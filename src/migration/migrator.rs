use ::migration::{SourceFile, TargetFile};
use std::fs::File;
use std::path::{PathBuf};
use migration::bip_metainfo::{Metainfo};
use std::fs;
use std::io::prelude::*;
use std::io::{Read,SeekFrom};

pub fn migrate(torrent_meta:&Metainfo, inputs: &mut Vec<SourceFile>, targets: &mut Vec<TargetFile>, output: &str) {
    // create all the target files
    for target in targets {
        // build the path to the file
        let mut path = PathBuf::from(output);
        if let Some(p) = torrent_meta.info().directory() {
            path.push(p);
        }
        path.push(&target.path);
        let parent = path.parent().expect("Parent directory not found");
        // create the directory if it doesn't exist
        if !parent.exists() {
            fs::create_dir_all(parent).unwrap();
        }
        // write the file based on input data
        let mut file = File::create(&path).expect("Unable to write file");
        match target.mapping {
            Some(m) => {
                // read the sourcefile
                let mut sourcefile = File::open(&inputs[m].path).expect("Unable to read input file");
                // if we have a positive offset, seek into the file and write it out
                if target.offset > 0 {
                    sourcefile.seek(SeekFrom::Start(target.offset as u64)).expect("Unable to seek in input file");
                }
                // if we have a negative offset, add initial padding to the size of the offset
                if target.offset < 0 {
                    file.set_len((-target.offset) as u64).unwrap();
                }
                // write the input to the output
                let mut buf = Vec::new();
                sourcefile.read_to_end(&mut buf).expect("Unable to read input file");
                file.write_all(&buf).expect("Unable to write to output file");
                // adjust the length to the right target length
                file.set_len(target.size).unwrap();
            },
            None => {
                // no mapping, just expand the filesize to target size, this has to advantage to reserve the disk space
                file.set_len(target.size).unwrap();
            }
        }
    }
    println!("Migration complete!");
}