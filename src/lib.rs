#![allow(dead_code)]
#![allow(unused_imports)]

#[macro_use]
extern crate nom;

extern crate crc;

use nom::{HexDisplay,Needed,IResult,ErrorKind,le_i32,le_u32,le_u8,le_u16,length_value,FileProducer};
use nom::Err;
use nom::IResult::*;

struct A {
  a: u8,
  b: u8
}

enum RProp {
    Array(Box<[RProp]>),
    Bool(bool),
    Byte(u8),
    Float(f32),
    Int(u32),
    Name(String),
    QWord(u64),
    Str(String)
}

named!(length_encoded,
    chain!(
        size: le_u32 ~
        crc: le_u32 ~
        data: take!(size),
        || {data}
    )
);

/// Text is encoded with a leading int that denotes the number of bytes that
/// the text spans. The last byte in the text will be null terminated, so we trim
/// it off. It may seem redundant to store this information, but stackoverflow contains
/// a nice reasoning for why it may have been done this way:
/// http://stackoverflow.com/questions/6293457/why-are-c-net-strings-length-prefixed-and-null-terminated
named!(text_encoded<&[u8], &str>,
    chain!(
        size: le_u32 ~
        data: take_str!(size - 1) ~
        take!(1),
        || {data}
    )
);

named!(str_prop<&[u8], RProp>, map!(map!(text_encoded, str::to_string), RProp::Str));

named!(rprop_encoded<&[u8], RProp>,
  switch!(text_encoded,
    "ArrayProperty" => call!(str_prop) |
    "BoolProperty" => call!(str_prop) |
    "ByteProperty" => call!(str_prop)|
    "FloatProperty" => call!(str_prop) |
    "IntProperty" => call!(str_prop) |
    "NameProperty" => call!(str_prop) |
    "QWordProperty" => call!(str_prop) |
    "StrProperty" => call!(str_prop)
  )
);

named!(f<&[u8],A>,
    chain!(
        header_size:  le_i32 ~
        header_crc:   le_i32 ~
        header:       take!(header_size) ~
        content_size: le_i32 ~
        content_crc:  le_i32 ~
        content:      take!(content_size),
        || {A {a: 0, b: 0}}
    )
);


#[cfg(test)]
mod tests {
    use nom::IResult::{Done, Error, Incomplete};
    use nom::Needed::Size;

    #[test]
    fn missing_header_data() {
        let data = include_bytes!("../assets/rumble.replay");
        let r = super::length_encoded(&data[..8]);
        assert_eq!(r, Incomplete(Size(4776)));
    }

    #[test]
    fn incomplete_header_data() {
        let data = include_bytes!("../assets/rumble.replay");
        let r = super::length_encoded(&data[..9]);
        assert_eq!(r, Incomplete(Size(4776)));
    }

    #[test]
    fn missing_header() {
        let r = super::length_encoded(&[]);
        assert_eq!(r, Incomplete(Size(4)));
    }

    #[test]
    fn missing_crc_data() {
        let data = include_bytes!("../assets/rumble.replay");
        let r = super::length_encoded(&data[..4]);
        assert_eq!(r, Incomplete(Size(8)));
    }

    #[test]
    fn parse_a_header_with_zero_data() {
        let data = [0, 0, 0, 0, 0, 0, 0, 0];
        let r = super::length_encoded(&data);
        assert_eq!(r, Done(&[][..], &[][..]));
    }

    #[test]
    fn parse_text_encoding() {
        // dd skip=16 count=28 if=rumble.replay of=text.replay bs=1
        let data = include_bytes!("../assets/text.replay");
        let r = super::text_encoded(data);
        assert_eq!(r, Done(&[][..], "TAGame.Replay_Soccar_TA"));
    }
}
