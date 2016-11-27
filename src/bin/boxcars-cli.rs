//! # CLI
//!
//! The command line version of the library. Given a file path argument, it will ingest all the
//! data into memory, and attempt parsing. If the parsing is successful, JSON is outputted to
//! stdout, else a non-helpful error message is printed. Sorry!
#![cfg_attr(feature = "afl-feat", feature(plugin))]
#![cfg_attr(feature = "afl-feat", plugin(afl_plugin))]
#![cfg(feature = "afl-feat")]
extern crate afl;
extern crate boxcars;
extern crate nom;
extern crate serde_json;

use std::env;
use std::fs::File;
use nom::IResult;
use std::io::Read;

#[cfg(not(feature = "afl-feat"))]
fn main() {
    let mut args: Vec<_> = env::args().collect();
    let filename = args.remove(1);
    let mut f = File::open(filename).unwrap();
    let mut buffer = vec![];
    f.read_to_end(&mut buffer).unwrap();
    let b = boxcars::parse(&buffer, true);
    match b {
        IResult::Done(_, val) => {
            let serialized = serde_json::to_string(&val).unwrap();
            println!("{}", serialized);
        }
        _ => {
            println!("Oh no we failed to parse");
        }
    }
}

#[cfg(feature = "afl-feat")]
fn main() {
    afl::handle_bytes(|s| {
        let _ = boxcars::parse(&s, true);
    });
}
