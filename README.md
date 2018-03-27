# mtmigrate

mtmigrate (music torrent migrate) is a tool to help archivists with migrating old data to a new torrent after a trump or generally when trying to join a swarm with existing data. It will help to rename the files, will try to repad/realign the file data if the data doesn't match and will even try to scan the whole file with a sliding window to find a match for a piece hash. If the files are binary compatible, this tool should be able to find the match.

It is primarily target towards FLAC files as these are lossless and digitally (CD/WEB) sourced files should be binary compatible if the same encoding settings were used.

It is a reimplementation of my original tool [whatmigrate](https://github.com/ThomasColliers/whatmigrate) in Rust.

## Dependencies
- rustc/cargo

## Installation and usage
- Make sure rustc/cargo are installed
- Clone the repository
- Run cargo build --release
- Run the binary in the target/release directory, running it will list all the available arguments
- On first run, a JSON config file will be created in a platform specific configuration directory. (Linux: ~/.config/mtmigrate, macOS: $HOME/Library/Application Support/mtmigrate, Windows: %APPDATA%\mtmigrate\mtmigrate)

## Todo
- Optimize the piece search further, it's still fairly slow on low-end systems
- If FLAC files fail to match a torrent, try reencoding a piece with some different encoding settings to try and replicate the encoding settings
- Add integration with Torrent clients and Redacted