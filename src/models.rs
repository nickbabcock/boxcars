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
use crate::network::Frame;
use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Serialize, Serializer};
use std::collections::HashMap;

/// The structure that a rocket league replay is parsed into.
#[derive(Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "py", derive(dict_derive::IntoPyObject))]
pub struct Replay {
    pub header_size: i32,
    pub header_crc: u32,
    pub major_version: i32,
    pub minor_version: i32,
    pub net_version: Option<i32>,
    pub game_type: String,

    /// Could use a map to represent properties but I don't want to assume that duplicate keys
    /// can't exist, so to be safe, use a traditional vector.
    #[serde(serialize_with = "pair_vec")]
    pub properties: Vec<(String, HeaderProp)>,
    pub content_size: i32,
    pub content_crc: u32,
    pub network_frames: Option<NetworkFrames>,
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

/// The frames decoded from the network data
#[derive(Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "py", derive(dict_derive::IntoPyObject))]
pub struct NetworkFrames {
    pub frames: Vec<Frame>,
}

/// In Rocket league replays, there are tickmarks that typically represent a significant event in
/// the game (eg. a goal). The tick mark is placed before the event happens so there is a ramp-up
/// time. For instance, a tickmark could be at frame 396 for a goal at frame 441. At 30 fps, this
/// would be 1.5 seconds of ramp up time.
#[derive(Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "py", derive(dict_derive::IntoPyObject))]
pub struct TickMark {
    pub description: String,
    pub frame: i32,
}

/// Keyframes as defined by the video compression section in the [wikipedia][] article, are the
/// main frames that are derived from in the following frame data. The key frames decoded will
/// match up with the frames decoded from the network data.
///
/// [wikipedia]: https://en.wikipedia.org/wiki/Key_frame#Video_compression
#[derive(Serialize, PartialEq, Debug, Clone, Copy)]
#[cfg_attr(feature = "py", derive(dict_derive::IntoPyObject))]
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
pub enum HeaderProp {
    Array(Vec<Vec<(String, HeaderProp)>>),
    Bool(bool),
    Byte,
    Float(f32),
    Int(i32),
    Name(String),
    QWord(u64),
    Str(String),
}

impl HeaderProp {
    /// If the `HeaderProp` is an array of properties, returns the array
    /// ```
    /// # use boxcars::HeaderProp;
    /// let v = HeaderProp::Array(vec![
    ///     vec![("abc".to_string(), HeaderProp::Byte)]
    /// ]);
    ///
    /// assert_eq!(v.as_array().unwrap().len(), 1);
    /// assert_eq!(v.as_array().unwrap()[0][0].1.as_array(), None);
    /// ```
    pub fn as_array(&self) -> Option<&Vec<Vec<(String, HeaderProp)>>> {
        if let HeaderProp::Array(arr) = self {
            Some(arr)
        } else {
            None
        }
    }

    /// If the `HeaderProp` is a boolean, returns the value
    /// ```
    /// # use boxcars::HeaderProp;
    /// let v = HeaderProp::Bool(true);
    /// let b = HeaderProp::Byte;
    ///
    /// assert_eq!(v.as_bool(), Some(true));
    /// assert_eq!(b.as_bool(), None);
    /// ```
    pub fn as_bool(&self) -> Option<bool> {
        if let HeaderProp::Bool(val) = self {
            Some(*val)
        } else {
            None
        }
    }

    /// If the `HeaderProp` is a float, returns the value
    /// ```
    /// # use boxcars::HeaderProp;
    /// let v = HeaderProp::Float(2.50);
    /// let b = HeaderProp::Byte;
    ///
    /// assert_eq!(v.as_float(), Some(2.50));
    /// assert_eq!(b.as_float(), None);
    /// ```
    pub fn as_float(&self) -> Option<f32> {
        if let HeaderProp::Float(val) = self {
            Some(*val)
        } else {
            None
        }
    }

    /// If the `HeaderProp` is a QWord, returns the value
    /// ```
    /// # use boxcars::HeaderProp;
    /// let v = HeaderProp::QWord(250);
    /// let b = HeaderProp::Byte;
    ///
    /// assert_eq!(v.as_u64(), Some(250));
    /// assert_eq!(b.as_u64(), None);
    /// ```
    pub fn as_u64(&self) -> Option<u64> {
        if let HeaderProp::QWord(val) = self {
            Some(*val)
        } else {
            None
        }
    }

