extern crate bip_metainfo;
extern crate walkdir;
use self::bip_metainfo::{Metainfo};
use self::walkdir::{DirEntry, WalkDir};
use std::path::{PathBuf};
use std::collections::{HashSet};
use std::io;

static AUDIO_FORMATS:&'static[&'static str] = &["flac","mp3","ogg","aac","ac3","dts"];

pub type MigrationError = Box<::std::error::Error>;

#[derive(Debug)]
struct SourceFile {
    path: PathBuf,
    display: String,
    extension: Option<String>,
    is_audio: bool,
    size: u64,
    mapping: Option<usize>
}

#[derive(Debug)]
struct TargetFile {
    index: usize,
    path: PathBuf,
    extension: Option<String>,
    is_audio: bool,
    size: u64
}

// ignore all results that are not a file
fn is_file(entry: &Result<DirEntry, self::walkdir::Error>) -> bool {
    match *entry {
        Ok(ref e) => (*e).file_type().is_file(),
        Err(_) => false
    }
}

fn print_mapping(inputs: &Vec<SourceFile>, targets: &Vec<TargetFile>) {
    // determine the maximume length so we can pad this mapping out
    let max = inputs.iter().map(|e| e.display.len()).max().unwrap();
    // print all the mappings we found
    for input in inputs.iter() {
        if let Some(ref i) = input.mapping {
            println!("  {:2$} => {}", input.display, targets[*i].path.to_string_lossy(), max+1);
        } else {
            println!("  {:1$} => None", input.display, max+1);
        }
    }
}

pub fn run<B>(buffer: B, input: &str, output: &str) -> Result<(),MigrationError> 
    where B: AsRef<[u8]> {
    // build the set of audio formats
    let audio_formats:HashSet<String> = AUDIO_FORMATS.into_iter().map(|x| x.to_string()).collect();

    // extract the metadata
    let torrent_meta = Metainfo::from_bytes(&buffer).expect("Failed to parse torrent file");

    // get files (recursively) from the input directory
    let mut inputs = Vec::new();
    let walker = WalkDir::new(input).into_iter();
    for entry in walker.filter(|e| is_file(e)) {
        let entry = entry?;
        let path = entry.path();
        let size = path.metadata().expect("Failed to access file metadata").len();
        let extension = match path.extension() {
            Some(e) => Some(e.to_string_lossy().into_owned()),
            None => None
        };
        let is_audio = if let Some(ref i) = extension {
            audio_formats.contains(i)
        } else {
            false
        };
        let display = path.strip_prefix(input).unwrap().to_string_lossy().into_owned();
        //let is_audio =  extension.is_some() && audio_formats.cont
        inputs.push(SourceFile { path:path.to_path_buf(), display, extension, is_audio, size, mapping:None });
    }
    // sort these files so they are easier to use
    inputs.sort_by(|a, b| a.path.cmp(&b.path));

    // get the target files from the torrent metadata
    let mut targets = Vec::new();
    {
        let torrent_info = &torrent_meta.info();
        for (i, file) in torrent_info.files().enumerate() {
            let path = file.path();
            let extension = match path.extension() {
                Some(e) => Some(e.to_string_lossy().into_owned()),
                None => None
            };
            let is_audio = if let Some(ref i) = extension {
                audio_formats.contains(i)
            } else {
                false
            };
            targets.push(TargetFile { index:i, path:path.to_path_buf(), extension, is_audio, size:file.length() });
        }
    }

    // BUILD MAPPINGS

    // try to exact match non-audio files, these are usually small so not really worth trying something else on these
    for input in inputs.iter_mut().filter(|f| !f.is_audio) {
        match targets.iter().position(|target| {
            if input.size == target.size && input.extension == target.extension {
                true
            }else{
                false
            }
        }) {
            Some(pos) => input.mapping = Some(pos),
            None => {}
        }
    }

    // next, we'll want to sort the targets as well, goal is to use normal sorting to determine the mapping
    {
        let mut targets_audio:Vec<&TargetFile> = targets.iter().filter(|f| f.is_audio).collect();
        targets_audio.sort_by(|a, b| a.path.cmp(&b.path));
        println!("{:?}", targets_audio);
        for (i, input) in inputs.iter_mut().filter(|f| f.is_audio).enumerate() {
            if let Some(e) = targets_audio.get(i) {
                input.mapping = Some(e.index);
            }
        }
    }

    // for audio files
    println!("Suggested mapping based on filename sort:");
    print_mapping(&inputs, &targets);
    loop {
        println!("Please input 'c' to continue, 's' to try filesize remap or 'm' to manually adjust the mapping [c]");
        let mut reply = String::new();
        io::stdin().read_line(&mut reply);
        match reply.trim() {
            "" | "c" => {
                break;
            },
            "s" => {
                // adjust the remapping based on size
            },
            "m" => {
                // interactive remapping
            },
            _ => {
                println!("Unrecognized option.");
            }
        }
    }

    run_matches(torrent_meta, inputs, targets);

    Ok(())
}

// this will take our matches and run them
fn run_matches(torrent_meta:Metainfo, inputs:Vec<SourceFile>, targets:Vec<TargetFile>) {
    // check the filesizes

    // repad the files if filesizes differ

    // do a hash check on the data

    // if we don't have a match yet, try the sliding window approach
}