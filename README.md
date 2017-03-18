#### hastebin / general pasting client

Simple client for uploading to hastebin.com or any site that accepts posting and viewing pastes


## Usage
```
# simple
cat file | rpaste
rpaste -f <file>

# raw
cat file | rpaste --raw
rpaste --file <file> --raw

# custom paste/read locations
rpaste --file <file> --paste-root https://hastebin.com/documents --read-root https://hastebin.com
```
