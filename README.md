## upaste [![Build Status](https://travis-ci.org/jaemk/upaste.svg?branch=master)](https://travis-ci.org/jaemk/upaste) [![crates.io](https://img.shields.io/crates/v/upaste.svg)](https://crates.io/crates/upaste)
> paste.rs / hastebin / general pasting client

Simple client for uploading to paste.rs, hastebin.com, or any site that accepts posting and viewing pastes

## Installation

Binary releases available for linux & osx. See [releases](https://github.com/jaemk/upaste/releases).

For installation on other platforms use cargo:
```
cargo install upaste
```

## Usage
```
# simple
cat file | upaste
upaste -f <file>
# ->  ** Success! Content available at: https://hastebin.com/<some-key>

# raw
cat file | upaste --raw
upaste --file <file> --raw
# ->  ** Success! Content available at: https://hastebin.com/raw/<some-key>

# custom paste/read locations
upaste --file <file> --paste-root https://hastebin.com/documents --read-root https://hastebin.com
# ->  ** Success! Content available at: <read-root>/<some-key>

# Or specify your alternate roots as ENV vars
UPASTE_PASTEROOT=https://paste.rs
UPASTE_READROOT=https://paste.rs
upaste --file <file>
# ->  ** Success! Content available at: <UPASTE_READROOT>/<some-key>

# specifying a range of lines (start at line 15, read 30 lines)
upaste --file <file> --start 15 --lines 30

# pulling existing paste into file
upaste --pull <key> > <file>
```
