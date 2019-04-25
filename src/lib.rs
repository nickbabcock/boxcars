//! # Boxcars
//!
//! Boxcars is a [Rocket League](http://www.rocketleaguegame.com/) replay parser
//! library written in Rust, designed to be fast and safe: 10-100x faster than
//! established parsers. Boxcars is extensively fuzzed to ensure potentially
//! malicious user input is handled gracefully.
//!
//! A key feature of boxcars is the ability to dictate what sections of the replay
//! to parse. A replay is broken up into two main parts: the header (where tidbits
//! like goals and scores are stored) and the network body, which contains
//! positional, rotational, and speed data (among other attributes). Since the
//! network data fluctuates between Rocket League patches and accounts for 99.8% of
//! the parsing time, one can tell boxcars to skip the network data or ignore
//! errors from the network data.
//!
//! - By skipping network data one can parse and aggregate thousands of replays in
//!   under a second to provide an immediate response to the user. Then a full
//!   parsing of the replay data can provide additional insights when given time.
//! - By ignoring network data errors, boxcars can still provide details about
//!   newly patched replays based on the header.
//!
//! Boxcars will also check for replay corruption on error, but this can be
//! configured to always check for corruption or never check.
//!
//! Serialization support is provided through [serde](https://github.com/serde-rs/serde).
//!
//! Below is an example to output the replay structure to json:
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
extern crate multimap;
extern crate phf;
#[macro_use]
extern crate serde;

#[cfg(test)]
extern crate serde_json;

pub use self::models::*;
pub use self::network::{Frame, NewActor, Rotation, Trajectory, UpdatedAttribute, Vector};
pub use self::parsing::{CrcCheck, NetworkParse, ParserBuilder};
mod attributes;
pub mod crc;
mod errors;
mod models;
mod network;
mod parsing;
mod serde_utils;

#[cfg_attr(feature = "cargo-clippy", allow(clippy::all))]
mod hashes {
    include!(concat!(env!("OUT_DIR"), "/generated.rs"));
}
