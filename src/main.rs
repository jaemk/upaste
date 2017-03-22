#![recursion_limit = "1024"]

#[macro_use] extern crate clap;
extern crate itertools;
extern crate reqwest;
extern crate serde_json;
#[macro_use] extern crate error_chain;
mod errors {
    error_chain! { }
}

use std::env;
use std::io::{self, Read, BufReader, BufRead};
use std::fs::File;
use std::path::PathBuf;
use clap::{Arg, App, ArgMatches};
use serde_json::Value;
use itertools::Itertools;
use errors::*;


fn main() {
    let matches = App::new("upaste")
        .version(crate_version!())
        .author("James K. <james.kominick@gmail.com>")
        .about(r##"
** CLI pasting client -- defaults to https://hastebin.com
** Reads from stdin or a specified file:
   >> cat file.txt | upaste
   >> upaste -f file.txt"##)
        .arg(Arg::with_name("file")
                .short("f")
                .long("file")
                .takes_value(true)
                .help("file to upload"))
        .arg(Arg::with_name("start")
                .short("s")
                .long("start")
                .takes_value(true)
                .help("line number to start reading at (1 being the first)"))
        .arg(Arg::with_name("lines")
                .short("l")
                .long("lines")
                .takes_value(true)
                .help("number of lines to read"))
        .arg(Arg::with_name("pull")
                .short("p")
                .long("pull")
                .takes_value(true)
                .help("pull an existing paste to stdout"))
        .arg(Arg::with_name("raw")
                .short("r")
                .long("raw")
                .help("return link to raw version"))
        .arg(Arg::with_name("paste-root")
                .long("paste-root")
                .takes_value(true)
                .help("Host url to upload to. Defaults to https://hastebin.com/documents"))
        .arg(Arg::with_name("read-root")
                .long("read-root")
                .takes_value(true)
                .help("Host url-root to use when linking to and pulling down posts. Defaults to https://hastebin.com/"))
        .get_matches();

    if let Err(ref e) = run(matches) {
        use ::std::io::Write;
        let stderr = &mut ::std::io::stderr();
        let stderr_msg = "Error writing to stderr";
        writeln!(stderr, "error: {}", e).expect(stderr_msg);

        for e in e.iter().skip(1) {
            writeln!(stderr, "caused by: {}", e).expect(stderr_msg);
        }

        // `RUST_BACKTRACE=1`
        if let Some(backtrace) = e.backtrace() {
            writeln!(stderr, "backtrace: {:?}", backtrace).expect(stderr_msg);
        }

        ::std::process::exit(1);
    }
}


/// Pull down content from `read_root` associated with a given key
fn pull_content(read_root: &str, key: &str) -> Result<(String, PathBuf)> {
    let mut p = PathBuf::from(read_root);
    p.push("raw");
    p.push(key);

    let client = reqwest::Client::new().unwrap();
    let mut resp = client.get(p.to_str().unwrap())
                         .send()
                         .chain_err(|| format!("Error sending request to: {}", p.display()))?;
    let mut content = String::new();
    let _ = resp.read_to_string(&mut content).unwrap();
    Ok((content, p))
}


/// Post content to `paste_root` and return a url to where we can view it.
/// Url returned is constructed with respect to `read_root` and `raw`.
fn post_content(paste_root: &str, read_root: &str, content: &str, raw: bool) -> Result<PathBuf> {
    let client = reqwest::Client::new().unwrap();
    let mut resp = client.post(paste_root)
                         .body(content)
                         .send()
                         .chain_err(|| format!("Error sending info to: {}", paste_root))?;
    let resp: Value = resp.json().chain_err(|| {
        let mut body = String::new();
        let _ = resp.read_to_string(&mut body).unwrap();
        format!("Error decoding response: {:?}", body)
    })?;

    // build the url where our content is located
    let key = resp["key"].to_string();
    let key = key.trim_matches('"');
    let raw = if raw { "raw" } else { "" };

    let mut p = PathBuf::from(read_root);
    p.push(raw);
    p.push(key);
    Ok(p)
}


/// Helper to read both file-buffers and stdin-buffers into a String.
/// Skips over `lines_to_skip` and only reads `lines_to_read` lines if
/// `lines_to_read` is specified and is a valid usize.
fn read<T: BufRead>(reader: T, lines_to_skip: usize, lines_to_read: Option<&str>) -> Result<String> {
    let mut lines = reader.lines().map(|l| l.unwrap_or("".into())).skip(lines_to_skip);
    Ok(match lines_to_read {
        Some(n_lines) => {
            let n = n_lines.parse::<usize>().chain_err(|| format!("Invalid lines param: {}", n_lines))?;
            lines.take(n).join("\n")
        }
        None => lines.join("\n"),
    })
}

/// Read content from either a specified file or from stdin into a String.
/// If `start` is specified, start reading at line number `start`.
/// If `lines` is specified, only read the number of `lines` specified.
fn read_input(matches: &ArgMatches) -> Result<String> {
    let start = matches.value_of("start").unwrap_or("1");
    let lines_to_skip = start.parse::<usize>()
        .chain_err(|| format!("Invalid start param: {}", start))? - 1;
    let lines_to_read = matches.value_of("lines");

    match matches.value_of("file") {
        Some(file_name) => {
            let file = File::open(file_name)
                .chain_err(|| format!("Unable to open file: {}", file_name))?;
            read(BufReader::new(file), lines_to_skip, lines_to_read)
        }
        None => {
            let stdin = io::stdin();
            let std_buf = stdin.lock();
            read(std_buf, lines_to_skip, lines_to_read)
        }
    }
}


fn run(matches: ArgMatches) -> Result<()> {
    // Get url roots
    let paste_root_default = env::var("UPASTE_PASTEROOT")
        .unwrap_or("https://hastebin.com/documents".into());
    let read_root_default = env::var("UPASTE_READROOT")
        .unwrap_or("https://hastebin.com".into());
    let paste_root = matches.value_of("paste-root").unwrap_or(&paste_root_default);
    let read_root = matches.value_of("read-root").unwrap_or(&read_root_default);

    // Handle pulling down existing pastes
    if let Some(existing_key) = matches.value_of("pull") {
        let (content, url) = pull_content(read_root, existing_key)
            .chain_err(|| format!("Error pulling content for key: {}", existing_key))?;
        println!("** {} **\n\n{}", url.display(), content);
        return Ok(())
    }

    // Read in content to post. Either from a file or stdin
    let content = read_input(&matches).chain_err(|| "Error reading input")?;

    // Post content
    let url = post_content(paste_root, read_root, &content, matches.is_present("raw"))
        .chain_err(|| format!("Error posting content to: {}", paste_root))?;
    println!(" ** Success! Content available at: {}", url.display());
    Ok(())
}

