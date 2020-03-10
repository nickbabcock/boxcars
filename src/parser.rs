//! # Parsing
//!
//! A Rocket League replay is a binary file that is little endian encoded. What follows below is a
//! detailed account of the Rocket League replay format.
//!
//! A replay is split into three major sections, a header, body, and footer.
//!
//! ## Header
//!
//! - First 32 bits: the number of bytes that comprises the header data
//! - Second 32 bits: the [cyclic redundancy check
//! (CRC)](https://en.wikipedia.org/wiki/Cyclic_redundancy_check) (the checksum to ensure the
//! replay isn't corrupt). It should be unsigned.
//! - Now we arrive at the header data
//! - Third 32 bits: the replay major version (it'll be something like 868)
//! - Fourth 32 bits: the replay minor version (it'll be something like 20)
//! - Fifth 32 bits:  the replay network version (very old replays won't have this, you'll need
//! to check that the major _version > 865 and minor_version > 17.
//!
//! Now we get to where the game type is encoded as a string. Below is the formula for decoding a
//! string.
//!
//! - The size of text as a 32bit integer
//!   - If the size is positive, we're dealing with a windows-1252 encoding, so we don't need to do
//!   anything to get the number of bytes that the string consumes (as windows-1252 is a 8bit
//!   encoding).
//!   - If the size is negative, the string is encoded with UTF-16, so multiply it by -2 to get the
//!   number of bytes needed to read the string.
//! - Consume the number of bytes determined, but drop the last letter (1 byte for windows-1252, 2
//! for UTF-16) as this will be a null character which we don't want.
//!
//! ### Header Properties
//!
//! The properties is where all the good nuggets of info reside (goals, player stats, etc).
//! Visualize the properties as a map of key-value pairs. The format is to:
//!
//!  - Read string. This will be the key
//!  - If the key is "None" we're done with given key value pair
//!  - Read string to determine the value type
//!  - Skip next 8 bytes (there's debate about what this means, but it doesn't matter)
//!  - Decode the value based on the value type:
//!    - "BoolProperty": read byte. Does it equal 1?
//!    - "ByteProperty": read two strings (unless the first seen is related to steam / ps4)
//!    - "FloatProperty": little endian encoded 32bit float
//!    - "IntProperty": 32bit signed integer
//!    - "NameProperty": read string
//!    - "StrProperty": read string
//!    - "QWordProperty": read 64bit signed integer
//!    - "ArrayProperty": A nested map of key-value pairs (recurse into step 1).
//!  - Prepare to read the next property, go to step 1.
//!
//! ## Body
//!
//! Next comes the bulk of the replay, the body. Just like the header, the body starts out with a
//! pair of 32 bit integers representing the number of bytes and the CRC of the section.
//!
//! The first data point in the body is a list of levels encoded as a list of strings. A list in a
//! replay is prefixed by the number of elements contained in the list as a 32 bit integer. Use
//! this integer and decode said number of elements.
//!
//! Then it's a list of keyframes where a keyframe is 12 bytes (time: 32 bit float, frame: 32 bit
//! integer, and position: 32 bit integer).
//!
//! Network data is next. Since the network data complex enough to warrant it's own section later.
//! Thankfully it is prefixed with a 32 bit integer denoting its size, so we can skip it with ease.
//!
//! ## Footer
//!
//! Since the network data is about 95% of the data, everything that comes after it is the footer
//! (it's not technically a dedicated section with a length prefix + crc, but I like to think of it
//! as a separate section).
//!
//! Unless parsing the network data, there isn't too much that is interesting in the footer.
//!
//! - List of debug info: (frame: 32 bit integer, user: string, text: string)
//! - List of tickmarks: (description: string, frame: 32 bit integer)
//! - Packages: a string list (just like the list of levels we saw earlier)
//! - Objects: a string list
//! - Names: a string list
//! - List of class indices: (class: string, index: 32 bit integer)
//! - List of network attribute encodings: (object_ind: 32 bit integer, parent_id: 32 bit integer,
//! cache_id: 32 bit integer, properties: a list of 32bit pairs (object_ind and stream_id))
//!
//! ## Network Body
//!
//! The number of frames in the body is hinted by the "NumFrames" property in the header.
//!
//! Do note that the network data is bit level. Meaning that a parser will read bit by bit. Thus a
//! request for 32bits of data may span 5 bytes if the parser was in the middle of a byte.
//!
//! Each frame is led by a pair of 32bit floating point numbers representing the absolute time of a
//! frame and the delta, or elapsed time, from the previous frame.
//!
//! Next comes actor data.
//!
//! - While there is more actor data in the frame (bit is on)
//!   - Decode the actor id, which is the number of bits needed to represent the "NumChannels" value in the header
//!   - If the actor is alive (bit is on)
//!     - If actor is new (bit is on)
//!       - Parse new actor:
//!         - 32bit integer representing the index in the `names` list
//!         - Unused bit
//!         - 32bit integer representing the ObjectId
//!         - Decode initial position if available (basically is the object not part of the crowd): Vector3i
//!         - Decode initial rotation if available (basically is the object a ball or car): Rotation
//!
//! Quick aside:
//!
//! Vector3i:
//!
//!   - size in bits: read five bits not exceeding 22
//!   - bias: 2^(size in bits + 1)
//!   - bit_limit: size in bits + 2
//!   - x: (read bits (bit_limit) - bias)
//!   - y: (read bits (bit_limit) - bias)
//!   - z: (read bits (bit_limit) - bias)
//!
//! Rotation:
//!
//!   - yaw: signed 8 bits
//!   - pitch: signed 8 bits
//!   - roll: signed 8 bits
//!
//! Ok, we can decode a new actor! How about update an existing actor with a new attribute? The
//! actor is new bit would be off. Every actor receiving an update, is an actor we've already seen.
//! So we'll look up the actor's ObjectId using the decoded ActorId seen when the actor was
//! instantiated.
//!
//! - ObjectId: the index that the object appears in the `objects` list in the footer that we
//!   previously parsed. For instance, "Core.Object" would have an ObjectId of 0.
//! - ActorId: the temporary id of an actor while it is in game. Once an actor is deleted, the
//!   actor id may be recycled.
//!
//! Then while the next bit is on, there are more attributes for our actor.
//!
//! We need to read from the stream, the number of bits that can compose the largest stream id /
//! attribute id for the object without exceeding this max value. Very confusing, so let's take a
//! step back with an example. Let's say we need to read the next 5 bits for the attribute stream
//! id. If the result is 16, the attribute resolves differently depending on the actor's ObjectId.
//! A contrived example: "16" for an "Engine.Pawn" ObjectId could point to the attribute
//! "Engine.Pawn:bCanSwatTurn", while a "16" for "TAGame.Vehicle_TA" could be
//! "TAGame.Vehicle_TA:bDriving".
//!
//! So we need to construct a lookup for an ObjectId where we can find out how much to read for the
//! attribute's stream id and resolve the stream id to an actual attribute.
//!
//! To construct this lookup, we need to examine the net_cache previously decoded from the footer.
//! The net_cache represents a hierarchy of attributes that children inherit from parents.
//!
//! ```json
//! [{
//!   "object_ind": 0, // "Core.Object"
//!   "parent_id": 0,
//!   "cache_id": 0,   // The cache id referenced by children
//!   "properties": [] // There are no attributes on a base object
//! },
//! {
//!   "object_ind": 22, // "Engine.Actor"
//!   "parent_id": 0,   // points to the "cache_id" of it's parent (Core.Object)
//!   "cache_id": 21,
//!   "properties": [
//!     {
//!       "object_ind": 2, // "Engine.Actor:RelativeLocation"
//!       "stream_id": 1   // Reading a 1 from the bits maps to the RelativeLocation
//!     },
//!   ]
//! },
//! {
//!   "object_ind": 24, // "TAGame.VehiclePickup_TA"
//!   "parent_id": 21,  // points to the "cache_id" of it's parent (Engine.Actor)
//!   "cache_id": 22,
//!   "properties": [   // Properties include all of parent properties
//!     {
//!       "object_ind": 23, // "TAGame.VehiclePickup_TA:ReplicatedPickupData"
//!       "stream_id": 21
//!     }
//!   ]
//! }]
//! ```
//!
//! Going back to "We need to read from the stream, the number of bits that can compose the largest
//! stream id / attribute id", we can see in the previous example that the largest stream_id for a
//! TAGame.VehiclePickup_TA is 21, so 4 bits are needed with an optional 5th bit if the accumulator
//! couldn't exceed the max value with another "on" bit of information. These bits are the
//! attribute stream id. For Engine.Actor, only 1 bit needs to be read.
//!
//! With all this information, we can decode the attribute! Since there are 40 attribute types,
//! it's not feasible to document them all here. It may take a lot of guesswork to determine the
//! attribute type of a new attribute introduced in a released patch. Basically the recommendation
//! is to look at the source code. Attribute parsing reuses all the concepts we've gone over
//!
//! The only thing left is the other branch when the "actor is alive" bit is off. This means that
//! the actor is deleted and that the given actor id can be recycled.

