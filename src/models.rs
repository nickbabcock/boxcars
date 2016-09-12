use serde::{Serialize, Serializer};
use std::collections::HashMap;

#[derive(Serialize, PartialEq, Debug)]
pub struct Replay {
    pub header_size: u32,
    pub header_crc: u32,
    pub major_version: u32,
    pub minor_version: u32,
    pub game_type: String,

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

#[derive(Serialize, PartialEq, Debug)]
pub struct TickMark {
    pub description: String,
    pub frame: u32,
}

#[derive(Serialize, PartialEq, Debug)]
pub struct KeyFrame {
    pub time: f32,
    pub frame: u32,
    pub position: u32,
}

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

#[derive(Serialize, PartialEq, Debug)]
pub struct DebugInfo {
    pub frame: u32,
    pub user: String,
    pub text: String,
}

#[derive(Serialize, PartialEq, Debug)]
pub struct ClassIndex {
    pub class: String,
    pub index: u32,
}

#[derive(Serialize, PartialEq, Debug)]
pub struct CacheProp {
    pub index: u32,
    pub id: u32,
}

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
        let actual = HeaderProp::Array(data);
        assert_eq!(to_json(&actual), "[{\"PlayerName\":\"rust is awesome\",\"frame\":441},{\"PlayerName\":\"rusty\",\"frame\":1738}]");
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
