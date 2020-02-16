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
//! use boxcars::{ParseError, Replay};
//! use std::error;
//! use std::fs;
//! use std::io::{self, Read};
//!
//! fn parse_rl(data: &[u8]) -> Result<Replay, ParseError> {
//!     boxcars::ParserBuilder::new(data)
//!         .on_error_check_crc()
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

#![recursion_limit = "1000"]

#[macro_use]
extern crate if_chain;

#[macro_use]
extern crate serde;

pub use self::errors::{AttributeError, FrameContext, FrameError, NetworkError, ParseError};
pub use self::models::*;
pub use self::network::attributes::Attribute;
pub use self::network::*;
pub use self::parser::{CrcCheck, NetworkParse, ParserBuilder};
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

#[cfg(feature = "py")]
mod py;
