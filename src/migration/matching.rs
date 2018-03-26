use ::migration::{SourceFile, TargetFile};
use std::io::prelude::*;
use std::io;
use std::io::{Read,SeekFrom};
use std::fs::File;
use std::collections::HashMap;
use migration::bip_metainfo::{Metainfo};
use migration::sha1::{Sha1, Digest};
use migration::rayon::prelude::*;
use migration::rayon::iter::{IntoParallelIterator, ParallelIterator};

#[derive(Debug)]
struct PieceResult {
    files: Vec<usize>,
    success: bool,
}

#[derive(Debug, Clone)]
struct FileResult {
    good:u32,
    total:u32,
}

// run a hash check on the given configuration, onlyscanfile allows for hash checking a single file
fn hash_check(torrent_meta:&Metainfo, inputs:&Vec<SourceFile>, targets:&Vec<TargetFile>, onlyscanfile:Option<usize>) -> Vec<FileResult> {
    let info = torrent_meta.info();
    let piece_length = info.piece_length();
    let pieces = info.pieces();
    // put the pieces into a vec so we can use Rayon on them
    let pieces_vec:Vec<&[u8]> = pieces.collect();

    // optionally filter a piece range if we ask for a single file
    let mut scan_range:bool = false;
    let mut piece_range = (0,0);
    if let Some(f) = onlyscanfile {
        scan_range = true;
        // get the byte range of the target
        let mut file_offset = 0;
        let mut file_size = 0;
        for target in targets.iter().enumerate() {
            if target.0 == f {
                file_size = target.1.size;
                break;
            }
            file_offset += target.1.size;
        }
        // calculate the piece ranges for this
        piece_range = (
            (file_offset as f32 / piece_length as f32) as usize,
            ((file_offset + file_size) as f32 / piece_length as f32) as usize
        );
    }

    // iterate the pieces and calculate the results in parallel
    let results:Vec<PieceResult> = pieces_vec.par_iter().enumerate().filter(|item| {
        !scan_range || (item.0 >= piece_range.0 && item.0 < piece_range.1)
    }).map(|item| {
        // calculate the offset of this piece
        let piece_offset = item.0 as u64 * piece_length;

        // define the starting point file
        let mut file_offset = 0;
        let mut curfile = &targets[0];
        for target in targets {
            curfile = target;
            file_offset += target.size;
            if file_offset > piece_offset {
                file_offset -= target.size;
                break;
            }
        }
        file_offset = piece_offset - file_offset;

        // keep track of result and files that are part of this piece
        let mut piece_result = PieceResult { files:Vec::new(), success:false };

        // create buffer for this piece
        let mut buffer:Vec<u8> = Vec::with_capacity(piece_length as usize);
        
        // fill up the buffer
        while buffer.len() < piece_length as usize {
            piece_result.files.push(curfile.index);
            if let Some(mapping) = curfile.mapping {
                // load piece of the file into buffer
                let mut file = File::open(&inputs[mapping].path).unwrap();
                // check if we are not trying negative seek, in that case, mark the piece as failed immediately
                if (file_offset as i64 + curfile.offset) < 0 {
                    piece_result.success = false;
                    return piece_result;
                }
                let total_offset  = (file_offset as i64 + curfile.offset) as u64;
                file.seek(SeekFrom::Start(total_offset as u64)).expect("Unable to seek in file");
                // put a cap on the number of bytes taken to not get any bytes of the (offset adjusted) end of the file
                let mut numbytes = piece_length-buffer.len() as u64;
                if total_offset + numbytes > (curfile.size as i64 + curfile.offset) as u64 {
                    numbytes = (curfile.size as i64 - total_offset as i64 + curfile.offset) as u64;
                }
                file.take(numbytes).read_to_end(&mut buffer).expect("Unable to read file");
                // (Minor TODO: if the input is smaller than the target and we haven't filled the buffer, 
                // we might want to add padding to make sure we don't incorrectly mark a subsequent file as part of this piece)
            } else {
                // no mapping available, which means we can give up on this piece
                // (Minor TODO: this can cause pieces not to show up as non matches for boundary pieces)
                piece_result.success = false;
                return piece_result;
            }
            // take next file if available
            if let Some(newf) = targets.get(curfile.index + 1) {
                file_offset = 0;
                curfile = newf;
            } else {
                break;
            }
        }
        // sha1 on the created buffer, check if it matches the piece
        piece_result.success = Sha1::digest(&buffer).as_slice() == *item.1;

        piece_result
    }).collect();

    // merge these results into a hashmap
    let merged_results:HashMap<usize, Vec<bool>> = results.iter().fold(HashMap::new(), |mut acc, x| {
        for index in &x.files {
            if !acc.contains_key(index) {
                acc.insert(*index, Vec::new());
            }
            acc.get_mut(&index).unwrap().push(x.success);
        }
        acc
    });

    // and get the results for each target file into a vec
    let result:Vec<FileResult> = targets.iter().map(|target| {
        let mut good = 0;
        let mut total = 1;
        if let Some(info) = merged_results.get(&target.index) {
            good = info.iter().fold(0,|mut acc,x| {
                if *x { acc += 1; }
                acc
            });
            total = info.len() as u32
        } 
        FileResult { good, total }
    }).collect();

    // if this is a single file hash, just return a vec with that only result
    if let Some(f) = onlyscanfile {
        return vec![result[f].clone()];
    };

    result
}

