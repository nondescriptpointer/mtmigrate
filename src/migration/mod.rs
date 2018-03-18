extern crate bip_metainfo;
extern crate walkdir;
extern crate sha1;
extern crate rayon;
use self::bip_metainfo::{Metainfo};
use self::walkdir::{DirEntry, WalkDir};
use std::path::{PathBuf};
use std::collections::{HashSet};
mod filemapping;
mod matching;

static AUDIO_FORMATS:&'static[&'static str] = &["flac","mp3","ogg","aac","ac3","dts"];

pub type MigrationError = Box<::std::error::Error>;

#[derive(Debug)]
pub struct SourceFile {
    path: PathBuf,
    display: String,
    extension: Option<String>,
    is_audio: bool,
    size: u64,
    mapping: Option<usize>, // this holds which target file this maps to
}

#[derive(Debug)]
pub struct TargetFile {
    index: usize,
    path: PathBuf,
    extension: Option<String>,
    is_audio: bool,
    size: u64,
    mapping: Option<usize>, // this holds which source file this maps to
    finalized: bool,
}

// ignore all results that are not a file
fn is_file(entry: &Result<DirEntry, self::walkdir::Error>) -> bool {
    match *entry {
        Ok(ref e) => (*e).file_type().is_file(),
        Err(_) => false
    }
}

pub fn run<B>(buffer: B, input: &str, _output: &str) -> Result<(),MigrationError> 
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
            targets.push(TargetFile { index:i, path:path.to_path_buf(), extension, is_audio, size:file.length(), mapping:None, finalized:false });
        }
    }

    // define the mappings
    filemapping::create_mapping(&mut inputs, &mut targets);

    // run the matcher
    matching::run_matcher(torrent_meta, inputs, targets);

    Ok(())
}