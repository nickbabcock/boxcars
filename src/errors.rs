use attributes::Attribute;
use network::{ActorId, ObjectId, StreamId};
use std::str;

#[derive(PartialEq, Debug, Clone, Fail)]
pub enum ParseError {
    #[fail(display = "A size of zero is not valid")]
    ZeroSize,

    #[fail(display = "Unable decode data as utf8: {}", _0)]
    Utf8Error(#[cause] str::Utf8Error),

    #[fail(display = "Text of size {} is too large", _0)]
    TextTooLarge(i32),

    #[fail(
        display = "Insufficient data. Expected {} bytes, but only {} left",
        _0, _1
    )]
    InsufficientData(i32, i32),

    #[fail(display = "Did not expect a property of: {}", _0)]
    UnexpectedProperty(String),

    #[fail(display = "Crc mismatch. Expected {} but received {}", _0, _1)]
    CrcMismatch(u32, u32),

    #[fail(display = "list of size {} is too large", _0)]
    ListTooLarge(usize),
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

    #[fail(display = "Unexpected size for string: {}", _0)]
    TooBigString(i32),
}

#[derive(PartialEq, Debug, Clone, Fail)]
pub enum NetworkError {
    #[fail(display = "Not enough data to decode {}", _0)]
    NotEnoughDataFor(&'static str),

    #[fail(display = "Time is out of range: {}", _0)]
    TimeOutOfRange(f32),

    #[fail(
        display = "Time was out of range. Backtracking from frame: {} to {}, the last actor ({}) had a stream id of {}. This may mean that a new update of Rocket League updated this attribute. Decoded into {:?}",
        _0, _1, _2, _3, _4
    )]
    TimeOutOfRangeUpdate(usize, usize, ActorId, StreamId, Attribute),

    #[fail(display = "Delta is out of range: {}", _0)]
    DeltaOutOfRange(f32),

    #[fail(display = "Too many stream ids ({}) for object id: {}", _0, _1)]
    MaxStreamIdTooLarge(i32, ObjectId),

    #[fail(display = "Number of channels exceeds maximum: {}", _0)]
    ChannelsTooLarge(i32),

    #[fail(display = "Object Id of {} exceeds range", _0)]
    ObjectIdOutOfRange(ObjectId),

    #[fail(
        display = "Stream id of {} references out of range object index: {}",
        _0, _1
    )]
    StreamTooLargeIndex(i32, i32),

    #[fail(
        display = "Replay contained object: {} but not the parent class: {}",
        _0, _1
    )]
    MissingParentClass(String, String),

    #[fail(
        display = "Parent id of {} for object id of {} was not recognized to have attributes",
        _0, _1
    )]
    ParentHasNoAttributes(ObjectId, ObjectId),

    #[fail(display = "Actor id: {} was not found", _0)]
    MissingActor(ActorId),

    #[fail(
        display = "Actor id: {} of object id: {} ({}) but no attributes found",
        _0, _1, _2
    )]
    MissingCache(ActorId, ObjectId, String),

    #[fail(
        display = "Actor id: {} of object id: {} ({}) but stream id: {} not found in {}",
        _0, _1, _2, _3, _4
    )]
    MissingAttribute(ActorId, ObjectId, String, StreamId, String),

    #[fail(
        display = "Actor id: {} of object id: {} ({}) but stream id: {} ({}) was not implemented. Possible missing implementations for stream id {}\n{}",
        _0, _1, _2, _3, _4, _3, _5
    )]
    UnimplementedAttribute(ActorId, ObjectId, String, StreamId, String, String),

    #[fail(display = "Attribute error: {}", _0)]
    AttributeError(#[cause] AttributeError),
}
