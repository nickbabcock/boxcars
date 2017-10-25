//! # Parsing
//!
//! A Rocket League game replay is a binary encoded file with an emphasis on little endian
//! encodings. The number 100 would be represented as the four byte sequence:
//!
//! ```plain
//! 0x64 0x00 0x00 0x00
//! ```
//!
//! This in contrast to big-endian, which would represent the number as:
//!
//! ```plain
//! 0x00 0x00 0x00 0x64
//! ```
//!
//! Remember, little endian means least significant bit first!
//!
//! Rust and nom makes the parsing easy and fast. A combination of Rust's language level features
//! and nom's syntatic macros make for concise implementations of parser combinators, which allow
//! for extremely composable statements.
//!
//! A replay is split into three major sections, a header, body, and footer.
//!
//! ## Header
//!
//! The first four bytes of a replay is the number of bytes that comprises the header. A length
//! prefixed integer is very common throughout a replay. This prefix may either be in reference to
//! the number of bytes an elements takes up, as just seen, or the number of elements in a list.
//!
//! The next four bytes make up the [cyclic redundancy check
//! (CRC)](https://en.wikipedia.org/wiki/Cyclic_redundancy_check) for the header. The check ensures
//! that the data has not be tampered with or, more likely, corrupted. Unfortunately, it remains an
//! outstanding issue to implement this check. I tried utilizing
//! [crc-rs](https://github.com/mrhooray/crc-rs) with [community-calculated
//! parameters](https://github.com/tfausak/octane/issues/10#issuecomment-226910062), but didn't get
//! anywhere.
//!
//! The game's major and minor version follow, each 32bit integers.
//!
//! Subsequently, the game type is encoded as a string. Strings in Rocket League Replay files are
//! length prefixed and null terminated.
//!
//! The properties is where all the good nuggets of info reside. Visualize the properties as a map
//! of strings to various types (number, string, array) that continues until a "None" key is found.
//!
//! ## Body
//!
//! The body is the least implemented section, but it contains some familiar notions, such as
//! length prefixed data strutures.
//!
//! Out of the body we get:
//!
//! - Levels (what level did the match take place)
//! - `KeyFrames`
//! - The body's crc. This check is actually for the rest of the content (including the footer).
//!
//! Since everything is length prefixed, we're able to skip the network stream data. This would be
//! 90% of the file, and it's a shame that my enthusiasm for implementing this section waned. When
//! the developers of the game say the section isn't easy to parse, the major rocket league
//! libraries dedicate half of their code to parsing the section, and the with each patch
//! everything breaks, it's an incredible feat for anyone to retain enthusiam. Way to go
//! maintainers!
//!
//! Most of the interesting bits like player stats and goals are contained in the header, so it's
//! not a tremendous loss if we can't parse the network data. If we were able to parse the network
//! data, it would allow us to run benchmark against other implementations. Octane's readme states:
//!
//! > Octane parses most replays in less than 5 seconds.
//!
//! Which is what initially got me curious if, utilizing the right tools, I could do better.
//! Running Octane on `assets/rumble.replay` found in the repo, it decoded the information and
//! converted it to JSON in 2.3s. Considering the file is 1MB, I saw room for improvement. Using
//! the implementation here to output the header and footer data in JSON took 1ms. Yes, this is not an
//! apples to apples comparison, and one should continue using proven tools, not some example
//! project, but if I were to extrapolate, there isn't 1000x additional work needed.
//!
//! ## Footer
//!
//! After the network stream there isn't too much of interest to us, as it relates more to the
//! network stream, but there was a low barrier to parse it. From the footer we see:
//!
//! - Debug info
//! - Tickmarks
//! - Followed by several string info and other classes that seem totally worthless if the network
//! data isn't parsed

use nom::{self, IResult, le_f32, le_i32, le_u32, le_u64, le_u8};
use encoding::{DecoderTrap, Encoding};
use encoding::all::{UTF_16LE, WINDOWS_1252};
use models::*;
use crc::calc_crc;
use errors::*;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum BoxcarError {
    Code(u32),
    TextIncomplete,
    TextString,
    CrcIncomplete,
    UnexpectedCrc { expected: u32, actual: u32 },
    Utf16,
    Windows1252,
    ArrayProp,
    BoolProp,
    ByteProp,
    FloatProp,
    IntProp,
    NameProp,
    QWordProp,
    StrProp,
}

