[![Build
Status](https://travis-ci.org/nickbabcock/boxcars.svg?branch=master)](https://travis-ci.org/nickbabcock/boxcars)

# Boxcars

Boxcars is a [Rocket League](http://www.rocketleaguegame.com/) replay parser written in Rust
with [serde](https://github.com/serde-rs/serde) support for serialization oftentimes into JSON.
The focus is correctness and performance with boxcar users able to dictate what sections of the
replay to parse. For instance, parsing just the header (where tidbits like goals and scores are
stored) completes in under 50 microseconds. However, if you want to check against replay
corruption and parse the network data, it would cost you 30 milliseconds (~1000x increase).
While a 1000x increase in time sounds significant; keep in mind, 30ms is phenomenal compared to
current state of the art Rocket League Replay parsers.

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

Since Boxcars allows you to pick and choose what to parse, below is a table with the following options and the estimated elapsed time.

| Header | Corruption Check | Body | Output JSON | Elapsed |
| -      | -                | -    | -           | -       |
| ✔      |                  |      |             | 45 µs   |
| ✔      | ✔                | ✔    |             | 30ms    |
| ✔      |                  | ✔    |             | 27ms    |
| ✔      | ✔                | ✔    | ✔           | 50ms    |
| ✔      |                  | ✔    | ✔           | 45ms    |

The most astounding number is that boxcars can parse 20,000 replays per second
per core. The best thing is that Boxcars will scale linearly as more cores are
dedicated to parsing replays in parallel.

# Special Thanks

Special thanks needs to be given to everyone in the Rocket League community who figured out the replay format and all its intricacies. Boxcars wouldn't exist if it weren't for them. I heavily leaned on implementations in [rattletrap](https://github.com/tfausak/rattletrap) and [RocketLeagueReplayParser](https://github.com/jjbott/RocketLeagueReplayParser). One of those should be your go to Rocket League Replay tool, unless you need speed, as those implementations are more mature than boxcars.
