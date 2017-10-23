//! # CLI
//!
//! The command line version of the library. Given a file path argument, it will ingest all the
//! data into memory, and attempt parsing. If the parsing is successful, JSON is outputted to
//! stdout, else a non-helpful error message is printed. Sorry!
#[macro_use]
extern crate structopt_derive;
extern crate structopt;
extern crate boxcars;
extern crate serde_json;

use structopt::StructOpt;
use std::fs::File;
use std::io::{self, Read};

#[derive(StructOpt, Debug, Clone, PartialEq)]
#[structopt(name = "rrrocket", about = "Parses a Rocket League replay file and outputs JSON")]
struct Opt {
    #[structopt(short = "c", long = "crc-check", help = "validate replay is not corrupt", default_value = "true")]
    crc: bool,

    #[structopt(help = "Rocket League replay file")]
    input: String,
}

fn parse_file(input: &String, crc: bool) -> boxcars::Replay {
    let mut f = File::open(input).unwrap();
    let mut buffer = vec![];
    f.read_to_end(&mut buffer).unwrap();
	boxcars::parse(&buffer, crc).unwrap()
}

fn main() {
    let opt = Opt::from_args();
    let data = parse_file(&opt.input, opt.crc);
	serde_json::to_writer(&mut io::stdout(), &data).unwrap();
}