use BoxcarError::*;

/// Text is encoded with a leading int that denotes the number of bytes that
/// the text spans.
named!(text_encoded<&[u8], &str, BoxcarError>,
    return_error!(nom::ErrorKind::Custom(TextIncomplete),
    fix_error!(BoxcarError,
    complete!(do_parse!(size: le_u32 >> data: apply!(decode_str, size) >> (data))))));

/// Reads a string of a given size from the data. The size includes a null
/// character as the last character, so we drop it in the returned string
/// slice. It may seem redundant to store this information, but stackoverflow
/// contains a nice reasoning for why it may have been done this way:
/// http://stackoverflow.com/q/6293457/433785
fn decode_str(input: &[u8], size: u32) -> IResult<&[u8], &str> {
    if size == 0 {
        // TODO: This magic number represents that the string is too short
        IResult::Error(nom::Err::Code(nom::ErrorKind::Custom(3435)))
    } else {
        do_parse!(input, data: take_str!(size - 1) >> take!(1) >> (data))
    }
}

/// Decode a byte slice as UTF-16 into Rust's UTF-8 string. If unknown or
/// invalid UTF-16 sequences are encountered, ignore them.
fn decode_utf16(input: &[u8]) -> String {
    UTF_16LE.decode(input, DecoderTrap::Ignore).unwrap()
}

/// Decode a byte slice as Windows 1252 into Rust's UTF-8 string. If unknown or
/// invalid Windows 1252 sequences are encountered, ignore them.
fn decode_windows1252(input: &[u8]) -> String {
    WINDOWS_1252.decode(input, DecoderTrap::Ignore).unwrap()
}

/// Given a slice of the data and the number of characters contained, decode
/// into a `String`. If the size is negative, that means we're dealing with a
/// UTF-16 string, else it's a regular string.
fn inner_text(input: &[u8], size: i32) -> IResult<&[u8], String> {
    // size.abs() will panic at min_value, so we eschew it for manual checking
    if size > 10000 || size < -10000 || size == 0 {
        // TODO: This magic number represents that the string is too long
        IResult::Error(nom::Err::Code(nom::ErrorKind::Custom(3434)))
    } else if size < 0 {
        // We're dealing with UTF-16 and each character is two bytes, we
        // multiply the size by 2. The last two bytes included in the count are
        // null terminators, we trim those off.
        do_parse!(input,
          data: map!(take!(size * -2 - 2), decode_utf16) >>
          take!(2) >>
          (data))
    } else {
        do_parse!(input,
          data: map!(take!(size - 1), decode_windows1252) >>
          take!(1) >>
          (data))
    }
}

/// The first four bytes are the number of characters in the string and rest is
/// the contents of the string.
named!(text_string<&[u8], String, BoxcarError>,
    return_error!(nom::ErrorKind::Custom(TextString),
    fix_error!(BoxcarError,
    complete!(
    do_parse!(size: le_i32 >> data: apply!(inner_text, size) >> (data))))));

/// Header properties are encoded in a pretty simple format, with some oddities. The first 64bits
/// is data that can be discarded, some people think that the 64bits is the length of the data
/// while others think that the first 32bits is the header length in bytes with the subsequent
/// 32bits unknown. Doesn't matter to us, we throw it out anyways. The rest of the bytes are
/// decoded property type specific.

named!(str_prop<&[u8], HeaderProp, BoxcarError>,
  return_error!(nom::ErrorKind::Custom(StrProp),
  complete!(
  do_parse!(fix_error!(BoxcarError, le_u64) >>
            x: text_string >> (HeaderProp::Str(x))))));

named!(name_prop<&[u8], HeaderProp, BoxcarError>,
  complete!(
  do_parse!(fix_error!(BoxcarError, le_u64) >>
            x: text_string >> (HeaderProp::Name(x)))));

named!(int_prop<&[u8], HeaderProp, BoxcarError>,
  fix_error!(BoxcarError,
  complete!(
  do_parse!(le_u64 >> x: le_u32 >> (HeaderProp::Int(x))))));

named!(bool_prop<&[u8], HeaderProp, BoxcarError>,
  fix_error!(BoxcarError,
  complete!(
  do_parse!(le_u64 >> x: le_u8 >> (HeaderProp::Bool(x == 1))))));

