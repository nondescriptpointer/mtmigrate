extern crate clap;
use clap::{Arg, App};
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
mod migration;

fn main() {
    let matches = App::new("mtmigrate")
                    .version("0.1.0")
                    .author("Thomas Colliers <mail@thomascolliers.com>")
                    .about("mtmigrate (music torrent migrate) is a tool to help you with migrating your old data to a new torrent after a trump or generally when trying to join a swarm with existing data.")
                    .arg(Arg::with_name("torrent")
                        .long("torrent")
                        .value_name("torrent file")
                        .help("Torrent file to try to map to")
                        .required(true)
                        .index(1)
                        .takes_value(true))
                    .arg(Arg::with_name("input")
                        .long("input")
                        .value_name("input")
                        .help("Directory to try and map")
                        .required(true)
                        .index(2)
                        .takes_value(true))
                    .arg(Arg::with_name("output")
                        .long("output")
                        .value_name("output")
                        .help("Output directory")
                        .required(true)
                        .index(3)
                        .takes_value(true))
                    .get_matches();

    let torrent_file = matches.value_of("torrent").unwrap();
    let input = matches.value_of("input").unwrap();
    let output = matches.value_of("output").unwrap();

    // read the torrent file into a byte vector
    let mut f = File::open(torrent_file).expect("Failed to open torrent file");
    let path = Path::new(torrent_file);
    let size = path.metadata().expect("Failed to access torrent file metadata").len() as usize;
    let mut buffer = Vec::with_capacity(size);
    f.read_to_end(&mut buffer).expect("Faled to load file into buffer");

    migration::run(buffer, input, output).expect("Migration failed");
}