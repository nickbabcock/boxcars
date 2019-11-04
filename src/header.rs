use crate::core_parser::CoreParser;
use crate::errors::ParseError;
use crate::models::HeaderProp;
use crate::parsing_utils::{le_f32, le_i32, le_u64};

/// Intermediate parsing structure for the header
#[derive(Debug, PartialEq)]
pub struct Header {
    pub major_version: i32,
    pub minor_version: i32,
    pub net_version: Option<i32>,
    pub game_type: String,
    pub properties: Vec<(String, HeaderProp)>,
}

impl Header {
    pub fn num_frames(&self) -> Option<i32> {
        self.properties
            .iter()
            .find(|&(key, _)| key == "NumFrames")
            .and_then(|&(_, ref prop)| prop.as_i32())
    }

    pub fn max_channels(&self) -> Option<i32> {
        self.properties
            .iter()
            .find(|&(key, _)| key == "MaxChannels")
            .and_then(|&(_, ref prop)| prop.as_i32())
    }
}

pub fn parse_header(rlp: &mut CoreParser) -> Result<Header, ParseError> {
    let major_version = rlp
        .take(4, le_i32)
        .map_err(|e| ParseError::ParseError("major version", rlp.bytes_read(), Box::new(e)))?;

    let minor_version = rlp
        .take(4, le_i32)
        .map_err(|e| ParseError::ParseError("minor version", rlp.bytes_read(), Box::new(e)))?;

    let net_version =
        if major_version > 865 && minor_version > 17 {
            Some(rlp.take(4, le_i32).map_err(|e| {
                ParseError::ParseError("net version", rlp.bytes_read(), Box::new(e))
            })?)
        } else {
            None
        };

    let game_type = rlp
        .parse_text()
        .map_err(|e| ParseError::ParseError("game type", rlp.bytes_read(), Box::new(e)))?;

    let properties = parse_rdict(rlp)
        .map_err(|e| ParseError::ParseError("header properties", rlp.bytes_read(), Box::new(e)))?;

    Ok(Header {
        major_version,
        minor_version,
        net_version,
        game_type,
        properties,
    })
}

fn parse_rdict(rlp: &mut CoreParser) -> Result<Vec<(String, HeaderProp)>, ParseError> {
    // Other the actual network data, the header property associative array is the hardest to parse.
    // The format is to:
    // - Read string
    // - If string is "None", we're done
    // - else we're dealing with a property, and the string just read is the key. Now deserialize the
    //   value.
    // The return type of this function is a key value vector because since there is no format
    // specification, we can't rule out duplicate keys. Possibly consider a multi-map in the future.

    let mut res: Vec<_> = Vec::new();
    loop {
        let key = rlp.parse_str()?;
        if key == "None" || key == "\0\0\0None" {
            break;
        }

        let val = match rlp.parse_str()? {
            "ArrayProperty" => array_property(rlp),
            "BoolProperty" => bool_property(rlp),
            "ByteProperty" => byte_property(rlp),
            "FloatProperty" => float_property(rlp),
            "IntProperty" => int_property(rlp),
            "NameProperty" => name_property(rlp),
            "QWordProperty" => qword_property(rlp),
            "StrProperty" => str_property(rlp),
            x => Err(ParseError::UnexpectedProperty(String::from(x))),
        }?;

        res.push((String::from(key), val));
    }

    Ok(res)
}

// Header properties are encoded in a pretty simple format, with some oddities. The first 64bits
// is data that can be discarded, some people think that the 64bits is the length of the data
// while others think that the first 32bits is the header length in bytes with the subsequent
// 32bits unknown. Doesn't matter to us, we throw it out anyways. The rest of the bytes are
// decoded property type specific.

fn byte_property(rlp: &mut CoreParser) -> Result<HeaderProp, ParseError> {
    // It's unknown (to me at least) why the byte property has two strings in it.
    rlp.take(8, |_d| ())?;
    match rlp.parse_str()? {
        "OnlinePlatform_Steam" | "OnlinePlatform_PS4" => Ok(()),
        _ => rlp.parse_str().map(|_| ()),
    }?;
    Ok(HeaderProp::Byte)
}

fn str_property(rlp: &mut CoreParser) -> Result<HeaderProp, ParseError> {
    rlp.take(8, |_d| ())?;
    Ok(HeaderProp::Str(rlp.parse_text()?))
}

