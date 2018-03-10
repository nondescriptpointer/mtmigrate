# mtmigrate

mtmigrate (music torrent migrate) is a tool to help you with migrating your old data to a new torrent after a trump or generally when trying to join a swarm with existing data. It will suggest file renaming, will try to repad the files if the filesizes don't match and will even try to scan the whole file with a sliding window to find a match for a piece hash.

It is primarily target towards FLAC files as these are lossless and 2 rips of the same CD should be binary compatible if the same encoding settings were used.

It is a reimplementation of my original tool [whatmigrate](https://github.com/ThomasColliers/whatmigrate) in Rust.

## Dependencies
- rustc/cargo

## Installation and usage
- Make sure rustc/cargo are installed
- Clone the repository
- Run cargo build --release
- Run the binary in the target/release directory, running it will list all the available arguments

## Todo
- If FLAC files fail to match a torrent, try reencoding the torrent with some different encoding settings
- Add support for torrent clients