named!(float_prop<&[u8], HeaderProp, BoxcarError>,
  fix_error!(BoxcarError,
  complete!(
  do_parse!(le_u64 >> x: le_f32 >> (HeaderProp::Float(x))))));

named!(qword_prop<&[u8], HeaderProp, BoxcarError>,
  fix_error!(BoxcarError,
  complete!(
  do_parse!(le_u64 >> x: le_u64 >> (HeaderProp::QWord(x))))));

/// The byte property is the odd one out. It's two strings following each other. No rhyme or
/// reason.
named!(byte_prop<&[u8], HeaderProp, BoxcarError>,
  do_parse!(fix_error!(BoxcarError, le_u64) >>
    text_encoded >> text_encoded >> (HeaderProp::Byte)));

/// The array property has the same leading 64bits that are discarded but also contains the length
/// of the aray as the next 32bits. Each element in the array is a dictionary so we decode `size`
/// number of dictionaries.
named!(array_prop<&[u8], HeaderProp, BoxcarError>,
    fix_error!(BoxcarError,
    complete!(
    do_parse!(
        le_u64 >>
        size: le_u32 >>
        elems: count!(rdict, size as usize) >>
        (HeaderProp::Array(elems))))));

/// The next string in the data tells us how to decode the property and what type it is.
named!(rprop_encoded<&[u8], HeaderProp, BoxcarError>,
  switch!(text_encoded,
    "ArrayProperty" => call!(array_prop) |
    "BoolProperty" => call!(bool_prop) |
    "ByteProperty" => call!(byte_prop) |
    "FloatProperty" => call!(float_prop) |
    "IntProperty" => call!(int_prop) |
    "NameProperty" => call!(name_prop) |
    "QWordProperty" => call!(qword_prop) |
    "StrProperty" => call!(str_prop)
  )
);

/// Other the actual network data, the header property associative array is the hardest to parse.
/// The format is to:
/// - Read string
/// - If string is "None", we're done
/// - else we're dealing with a property, and the string just read is the key. Now deserialize the
///   value.
/// The return type of this function is a key value vector because since there is no format
/// specification, we can't rule out duplicate keys. Possibly consider a multi-map in the future.
fn rdict(input: &[u8]) -> IResult<&[u8], Vec<(String, HeaderProp)>, BoxcarError> {
    let mut v: Vec<(String, HeaderProp)> = Vec::new();

    // Initialize to a dummy value to avoid unitialized errors
    let mut res: IResult<&[u8], Vec<(String, HeaderProp)>, BoxcarError> = IResult::Done(input, Vec::new());

    // Done only if we see an error or if we see "None"
    let mut done = false;

    // Keeps track of where we currently are in the slice.
    let mut cslice = input;

    while !done {
        match text_encoded(cslice) {
            IResult::Done(i, txt) => {
                cslice = i;
                match txt {
                    "None" => done = true,
                    _ => match rprop_encoded(cslice) {
                        IResult::Done(inp, val) => {
                            cslice = inp;
                            v.push((txt.to_string(), val));
                        }
                        IResult::Incomplete(a) => {
                            res = IResult::Incomplete(a);
                            done = true
                        }
                        IResult::Error(a) => {
                            res = IResult::Error(a);
                            done = true
                        }
                    },
                }
            }

            IResult::Incomplete(a) => {
                done = true;
                res = IResult::Incomplete(a);
            }

            IResult::Error(a) => {
                done = true;
                res = IResult::Error(a);
            }
        }
    }

    match res {
        IResult::Done(_, _) => IResult::Done(cslice, v),
        _ => res,
    }
}

pub fn parse(input: &[u8], crc_check: bool) -> Result<Replay> {
    if crc_check {
        full_crc_check(input).to_result()
            .and_then(|_| data_parse(input).to_result())
            .map_err(|e| Error::from(ErrorKind::Parsing(format!("error list: {}", &e))))
    } else {
        data_parse(input).to_result()
            .map_err(|e| Error::from(ErrorKind::Parsing(format!("error list: {}", &e))))
    }
}

