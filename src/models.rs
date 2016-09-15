//! # Models
//!
//! Here lies the data structures that a rocket league replay is decoded into. All of the models
//! are contained in this one file because of serde. On Rust nightly, which is what this library
//! needs to compile, no special actions are needed for serde to work. However, to achieve
//! compilation on Rust stable, an extra [build step can be
//! introduced](https://serde.rs/codegen-stable.html). Part of the intermediate step is to split
//! all the serde specific code into a separate file, which is what I've done here. To me, it is
//! not critical to have this library compile on stable, hence this work remains unfinished.
//!
//! For serde, we only care about serialization, JSON serialization. Deserialization is not
//! implemented from our JSON output because it is lossy (JSON isn't the best with different
//! numeric/string types). Asking "why JSON" would be next logical step, and that's due to other
//! rocket league replay parsers (like Octane) using JSON; however, the output of this library is
//! not compatible with that of other rocket league replay parsers.

use serde::{Serialize, Serializer};
use std::collections::HashMap;

/// The structure that a rocket league replay is parsed into.
#[derive(Serialize, PartialEq, Debug)]
pub struct Replay {
    pub header_size: u32,
    pub header_crc: u32,
    pub major_version: u32,
    pub minor_version: u32,
    pub game_type: String,

    /// Could use a map to represent properties but I don't want to assume that duplicate keys
    /// can't exist, so to be safe, use a traditional vector.
    #[serde(serialize_with = "pair_vec")]
    pub properties: Vec<(String, HeaderProp)>,
    pub content_size: u32,
    pub content_crc: u32,
    pub levels: Vec<String>,
    pub keyframes: Vec<KeyFrame>,
    pub debug_info: Vec<DebugInfo>,
    pub tick_marks: Vec<TickMark>,
    pub packages: Vec<String>,
    pub objects: Vec<String>,
    pub names: Vec<String>,
    pub class_indices: Vec<ClassIndex>,
    pub net_cache: Vec<ClassNetCache>,
}

/// In Rocket league replays, there are tickmarks that typically represent a significant event in
/// the game (eg. a goal). The tick mark is placed before the event happens so there is a ramp-up
/// time. For instance, a tickmark could be at frame 396 for a goal at frame 441. At 30 fps, this
/// would be 1.5 seconds of ramp up time.
#[derive(Serialize, PartialEq, Debug)]
pub struct TickMark {
    pub description: String,
    pub frame: u32,
}

/// Keyframes as defined by the video compression section in the [wikipedia][] article, are the
/// main frames that are derived from in the following frame data. Since we are not decoding the
/// network stream, this is more a nice-to-decode than a necessity
///
/// [wikipedia]: https://en.wikipedia.org/wiki/Key_frame#Video_compression
#[derive(Serialize, PartialEq, Debug)]
pub struct KeyFrame {
    pub time: f32,
    pub frame: u32,
    pub position: u32,
}

/// All the interesting data are stored as properties in the header, properties such as:
/// - When and who scored a goal
/// - Player stats (goals, assists, score, etc).
/// - Date and level played on
/// A property can be a number, string, or a more complex object such as an array containing
/// additional properties.
#[derive(PartialEq, Debug)]
pub enum HeaderProp {
    Array(Vec<Vec<(String, HeaderProp)>>),
    Bool(bool),
    Byte,
    Float(f32),
    Int(u32),
    Name(String),
    QWord(u64),
    Str(String),
}

/// Debugging info stored in the replay if debugging is enabled.
#[derive(Serialize, PartialEq, Debug)]
pub struct DebugInfo {
    pub frame: u32,
    pub user: String,
    pub text: String,
}

/// Contains useful information when decoding the network stream, which we aren't
#[derive(Serialize, PartialEq, Debug)]
pub struct ClassIndex {
    pub class: String,
    pub index: u32,
}

/// Contains useful information when decoding the network stream, which we aren't
#[derive(Serialize, PartialEq, Debug)]
pub struct CacheProp {
    pub index: u32,
    pub id: u32,
}

/// Contains useful information when decoding the network stream, which we aren't
#[derive(Serialize, PartialEq, Debug)]
pub struct ClassNetCache {
    pub index: u32,
    pub parent_id: u32,
    pub id: u32,
    pub properties: Vec<CacheProp>,
}