use crate::core_parser::CoreParser;
use crate::crc::calc_crc;
use crate::errors::{NetworkError, ParseError};
use crate::header::{self, Header};
use crate::models::*;
use crate::network;
use crate::parsing_utils::{le_f32, le_i32};

/// Determines under what circumstances the parser should perform the crc check for replay
/// corruption. Since the crc check is the most time consuming part when parsing the header,
/// clients should choose under what circumstances a crc check is performed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrcCheck {
    /// Always perform the crc check. Useful when the replay has had its contents modified. This
    /// will catch a user that increased the number of goals they scored (easy) but only if they
    /// didn't update the crc as well (not as easy).
    Always,

    /// Never perform the crc check. Useful only when it doesn't matter to know if a replay is
    /// corrupt or not, you either want the data or the parsing error.
    Never,

    /// Only perform the crc check when parsing a section fails. This option gets the best of both
    /// worlds. If parsing fails, the crc check will determine if it is a programming error or the
    /// replay is corrupt. If parsing succeeds it won't precious time performing the check. This
    /// option is the default for parsing.
    OnError,
}

/// Determines how the parser should handle the network data, which is the most
/// intensive and volatile section of the replay.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkParse {
    /// If the network data fails parse return an error
    Always,

    /// Skip parsing the network data
    Never,

    /// Attempt to parse the network data, but if unsuccessful ignore the error
    /// and continue parsing
    IgnoreOnError,
}

