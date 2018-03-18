
use ::migration::{SourceFile, TargetFile};
use std::io::prelude::*;
use std::io::{Read,SeekFrom};
use std::fs::File;
use std::collections::HashMap;
use migration::bip_metainfo::{Metainfo};
use migration::sha1::{Sha1, Digest};
use migration::rayon::prelude::*;

#[derive(Debug)]
struct PieceResult {
    files: Vec<usize>,
    success: bool,
}

#[derive(Debug)]
struct FileResult {
    good:i32,
    total:i32,
}

// run a hash check on the given configuration
fn hash_check(torrent_meta:&Metainfo, inputs:&Vec<SourceFile>, targets:&Vec<TargetFile>) -> Vec<FileResult> {
    let info = torrent_meta.info();
    let piece_length = info.piece_length();
    let pieces = info.pieces();
    // put the pieces into a vec so we can use Rayon on them
    let pieces_vec:Vec<&[u8]> = pieces.collect();

    // calculate the results
    let results:Vec<PieceResult> = pieces_vec.par_iter().enumerate().map(|item| {
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

        // result building
        let mut piece_result = PieceResult { files:Vec::new(), success:false };
        // create buffer
        let mut buffer:Vec<u8> = Vec::with_capacity(piece_length as usize);
        
        // fill up the buffer
        while buffer.len() < piece_length as usize {
            piece_result.files.push(curfile.index);
            if let Some(mapping) = curfile.mapping {
                // load piece of the file into buffer
                let mut file = File::open(&inputs[mapping].path).unwrap();
                file.seek(SeekFrom::Start(file_offset)).expect("Unable to seek in file");
                file.take(piece_length-buffer.len() as u64).read_to_end(&mut buffer).expect("Unable to seek in file");
            } else {
                // no mapping available, which means we can give up on this piece
                // (Minor TODO: this can cause pieces not to show up as matches for boundary pieces)
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
        let output:Vec<u8> = Sha1::digest(&buffer).to_vec();
        piece_result.success = output == *item.1;

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

    let result:Vec<FileResult> = targets.iter().map(|target| {
        let mut good = 0;
        let mut total = 1;
        if let Some(info) = merged_results.get(&target.index) {
            good = info.iter().fold(0,|mut acc,x| {
                if *x { acc += 1; }
                acc
            });
            total = info.len() as i32
        } 
        FileResult { good, total }
    }).collect();

    result
}

fn print_hash_result(result:&Vec<FileResult>, targets:&Vec<TargetFile>) {
    println!("Hash test result:");
    let max = targets.iter().map(|e| e.path.to_string_lossy().len()).max().unwrap();
    for (index, target) in targets.iter().enumerate() {
        let info = &result[index];
        println!("  {:3$} = {:.1}%{}", 
            target.path.to_string_lossy(),
            (info.good as f32/info.total as f32)*100.0,
            if target.mapping == None { " (unmapped)" } else { "" },
            max+1,
        );
    }
}

pub fn run_matcher(torrent_meta:Metainfo, inputs:Vec<SourceFile>, mut targets:Vec<TargetFile>) {
    // do a first hash check
    let result = hash_check(&torrent_meta, &inputs, &targets);
    print_hash_result(&result,&targets);

    // take actions on the failed files
    for (index, target) in targets.iter_mut().enumerate() {
        let info = &result[index];
        if (info.good as f32/info.total as f32) < 0.2 && target.is_audio {
            // only if we have a mapping
            if let Some(mapping) = target.mapping {
                // set the offset
                target.offset = inputs[mapping].size - target.size;
                // recheck the file
                
                // if still not ok, try sliding window approach to find a piece match and find the offset
                
            }
        }
    }

    let result = hash_check(&torrent_meta, &inputs, &targets);
    print_hash_result(&result,&targets);
    // do a hash check on the data

    // try the sliding window approach

    // execute the migration
}