/// Serialize a vector of key value tuples into a map. This is useful when the data we're ingesting
/// (rocket league replay data) doesn't have a defined spec, so it may be assuming too much to
/// store it into an associative array, so it's stored as a normal sequence. Here we serialize as a
/// map structure because most replay parser do this, so we should be compliant and the data format
/// doesn't dictate that the keys in a sequence of key value pairs must be distinct. It's true,
/// JSON doesn't need the keys to be unique: http://stackoverflow.com/q/21832701/433785
fn pair_vec<K, V, S>(inp: &[(K, V)], serializer: &mut S) -> Result<(), S::Error>
    where K: Serialize,
          V: Serialize,
          S: Serializer
{
    let mut state = try!(serializer.serialize_map(Some(inp.len())));
    for &(ref key, ref val) in inp.iter() {
        try!(serializer.serialize_map_key(&mut state, key));
        try!(serializer.serialize_map_value(&mut state, val));
    }
    return serializer.serialize_map_end(state);
}

/// By default serde will generate a serialization method that writes out the enum as well as the
/// enum value. Since header values are self describing in JSON, we do not need to serialize the
/// enum type. This is slightly lossy as in the serialized format it will be ambiguous if a value
/// is a `Name` or `Str`, as well as `Byte`, `Float`, `Int`, or `QWord`.
impl Serialize for HeaderProp {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer
    {
        match *self {
            HeaderProp::Array(ref x) => {
                let mut state = try!(serializer.serialize_seq(Some(x.len())));
                for inner in x {
                    // Look for a better way to do this instead of allocating the intermediate map
                    let mut els = HashMap::new();
                    for &(ref key, ref val) in inner.iter() {
                        els.insert(key, val);
                    }
                    try!(serializer.serialize_seq_elt(&mut state, els));
                }
                return serializer.serialize_seq_end(state);
            }
            HeaderProp::Bool(ref x) => serializer.serialize_bool(*x),
            HeaderProp::Byte => serializer.serialize_u8(0),
            HeaderProp::Float(ref x) => serializer.serialize_f32(*x),
            HeaderProp::Int(ref x) => serializer.serialize_u32(*x),
            HeaderProp::QWord(ref x) => serializer.serialize_u64(*x),
            HeaderProp::Name(ref x) | HeaderProp::Str(ref x) => serializer.serialize_str(x)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde;
    use std;
    use serde_json;

    fn to_json<T: serde::Serialize>(input: &T) -> std::string::String {
        return serde_json::to_string(input).unwrap();
    }

    #[test]
    fn serialize_header_array() {
        let data = vec![
            vec![
                ("frame".to_string(), HeaderProp::Int(441)),
                ("PlayerName".to_string(), HeaderProp::Str("rust is awesome".to_string()))
            ], vec![
                ("frame".to_string(), HeaderProp::Int(1738)),
                ("PlayerName".to_string(), HeaderProp::Str("rusty".to_string()))
            ]
          ];
        let actual = to_json(&HeaderProp::Array(data));
        assert!(actual.contains("\"PlayerName\":\"rust is awesome\""));
        assert!(actual.contains("\"PlayerName\":\"rusty\""));
        assert!(actual.contains("\"frame\":441"));
        assert!(actual.contains("\"frame\":1738"));
    }

    #[test]
    fn serialize_header_bool() {
        assert_eq!(to_json(&HeaderProp::Bool(false)), "false");
        assert_eq!(to_json(&HeaderProp::Bool(true)), "true");
    }

    #[test]
    fn serialize_header_numbers() {
        assert_eq!(to_json(&HeaderProp::Byte), "0");
        assert_eq!(to_json(&HeaderProp::QWord(10)), "10");
        assert_eq!(to_json(&HeaderProp::Float(10.2)), "10.2");
        assert_eq!(to_json(&HeaderProp::Int(11)), "11");
    }

    #[test]
    fn serialize_header_str() {
        let val = "hello world";
        assert_eq!(to_json(&HeaderProp::Str(val.to_string())), "\"hello world\"");
        assert_eq!(to_json(&HeaderProp::Name(val.to_string())), "\"hello world\"");
    }
}
