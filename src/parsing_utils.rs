use crate::errors::ParseError;
use encoding_rs::{UTF_16LE, WINDOWS_1252};
use std::convert::TryInto;

#[inline]
pub fn le_i32(d: &[u8]) -> i32 {
    i32::from_le_bytes(d[..4].try_into().unwrap())
}

#[inline]
pub fn le_f32(d: &[u8]) -> f32 {
    f32::from_le_bytes(d[..4].try_into().unwrap())
}

#[inline]
pub fn le_u64(d: &[u8]) -> u64 {
    u64::from_le_bytes(d[..8].try_into().unwrap())
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

pub fn decode_utf16(input: &[u8]) -> Result<String, ParseError> {
    if input.len() < 2 {
        Err(ParseError::ZeroSize)
    } else {
        let (s, _) = UTF_16LE.decode_without_bom_handling(&input[..input.len() - 2]);
        Ok(String::from(s))
    }
}

pub fn decode_windows1252(input: &[u8]) -> Result<String, ParseError> {
    if input.is_empty() {
        Err(ParseError::ZeroSize)
    } else {
        let (s, _) = WINDOWS_1252.decode_without_bom_handling(&input[..input.len() - 1]);
        Ok(String::from(s))
    }
}