/// The main entry point to parsing replays in boxcars. Allows one to customize parsing options,
/// such as only parsing the header and forgoing crc (corruption) checks.
#[derive(Debug, Clone, PartialEq)]
pub struct ParserBuilder<'a> {
    data: &'a [u8],
    crc_check: Option<CrcCheck>,
    network_parse: Option<NetworkParse>,
}

impl<'a> ParserBuilder<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        ParserBuilder {
            data,
            crc_check: None,
            network_parse: None,
        }
    }

    pub fn always_check_crc(mut self) -> ParserBuilder<'a> {
        self.crc_check = Some(CrcCheck::Always);
        self
    }

    pub fn never_check_crc(mut self) -> ParserBuilder<'a> {
        self.crc_check = Some(CrcCheck::Never);
        self
    }

    pub fn on_error_check_crc(mut self) -> ParserBuilder<'a> {
        self.crc_check = Some(CrcCheck::OnError);
        self
    }

    pub fn with_crc_check(mut self, check: CrcCheck) -> ParserBuilder<'a> {
        self.crc_check = Some(check);
        self
    }

    pub fn must_parse_network_data(mut self) -> ParserBuilder<'a> {
        self.network_parse = Some(NetworkParse::Always);
        self
    }

    pub fn never_parse_network_data(mut self) -> ParserBuilder<'a> {
        self.network_parse = Some(NetworkParse::Never);
        self
    }

    pub fn ignore_network_data_on_error(mut self) -> ParserBuilder<'a> {
        self.network_parse = Some(NetworkParse::IgnoreOnError);
        self
    }

    pub fn with_network_parse(mut self, parse: NetworkParse) -> ParserBuilder<'a> {
        self.network_parse = Some(parse);
        self
    }

    pub fn parse(self) -> Result<Replay, ParseError> {
        let mut parser = Parser::new(
            self.data,
            self.crc_check.unwrap_or(CrcCheck::OnError),
            self.network_parse.unwrap_or(NetworkParse::IgnoreOnError),
        );
        parser.parse()
    }
}

/// Intermediate parsing structure for the body / footer
#[derive(Debug, PartialEq)]
pub struct ReplayBody<'a> {
    pub levels: Vec<String>,
    pub keyframes: Vec<KeyFrame>,
    pub debug_info: Vec<DebugInfo>,
    pub tick_marks: Vec<TickMark>,
    pub packages: Vec<String>,
    pub objects: Vec<String>,
    pub names: Vec<String>,
    pub class_indices: Vec<ClassIndex>,
    pub net_cache: Vec<ClassNetCache>,
    pub network_data: &'a [u8],
}

