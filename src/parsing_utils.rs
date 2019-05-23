use crate::errors::ParseError;
use byteorder::{ByteOrder, LittleEndian};
use encoding_rs::{UTF_16LE, WINDOWS_1252};
use std::borrow::Cow;

#[inline]
pub fn le_i32(d: &[u8]) -> i32 {
    LittleEndian::read_i32(d)
}

#[inline]
pub fn le_f32(d: &[u8]) -> f32 {
    LittleEndian::read_f32(d)
}

#[inline]
pub fn le_u64(d: &[u8]) -> u64 {
    LittleEndian::read_u64(d)
}

const MULTIPLY_DE_BRUIJN_BIT_POSITION2: [u32; 32] = [
    0, 1, 28, 2, 29, 14, 24, 3, 30, 22, 20, 15, 25, 17, 4, 8, 31, 27, 13, 23, 21, 19, 16, 7, 26,
    12, 18, 6, 11, 5, 10, 9,
];

// https://graphics.stanford.edu/~seander/bithacks.html#IntegerLogDeBruijn
pub fn log2(v: u32) -> u32 {
    MULTIPLY_DE_BRUIJN_BIT_POSITION2[((v.wrapping_mul(0x077C_B531)) >> 27) as usize]
}

/// Reads a string of a given size from the data. The size includes a null
/// character as the last character, so we drop it in the returned string
/// slice. It may seem redundant to store this information, but stackoverflow
/// contains a nice reasoning for why it may have been done this way:
/// <http://stackoverflow.com/q/6293457/433785>
pub fn decode_str(input: &[u8]) -> Result<&str, ParseError> {
    if input.is_empty() {
        Err(ParseError::ZeroSize)
    } else {
        Ok(::std::str::from_utf8(&input[..input.len() - 1])?)
    }
}

pub fn decode_utf16(input: &[u8]) -> Result<Cow<'_, str>, ParseError> {
    if input.len() < 2 {
        Err(ParseError::ZeroSize)
    } else {
        let (s, _) = UTF_16LE.decode_without_bom_handling(&input[..input.len() - 2]);
        Ok(s)
    }
}

pub fn decode_windows1252(input: &[u8]) -> Result<Cow<'_, str>, ParseError> {
    if input.is_empty() {
        Err(ParseError::ZeroSize)
    } else {
        let (s, _) = WINDOWS_1252.decode_without_bom_handling(&input[..input.len() - 1]);
        Ok(s)
    }
}

pub fn err_str(bytes_read: i32, desc: &'static str, e: &ParseError) -> String {
    format!(
        "Could not decode replay {} at offset ({}): {}",
        desc, bytes_read, e
    )
}
