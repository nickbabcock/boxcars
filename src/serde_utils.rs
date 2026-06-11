use serde::de::{self, Visitor};
use serde::{Deserializer, Serializer};
use std::fmt::{self, Display};
use std::str::FromStr;

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

/// Inverse of [`display_it`]: parse a value from its `Display` representation (a
/// string). Using `deserialize_str` keeps this honest as the strict inverse of
/// [`display_it`] and works for non-self-describing formats too.
pub fn parse_it<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: FromStr,
    <T as FromStr>::Err: Display,
    D: Deserializer<'de>,
{
    struct DisplayVisitor<T>(std::marker::PhantomData<T>);

    impl<'de, T> Visitor<'de> for DisplayVisitor<T>
    where
        T: FromStr,
        <T as FromStr>::Err: Display,
    {
        type Value = T;

        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("a stringified integer")
        }

        fn visit_str<E: de::Error>(self, v: &str) -> Result<T, E> {
            v.parse().map_err(de::Error::custom)
        }
    }

    deserializer.deserialize_str(DisplayVisitor(std::marker::PhantomData))
}
