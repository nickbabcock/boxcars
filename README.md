# Boxcars

[![Build Status](https://travis-ci.org/nickbabcock/boxcars.svg?branch=master)](https://travis-ci.org/nickbabcock/boxcars) [![Build status](https://ci.appveyor.com/api/projects/status/v0l0okwfqa5vg13v?svg=true)](https://ci.appveyor.com/project/nickbabcock/boxcars) [![](https://docs.rs/boxcars/badge.svg)](https://docs.rs/boxcars) [![Version](https://img.shields.io/crates/v/boxcars.svg?style=flat-square)](https://crates.io/crates/boxcars)

*Looking for rrrocket (the commandline app that parses replays and outputs JSON for analysis)? It [recently moved](https://github.com/nickbabcock/rrrocket)*

Boxcars is a [Rocket League](http://www.rocketleaguegame.com/) replay parser
library written in Rust, designed to be fast and safe: 10-100x faster than
established parsers. Boxcars is extensively fuzzed to ensure potentially
malicious user input is handled gracefully.

A key feature of boxcars is the ability to dictate what sections of the replay
to parse. A replay is broken up into two main parts: the header (where tidbits
like goals and scores are stored) and the network body, which contains
positional, rotational, and speed data (among other attributes). Since the
network data fluctuates between Rocket League patches and accounts for 99.8% of
the parsing time, one can tell boxcars to skip the network data or ignore
errors from the network data.

- By skipping network data one can parse and aggregate thousands of replays in
  under a second to provide an immediate response to the user. Then a full
  parsing of the replay data can provide additional insights when given time.
- By ignoring network data errors, boxcars can still provide details about
  newly patched replays based on the header.

Boxcars will also check for replay corruption on error, but this can be
configured to always check for corruption or never check.

Serialization support is provided through [serde](https://github.com/serde-rs/serde).

Below is an example to output the replay structure to json:

```rust
extern crate boxcars;
extern crate serde_json;
extern crate failure;

use std::fs::File;
use std::io::{self, Read};

fn run() -> Result<(), ::failure::Error> {
    let filename = "assets/replays/good/rumble.replay";
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

## Benchmarks

Since Boxcars allows you to pick and choose what to parse, below is a table
with the following options and the estimated elapsed time.

| Header | Corruption Check | Body | Output JSON | Elapsed |
| -      | -                | -    | -           | -       |
| ✔      |                  |      |             | 40 µs   |
| ✔      | ✔                | ✔    |             | 24ms    |
| ✔      |                  | ✔    |             | 20ms    |
| ✔      | ✔                | ✔    | ✔           | 38ms    |
| ✔      |                  | ✔    | ✔           | 35ms    |

## Special Thanks

Special thanks needs to be given to everyone in the Rocket League community who figured out the replay format and all its intricacies. Boxcars wouldn't exist if it weren't for them. I heavily leaned on implementations in [rattletrap](https://github.com/tfausak/rattletrap) and [RocketLeagueReplayParser](https://github.com/jjbott/RocketLeagueReplayParser). One of those should be your go to Rocket League Replay tool, unless you need speed, as those implementations are more mature than boxcars.

## Difference between rattletrap and boxcars

- Rattletrap can binary that ingests rocket league replays and outputs, while boxcars is a lower level parsing library for rocket league. Boxcars underpins Rrrocket, a cli binary that outputs JSON similar to Rattletrap
- Rattletrap can roundtrip replays (convert them into JSON and then write them out back to a replay losslessly). Boxcars is focussed on parsing replays.
- In part due to allowing roundtrip parsing, rattletrap JSON output is 2x larger than boxcars (rrrocket) even when accounting for output minification.

Below are some differences in the model:

rattletrap:

```json
"properties": {
  "value": {
    "BuildID": {
      "kind": "IntProperty",
      "size": "4",
      "value": {
        "int": 1401925076
      }
    },
  }
}
```

boxcars:

```json
"properties": {
  "BuildID": 1401925076
}
```

---

rattletrap:

```json
"actor_id": {
  "limit": 2047,
  "value": 1
},
```

boxcars:

```json
"actor_id": 1
```

---

rattletrap:

```json
"value": {
  "spawned": {
    "class_name": "TAGame.GameEvent_Soccar_TA",
    "flag": true,
    "initialization": {
      "location": {
        "bias": 2,
        "size": {
          "limit": 21,
          "value": 0
        },
        "x": 0,
        "y": 0,
        "z": 0
      }
    },
    "name": "GRI_TA_1",
    "name_index": 0,
    "object_id": 85,
    "object_name": "Archetypes.GameEvent.GameEvent_Soccar"
  }
}
```

boxcars:

```json
"actor_id": 1,
"name_id": 1,
"object_id": 85,
"initial_trajectory": {
  "location": {
    "bias": 2,
    "dx": 2,
    "dy": 2,
    "dz": 2
  },
  "rotation": null
}
```

While rattletrap provides convenience conversions, boxcars omit them in favor of a more raw view of the replay:

- to derive `object_name`: `replay.objects[x.object_id]`
- to derive `name`: `replay.names[x.name_id]`

The raw formula for calculating x,y,z from dx, dy, dz, and bias is:

```
x = dx - bias
y = dy - bias
z = dz - bias
```

So:

```json
"location": {
  "bias": 4096,
  "dx": 6048,
  "dy": 1632,
  "dz": 4113
},
```

Would translate into

```
x: 1952
y: -2464
z: 17
```
