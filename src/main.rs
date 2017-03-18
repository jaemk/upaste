#[macro_use] extern crate clap;
extern crate reqwest;
extern crate serde_json;
#[macro_use] extern crate error_chain;

mod errors {
    error_chain! { }
}

use std::io::{self, Read};
use std::fs::File;
use std::path::PathBuf;
use clap::{Arg, App, ArgMatches};
use serde_json::Value;
use errors::*;


fn main() {
    let matches = App::new("upaste")
        .version(crate_version!())
        .author("James K. <james.kominick@gmail.com")
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
                .help("Host url-root to use when linking to post. Defaults to https://hastebin.com/"))
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


fn run(matches: ArgMatches) -> Result<()> {
    // Get url roots
    let paste_root = matches.value_of("paste-root").unwrap_or("https://hastebin.com/documents");
    let read_root = matches.value_of("read-root").unwrap_or("https://hastebin.com");

    // Handle pulling down existing pastes
    if let Some(existing_key) = matches.value_of("pull") {
        let mut p = PathBuf::from(read_root);
        p.push("raw");
        p.push(existing_key);

        let client = reqwest::Client::new().unwrap();
        let mut resp = client.get(p.to_str().unwrap())
                             .send()
                             .chain_err(|| format!("Error sending request to: {}", p.display()))?;
        let mut content = String::new();
        let _ = resp.read_to_string(&mut content).unwrap();
        println!("** {} **\n\n{}", p.display(), content);
        return Ok(());
    }

    // Read in content to post. Either from a file or stdin
    let mut content = String::new();
    match matches.value_of("file") {
        Some(file_name) => {
            content = "read from file".to_string();
            let mut file = File::open(file_name)
                .chain_err(|| format!("Unable to open file: {}", file_name))?;
            let _ = file.read_to_string(&mut content)
                .chain_err(|| format!("Error reading from file: {}", file_name))?;
        }
        None => {
            let stdin = io::stdin();
            let _ = stdin.lock().read_to_string(&mut content)
                .chain_err(|| "Error reading from stdin")?;
        }
    };

    // Post content
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

    // Display the url where our content is located
    let key = resp["key"].to_string();
    let key = key.trim_matches('"');
    let raw = if matches.is_present("raw") { "raw" } else { "" };

    let mut p = PathBuf::from(read_root);
    p.push(raw);
    p.push(key);
    println!(" ** Success! Content available at: {}", p.display());
    Ok(())
}
