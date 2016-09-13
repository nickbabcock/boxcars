//! # boxcars (also written boxca-rs)
//!
//! boxcars is an example of a [Rocket League](http://www.rocketleaguegame.com/) replay parser
//! written in rust using [nom](https://github.com/Geal/nom) for parsing and
//! [serde](https://github.com/serde-rs/serde) for serialization. Emphasis on example, as this
//! library in no way competes with the other feature complete parsers such as
//! [Octane](https://github.com/tfausak/octane) and
//! [RocketLeagueReplayParser](https://github.com/jjbott/RocketLeagueReplayParser). Rather let the
//! project be a good example of Rust code using nom, and serde as extensive examples are hard to
//! come by. While lacking feature completeness and user friendly error message -- among other
//! issues, tests and documentation strive to be thorough.
#![feature(plugin, custom_derive)]
#![plugin(serde_macros)]

#[macro_use]
extern crate nom;
extern crate serde;

#[cfg(test)] extern crate serde_json;

pub use self::models::*;
pub use self::parsing::*;
mod models;
mod parsing;

