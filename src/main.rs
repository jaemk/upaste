#![recursion_limit = "1024"]

extern crate clap;
extern crate itertools;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;
extern crate url;

use clap::{App, Arg, ArgMatches};
use itertools::Itertools;
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path;
use url::Url;

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

pub fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let matches = App::new("upaste")
        .version(clap::crate_version!())
        .author("James K. <james@kominick.com>")
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
        .arg(Arg::with_name("ttl-seconds")
                .short("t")
                .long("ttl-seconds")
                .takes_value(true)
                .help("seconds after which to expire"))
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
                .help("Host url to upload to. Defaults to https://hastebin.com/documents or $UPASTE_PASTEROOT"))
        .arg(Arg::with_name("read-root")
                .long("read-root")
                .takes_value(true)
                .help("Host url-root to use when linking to and pulling down posts. Defaults to https://hastebin.com/ or $UPASTE_READROOT"))
        .get_matches();

    #[cfg(target_os = "linux")]
    {
        if env::var_os("SSL_CERT_FILE").is_none() {
            env::set_var("SSL_CERT_FILE", "/etc/ssl/certs/ca-certificates.crt");
        }
        if env::var_os("SSL_CERT_DIR").is_none() {
            env::set_var("SSL_CERT_DIR", "/etc/ssl/certs");
        }
    }

    // Get url roots
    let paste_root_default =
        env::var("UPASTE_PASTEROOT").unwrap_or_else(|_| "https://hastebin.com/documents".into());
    let read_root_default =
        env::var("UPASTE_READROOT").unwrap_or_else(|_| "https://hastebin.com".into());
    let paste_root = matches
        .value_of("paste-root")
        .unwrap_or(&paste_root_default);
    let read_root = matches.value_of("read-root").unwrap_or(&read_root_default);

    // Handle pulling down existing pastes
    if let Some(existing_key) = matches.value_of("pull") {
        let read_root_p = path::PathBuf::from(read_root);
        let read_root_p = if read_root.starts_with("https://paste.rs") {
            read_root_p
        } else {
            read_root_p.join("raw")
        };
        let (content, url) = pull_content(&read_root_p, existing_key)
            .map_err(|e| format!("Error pulling content for key: {}, {}", existing_key, e))?;
        println!("** {} **\n\n{}", url, content);
        return Ok(());
    }

    // Read in content to post. Either from a file or stdin
    let content = read_input(&matches).map_err(|e| format!("Error reading input: {}", e))?;

    // Post content
    let ttl = match matches.value_of("ttl-seconds") {
        None => None,
        Some(ttl) => Some(
            ttl.parse::<u32>()
                .map_err(|e| format!("invalid ttl {}", e))?,
        ),
    };
    let url = post_content(
        paste_root,
        read_root,
        &content,
        matches.is_present("raw"),
        ttl,
    )
    .map_err(|e| format!("Error posting content to: {}, {}", paste_root, e))?;
    println!(" ** Success! Content available at: {}", url);
    Ok(())
}

/// Pull down content from `read_root` associated with a given key
fn pull_content(read_root: &path::Path, key: &str) -> Result<(String, Url)> {
    let read_root = Url::parse(read_root.join(key).to_str().expect("invalid path"))?;

    let resp = ureq::get(read_root.as_str()).call();
    if resp.error() {
        return Err(format!("Error: {}, {}", resp.status(), resp.status_text()).into());
    }
    let content = resp.into_string()?;
    Ok((content, read_root))
}

#[derive(Debug, Clone, serde::Deserialize)]
struct PostResponse {
    key: String,
}

/// Post content to `paste_root` and return a url to where we can view it.
/// Url returned is constructed with respect to `read_root` and `raw`.
fn post_content(
    paste_root: &str,
    read_root: &str,
    content: &str,
    raw: bool,
    ttl_seconds: Option<u32>,
) -> Result<Url> {
    let mut r = ureq::post(paste_root);
    if let Some(ttl) = ttl_seconds {
        r.query_str(&format!("?ttl_seconds={}", ttl));
    }
    let resp = r.send_string(content);
    if resp.error() {
        return Err(format!("Error: {}, {}", resp.status(), resp.status_text()).into());
    }
    let content = resp.into_string()?;

    // paste.rs returns key in the body
    if paste_root.starts_with("https://paste.rs") {
        Ok(Url::parse(&content)?)
    } else {
        // everything else returns a key in json
        let resp = serde_json::from_str::<PostResponse>(&content)
            .map_err(|e| format!("Failed parsing: {:?}, {}", content, e))?;
        let key = resp.key.trim_matches('"');
        let url = path::PathBuf::from(read_root);
        let url = if raw { url.join("raw") } else { url };
        Ok(Url::parse(url.join(key).to_str().expect("invalid path"))?)
    }
}

/// Helper to read both file-buffers and stdin-buffers into a String.
/// Skips over `lines_to_skip` and only reads `lines_to_read` lines if
/// `lines_to_read` is specified and is a valid usize.
fn read<T: BufRead>(
    reader: T,
    lines_to_skip: usize,
    lines_to_read: Option<&str>,
) -> Result<String> {
    let mut lines = reader
        .lines()
        .map(|l| l.unwrap_or_else(|_| "".into()))
        .skip(lines_to_skip);
    Ok(match lines_to_read {
        Some(n_lines) => {
            let n = n_lines
                .parse::<usize>()
                .map_err(|e| format!("Invalid lines param: {}, expected int, {}", n_lines, e))?;
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
    let lines_to_skip = start
        .parse::<usize>()
        .map_err(|e| format!("Invalid start param: {}, expected int, {}", start, e))?
        - 1;
    let lines_to_read = matches.value_of("lines");

    match matches.value_of("file") {
        Some(file_name) => {
            let file = File::open(file_name)
                .map_err(|e| format!("Unable to open file: {}, {}", file_name, e))?;
            read(BufReader::new(file), lines_to_skip, lines_to_read)
        }
        None => {
            let stdin = io::stdin();
            let std_buf = stdin.lock();
            read(std_buf, lines_to_skip, lines_to_read)
        }
    }
}