named!(data_parse<&[u8], Replay, BoxcarError>,
    complete!(do_parse!(
        header_size:  fix_error!(BoxcarError, le_u32) >>
        header_crc:   fix_error!(BoxcarError, le_u32) >>
        major_version: fix_error!(BoxcarError, le_u32) >>
        minor_version: fix_error!(BoxcarError, le_u32) >>
        game_type: text_encoded >>
        properties: rdict >>
        content_size: fix_error!(BoxcarError, le_u32) >>
        content_crc: fix_error!(BoxcarError, le_u32) >>
        levels: text_list >>
        keyframes: keyframe_list >>
        network_size: fix_error!(BoxcarError, le_u32) >>

// This is where this example falls short is that decoding the network data is not
// implemented. See Octane or RocketLeagueReplayParser for more info.
        fix_error!(BoxcarError, take!(network_size)) >>
        debug_info: debuginfo_list >>
        tick_marks: tickmark_list >>
        packages: text_list >>
        objects: text_list >>
        names: text_list >>
        class_indices: classindex_list >>
        net_cache: classnetcache_list >>

        (Replay {
          header_size: header_size,
          header_crc: header_crc,
          major_version: major_version,
          minor_version: minor_version,
          game_type: game_type.to_string(),
          properties: properties,
          content_size: content_size,
          content_crc: content_crc,
          levels: levels,
          keyframes: keyframes,
          debug_info: debug_info,
          tick_marks: tick_marks,
          packages: packages,
          objects: objects,
          names: names,
          class_indices: class_indices,
          net_cache: net_cache
        })
    ))
);

/// Below are a series of decoding functions that take in data and returns some domain object (eg:
/// `TickMark`, `KeyFrame`, etc.

named!(keyframe_encoded<&[u8], KeyFrame, BoxcarError>,
  fix_error!(BoxcarError,
  do_parse!(time: le_f32 >>
           frame: le_u32 >>
           position: le_u32 >>
           (KeyFrame {time: time, frame: frame, position: position}))));

named!(debuginfo_encoded<&[u8], DebugInfo, BoxcarError>,
  do_parse!(frame: fix_error!(BoxcarError, le_u32) >> user: text_string >> text: text_string >>
    (DebugInfo { frame: frame, user: user, text: text })));

named!(tickmark_encoded<&[u8], TickMark, BoxcarError>,
  do_parse!(description: text_string >>
           frame: fix_error!(BoxcarError, le_u32) >>
           (TickMark {description: description, frame: frame})));

named!(classindex_encoded<&[u8], ClassIndex, BoxcarError>,
  do_parse!(class: text_string >> index: fix_error!(BoxcarError, le_u32) >>
    (ClassIndex { class: class, index: index })));

named!(cacheprop_encoded<&[u8], CacheProp, BoxcarError>,
  fix_error!(BoxcarError,
  do_parse!(index: le_u32 >> id: le_u32 >>
    (CacheProp { index: index, id: id }))));

named!(classnetcache_encoded<&[u8], ClassNetCache, BoxcarError>,
  fix_error!(BoxcarError,
  do_parse!(index: le_u32 >>
            parent_id: le_u32 >>
            id: le_u32 >>
            prop_size: le_u32 >>
            properties: count!(cacheprop_encoded, prop_size as usize) >>
            (ClassNetCache {
             index: index,
             parent_id: parent_id,
             id: id,
             properties: properties
            }))));

/// All the domain objects can be observed in a list that is initially prefixed by the length.
/// There may be a way to consolidate the implementations, but they're already currently concise.

named!(text_list<&[u8], Vec<String>, BoxcarError>,
  fix_error!(BoxcarError,
  do_parse!(size: le_u32 >> elems: count!(text_string, size as usize) >> (elems))));

named!(keyframe_list<&[u8], Vec<KeyFrame>, BoxcarError>,
  fix_error!(BoxcarError,
  do_parse!(size: le_u32 >> elems: count!(keyframe_encoded, size as usize) >> (elems))));

named!(debuginfo_list<&[u8], Vec<DebugInfo>, BoxcarError>,
  fix_error!(BoxcarError,
  do_parse!(size: le_u32 >> elems: count!(debuginfo_encoded, size as usize) >> (elems))));

named!(tickmark_list<&[u8], Vec<TickMark>, BoxcarError>,
  fix_error!(BoxcarError,
  do_parse!(size: le_u32 >> elems: count!(tickmark_encoded, size as usize) >> (elems))));

named!(classindex_list<&[u8], Vec<ClassIndex>, BoxcarError>,
  fix_error!(BoxcarError,
  do_parse!(size: le_u32 >> elems: count!(classindex_encoded, size as usize) >> (elems))));

