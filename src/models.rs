/// # Models
///
/// Here lies the data structures that a rocket league replay is decoded into. All of the models
/// are contained in this one file because of serde.
///
/// For serde, we only care about serialization, JSON serialization. Deserialization is not
/// implemented from our JSON output because it is lossy (JSON isn't the best with different
/// numeric/string types). Asking "why JSON" would be next logical step, and that's due to other
/// rocket league replay parsers (like Octane) using JSON; however, the output of this library is
/// not compatible with that of other rocket league replay parsers.

use serde::{Serialize, Serializer};
use serde::ser::{SerializeMap, SerializeSeq};
use std::collections::HashMap;
use std::borrow::Cow;
use network::Frame;

/// The structure that a rocket league replay is parsed into.
#[derive(Serialize, PartialEq, Debug, Clone)]
pub struct Replay<'a> {
    pub header_size: i32,
    pub header_crc: i32,
    pub major_version: i32,
    pub minor_version: i32,
    pub net_version: Option<i32>,
    pub game_type: Cow<'a, str>,

    /// Could use a map to represent properties but I don't want to assume that duplicate keys
    /// can't exist, so to be safe, use a traditional vector.
    #[serde(serialize_with = "pair_vec")]
    pub properties: Vec<(&'a str, HeaderProp<'a>)>,
    pub content_size: i32,
    pub content_crc: i32,
    pub network_frames: Option<NetworkFrames>,
    pub levels: Vec<Cow<'a, str>>,
    pub keyframes: Vec<KeyFrame>,
    pub debug_info: Vec<DebugInfo<'a>>,
    pub tick_marks: Vec<TickMark<'a>>,
    pub packages: Vec<Cow<'a, str>>,
    pub objects: Vec<Cow<'a, str>>,
    pub names: Vec<Cow<'a, str>>,
    pub class_indices: Vec<ClassIndex<'a>>,
    pub net_cache: Vec<ClassNetCache>,
}

/// The frames decoded from the network data
#[derive(Serialize, PartialEq, Debug, Clone)]
pub struct NetworkFrames {
    pub frames: Vec<Frame>,
}

/// In Rocket league replays, there are tickmarks that typically represent a significant event in
/// the game (eg. a goal). The tick mark is placed before the event happens so there is a ramp-up
/// time. For instance, a tickmark could be at frame 396 for a goal at frame 441. At 30 fps, this
/// would be 1.5 seconds of ramp up time.
#[derive(Serialize, PartialEq, Debug, Clone)]
pub struct TickMark<'a> {
    pub description: Cow<'a, str>,
    pub frame: i32,
}

/// Keyframes as defined by the video compression section in the [wikipedia][] article, are the
/// main frames that are derived from in the following frame data. The key frames decoded will
/// match up with the frames decoded from the network data.
///
/// [wikipedia]: https://en.wikipedia.org/wiki/Key_frame#Video_compression
#[derive(Serialize, PartialEq, Debug, Clone, Copy)]
pub struct KeyFrame {
    pub time: f32,
    pub frame: i32,
    pub position: i32,
}

/// All the interesting data are stored as properties in the header, properties such as:
///
/// - When and who scored a goal
/// - Player stats (goals, assists, score, etc).
/// - Date and level played on
///
/// A property can be a number, string, or a more complex object such as an array containing
/// additional properties.
#[derive(PartialEq, Debug, Clone)]
pub enum HeaderProp<'a> {
    Array(Vec<Vec<(&'a str, HeaderProp<'a>)>>),
    Bool(bool),
    Byte,
    Float(f32),
    Int(i32),
    Name(Cow<'a, str>),
    QWord(i64),
    Str(Cow<'a, str>),
}

/// Debugging info stored in the replay if debugging is enabled.
#[derive(Serialize, PartialEq, Debug, Clone)]
pub struct DebugInfo<'a> {
    pub frame: i32,
    pub user: Cow<'a, str>,
    pub text: Cow<'a, str>,
}

/// A mapping between an object's name and its index. Largely redundant
#[derive(Serialize, PartialEq, Debug, Clone)]
pub struct ClassIndex<'a> {
    /// Should be equivalent to `Replay::objects(self.index)`
    pub class: &'a str,

    /// The index that the object appears in the `Replay::objects`
    pub index: i32,
}

