use serde::Serializer;
use std::fmt::Display;

/// For the times when the `Display` string is more appropriate than the default serialization
/// strategy
pub fn display_it<T, S>(data: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: Display,
    S: Serializer,
{
    serializer.collect_str(data)
}
