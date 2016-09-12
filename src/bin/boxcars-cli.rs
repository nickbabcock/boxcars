extern crate boxcars;
extern crate nom;
extern crate serde_json;

use std::env;
use std::fs::File;
use nom::IResult;
use std::io::Read;

fn main() {
    let mut args: Vec<_> = env::args().collect();
    let filename = args.remove(1);
    let mut f = File::open(filename).unwrap();
    let mut buffer = vec![];
    f.read_to_end(&mut buffer).unwrap();
    let b = boxcars::parse(&buffer);
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
