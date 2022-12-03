# v0.9.8 - December 3rd, 2022

- Support parsing patch v2.23 replays

# v0.9.7 - November 7th, 2022

- Improve dropshot compatibility

# v0.9.6 - October 15th, 2022

- Support parsing patch v2.21 replays

# v0.9.5 - August 31st, 2022

- 20% performance increase from bitter update from 0.5 to 0.6

# v0.9.4 - May 10th, 2022

- Support parsing voice chat related attributes (patch v2.15)

# v0.9.3 - November 28th, 2021

- Support parsing impulse attributes (patch v2.08)

# v0.9.2 - August 21th, 2021

- Additional support for recent rumble replays

# v0.9.1 - August 13th, 2021

- Support parsing rocket league v2.01 replays
- Derive `Hash` for `UniqueId` and `RemoteId` 

# v0.9.0 - July 30th, 2021

- `HeaderProp::Byte` now exposes parsed fields (breaking change for those who operate on this enum. This is in contrast to breaking changes in network attributes introduced in patch versions, as updating network attributes are needed to parse latest RL replays and should be considered routine).
- MSRV is now 1.46.0
- `phf` dependency updated to 0.9

# v0.8.10 - February 7th, 2021

- Support gridiron replays

# v0.8.9 - January 24th, 2021

No changes in behavior, bump `bitter` internal dependency to latest

# v0.8.8 - January 7th, 2021

* Allow missing trailers on replays instead of erroring as trailers are unused and not exposed

# v0.8.7 - December 24th, 2020

* Support additional LAN / RLCS replays

# v0.8.6 - December 8th, 2020

* Support replays that contain the rumble pickup attribute

# v0.8.5 - October 4th, 2020

* Support replays that contain the difficulty attribute

# v0.8.4 - October 1st, 2020

* Support for tournament replays:
  * `MaxTimeWarningData_TA` and crew added
  * `Engine.ReplicatedActor_ORS:ReplicatedOwner` attribute added

# v0.8.3 - September 27th, 2020

* Support latest rocket league patch (1.82) for Epic IDs

# v0.8.2 - June 17th, 2020

* Support latest rocket league patch (1.78) for demolish fx attributes

# v0.8.1 - April 17th, 2020

* Support latest rocket league patch (1.76) for heatseeker matches with godball.

# v0.8.0 - April 1st, 2020

A few breaking changes:

* Many data structures now use `i32` instead of `u32`, else `-1` values would be recorded as 2^32 - 1 (4294967295)
* APIs that represented actor ids are now wrapped in the `ActorId` type instead of the raw data type (eg: `i32`). A good example of this is the Demolish attribute that changed `attacker_actor_id: u32` to `attacker: ActorId`
* `Attribute::Loadout::unknown3` has been renamed to `product_id`
* `Attribute::Flagged(bool, u32)` has been moved to `Attribute::ActiveActor(ActiveActor)`

```rust
pub struct ActiveActor {
    pub active: bool,
    pub actor: ActorId,
}
```

* `Attribute::AppliedDamage` has moved to a dedicated type:

```rust
pub struct AppliedDamage {
    pub id: u8,
    pub position: Vector3f,
    pub damage_index: i32,
    pub total_damage: i32,
}
```

* `Attribute::DamageState` has moved to a dedicated type:

```rust
pub struct DamageState {
    /// State of the dropshot tile (0 - undamaged, 1 - damaged, 2 - destroyed)
    pub tile_state: u8,

    /// True if damaged
    pub damaged: bool,

    /// Player actor that inflicted the damage
    pub offender: ActorId,

    /// Position of the ball at the time of the damage
    pub ball_position: Vector3f,

    /// True for the dropshot tile that was hit by the ball (center tile of the damage area)
    pub direct_hit: bool,
    pub unknown1: bool,
}
```

* `Attribute::ExtendedExplosion` has moved to a dedicated type:

```rust
pub struct ExtendedExplosion {
    pub explosion: Explosion,
    pub unknown1: bool,
    pub secondary_actor: ActorId,
}
```

