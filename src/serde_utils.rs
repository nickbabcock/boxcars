use serde::Serializer;
use std::fmt::Display;

/// For the times when the `Display` string is more appropriate than the default serialization
/// strategy. This function is useful for 64bit integers, as 64bit integers can't be represented
/// wholly in floating point notation, which is how Javascript views all numbers. Thus serialize
/// 64bit integers as strings so that downstream applications can decide on how best to interpret
/// large numbers. To give an example: the 64bit integer of 76561198122624102 is represented as
/// 76561198122624100.0 (off by 2) in 64bit floating point.
pub fn display_it<T, S>(data: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: Display,
    S: Serializer,
{
    serializer.collect_str(data)
}