named!(classnetcache_list<&[u8], Vec<ClassNetCache>, BoxcarError>,
  fix_error!(BoxcarError,
  do_parse!(size: le_u32 >> elems: count!(classnetcache_encoded, size as usize) >> (elems))));

/// Given a pair of expected crc value and data, perform crc on the data and return `Ok`
/// if the expected matched the actual, else an `Err`
fn confirm_crc(pair: (u32, &[u8])) -> IResult<&[u8], (), BoxcarError> {
    let (crc, data) = pair;
    let res = calc_crc(data);
    if res == crc {
        IResult::Done(data, ())
    } else {
        IResult::Error(nom::Err::Code(nom::ErrorKind::Custom(UnexpectedCrc { expected: crc, actual: res })))
    }
}

/// Gather the expected crc and data to perform the crc on in a tuple
named!(crc_gather<&[u8], (u32, &[u8])>,
    complete!(
    do_parse!(
        size: le_u32 >>
        crc: le_u32 >>
        data: take!(size) >>
        ((crc, data)))));

/// Extracts crc data and ensures that it is correct
named!(crc_check<&[u8], (), BoxcarError>,
  flat_map!(fix_error!(BoxcarError, crc_gather), confirm_crc));

/// A Rocket League replay is split into two parts with respect to crc calculation. The header and
/// body. Each section is prefixed by the length of the section and the expected crc.
named!(full_crc_check<&[u8], &[u8], BoxcarError>,
    fix_error!(BoxcarError,
    recognize!(do_parse!(crc_check >> crc_check >> (())))));


#[cfg(test)]
mod tests {
    use nom::IResult::Done;
    use nom::ErrorKind;
    use nom::error_to_list;
    use models::*;
    use models::HeaderProp::*;
    use super::BoxcarError::*;

    #[test]
    fn parse_text_encoding() {
        // dd skip=16 count=28 if=rumble.replay of=text.replay bs=1
        let data = include_bytes!("../assets/text.replay");
        let r = super::text_encoded(data);
        assert_eq!(r, Done(&[][..], "TAGame.Replay_Soccar_TA"));
    }

    #[test]
    fn parse_text_encoding_bad() {
        // dd skip=16 count=28 if=rumble.replay of=text.replay bs=1
        let data = include_bytes!("../assets/text.replay");
        let r = super::text_encoded(&data[..data.len() - 1]);
        let errors = r.unwrap_err();
        let v = error_to_list(&errors);
        assert_eq!(v[0], ErrorKind::Custom(TextIncomplete));
    }

    #[test]
    fn parse_text_zero_size() {
        let data = [0, 0, 0, 0, 0];
        let r = super::text_encoded(&data[..]);
        assert!(r.is_err())
    }

    #[test]
    fn parse_text_encoding_bad_2() {
        // Test for when there is not enough data to decode text length
        // dd skip=16 count=28 if=rumble.replay of=text.replay bs=1
        let data = include_bytes!("../assets/text.replay");
        let r = super::text_encoded(&data[..2]);
        let errors = r.unwrap_err();
        let v = error_to_list(&errors);
        assert_eq!(v[0], ErrorKind::Custom(TextIncomplete));
    }

    #[test]
    fn parse_utf16_string() {
        // dd skip=((0x120)) count=28 if=utf-16.replay of=utf-16-text.replay bs=1
        let data = include_bytes!("../assets/utf-16-text.replay");
        let r = super::text_string(data);
        assert_eq!(r, Done(&[][..], "\u{2623}D[e]!v1zz\u{2623}".to_string()));
    }

    #[test]
    fn test_windows1252_string() {
        let data = include_bytes!("../assets/windows_1252.replay");
        let actual = super::text_string(&data[0x1ad..0x1c4]);
        assert_eq!(actual, Done(&[][..], "caudillman6000\u{b3}(2)".to_string()));
    }

    /// Define behavior on invalid UTF-16 sequences.
    #[test]
    fn parse_invalid_utf16_string() {
        let data = [0xfd, 0xff, 0xff, 0xff, 0xd8, 0xd8, 0x00, 0x00, 0x00, 0x00];
        let r = super::text_string(&data);
        assert_eq!(r, Done(&[][..], "\u{0}".to_string()));
    }

