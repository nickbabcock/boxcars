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
extern crate rayon;

#[macro_use]
extern crate error_chain;

mod errors {
    use boxcars;
    use serde_json;
    error_chain! {
        foreign_links {
            Io(::std::io::Error);
            Serde(serde_json::Error);
        }

        links {
            Boxcar(boxcars::Error, boxcars::ErrorKind);
        }
    }
}

use errors::*;
use structopt::StructOpt;
use std::fs::File;
use std::io::{Read, Write, BufWriter};
use error_chain::ChainedError;
use rayon::prelude::*;

#[derive(StructOpt, Debug, Clone, PartialEq)]
#[structopt(name = "rrrocket", about = "Parses Rocket League replay files and outputs a .json file with the decoded information")]
struct Opt {
    #[structopt(short = "c", long = "crc-check", help = "validate replay is not corrupt", default_value = "true")]
    crc: bool,

    #[structopt(help = "Rocket League replay files")]
    input: Vec<String>,
}

fn parse_file(input: &str, crc: bool) -> Result<boxcars::Replay> {
    let mut f = File::open(input)?;
    let mut buffer = vec![];
    f.read_to_end(&mut buffer)?;
	Ok(boxcars::parse(&buffer, crc)?)
}

fn run() -> Result<()> {
    let opt = Opt::from_args();
    let compute_crc = opt.crc;
    println!("{}", compute_crc);
    let res: Result<Vec<()>> = opt.input.par_iter()
        .map(|file| {
            let outfile = format!("{}.json", file);
            let data = parse_file(file, compute_crc)?;
            let mut out_file = BufWriter::new(File::open(outfile)?);
            serde_json::to_writer(&mut out_file, &data)?;
            Ok(())
        }).collect();
    res?;
    Ok(())
}

fn main() {
    if let Err(ref e) = run() {
        writeln!(::std::io::stderr(), "{}", e.display_chain())
			.expect("Error writing to stderr");
        ::std::process::exit(1);
    }
}