/// Holds the current state of parsing a replay
#[derive(Debug, Clone, PartialEq)]
pub struct Parser<'a> {
    core: CoreParser<'a>,
    crc_check: CrcCheck,
    network_parse: NetworkParse,
}

impl<'a> Parser<'a> {
    fn new(data: &'a [u8], crc_check: CrcCheck, network_parse: NetworkParse) -> Self {
        Parser {
            core: CoreParser::new(data),
            crc_check,
            network_parse,
        }
    }

    fn parse(&mut self) -> Result<Replay, ParseError> {
        let header_size = self.core.take_i32("header size")?;
        let header_crc = self.core.take_u32("header crc")?;

        let header_data = self.core.view_data(header_size as usize).map_err(|e| {
            ParseError::ParseError("header data", self.core.bytes_read(), Box::new(e))
        })?;

        let header =
            self.crc_section(header_data, header_crc as u32, "header", Self::parse_header)?;

        let content_size = self.core.take_i32("content size")?;
        let content_crc = self.core.take_u32("content crc")?;

        let content_data = self.core.view_data(content_size as usize).map_err(|e| {
            ParseError::ParseError("content data", self.core.bytes_read(), Box::new(e))
        })?;

        let body = self.crc_section(content_data, content_crc as u32, "body", Self::parse_body)?;

        let network: Option<NetworkFrames> = match self.network_parse {
            NetworkParse::Always => Some(
                self.parse_network(&header, &body)
                    .map_err(|x| ParseError::NetworkError(Box::new(x)))?,
            ),
            NetworkParse::IgnoreOnError => self
                .parse_network(&header, &body)
                .map_err(|x| ParseError::NetworkError(Box::new(x)))
                .ok(),
            NetworkParse::Never => None,
        };

        Ok(Replay {
            header_size,
            header_crc,
            major_version: header.major_version,
            minor_version: header.minor_version,
            net_version: header.net_version,
            game_type: header.game_type,
            properties: header.properties,
            content_size,
            content_crc,
            network_frames: network,
            levels: body.levels,
            keyframes: body.keyframes,
            debug_info: body.debug_info,
            tick_marks: body.tick_marks,
            packages: body.packages,
            objects: body.objects,
            names: body.names,
            class_indices: body.class_indices,
            net_cache: body.net_cache,
        })
    }

    fn parse_network(
        &mut self,
        header: &Header,
        body: &ReplayBody<'_>,
    ) -> Result<NetworkFrames, NetworkError> {
        network::parse(header, body)
    }

    fn parse_header(&mut self) -> Result<Header, ParseError> {
        header::parse_header(&mut self.core)
    }

    /// Parses a section and performs a crc check as configured
    fn crc_section<T, F>(
        &mut self,
        data: &[u8],
        crc: u32,
        section: &str,
        mut f: F,
    ) -> Result<T, ParseError>
    where
        F: FnMut(&mut Self) -> Result<T, ParseError>,
    {
        let result = f(self);

        match self.crc_check {
            CrcCheck::Always => {
                let actual = calc_crc(data);
                if actual != crc as u32 {
                    Err(ParseError::CrcMismatch(crc, actual))
                } else {
                    result
                }
            }
            CrcCheck::OnError => result.map_err(|e| -> ParseError {
                let actual = calc_crc(data);
                if actual != crc as u32 {
                    ParseError::CorruptReplay(String::from(section), Box::new(e))
                } else {
                    e
                }
            }),
            CrcCheck::Never => result,
        }
    }

    fn parse_body(&mut self) -> Result<ReplayBody<'a>, ParseError> {
        let levels = self
            .core
            .text_list()
            .map_err(|e| ParseError::ParseError("levels", self.core.bytes_read(), Box::new(e)))?;

        let keyframes = self.parse_keyframe().map_err(|e| {
            ParseError::ParseError("keyframes", self.core.bytes_read(), Box::new(e))
        })?;

        let network_size = self.core.take_i32("network size")?;

        let network_data = self.core.take(network_size as usize, |d| d).map_err(|e| {
            ParseError::ParseError("network data", self.core.bytes_read(), Box::new(e))
        })?;

