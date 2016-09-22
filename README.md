[![Build
Status](https://travis-ci.org/nickbabcock/boxcars.svg?branch=master)](https://travis-ci.org/nickbabcock/boxcars)

# boxcars

[Boxcars](https://github.com/nickbabcock/boxcars), also stylized as boxca-rs,
is an example of a [Rocket League](http://www.rocketleaguegame.com/) replay
parser written in [Rust](https://www.rust-lang.org/en-US/) using
[nom](https://github.com/Geal/nom) for parsing and
[serde](https://github.com/serde-rs/serde) for serialization. As stated in the
title, this is an example, as this library in no way competes with the other
feature complete parsers such as [Octane](https://github.com/tfausak/octane)
and
[RocketLeagueReplayParser](https://github.com/jjbott/RocketLeagueReplayParser).
Rather, let boxcars be a good example of Rust code using nom, and serde as
extensive examples are hard to come by. While lacking feature completeness and
user friendly error message -- [among other
issues](https://github.com/nickbabcock/boxcars/issues), tests and documentation
strive to be thorough.

The code is well documented, give it a read!

# Benchmarks

The benchmarks several things:

- How long to just parse the raw data
- How long to parse the raw data and ensure that the replay is not corrupt using a crc check
- How long to parse the data and output json data of the replay
- How long to parse the data with crc check and output json of the replay

The benchmark data is below. The most astounding number is that boxcars can
parse nearly 15,000 replays per second per core. An eight core machine would
process 120,000 replays a second. This is, without a doubt, an optimistic
number, but still mightily impressive.

```
bench_parsing_data                    ... bench:      72,353 ns/iter (+/- 2,329)
bench_parsing_data_crc_check          ... bench:   2,209,007 ns/iter (+/- 113,528)
bench_parse_and_json                  ... bench:     138,937 ns/iter (+/- 16,224)
bench_parse_and_json_crc_check        ... bench:   2,273,926 ns/iter (+/- 62,011)
```
