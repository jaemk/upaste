#![recursion_limit = "1024"]

#[macro_use] extern crate clap;
#[macro_use] extern crate error_chain;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate itertools;
extern crate reqwest;
extern crate url;

error_chain! {
    foreign_links {
        Reqwest(reqwest::Error);
        Io(std::io::Error);
        Url(url::ParseError);
    }
}

use std::env;
use std::path;
use std::io::{self, Read, BufReader, BufRead};
use std::fs::File;
use clap::{Arg, App, ArgMatches};
use itertools::Itertools;
use url::Url;


quick_main!(run);


fn run() -> Result<()> {
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
                .help("Host url to upload to. Defaults to https://hastebin.com/documents or $UPASTE_PASTEROOT"))
        .arg(Arg::with_name("read-root")
                .long("read-root")
                .takes_value(true)
                .help("Host url-root to use when linking to and pulling down posts. Defaults to https://hastebin.com/ or $UPASTE_READROOT"))
        .get_matches();

    #[cfg(target_os="linux")]
    {
        if env::var_os("SSL_CERT_FILE").is_none() {
            env::set_var("SSL_CERT_FILE", "/etc/ssl/certs/ca-certificates.crt");
        }
        if env::var_os("SSL_CERT_DIR").is_none() {
            env::set_var("SSL_CERT_DIR", "/etc/ssl/certs");
        }
    }

    // Get url roots
    let paste_root_default = env::var("UPASTE_PASTEROOT")
        .unwrap_or("https://hastebin.com/documents".into());
    let read_root_default = env::var("UPASTE_READROOT")
        .unwrap_or("https://hastebin.com".into());
    let paste_root = matches.value_of("paste-root").unwrap_or(&paste_root_default);
    let read_root = matches.value_of("read-root").unwrap_or(&read_root_default);

    // Handle pulling down existing pastes
    if let Some(existing_key) = matches.value_of("pull") {
        let read_root_p = path::PathBuf::from(read_root);
        let read_root_p = if read_root.starts_with("https://paste.rs") { read_root_p } else { read_root_p.join("raw") };
        let (content, url) = pull_content(&read_root_p, existing_key)
            .chain_err(|| format!("Error pulling content for key: {}", existing_key))?;
        println!("** {} **\n\n{}", url, content);
        return Ok(())
    }

    // Read in content to post. Either from a file or stdin
    let content = read_input(&matches).chain_err(|| "Error reading input")?;

    // Post content
    let url = post_content(paste_root, read_root, &content, matches.is_present("raw"))
        .chain_err(|| format!("Error posting content to: {}", paste_root))?;
    println!(" ** Success! Content available at: {}", url);
    Ok(())
}


/// Pull down content from `read_root` associated with a given key
fn pull_content(read_root: &path::Path, key: &str) -> Result<(String, Url)> {
    let read_root = Url::parse(read_root.join(key).to_str().expect("invalid path"))?;

    let client = reqwest::Client::new();
    let mut resp = client.get(read_root.as_str())
                         .send()
                         .chain_err(|| format!("Error sending request to: {}", read_root))?;
    let mut content = String::new();
    let _ = resp.read_to_string(&mut content)?;
    Ok((content, read_root))
}


#[derive(Debug, Clone, Deserialize)]
struct PostResponse {
    key: String,
}

/// Post content to `paste_root` and return a url to where we can view it.
/// Url returned is constructed with respect to `read_root` and `raw`.
fn post_content(paste_root: &str, read_root: &str, content: &str, raw: bool) -> Result<Url> {
    let client = reqwest::Client::new();
    let mut resp = client.post(paste_root)
                         .body(content.to_owned())
                         .send()?
                         .error_for_status()?;

    let mut content = String::new();
    resp.read_to_string(&mut content)?;

    // paste.rs returns key in the body
    if paste_root.starts_with("https://paste.rs") {
        Ok(Url::parse(&content)?)
    } else {
        // everything returns key in json
        // body has already been read into content
        let resp = serde_json::from_str::<PostResponse>(&content)
            .chain_err(|| format!("Failed parsing: {:?}", content))?;
        let key = resp.key.trim_matches('"');
        let url = path::PathBuf::from(read_root);
        let url = if raw { url.join("raw") } else { url };
        Ok(Url::parse(url.join(key).to_str().expect("invalid path"))?)
    }
}


/// Helper to read both file-buffers and stdin-buffers into a String.
/// Skips over `lines_to_skip` and only reads `lines_to_read` lines if
/// `lines_to_read` is specified and is a valid usize.
fn read<T: BufRead>(reader: T, lines_to_skip: usize, lines_to_read: Option<&str>) -> Result<String> {
    let mut lines = reader.lines().map(|l| l.unwrap_or("".into())).skip(lines_to_skip);
    Ok(match lines_to_read {
        Some(n_lines) => {
            let n = n_lines.parse::<usize>().chain_err(|| format!("Invalid lines param: {}, expected int", n_lines))?;
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
        .chain_err(|| format!("Invalid start param: {}, expected int", start))? - 1;
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