        let debug_infos = self.parse_debuginfo().map_err(|e| {
            ParseError::ParseError("debug info", self.core.bytes_read(), Box::new(e))
        })?;

        let tickmarks = self.parse_tickmarks().map_err(|e| {
            ParseError::ParseError("tickmarks", self.core.bytes_read(), Box::new(e))
        })?;

        let packages = self
            .core
            .text_list()
            .map_err(|e| ParseError::ParseError("packages", self.core.bytes_read(), Box::new(e)))?;
        let objects = self
            .core
            .text_list()
            .map_err(|e| ParseError::ParseError("objects", self.core.bytes_read(), Box::new(e)))?;
        let names = self
            .core
            .text_list()
            .map_err(|e| ParseError::ParseError("names", self.core.bytes_read(), Box::new(e)))?;

        let class_index = self.parse_classindex().map_err(|e| {
            ParseError::ParseError("class index", self.core.bytes_read(), Box::new(e))
        })?;

        let net_cache = self.parse_classcache().map_err(|e| {
            ParseError::ParseError("net cache", self.core.bytes_read(), Box::new(e))
        })?;

        Ok(ReplayBody {
            levels,
            keyframes,
            debug_info: debug_infos,
            tick_marks: tickmarks,
            packages,
            objects,
            names,
            class_indices: class_index,
            net_cache,
            network_data,
        })
    }

    fn parse_tickmarks(&mut self) -> Result<Vec<TickMark>, ParseError> {
        self.core.list_of(|s| {
            Ok(TickMark {
                description: s.parse_text()?,
                frame: s.take(4, le_i32)?,
            })
        })
    }

    fn parse_keyframe(&mut self) -> Result<Vec<KeyFrame>, ParseError> {
        self.core.list_of(|s| {
            Ok(KeyFrame {
                time: s.take(4, le_f32)?,
                frame: s.take(4, le_i32)?,
                position: s.take(4, le_i32)?,
            })
        })
    }

    fn parse_debuginfo(&mut self) -> Result<Vec<DebugInfo>, ParseError> {
        self.core.list_of(|s| {
            Ok(DebugInfo {
                frame: s.take(4, le_i32)?,
                user: s.parse_text()?,
                text: s.parse_text()?,
            })
        })
    }

    fn parse_classindex(&mut self) -> Result<Vec<ClassIndex>, ParseError> {
        self.core.list_of(|s| {
            Ok(ClassIndex {
                class: s.parse_str().map(String::from)?,
                index: s.take(4, le_i32)?,
            })
        })
    }

    fn parse_classcache(&mut self) -> Result<Vec<ClassNetCache>, ParseError> {
        self.core.list_of(|x| {
            Ok(ClassNetCache {
                object_ind: x.take(4, le_i32)?,
                parent_id: x.take(4, le_i32)?,
                cache_id: x.take(4, le_i32)?,
                properties: x.list_of(|s| {
                    Ok(CacheProp {
                        object_ind: s.take(4, le_i32)?,
                        stream_id: s.take(4, le_i32)?,
                    })
                })?,
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::TickMark;
    use std::error::Error;

    #[test]
    fn key_frame_list() {
        let data = include_bytes!("../assets/replays/good/rumble.replay");

        // List is 2A long, each keyframe is 12 bytes. Then add four for list length = 508
        let mut parser = Parser::new(
            &data[0x12ca..0x12ca + 508],
            CrcCheck::Never,
            NetworkParse::Never,
        );
        let frames = parser.parse_keyframe().unwrap();
        assert_eq!(frames.len(), 42);
    }

    #[test]
    fn tickmark_list() {
        let data = include_bytes!("../assets/replays/good/rumble.replay");

        // 7 tick marks at 8 bytes + size of tick list
        let mut parser = Parser::new(
            &data[0xf6cce..0xf6d50],
            CrcCheck::Never,
            NetworkParse::Never,
        );
        let ticks = parser.parse_tickmarks().unwrap();

        assert_eq!(ticks.len(), 7);
        assert_eq!(
            ticks[0],
            TickMark {
                description: String::from("Team1Goal"),
                frame: 396,
            }
        );
    }

    #[test]
    fn test_the_parsing_empty() {
        let mut parser = Parser::new(&[], CrcCheck::Never, NetworkParse::Never);
        assert!(parser.parse().is_err());
    }

    #[test]
    fn test_the_parsing_text_too_long() {
        let data = include_bytes!("../assets/replays/bad/fuzz-string-too-long.replay");
        let mut parser = Parser::new(&data[..], CrcCheck::Never, NetworkParse::Never);
        assert!(parser.parse().is_err())
    }

    #[test]
    fn test_the_parsing_text_too_long2() {
        let data = include_bytes!("../assets/replays/bad/fuzz-string-too-long2.replay");
        let mut parser = Parser::new(&data[..], CrcCheck::Never, NetworkParse::Always);
        let err = parser.parse().unwrap_err();
        assert!(format!("{}", err).contains("Unexpected size for string: -1912602609"));
    }

    #[test]
    fn test_fuzz_corpus_slice_index() {
        let data = include_bytes!("../assets/replays/bad/fuzz-slice-index.replay");
        let mut parser = Parser::new(&data[..], CrcCheck::Never, NetworkParse::Never);
        assert!(parser.parse().is_err())
    }

    #[test]
    fn test_the_fuzz_corpus_abs_panic() {
        let data = include_bytes!("../assets/replays/bad/fuzz-corpus.replay");
        let mut parser = Parser::new(&data[..], CrcCheck::Never, NetworkParse::Never);
        assert!(parser.parse().is_err())
    }

    #[test]
    fn test_the_fuzz_corpus_large_list() {
        let data = include_bytes!("../assets/replays/bad/fuzz-list-too-large.replay");
        let mut parser = Parser::new(&data[..], CrcCheck::Never, NetworkParse::Never);
        let err = parser.parse().unwrap_err();
        assert!(format!("{}", err)
            .starts_with("Could not decode replay debug info at offset (1010894): list of size"));
    }

    #[test]
    fn test_the_fuzz_corpus_large_list_on_error_crc() {
        let data = include_bytes!("../assets/replays/bad/fuzz-list-too-large.replay");
        let mut parser = Parser::new(&data[..], CrcCheck::OnError, NetworkParse::Never);
        let err = parser.parse().unwrap_err();
        assert_eq!(
            "Failed to parse body and crc check failed. Replay is corrupt",
            format!("{}", err)
        );

        assert!(format!("{}", err.source().unwrap())
            .starts_with("Could not decode replay debug info at offset (1010894): list of size"));
    }

    #[test]
    fn test_the_fuzz_corpus_large_list_always_crc() {
        let data = include_bytes!("../assets/replays/bad/fuzz-list-too-large.replay");
        let mut parser = Parser::new(&data[..], CrcCheck::Always, NetworkParse::Never);
        let err = parser.parse().unwrap_err();
        assert_eq!(
            "Crc mismatch. Expected 3765941959 but received 1314727725",
            format!("{}", err)
        );
        assert!(err.source().is_none());
    }

    #[test]
    fn test_the_fuzz_object_id_too_large() {
        let data = include_bytes!("../assets/replays/bad/fuzz-large-object-id.replay");
        let mut parser = Parser::new(&data[..], CrcCheck::Never, NetworkParse::Always);
        let err = parser.parse().unwrap_err();
        assert_eq!("Object Id of 1547 exceeds range", format!("{}", err));
        assert!(err.source().is_some());
    }

    #[test]
    fn test_the_fuzz_too_many_frames() {
        let data = include_bytes!("../assets/replays/bad/fuzz-too-many-frames.replay");
        let mut parser = Parser::new(&data[..], CrcCheck::Never, NetworkParse::Always);
        let err = parser.parse().unwrap_err();
        assert_eq!("Too many frames to decode: 738197735", format!("{}", err));
        assert!(err.source().is_some());
    }

    #[test]
    fn test_crc_check_with_bad() {
        let mut data = include_bytes!("../assets/replays/good/rumble.replay").to_vec();

        // Changing this byte won't make the parsing fail but will make the crc check fail
        data[4775] = 100;
        let mut parser = Parser::new(&data[..], CrcCheck::Always, NetworkParse::Never);
        let res = parser.parse();
        assert!(res.is_err());
        assert_eq!(
            "Crc mismatch. Expected 337843175 but received 2877465516",
            format!("{}", res.unwrap_err())
        );

        parser = Parser::new(&data[..], CrcCheck::OnError, NetworkParse::Never);
        assert!(parser.parse().is_ok());
    }
}
