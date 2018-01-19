use std::str;

#[derive(PartialEq, Debug, Clone, Fail)]
pub enum ParseError {
    #[fail(display = "A size of zero is not valid")] ZeroSize,

    #[fail(display = "Unable decode data as utf8: {}", _0)] Utf8Error(#[cause] str::Utf8Error),

    #[fail(display = "Text of size {} is too large", _0)] TextTooLarge(i32),

    #[fail(display = "Insufficient data. Expected {} bytes, but only {} left", _0, _1)]
    InsufficientData(i32, i32),

    #[fail(display = "Did not expect a property of: {}", _0)] UnexpectedProperty(String),

    #[fail(display = "Crc mismatch. Expected {} but received {}", _0, _1)] CrcMismatch(u32, u32),

    #[fail(display = "list of size {} is too large", _0)] ListTooLarge(usize),
}

impl From<str::Utf8Error> for ParseError {
    fn from(error: str::Utf8Error) -> Self {
        ParseError::Utf8Error(error)
    }
}

#[derive(PartialEq, Debug, Clone, Fail)]
pub enum AttributeError {
    #[fail(display = "Not enough data to decode attribute {}", _0)]
    NotEnoughDataFor(&'static str),

    #[fail(display = "Unrecognized remote id of {}", _0)]
    UnrecognizedRemoteId(u8),

    #[fail(display = "Does not have an attribute implementation")]
    Unimplemented,
}

#[derive(PartialEq, Debug, Clone, Fail)]
pub enum NetworkError {
    #[fail(display = "Not enough data to decode {}", _0)]
    NotEnoughDataFor(&'static str),

    #[fail(display = "Time is out of range: {}", _0)]
    TimeOutOfRange(f32),

    #[fail(display = "Delta is out of range: {}", _0)]
    DeltaOutOfRange(f32),  

    #[fail(display = "Too many prop ids ({}) for object index: {}", _0, _1)]
    PropIdsTooLarge(i32, i32),  

    #[fail(display = "Number of channels exceeds maximum: {}", _0)]
    ChannelsTooLarge(i32),  

    #[fail(display = "Type Id of {} exceeds range", _0)]
    TypeIdOutOfRange(i32),  

    #[fail(display = "Stream id of {} references out of range object index: {}", _0, _1)]
    StreamTooLargeIndex(i32, i32),  

    #[fail(display = "Parent index of {} for object index of {} was not recognized to have attributes", _0, _1)]
    ParentIndexHasNoAttributes(i32, i32),  
}