fn piece_search(index:usize,torrent_meta:&Metainfo, inputs:&Vec<SourceFile>, targets:&Vec<TargetFile>) -> Option<i64> {
    // get the offsets of our target file
    let mut file_offset = 0;
    let mut file_size = 0;
    for target in targets.iter().enumerate(){
        if target.0 == index {
            file_size = target.1.size;
            break;
        }
        file_offset += target.1.size;
    }
    
    // calculate the middle peace for this file
    let piece_length = torrent_meta.info().piece_length();
    let middle_piece = ((file_offset as f64 + (file_size as f64 / 2.0)) / piece_length as f64) as usize;
    
    // get the hash for this piece
    let piece = torrent_meta.info().pieces().nth(middle_piece).unwrap();
    
    // load the whole input file into a vec
    let mappedfile = targets[index].mapping.expect("Mapped file not found");
    let inputfile = &inputs[mappedfile];
    let mut buffer:Vec<u8> = Vec::with_capacity(inputfile.size as usize);
    let mut file = File::open(&inputfile.path).expect("Unable to open file for reading");
    file.read_to_end(&mut buffer).expect("Unable to read file");

    // determine our window size
    let windowsize = (targets[index].size as i64 - inputfile.size as i64).abs() + piece_length as i64;
    
    // do the search
    // (TODO: this iterator should start in the middle instead to potentially speed up the search considerably)
    let result = (((file_size as f64 / 2.0) as i64 - windowsize) as usize..((file_size as f64 / 2.0) as i64 + windowsize) as usize).into_par_iter().find_any(|x| {
        if Sha1::digest(&buffer[*x..*x+piece_length as usize]).as_slice() == piece {
            return true;
        }
        false
    });
    if let Some(offs) = result {
        // figure out where this piece offset should normally be
        let normal = piece_length as u64 * middle_piece as u64 - file_offset;
        let offset = normal as i64 - offs as i64;
        return Some(offset);
    }

    None
}

fn print_hash_result(result:&Vec<FileResult>, targets:&Vec<TargetFile>) {
    let max = targets.iter().map(|e| e.path.to_string_lossy().len()).max().unwrap();
    let mut total_good = 0;
    let mut total_total = 0;
    for (index, target) in targets.iter().enumerate() {
        let info = &result[index];
        total_good += info.good;
        total_total += info.total;
        println!("  {:3$} = {:.1}%{}", 
            target.path.to_string_lossy(),
            (info.good as f32/info.total as f32)*100.0,
            if target.mapping == None { " (unmapped)" } else { "" },
            max+1,
        );
    }
    println!("Overall hash result: {:.2}%", (total_good as f32 / total_total as f32) * 100.0);
}

pub fn run_matcher(torrent_meta:&Metainfo, inputs:&mut Vec<SourceFile>, targets:&mut Vec<TargetFile>) {
    // do a first hash check
    let result = hash_check(&torrent_meta, &inputs, &targets, None);
    println!("Initial hash test result:");
    print_hash_result(&result,&targets);

    // take actions on failed files music files
    let mut corrected_pieces = 0;
    let mut agreed_to_search = false;
    let mut rejected_search = false;
    for index in 0..targets.len() { // not iterating over targets directly to avoid reference
        let info = &result[index];
        if (info.good as f32/info.total as f32) < 0.2 && targets[index].is_audio {
            // only if we have a mapping
            if let Some(mapping) = targets[index].mapping {
                // keep track of how many we are correcting
                corrected_pieces += 1;
                // adjust the offset to right aligned instead of left aligned
                targets[index].offset = inputs[mapping].size as i64 - targets[index].size as i64;
                // hash check this single file
                let fileresult = &hash_check(&torrent_meta, &inputs, &targets, Some(index))[0];
                // if still not good, we'll try a more in depth piece search
                if (fileresult.good as f32/fileresult.total as f32) < 0.2 {
                    if !rejected_search {
                        if !agreed_to_search {
                            println!("Realign not succesful on at least one file, want to try a (slow and CPU heavy) piece search? (y/n) [n]");
                            let mut reply = String::new();
                            io::stdin().read_line(&mut reply).unwrap();
                            match reply.trim() {
                                "y" | "yes" => {
                                    agreed_to_search = true;
                                },
                                _ => {
                                    rejected_search = true;
                                }
                            }
                        }
                        if agreed_to_search {
                            // do a piece search on this file as final attempt
                            if let Some(offset) = piece_search(index,&torrent_meta, &inputs, &targets) {
                                targets[index].offset = -offset;
                            }
                        }
                    }
                }
            }
        }
    }

    // check if we made any corrections
    if corrected_pieces > 0 {
        // do a final hash check on the data
        let result = hash_check(&torrent_meta, &inputs, &targets, None);
        println!("Hash test result after optimization:");
        print_hash_result(&result,&targets);        
    }
}