    #[test]
    fn rdict_no_elements() {
        let data = [0x05, 0x00, 0x00, 0x00, b'N', b'o', b'n', b'e', 0x00];
        let r = super::rdict(&data);
        assert_eq!(r, Done(&[][..], Vec::new()));
    }

    #[test]
    fn rdict_one_element() {
        // dd skip=$((0x1269)) count=$((0x12a8 - 0x1269)) if=rumble.replay of=rdict_one.replay bs=1
        let data = include_bytes!("../assets/rdict_one.replay");
        let r = super::rdict(data);
        assert_eq!(r, Done(&[][..],  vec![("PlayerName".to_string(), Str("comagoosie".to_string()))]));
    }

    #[test]
    fn rdict_one_element_bad_str() {
        let mut data = Vec::new();
        data.extend([0x00; 8].iter().clone());
        data.extend([0x06, 0x00, 0x00, 0x00].iter().clone());
        data.extend(b"bobby");
        let r = super::str_prop(&data[..]);
        let errors = r.unwrap_err();
        let v = error_to_list(&errors);
        assert_eq!(v[0], ErrorKind::Custom(StrProp));
        assert_eq!(v[1], ErrorKind::Custom(TextString));
    }

    #[test]
    fn rdict_one_int_element() {
        // dd skip=$((0x250)) count=$((0x284 - 0x250)) if=rumble.replay of=rdict_int.replay bs=1
        let data = include_bytes!("../assets/rdict_int.replay");
        let r = super::rdict(data);
        assert_eq!(r, Done(&[][..], vec![("PlayerTeam".to_string(), Int(0))]));
    }

    #[test]
    fn rdict_one_bool_element() {
        // dd skip=$((0xa0f)) count=$((0xa3b - 0xa0f)) if=rumble.replay of=rdict_bool.replay bs=1
        let data = include_bytes!("../assets/rdict_bool.replay");
        let r = super::rdict(data);
        assert_eq!(r, Done(&[][..], vec![("bBot".to_string(), Bool(false))]));
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
        let data = append_none(include_bytes!("../assets/rdict_name.replay"));
        let r = super::rdict(&data);
        assert_eq!(r, Done(&[][..],  vec![("MatchType".to_string(), Name("Online".to_string()))]));

    }

    #[test]
    fn rdict_one_float_element() {
        // dd skip=$((0x10a2)) count=$((0x10ce - 0x10a2)) if=rumble.replay of=rdict_float.replay bs=1
        let data = append_none(include_bytes!("../assets/rdict_float.replay"));
        let r = super::rdict(&data);
        assert_eq!(r, Done(&[][..],  vec![("RecordFPS".to_string(), Float(30.0))]));
    }

    #[test]
    fn rdict_one_qword_element() {
        // dd skip=$((0x576)) count=$((0x5a5 - 0x576)) if=rumble.replay of=rdict_qword.replay bs=1
        let data = append_none(include_bytes!("../assets/rdict_qword.replay"));
        let r = super::rdict(&data);
        assert_eq!(r, Done(&[][..],  vec![("OnlineID".to_string(), QWord(76561198101748375))]));
    }

    #[test]
    fn rdict_one_array_element() {
        // dd skip=$((0xab)) count=$((0x3f7 + 36)) if=rumble.replay of=rdict_array.replay bs=1
        let data = append_none(include_bytes!("../assets/rdict_array.replay"));
        let r = super::rdict(&data);
        let expected = vec![
            vec![
                ("frame".to_string(), Int(441)),
                ("PlayerName".to_string(), Str("Cakeboss".to_string())),
                ("PlayerTeam".to_string(), Int(1))
            ], vec![
                ("frame".to_string(), Int(1738)),
                ("PlayerName".to_string(), Str("Sasha Kaun".to_string())),
                ("PlayerTeam".to_string(), Int(0))
            ], vec![
                ("frame".to_string(), Int(3504)),
                ("PlayerName".to_string(), Str("SilentWarrior".to_string())),
                ("PlayerTeam".to_string(), Int(0))
            ], vec![
                ("frame".to_string(), Int(5058)),
                ("PlayerName".to_string(), Str("jeffreyj1".to_string())),
                ("PlayerTeam".to_string(), Int(1))
            ], vec![
                ("frame".to_string(), Int(5751)),
                ("PlayerName".to_string(), Str("GOOSE LORD".to_string())),
                ("PlayerTeam".to_string(), Int(0))
            ], vec![
                ("frame".to_string(), Int(6083)),
                ("PlayerName".to_string(), Str("GOOSE LORD".to_string())),
                ("PlayerTeam".to_string(), Int(0))
            ], vec![
                ("frame".to_string(), Int(7021)),
                ("PlayerName".to_string(), Str("SilentWarrior".to_string())),
                ("PlayerTeam".to_string(), Int(0))
            ]
        ];
        assert_eq!(r, Done(&[][..],  vec![("Goals".to_string(), Array(expected))]));
    }