* `Attribute::StatEvent` has moved to a dedicated type:

```rust
pub struct StatEvent {
    pub unknown1: bool,
    pub object_id: i32,
}
```

# v0.7.2 - March 13, 2020

Add support for decoding new replays on the 1.74 patch via two new attributes:

* TAGame.PRI_TA:bIsDistracted (Attribute::Int)
* TAGame.GameEvent_Soccar_TA:MaxScore (Attribute::Boolean)

# v0.7.1 - March 5th, 2020

This is a performance release. Benchmarks improved by 40% when large network attributes were moved to a separate location on the heap, so that the size of the attribute data is the determined by the rigid body (the most common attribute). Below are the attributes that are now boxed:

- CamSettings
- Demolish
- Loadout
- TeamLoadout
- UniqueId
- Reservation
- PartyLeader
- PrivateMatch

While this is technically a breaking change to the API, I decided to release this as a patch release due to `Box` being easily dereferenced, so usage of these attributes shouldn't change drastically.

# v0.7.0 - February 21st, 2020

Couple of breaking changes with how rigid bodies (and other vector based network attributes) are represented.

## Decode Rigid Body Rotation Bits into Quaternions

While boxcars could parse all replays, it nonsensically stored the quaternion
information from rigid body attributes. v0.7.0 ensures that the quaternion
logic now matches rattletrap, bakkes, and jjbott's implementation.

The `x`, `y`, and `z` fields n a `RigidBody` have been replaced with a `rotation` field that is a quaternion:

```
Quaternion {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
}
```

## Replace Vector Bias with Computed Values

Previously boxcars would output the following vector data for rigid
bodies:

```
"location": {
  "bias": 262144,
  "dx": 457343,
  "dy": 15746,
  "dz": 263845
}
```

This is correct, but does not conform to the other parsers (bakkes and
jjbott) which look like:

```
"position": {
  "X": 1951.99,
  "Y": -2463.98,
  "Z": 17.01
},
```

The vector fields are computed as ((X - bias) / 100). Important, the `/ 100`
part of the formula is dropped for new actors. Thus there are two types
vectors: floating point and integer.

Having floating point and integer vectors necessitated splitting the
Vector class into two: Vector3f and Vector3i (one for integer and the
other for floating point -- side note: Vectorf and Vectori just didn't
look right).

# v0.6.2 - December 17th, 2019

* Fix potential integer overflow / underflow on malicious input
* Remove unused errors: NetworkError::{ChannelsTooLarge, MaxStreamIdTooLarge}

# v0.6.1 - November 18th, 2019

* Overhaul error handling for network frame decoding. Whenever an error occurs, a significant amount of context is captured (what was the last actor added / updated, all previously decoded frames, actor ids, names, etc). This should make it more apparent why a replay failed to decode, and while this technically is a breaking change in the error API -- it's unlikely anyone is relying on the bowels of boxcar's error structure.
* Support decoding replays that contain duplicate object ids
* Support decoding RLCS / Lan replays
* Support decoding replays with many actors

# v0.6.0 - November 7th, 2019

* Remove the `multimap` dependency
* Added convenience functions to extract properties from the header. Since properties in the header can be a multitude of types, one can extract the inside data if the type is known ahead of time. For instance, `MaxChannels` is a integer, so one can extract the integer with `HeaderProp::as_i32`

Before:

```rust
replay.properties
    .iter()
    .find(|&(key, _)| key == "MaxChannels")
    .and_then(|&(_, ref prop)| {
        if let HeaderProp::Int(v) = *prop {
            Some(v)
        } else {
            None
        }
    })
```

After:

```rust
self.properties
    .iter()
    .find(|&(key, _)| key == "MaxChannels")
    .and_then(|&(_, ref prop)| prop.as_i32())
```

