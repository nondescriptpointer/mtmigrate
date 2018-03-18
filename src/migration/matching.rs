
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

// run a hash check on the given configuration
fn hash_check(torrent_meta:&Metainfo, inputs:&Vec<SourceFile>, targets:&Vec<TargetFile>) -> HashMap<usize,Vec<bool>> {
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
        let mut starting = &targets[0];
        for target in targets {
            starting = target;
            file_offset += target.size;
            if file_offset > piece_offset {
                file_offset -= target.size;
                break;
            }
        }

        // result building
        let mut piece_result = PieceResult { files:vec!(starting.index), success:false };
        // check if we have a mapping for this file
        let mut result = false;
        if let Some(mapping) = starting.mapping {
            // create buffer
            let mut buffer:Vec<u8> = Vec::with_capacity(piece_length as usize);
            // load piece of the file into buffer
            let mut file = File::open(&inputs[mapping].path).unwrap();
            file.seek(SeekFrom::Start(piece_offset-file_offset)).expect("Unable to seek in file");
            file.take(piece_length).read_to_end(&mut buffer).expect("Unable to seek in file");
            // sha1 on this buffer, check if it matches the piece
            let output:Vec<u8> = Sha1::digest(&buffer).to_vec();
            result = output == *item.1;
        }
        piece_result.success = result;

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

    merged_results
}

fn print_hash_result(result:&HashMap<usize,Vec<bool>>, targets:&Vec<TargetFile>) {
    println!("Hash test result:");
    let max = targets.iter().map(|e| e.path.to_string_lossy().len()).max().unwrap();
    for target in targets {
        // get amount of good pieces for this file
        let mut good = 0;
        let mut total = 1;
        if let Some(item) = result.get(&target.index) {
            good = item.iter().fold(0,|mut acc,x| {
                if *x { acc += 1; }
                acc
            });
            total = item.len();
        }

        println!("  {:3$} = {:.1}%{}", 
            target.path.to_string_lossy(),
            (good as f32/total as f32)*100.0,
            if target.mapping == None { " (unmapped)" } else { "" },
            max+1,
        );
    }
}

pub fn run_matcher(torrent_meta:Metainfo, inputs:Vec<SourceFile>, targets:Vec<TargetFile>) {
    // check if filesizes don't match and repad the files if filesizes differ
    let result = hash_check(&torrent_meta, &inputs, &targets);
    print_hash_result(&result,&targets);
    // do a hash check on the data


    // if we don't have a match yet, finding the offset might be difficult as both front and back padding could be different

    // try the sliding window approach

}