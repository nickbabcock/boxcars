//! # boxcars (also written boxca-rs)
//!
//! boxcars is an example of a [Rocket League](http://www.rocketleaguegame.com/) replay parser
//! written in Rust using [nom](https://github.com/Geal/nom) for parsing and
//! [serde](https://github.com/serde-rs/serde) for serialization. Emphasis on example, as this
//! library in no way competes with the other feature complete parsers such as
//! [Octane](https://github.com/tfausak/octane) and
//! [`RocketLeagueReplayParser`](https://github.com/jjbott/RocketLeagueReplayParser). Rather, let
//! boxcars be a good example of Rust code using nom, and serde as extensive examples are hard to
//! come by. While lacking feature completeness and user friendly error message -- [among other
//! issues](https://github.com/nickbabcock/boxcars/issues), tests and documentation strive to be
//! thorough.
//!
//! ```
//! extern crate boxcars;
//! extern crate nom;
//! extern crate serde_json;
//!
//! use std::fs::File;
//! use std::io::Read;
//!
//! # let filename = "assets/rumble.replay";
//! let mut f = File::open(filename).unwrap();
//! let mut buffer = vec![];
//! f.read_to_end(&mut buffer).unwrap();
//! let b = boxcars::parse(&buffer);
//!
//! match b {
//!     nom::IResult::Done(_, val) => {
//!         let serialized = serde_json::to_string(&val).unwrap();
//!         println!("{}", serialized);
//!     }
//!     _ => {
//!         println!("Oh no we failed to parse");
//!     }
//! }
//! ```

#![cfg_attr(feature = "nightly", feature(test))]
#![cfg_attr(feature = "serde_macros", feature(plugin, custom_derive))]
#![cfg_attr(feature = "serde_macros", plugin(serde_macros))]

#[macro_use]
extern crate nom;
extern crate serde;
extern crate encoding;

#[cfg(feature = "nightly")]
extern crate test;

#[cfg(test)]
extern crate serde_json;

mod models {
    #[cfg(feature = "serde_macros")]
    include!("models.in.rs");

    #[cfg(feature = "serde_codegen")]
    include!(concat!(env!("OUT_DIR"), "/models.rs"));
}

pub use self::models::*;
pub use self::parsing::*;
mod parsing;
mod crc;

#[cfg(test)]
mod tests {
    #[cfg(feature = "nightly")]
    use test::Bencher;
    use super::*;
    use nom;
    use serde_json;

    #[cfg(feature = "nightly")]
    #[bench]
    fn bench_parse_and_json(b: &mut Bencher) {
        let data = include_bytes!("../assets/rumble.replay");
        b.iter(|| {
            match parse(data) {
                nom::IResult::Done(_, val) => {
                    serde_json::to_string(&val).unwrap();
                }
                _ => assert!(false)
            }
        });
    }
}