* Added support for a additional attributes:
  * Archetypes.GameEvent.GameEvent_SoccarLan
  * Engine.Actor:bTearOff
  * Engine.Pawn:HealthMax
  * TAGame.Car_TA:ReplicatedCarScale
  * TAGame.GameEvent_Soccar_TA:bMatchEnded
  * TAGame.GameEvent_Soccar_TA:bNoContest
  * TAGame.GameEvent_Soccar_TA:GameWinner
  * TAGame.GameEvent_Soccar_TA:MatchWinner
  * TAGame.GameEvent_Soccar_TA:MVP
  * TAGame.GameEvent_TA:bAllowReadyUp
  * TAGame.PRI_TA:RepStatTitles
  * TAGame.Vehicle_TA:bPodiumMode

# v0.5.2 - October 20th, 2019

* Parse replays with QQ ids
* Parse replays from the tutorial
* Parse replays with the anniversary ball
* Parse replays that contain a ps4 platform id in the header

# v0.5.1 - October 18th, 2019

* Update network parser to be compatible with v1.68 rocket league replays
* Update internal phf dependency from 0.7 to 0.8

# v0.5.0 - September 8th, 2019

* Checking against corrupt replays using a new crc algorithm improved crc performance by 8x. This translates to a potential 20% overall performance improvement when the crc is needed / requested to be calculated when decoding a replay
* The `Replay` structure now owns all parsing data, there is no more lifetime parameter that is tied to the original raw data slice. For instance, `replay.game_type` is now a `String`, not a `Cow<'a, str>`. The impetus of this change stems from the difficulty of making a replay long lived when it's tied to the lifetime of another object. See [issue 61 for more info](https://github.com/nickbabcock/boxcars/issues/61). The result of this change should be an API that is more ergonomic. While performance decreased for those only interested in parsing the header, there was an overall performance win by moving to the new API.

# v0.4.1 - September 3rd, 2019

* Expose error types as part of public API ([#54](https://github.com/nickbabcock/boxcars/pull/54)).
* Include attribute object id on actor update, so now one can more easily derive the attribute's name with `replay.objects[attribute.object_id]` ([#56](https://github.com/nickbabcock/boxcars/pull/56))

# v0.4.0 - September 2nd, 2019

* Update network parser to be compatible with v1.66 rocket league replays
* Update to multimap 0.6
* `Boxcars::ParserBuilder::parse` no longer returns a `failure::Error` as the failure dependency has been removed. Instead a `ParseError` is returned. The same error enums are available as well as the same string representation.

# v0.3.5 - August 12th, 2019

- Support for haunted and rugby games.
- Improvement to error handling that gives detailed error messages on what new / updated actor may have received changes in the RL update. These error messages should only be helpful debugging new updates.
- Several security fixes:
  - Malicious user could craft NumFrames property to be obscenely high and run the machine out of memory. An error is now thrown if the requested number of frames is greater than the number of bytes remaining.
  - A class's network cache that referenced an out of range object id would cause a index out of bound panic. Now an error is raised.
  - Other fixes are for panics in debug builds

# v0.3.4 - June 5th, 2019

* Update network parser to be compatible with v1.63 rocket league replays

# v0.3.3 - June 1st, 2019

- Update crc content from signed 32bits to unsigned 32bits as a negative checksum can be misleading.
- Additional decoding for PsyNet, Switch, and Ps4 remote ids. Instead of just a vector of opaque bytes, now the values contain a structure with additional fields like (`online_id` or `name`). Any leftover data is still captured as opaque bytes.

# v0.3.2 - May 24th 2019

- Update multimap requirement from 0.4 to 0.5
- Bugfix for newer replays with reservations involving psynet players

# v0.3.1 - May 23rd 2019

- Fix compilation edge case
- Update if_chain requirement from 0.1 to 1.0

# v0.3.0 - May 2nd 2019

* Minor version bump as the network API grew significantly. A lot of the network attributes were publicly opaque, so while one could access all the members (and write them out as JSON for instance) there was no way to access individual fields on these attributes (like RigidBody::sleeping was inaccessible). Hiding these fields was an oversight and has been fixed.
* Update encoding_rs from 0.7 to 0.8 (no discernible changes should be expected)

# v0.2.8 - April 25th 2019

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
