[![Build
Status](https://travis-ci.org/nickbabcock/boxcars.svg?branch=master)](https://travis-ci.org/nickbabcock/boxcars)

# boxcars

boxcars is a [Rocket League](http://www.rocketleaguegame.com/) replay parser written in Rust
using [serde](https://github.com/serde-rs/serde) for serialization. Currently, this library in
no way competes with the other feature complete parsers such as
[Octane](https://github.com/tfausak/octane) and
[`RocketLeagueReplayParser`](https://github.com/jjbott/RocketLeagueReplayParser). Rather, let
boxcars be a good example of Rust code.

The code is well documented, give it a read!

Below is an example to output the replay structure to json:

```rust
extern crate boxcars;
extern crate serde_json;
extern crate failure;

use std::fs::File;
use std::io::{self, Read};

fn run() -> Result<(), ::failure::Error> {
    let filename = "assets/rumble.replay";
    let mut f = File::open(filename)?;
    let mut buffer = vec![];
    f.read_to_end(&mut buffer)?;
    let replay = boxcars::ParserBuilder::new(&buffer)
        .on_error_check_crc()
        .parse()?;

    serde_json::to_writer(&mut io::stdout(), &replay)?;
    Ok(())
}

```

# rrrocket

Rrrocket is what a cli program might look like utilizing the boxcars library.
Rrrocket parses a Rocket League replay file and outputs JSON. The executable
has been built for many platforms, so head on over to the [latest
release](https://github.com/nickbabcock/boxcars/releases/latest) and download
the appropriate bundle. If you're not sure which bundle to download, here are
the most likely options:

- For Windows, you'll want the one labeled `windows-msvc`
- For Linux, you'll want the one labeled `linux-musl`
- For macOS, you'll want the only one labeled `apple`

A sample output of the JSON from rrrocket:

```json
{
  "header_size": 4768,
  "header_crc": 337843175,
  "major_version": 868,
  "minor_version": 12,
  "game_type": "TAGame.Replay_Soccar_TA",
  "properties": {
    "TeamSize": 3,
    "Team0Score": 5,
    "Team1Score": 2,
    "Goals": [
      {
        "PlayerName": "Cakeboss",
        "PlayerTeam": 1,
        "frame": 441
      },
      // all the goals
    ]
    // and many more properties
  }
```

# boxcapy

boxcapy is a python script that ingests given JSON files that have been created
by `rrrocket`. The command below is the one I use to generate the JSON files:

```bash
find . -type f -iname "*.replay" | xargs -n1 -I{} bash -c '~/rrrocket {} > {}.json'
```

To have your graphs saved into your directory follow the below instructions:

- Since the graphs are in the style of XKCD, one has to install the Humor Sans font before continuing (eg. `apt install fonts-humor-sans`)

## Python 2

- Install [pipenv](https://docs.pipenv.org/install.html#installing-pipenv)
- Install dependencies `pipenv --two && pipenv install`
- Run on generated JSON files: `pipenv run boxcapy/rocket-plot.py ~/Demos/*.json --headless`

## Python 3

- Install [pipenv](https://docs.pipenv.org/install.html#installing-pipenv)
- Install dependencies `pipenv --three && pipenv install --skip-lock`
- Run on generated JSON files: `pipenv run boxcapy/rocket-plot.py ~/Demos/*.json --headless`

# Benchmarks

The benchmarks several things:

- How long to just parse the raw data
- How long to parse the raw data and ensure that the replay is not corrupt using a crc check
- How long to parse the data and output json data of the replay
- How long to parse the data with crc check and output json of the replay

The benchmark data is below. The most astounding number is that boxcars can
parse nearly 30,000 replays per second per core. An eight core machine would
process 240,000 replays a second. This benchmark represents data already in
memory and doesn't include time to read from disk / network.

```
bench_parsing_data                    ... bench:      33,627 ns/iter (+/- 2,329)
bench_parsing_data_crc_check          ... bench:   2,209,007 ns/iter (+/- 113,528)
bench_parse_and_json                  ... bench:      62,557 ns/iter (+/- 16,224)
bench_parse_and_json_crc_check        ... bench:   2,273,926 ns/iter (+/- 62,011)
```