    /// If the `HeaderProp` is an int, returns the value
    /// ```
    /// # use boxcars::HeaderProp;
    /// let v = HeaderProp::Int(-250);
    /// let b = HeaderProp::Byte;
    ///
    /// assert_eq!(v.as_i32(), Some(-250));
    /// assert_eq!(b.as_i32(), None);
    /// ```
    pub fn as_i32(&self) -> Option<i32> {
        if let HeaderProp::Int(val) = self {
            Some(*val)
        } else {
            None
        }
    }

    /// If the `HeaderProp` is an string, returns the value
    /// ```
    /// # use boxcars::HeaderProp;
    /// let v = HeaderProp::Name("abc".to_string());
    /// let x = HeaderProp::Str("def".to_string());
    /// let b = HeaderProp::Byte;
    ///
    /// assert_eq!(v.as_string(), Some("abc"));
    /// assert_eq!(x.as_string(), Some("def"));
    /// assert_eq!(b.as_i32(), None);
    /// ```
    pub fn as_string(&self) -> Option<&str> {
        match self {
            HeaderProp::Name(val) => Some(val.as_str()),
            HeaderProp::Str(val) => Some(val.as_str()),
            _ => None,
        }
    }

    /// Returns if the `HeaderProp` is a byte
    /// ```
    /// # use boxcars::HeaderProp;
    /// let v = HeaderProp::Name("abc".to_string());
    /// let b = HeaderProp::Byte;
    ///
    /// assert_eq!(v.is_byte(), false);
    /// assert_eq!(b.is_byte(), true);
    /// ```
    pub fn is_byte(&self) -> bool {
        if let HeaderProp::Byte = self {
            true
        } else {
            false
        }
    }
}

/// Debugging info stored in the replay if debugging is enabled.
#[derive(Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "py", derive(dict_derive::IntoPyObject))]
pub struct DebugInfo {
    pub frame: i32,
    pub user: String,
    pub text: String,
}

/// A mapping between an object's name and its index. Largely redundant
#[derive(Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "py", derive(dict_derive::IntoPyObject))]
pub struct ClassIndex {
    /// Should be equivalent to `Replay::objects(self.index)`
    pub class: String,

    /// The index that the object appears in the `Replay::objects`
    pub index: i32,
}

/// A mapping between an object (that's an attribute)'s index and what its id will be when encoded
/// in the network data
#[derive(Serialize, PartialEq, Debug, Clone, Copy)]
#[cfg_attr(feature = "py", derive(dict_derive::IntoPyObject))]
pub struct CacheProp {
    /// The index that the object appears in the `Replay::objects`
    pub object_ind: i32,

    /// An attribute / property id that appears in the network data. Stream ids are often re-used
    /// between multiple different properties
    pub stream_id: i32,
}

/// Contains useful information when decoding the network stream
#[derive(Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "py", derive(dict_derive::IntoPyObject))]
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
impl Serialize for HeaderProp {
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
                    for (key, val) in inner.iter() {
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
            HeaderProp::QWord(ref x) => serializer.collect_str(x),
            HeaderProp::Name(ref x) | HeaderProp::Str(ref x) => serializer.serialize_str(x),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn to_json<T: serde::Serialize>(input: &T) -> std::string::String {
        serde_json::to_string(input).unwrap()
    }

    #[test]
    fn serialize_header_array() {
        let data = vec![
            vec![
                (String::from("frame"), HeaderProp::Int(441)),
                (
                    String::from("PlayerName"),
                    HeaderProp::Str(String::from("rust is awesome")),
                ),
            ],
            vec![
                (String::from("frame"), HeaderProp::Int(1738)),
                (
                    String::from("PlayerName"),
                    HeaderProp::Str(String::from("rusty")),
                ),
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
        assert_eq!(to_json(&HeaderProp::QWord(10)), "\"10\"");
        assert_eq!(to_json(&HeaderProp::Float(10.2)), "10.2");
        assert_eq!(to_json(&HeaderProp::Int(11)), "11");
    }

    #[test]
    fn serialize_header_str() {
        let val = "hello world";
        assert_eq!(
            to_json(&HeaderProp::Str(String::from(val))),
            "\"hello world\""
        );
        assert_eq!(
            to_json(&HeaderProp::Name(String::from(val))),
            "\"hello world\""
        );
    }
}
