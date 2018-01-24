//! # boxcars
//!
//! boxcars is a [Rocket League](http://www.rocketleaguegame.com/) replay parser written in Rust
//! with [serde](https://github.com/serde-rs/serde) support for serialization.
//!
//! ```
//! extern crate boxcars;
//! extern crate serde_json;
//! extern crate failure;
//!
//! use std::fs::File;
//! use std::io::{self, Read};
//! # fn main() {
//! #    let filename = "assets/rumble.replay";
//! #    run(filename).unwrap();
//! # }
//!
//! fn run(filename: &str) -> Result<(), ::failure::Error> {
//!     let mut f = File::open(filename)?;
//!     let mut buffer = vec![];
//!     f.read_to_end(&mut buffer)?;
//!     let replay = boxcars::ParserBuilder::new(&buffer)
//!         .on_error_check_crc()
//!         .parse()?;
//!
//!     serde_json::to_writer(&mut io::stdout(), &replay)?;
//!     Ok(())
//! }
//! ```

#![recursion_limit = "1000"]

extern crate bitter;
extern crate byteorder;
extern crate encoding_rs;
#[macro_use]
extern crate failure;
extern crate fnv;
#[macro_use]
extern crate if_chain;
extern crate phf;
extern crate serde;

#[cfg(test)]
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

pub use self::models::*;
pub use self::network::{Frame, NewActor, Rotation, Trajectory, UpdatedAttribute, Vector};
pub use self::parsing::{CrcCheck, NetworkParse, ParserBuilder};
mod network;
mod parsing;
mod models;
pub mod crc;
mod errors;
mod attributes;

#[cfg_attr(feature = "cargo-clippy", allow(clippy))]
mod hashes {
    include!(concat!(env!("OUT_DIR"), "/generated.rs"));
}