/// A mapping between an object (that's an attribute)'s index and what its id will be when encoded
/// in the network data
#[derive(Serialize, PartialEq, Debug, Clone, Copy)]
pub struct CacheProp {
    /// The index that the object appears in the `Replay::objects`
    pub object_ind: i32,

    /// An attribute / property id that appears in the network data. Stream ids are often re-used
    /// between multiple different properties
    pub stream_id: i32,
}

/// Contains useful information when decoding the network stream, which we aren't
#[derive(Serialize, PartialEq, Debug, Clone)]
pub struct ClassNetCache {
    /// The index that the object appears in the `Replay::objects`
    pub object_ind: i32,

    /// The cache id of the parent. The child class inherits all the parent's properties.
    pub parent_id: i32,

    /// The cache id of the object
    pub cache_id: i32,

    /// List of properties that is on the object.
    pub properties: Vec<CacheProp>,
}

/// Serialize a vector of key value tuples into a map. This is useful when the data we're ingesting
/// (rocket league replay data) doesn't have a defined spec, so it may be assuming too much to
/// store it into an associative array, so it's stored as a normal sequence. Here we serialize as a
/// map structure because most replay parser do this, so we should be compliant and the data format
/// doesn't dictate that the keys in a sequence of key value pairs must be distinct. It's true,
/// JSON doesn't need the keys to be unique: <http://stackoverflow.com/q/21832701/433785>
fn pair_vec<K, V, S>(inp: &[(K, V)], serializer: S) -> Result<S::Ok, S::Error>
where
    K: Serialize,
    V: Serialize,
    S: Serializer,
{
    let mut state = serializer.serialize_map(Some(inp.len()))?;
    for &(ref key, ref val) in inp.iter() {
        state.serialize_key(key)?;
        state.serialize_value(val)?;
    }
    state.end()
}

/// By default serde will generate a serialization method that writes out the enum as well as the
/// enum value. Since header values are self describing in JSON, we do not need to serialize the
/// enum type. This is slightly lossy as in the serialized format it will be ambiguous if a value
/// is a `Name` or `Str`, as well as `Byte`, `Float`, `Int`, or `QWord`.
impl<'a> Serialize for HeaderProp<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            HeaderProp::Array(ref x) => {
                let mut state = serializer.serialize_seq(Some(x.len()))?;
                for inner in x {
                    // Look for a better way to do this instead of allocating the intermediate map
                    let mut els = HashMap::new();
                    for &(key, ref val) in inner.iter() {
                        els.insert(key, val);
                    }
                    state.serialize_element(&els)?;
                }
                state.end()
            }
            HeaderProp::Bool(ref x) => serializer.serialize_bool(*x),
            HeaderProp::Byte => serializer.serialize_u8(0),
            HeaderProp::Float(ref x) => serializer.serialize_f32(*x),
            HeaderProp::Int(ref x) => serializer.serialize_i32(*x),
            HeaderProp::QWord(ref x) => serializer.serialize_i64(*x),
            HeaderProp::Name(ref x) | HeaderProp::Str(ref x) => serializer.serialize_str(x),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde;
    use std;
    use serde_json;
    use std::borrow::Cow;

    fn to_json<T: serde::Serialize>(input: &T) -> std::string::String {
        serde_json::to_string(input).unwrap()
    }

    #[test]
    fn serialize_header_array() {
        let data = vec![
            vec![
                ("frame", HeaderProp::Int(441)),
                (
                    "PlayerName",
                    HeaderProp::Str(Cow::Borrowed("rust is awesome")),
                ),
            ],
            vec![
                ("frame", HeaderProp::Int(1738)),
                ("PlayerName", HeaderProp::Str(Cow::Borrowed("rusty"))),
            ],
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
        assert_eq!(
            to_json(&HeaderProp::Str(Cow::Borrowed(val))),
            "\"hello world\""
        );
        assert_eq!(
            to_json(&HeaderProp::Name(Cow::Borrowed(val))),
            "\"hello world\""
        );
    }
}