fn name_property(rlp: &mut CoreParser) -> Result<HeaderProp, ParseError> {
    rlp.take(8, |_d| ())?;
    Ok(HeaderProp::Name(rlp.parse_text()?))
}

fn int_property(rlp: &mut CoreParser) -> Result<HeaderProp, ParseError> {
    rlp.take(12, |d| HeaderProp::Int(le_i32(&d[8..])))
}

fn bool_property(rlp: &mut CoreParser) -> Result<HeaderProp, ParseError> {
    rlp.take(9, |d| HeaderProp::Bool(d[8] == 1))
}

fn float_property(rlp: &mut CoreParser) -> Result<HeaderProp, ParseError> {
    rlp.take(12, |d| HeaderProp::Float(le_f32(&d[8..])))
}

fn qword_property(rlp: &mut CoreParser) -> Result<HeaderProp, ParseError> {
    rlp.take(16, |d| HeaderProp::QWord(le_u64(&d[8..])))
}

fn array_property(rlp: &mut CoreParser) -> Result<HeaderProp, ParseError> {
    let size = rlp.take(12, |d| le_i32(&d[8..]))?;
    let arr = CoreParser::repeat(size as usize, || parse_rdict(rlp))?;
    Ok(HeaderProp::Array(arr))
}

#[cfg(test)]
mod tests {
    use crate::core_parser::CoreParser;

    use super::*;

    #[test]
    fn rdict_no_elements() {
        let data = [0x05, 0x00, 0x00, 0x00, b'N', b'o', b'n', b'e', 0x00];
        let mut parser = CoreParser::new(&data[..]);
        let res = parse_rdict(&mut parser).unwrap();
        assert_eq!(res, Vec::new());
    }

    #[test]
    fn rdict_one_element() {
        // dd skip=$((0x1269)) count=$((0x12a8 - 0x1269)) if=rumble.replay of=rdict_one.replay bs=1
        let data = include_bytes!("../assets/replays/partial/rdict_one.replay");
        let mut parser = CoreParser::new(&data[..]);
        let res = parse_rdict(&mut parser).unwrap();
        assert_eq!(
            res,
            vec![(String::from("PlayerName"), HeaderProp::Str(String::from("comagoosie")))]
        );
    }

    #[test]
    fn rdict_one_int_element() {
        // dd skip=$((0x250)) count=$((0x284 - 0x250)) if=rumble.replay of=rdict_int.replay bs=1
        let data = include_bytes!("../assets/replays/partial/rdict_int.replay");
        let mut parser = CoreParser::new(&data[..]);
        let res = parse_rdict(&mut parser).unwrap();
        assert_eq!(res, vec![(String::from("PlayerTeam"), HeaderProp::Int(0))]);
    }

    #[test]
    fn rdict_one_bool_element() {
        // dd skip=$((0xa0f)) count=$((0xa3b - 0xa0f)) if=rumble.replay of=rdict_bool.replay bs=1
        let data = include_bytes!("../assets/replays/partial/rdict_bool.replay");
        let mut parser = CoreParser::new(&data[..]);
        let res = parse_rdict(&mut parser).unwrap();
        assert_eq!(res, vec![(String::from("bBot"), HeaderProp::Bool(false))]);
    }

    fn append_none(input: &[u8]) -> Vec<u8> {
        let append = [0x05, 0x00, 0x00, 0x00, b'N', b'o', b'n', b'e', 0x00];
        let mut v = Vec::new();
        v.extend_from_slice(input);
        v.extend_from_slice(&append);
        v
    }

    #[test]
    fn rdict_one_name_element() {
        // dd skip=$((0x1237)) count=$((0x1269 - 0x1237)) if=rumble.replay of=rdict_name.replay bs=1
        let data = append_none(include_bytes!(
            "../assets/replays/partial/rdict_name.replay"
        ));
        let mut parser = CoreParser::new(&data[..]);
        let res = parse_rdict(&mut parser).unwrap();
        assert_eq!(
            res,
            vec![(String::from("MatchType"), HeaderProp::Name(String::from("Online")))]
        );
    }

    #[test]
    fn rdict_one_float_element() {
        // dd skip=$((0x10a2)) count=$((0x10ce - 0x10a2)) if=rumble.replay of=rdict_float.replay bs=1
        let data = append_none(include_bytes!(
            "../assets/replays/partial/rdict_float.replay"
        ));
        let mut parser = CoreParser::new(&data[..]);
        let res = parse_rdict(&mut parser).unwrap();
        assert_eq!(res, vec![(String::from("RecordFPS"), HeaderProp::Float(30.0))]);
    }

