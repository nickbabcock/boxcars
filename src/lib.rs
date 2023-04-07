//! # Boxcars
//!
//! Boxcars is a [Rocket League](http://www.rocketleaguegame.com/) replay parser library written in
//! Rust.
//!
//! ## Features
//!
//! - ✔ Safe: Stable Rust with no unsafe
//! - ✔ Fast: Parse a hundred replays per second per CPU core
//! - ✔ Fuzzed: Extensively fuzzed against potential malicious input
//! - ✔ Ergonomic: Serialization support is provided through [serde](https://github.com/serde-rs/serde)
//!
//! See where Boxcars in used:
//!
//! - Inside the [rrrocket CLI app](https://github.com/nickbabcock/rrrocket) to turn Rocket League Replays into JSON
//! - Compiled to [WebAssembly and embedded in a web page](https://rl.nickb.dev/)
//! - Underpins the python analyzer of the [popular calculated.gg site](https://calculated.gg/)
//!
//! ## Quick Start
//!
//! Below is an example to output the replay structure to json:
//!
//! ```rust
//! use boxcars::{ParseError, Replay};
//! use std::error;
//! use std::fs;
//! use std::io::{self, Read};
//!
//! fn parse_rl(data: &[u8]) -> Result<Replay, ParseError> {
//!     boxcars::ParserBuilder::new(data)
//!         .must_parse_network_data()
//!         .parse()
//! }
//!
//! fn run(filename: &str) -> Result<(), Box<dyn error::Error>> {
//!     let filename = "assets/replays/good/rumble.replay";
//!     let buffer = fs::read(filename)?;
//!     let replay = parse_rl(&buffer)?;
//!     serde_json::to_writer(&mut io::stdout(), &replay)?;
//!     Ok(())
//! }
//!
//! # let filename = "assets/replays/good/rumble.replay";
//! # run(filename).unwrap();
//! ```
//!
//! The above example will parse both the header and network data of a replay file, and return an
//! error if there is an issue either header or network data.  Since the network data will often
//! change with each Rocket League patch, the default behavior is to ignore any errors from the
//! network data and still be able to return header information.
//!
//! ## Variations
//!
//! If you're only interested the header (where tidbits like goals and scores are stored) then you
//! can achieve an 1000x speedup by directing boxcars to only parse the header.
//!
//! - By skipping network data one can parse and aggregate thousands of replays in
//!   under a second to provide an immediate response to the user. Then a full
//!   parsing of the replay data can provide additional insights when given time.
//! - By ignoring network data errors, boxcars can still provide details about
//!   newly patched replays based on the header.
//!
//! Boxcars will also check for replay corruption on error, but this can be configured to always
//! check for corruption or never check.

#[macro_use]
extern crate serde;

#[macro_use]
mod macros;
pub use self::errors::{AttributeError, FrameContext, FrameError, NetworkError, ParseError};
pub use self::models::*;
pub use self::network::attributes::*;
pub use self::network::*;
pub use self::parser::{CrcCheck, NetworkParse, ParserBuilder};
mod bits;
mod core_parser;
pub mod crc;
mod data;
mod errors;
mod header;
mod models;
mod network;
mod parser;
mod parsing_utils;
mod serde_utils;
