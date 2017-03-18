## upaste [![Build Status](https://travis-ci.org/jaemk/upaste.svg?branch=master)](https://travis-ci.org/jaemk/upaste) [![crates.io](https://img.shields.io/crates/v/upaste.svg)](https://crates.io/crates/upaste)
> hastebin / general pasting client

Simple client for uploading to hastebin.com or any site that accepts posting and viewing pastes

## Installation

Static binary releases currently only exist for linux. See [releases](https://github.com/jaemk/upaste/releases).

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

# pulling existing paste into file
upaste --pull <key> > <file>
```

```
 $ upaste --help

   upaste 0.2.0
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
           --paste-root <paste-root>    Host url to upload to. Defaults to https://hastebin.com/documents
       -p, --pull <pull>                pull an existing paste to stdout
           --read-root <read-root>      Host url-root to use when linking to post. Defaults to https://hastebin.com/

```
