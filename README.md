## upaste [![Build Status](https://travis-ci.org/jaemk/upaste.svg?branch=master)](https://travis-ci.org/jaemk/upaste) [![crates.io](https://img.shields.io/crates/v/upaste.svg)](https://crates.io/crates/upaste)
> hastebin / general pasting client

Simple client for uploading to hastebin.com or any site that accepts posting and viewing pastes

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
UPASTE_PASTEROOT=https://mypasteservice.com/new
UPASTE_READROOT=https://mypasteservice.com
upaste --file <file>
# ->  ** Success! Content available at: <UPASTE_READROOT>/<some-key>

# specifying a range of lines (start at line 15, read 30 lines)
upaste --file <file> --start 15 --lines 30

# pulling existing paste into file
upaste --pull <key> > <file>
```

##
```
 $ upaste -h

   upaste 0.2.2
   James K. <james.kominick@gmail.com>
   
   ** CLI pasting client -- defaults to https://hastebin.com
   ** Reads from stdin or a specified file:
      >> cat file.txt | upaste
      >> upaste -f file.txt
   
   USAGE:
       upaste [FLAGS] [OPTIONS]
   
   FLAGS:
       -h, --help       Prints help information
       -r, --raw        return link to raw version
       -V, --version    Prints version information
   
   OPTIONS:
       -f, --file <file>                file to upload
       -l, --lines <lines>              number of lines to read
           --paste-root <paste-root>    Host url to upload to. Defaults to https://hastebin.com/documents or $UPASTE_PASTEROOT
       -p, --pull <pull>                pull an existing paste to stdout
           --read-root <read-root>      Host url-root to use when linking to and pulling down posts. Defaults to https://hastebin.com/ or
                                        $UPASTE_READROOT
       -s, --start <start>              line number to start reading at (1 being the first)
```