    #[test]
    fn rdict_one_qword_element() {
        // dd skip=$((0x576)) count=$((0x5a5 - 0x576)) if=rumble.replay of=rdict_qword.replay bs=1
        let data = append_none(include_bytes!(
            "../assets/replays/partial/rdict_qword.replay"
        ));
        let mut parser = CoreParser::new(&data[..]);
        let res = parse_rdict(&mut parser).unwrap();
        assert_eq!(
            res,
            vec![(String::from("OnlineID"), HeaderProp::QWord(76561198101748375))]
        );
    }

    #[test]
    fn rdict_one_array_element() {
        // dd skip=$((0xab)) count=$((0x3f7 + 36)) if=rumble.replay of=rdict_array.replay bs=1
        let data = append_none(include_bytes!(
            "../assets/replays/partial/rdict_array.replay"
        ));
        let mut parser = CoreParser::new(&data[..]);
        let res = parse_rdict(&mut parser).unwrap();
        let expected = vec![
            vec![
                (String::from("frame"), HeaderProp::Int(441)),
                (String::from("PlayerName"), HeaderProp::Str(String::from("Cakeboss"))),
                (String::from("PlayerTeam"), HeaderProp::Int(1)),
            ],
            vec![
                (String::from("frame"), HeaderProp::Int(1738)),
                (String::from("PlayerName"), HeaderProp::Str(String::from("Sasha Kaun"))),
                (String::from("PlayerTeam"), HeaderProp::Int(0)),
            ],
            vec![
                (String::from("frame"), HeaderProp::Int(3504)),
                (
                    String::from("PlayerName"),
                    HeaderProp::Str(String::from("SilentWarrior")),
                ),
                (String::from("PlayerTeam"), HeaderProp::Int(0)),
            ],
            vec![
                (String::from("frame"), HeaderProp::Int(5058)),
                (String::from("PlayerName"), HeaderProp::Str(String::from("jeffreyj1"))),
                (String::from("PlayerTeam"), HeaderProp::Int(1)),
            ],
            vec![
                (String::from("frame"), HeaderProp::Int(5751)),
                (String::from("PlayerName"), HeaderProp::Str(String::from("GOOSE LORD"))),
                (String::from("PlayerTeam"), HeaderProp::Int(0)),
            ],
            vec![
                (String::from("frame"), HeaderProp::Int(6083)),
                (String::from("PlayerName"), HeaderProp::Str(String::from("GOOSE LORD"))),
                (String::from("PlayerTeam"), HeaderProp::Int(0)),
            ],
            vec![
                (String::from("frame"), HeaderProp::Int(7021)),
                (
                    String::from("PlayerName"),
                    HeaderProp::Str(String::from("SilentWarrior")),
                ),
                (String::from("PlayerTeam"), HeaderProp::Int(0)),
            ],
        ];
        assert_eq!(res, vec![(String::from("Goals"), HeaderProp::Array(expected))]);
    }

    #[test]
    fn rdict_one_byte_element() {
        // dd skip=$((0xdf0)) count=$((0xe41 - 0xdf0)) if=rumble.replay of=rdict_byte.replay bs=1
        let data = append_none(include_bytes!(
            "../assets/replays/partial/rdict_byte.replay"
        ));
        let mut parser = CoreParser::new(&data[..]);
        let res = parse_rdict(&mut parser).unwrap();
        assert_eq!(res, vec![(String::from("Platform"), HeaderProp::Byte)]);
    }

    #[test]
    fn rdict_unrecognized_property() {
        // dd skip=$((0xdf0)) count=$((0xe41 - 0xdf0)) if=rumble.replay of=rdict_byte.replay bs=1
        let data = append_none(include_bytes!(
            "../assets/replays/partial/rdict_unrecognized.replay"
        ));
        let mut parser = CoreParser::new(&data[..]);
        let res = parse_rdict(&mut parser).unwrap_err();
        assert_eq!(res.to_string(), String::from("Did not expect a property of: BiteProperty"));
    }

    #[test]
    fn rdict_ps4_online_id() {
        let data = append_none(include_bytes!(
            "../assets/replays/partial/rdict_ps4_online_id.replay"
        ));
        let mut parser = CoreParser::new(&data[..]);
        let res = parse_rdict(&mut parser).unwrap();
        assert_eq!(res.len(), 14);
        assert_eq!(res[0], (String::from("TeamSize"), HeaderProp::Int(3)));
    }
}