    #[test]
    fn rdict_one_byte_element() {
        // dd skip=$((0xdf0)) count=$((0xe41 - 0xdf0)) if=rumble.replay of=rdict_byte.replay bs=1
        let data = append_none(include_bytes!("../assets/rdict_byte.replay"));
        let r = super::rdict(&data);
        assert_eq!(r, Done(&[][..], vec![("Platform".to_string(), Byte)]));
    }


    #[test]
    fn key_frame_decode() {
        let data = include_bytes!("../assets/rumble.replay");
        let r = super::keyframe_encoded(&data[0x12da..0x12da + 12]);
        assert_eq!(r, Done(&[][..], KeyFrame { time: 16.297668, frame: 208, position: 137273 } ));
    }

    #[test]
    fn key_frame_list() {
        let data = include_bytes!("../assets/rumble.replay");

        // List is 2A long, each keyframe is 12 bytes. Then add four for list length = 508
        let r = super::keyframe_list(&data[0x12ca..0x12ca + 508]);
        match r {
            Done(i, val) => {
                let left: &[u8] = &[][..];

                // There are 42 key frames in this list
                assert_eq!(val.len(), 42);
                assert_eq!(i, left);
            }
            _ => {
                assert!(false);
            }
        }
    }

    #[test]
    fn tickmark_list() {
        let data = include_bytes!("../assets/rumble.replay");

        // 7 tick marks at 8 bytes + size of tick list
        let r = super::tickmark_list(&data[0xf6cce..0xf6d50]);
        match r {
            Done(i, val) => {
                let left: &[u8] = &[][..];

                // There are 7 tick marks in this list
                assert_eq!(val.len(), 7);
                assert_eq!(val[0], TickMark { description: "Team1Goal".to_string(), frame: 396 });
                assert_eq!(i, left);
            }
            _ => {
                assert!(false);
            }
        }
    }

    #[test]
    fn test_the_whole_shebang() {
        let data = include_bytes!("../assets/rumble.replay");
        let left: &[u8] = &[][..];
        match super::data_parse(data) {
            Done(i, _) => assert_eq!(i, left),
            _ => assert!(false),
        }
    }

    #[test]
    fn test_the_parsing_empty() {
        match super::parse(&[][..], false) {
            Ok(_) => assert!(false),
            _ => assert!(true),
        }
    }

    #[test]
    fn test_the_parsing_text_too_long() {
        let data = include_bytes!("../assets/fuzz-string-too-long.replay");
        assert!(super::parse(&data[..], false).is_err());
    }

    #[test]
    fn test_the_fuzz_corpus_abs_panic() {
        let data = include_bytes!("../assets/fuzz-corpus.replay");
        assert!(super::parse(&data[..], false).is_err());
    }

    #[test]
    fn test_the_whole_shebang_with_crc() {
        let data = include_bytes!("../assets/rumble.replay");
        match super::parse(data, true) {
            Ok(_) => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn test_crc_check_header() {
        let data = include_bytes!("../assets/rumble.replay");
        let r = super::crc_check(&data[..4776]);
        assert_eq!(r, Done(&[][..], ()));
    }

    #[test]
    fn test_crc_check_header_bad() {
        let mut data = include_bytes!("../assets/rumble.replay").to_vec();
        data[4775] = 100;
        let r = super::crc_check(&data[..4776]);
        let errors = r.unwrap_err();
        let v = error_to_list(&errors);
        assert_eq!(v[0], ErrorKind::Custom(UnexpectedCrc { expected: 337843175, actual: 2877465516 }));
    }

    #[test]
    fn test_crc_check_full() {
        let data = include_bytes!("../assets/rumble.replay");
        let r = super::full_crc_check(&data[..]);
        assert_eq!(r, Done(&[][..], &data[..]));
    }
}
