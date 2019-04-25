# v0.2.8 - APril 25th 2019

* Serialize 64bit numbers as strings, so that JSON parsers don't lose any data
  in parsing them as 64bit floating point
  * Javascript numbers are 64bit floating point. 64bit integers can't be
    represented wholly in floating point notation. Thus serialize them as
    strings so that downstream applications can decide on how best to interpret
    large numbers (like 76561198122624102). Affects Int64, QWord, Steam, and
    XBox attributes.
* QWord header property changes from i64 to u64 as some pointed out that
  negative numbers didn't make sense for QWord properties (OnlineId)

# v0.2.7 - April 22nd 2019

* Update network parser to be compatible with v1.61 rocket league replays

# v0.2.6 - April 4th 2019

* Update network parser to be compatible with v1.59 rocket league replays

# v0.2.5 - September 6th 2018

* Update network parser to be compatible with v1.50 rocket league replays

# v0.2.4 - May 30th, 2018

* Update network parser to be compatible with v1.45 rocket league replays

# v0.2.3 - April 25th, 2018

* Update network parser to be compatible with latest rocket league replays
* Improve throughput of network parsing by up to 10%
* Additional detailed error messages

# v0.2.2 - March 18th, 2018

* Update network parser to the latest rocket league replays

# v0.2.1 - February 14th, 2018

* Fixed several bugs surrounding parsing of the network data. More replays are now parseable

# v0.2.0 - January 31st, 2018

Initial release of the boxcars Rust library. v0.1.0 was never released on crates.io, but was used transitively with v0.1.0 of rrrocket (hence the initial version being v0.2.0 instead of v0.1.0)
