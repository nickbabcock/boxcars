//! # CLI
//!
//! The command line version of the library. Given a file path argument, it will ingest all the
//! data into memory, and attempt parsing. If the parsing is successful, JSON is outputted to
//! stdout, else a non-helpful error message is printed. Sorry!
extern crate boxcars;
extern crate serde_json;

use std::env;
use std::fs::File;
use std::io::{self, Read};

fn main() {
    let mut args: Vec<_> = env::args().collect();
    let filename = args.remove(1);
    let mut f = File::open(filename).unwrap();
    let mut buffer = vec![];
    f.read_to_end(&mut buffer).unwrap();
    let b = boxcars::parse(&buffer, true);
    match b {
        Ok(val) => {
            serde_json::to_writer(&mut io::stdout(), &val).unwrap();
        }
        _ => {
            println!("Oh no we failed to parse");
        }
    }
}
