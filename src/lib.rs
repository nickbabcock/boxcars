//! # boxcars (also written boxca-rs)
//!
//! boxcars is a [Rocket League](http://www.rocketleaguegame.com/) replay parser written in Rust
//! using [serde](https://github.com/serde-rs/serde) for serialization. Currently, this library in
//! no way competes with the other feature complete parsers such as
//! [Octane](https://github.com/tfausak/octane) and
//! [`RocketLeagueReplayParser`](https://github.com/jjbott/RocketLeagueReplayParser). Rather, let
//! boxcars be a good example of Rust code.
//!
//! ```
//! extern crate boxcars;
//! extern crate serde_json;
//! extern crate failure;
//!
//! use std::fs::File;
//! use std::io::{self, Read};
//!
//! # fn main() {
//!     run().unwrap();
//! # }
//!
//! # fn run() -> Result<(), ::failure::Error> {
//! # let filename = "assets/rumble.replay";
//! let mut f = File::open(filename)?;
//! let mut buffer = vec![];
//! f.read_to_end(&mut buffer)?;
//! let replay = boxcars::ParserBuilder::new(&buffer)
//!     .on_error_check_crc()
//!     .parse()?;
//!
//! serde_json::to_writer(&mut io::stdout(), &replay)?;
//! Ok(())
//! # }
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
pub use self::network::{Frame, UpdatedAttribute, NewActor, Trajectory, Rotation, Vector};
pub use self::parsing::{ParserBuilder, CrcCheck, NetworkParse};
mod network;
mod parsing;
mod models;
mod crc;
mod errors;
mod attributes;

mod hashes {
  include!(concat!(env!("OUT_DIR"), "/generated.rs"));
}
