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

/// Reads a string of a given size from the data. The size includes a null
/// character as the last character, so we drop it in the returned string
/// slice. It may seem redundant to store this information, but stackoverflow
/// contains a nice reasoning for why it may have been done this way:
/// <http://stackoverflow.com/q/6293457/433785>
pub fn decode_str(input: &[u8]) -> Result<&str, ParseError> {
    let data = &input[..input.len().saturating_sub(1)];
    Ok(::std::str::from_utf8(data)?)
}

pub fn decode_utf16(input: &[u8]) -> Result<String, ParseError> {
    let data = &input[..input.len().saturating_sub(2)];
    let (s, _) = UTF_16LE.decode_without_bom_handling(data);
    Ok(String::from(s))
}

pub fn decode_windows1252(input: &[u8]) -> Result<String, ParseError> {
    let data = &input[..input.len().saturating_sub(1)];
    let (s, _) = WINDOWS_1252.decode_without_bom_handling(data);
    Ok(String::from(s))
}
