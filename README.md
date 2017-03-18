## upaste [![Build Status](https://travis-ci.org/jaemk/upaste.svg?branch=master)](https://travis-ci.org/jaemk/upaste)
> hastebin / general pasting client

Simple client for uploading to hastebin.com or any site that accepts posting and viewing pastes


## Usage
```
# simple
cat file | upaste
upaste -f <file>

# raw
cat file | upaste --raw
upaste --file <file> --raw

# custom paste/read locations
upaste --file <file> --paste-root https://hastebin.com/documents --read-root https://hastebin.com
```
