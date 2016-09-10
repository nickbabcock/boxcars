extern crate boxcars;
extern crate nom;

use std::env;
use std::fs::File;
use nom::IResult;
use std::io::Read;

fn main() {
    let mut args: Vec<_> = env::args().collect();
    let filename = args.remove(1);
    let mut f = File::open(filename).unwrap();
    let mut buffer = vec!();
    f.read_to_end(&mut buffer).unwrap();
    let b = boxcars::parse(&buffer);
    match b {
      IResult::Done(_, val) => {
        println!("header size: {}", val.header_size);
      }
      _ => {
        println!("Oh no we failed to parse");
      }
    }
}
