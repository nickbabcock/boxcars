use crate::network::{ActorId, Attribute, ObjectId, StreamId, Trajectory};
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::str;

#[derive(PartialEq, Debug, Clone)]
pub enum ParseError {
    ParseError(&'static str, i32, Box<ParseError>),
    ZeroSize,
    Utf8Error(str::Utf8Error),
    TextTooLarge(i32),
    InsufficientData(i32, i32),
    UnexpectedProperty(String),
    CrcMismatch(u32, u32),
    CorruptReplay(String, Box<ParseError>),
    ListTooLarge(usize),
    NetworkError(NetworkError),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ParseError::ZeroSize => write!(f, "A size of zero is not valid"),
            ParseError::Utf8Error(utf8_error) => {
                write!(f, "Unable decode data as utf8: {}", utf8_error)
            }
            ParseError::TextTooLarge(size) => write!(f, "Text of size {} is too large", size),
            ParseError::InsufficientData(expected, left) => write!(
                f,
                "Insufficient data. Expected {} bytes, but only {} left",
                expected, left
            ),
            ParseError::UnexpectedProperty(property) => {
                write!(f, "Did not expect a property of: {}", property)
            }
            ParseError::CrcMismatch(expected, found) => write!(
                f,
                "Crc mismatch. Expected {} but received {}",
                expected, found
            ),
            ParseError::CorruptReplay(section, _) => write!(
                f,
                "Failed to parse {} and crc check failed. Replay is corrupt",
                section
            ),
            ParseError::ListTooLarge(size) => write!(f, "list of size {} is too large", size),
            ParseError::ParseError(section, bytes_read, parse_error) => write!(
                f,
                "Could not decode replay {} at offset ({}): {}",
                section, bytes_read, parse_error
            ),
            ParseError::NetworkError(network_error) => write!(f, "{}", network_error),
        }
    }
}

impl Error for ParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ParseError::Utf8Error(utf8_error) => Some(utf8_error),
            ParseError::CorruptReplay(_, error) => Some(error),
            ParseError::ParseError(_, _, error) => Some(error),
            ParseError::NetworkError(error) => Some(error),
            _ => None,
        }
    }
}

impl From<str::Utf8Error> for ParseError {
    fn from(error: str::Utf8Error) -> Self {
        ParseError::Utf8Error(error)
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum AttributeError {
    NotEnoughDataFor(&'static str),
    UnrecognizedRemoteId(u8),
    Unimplemented,
    TooBigString(i32),
}

impl Error for AttributeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl Display for AttributeError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            AttributeError::NotEnoughDataFor(message) => {
                write!(f, "Not enough data to decode attribute {}", message)
            }
            AttributeError::UnrecognizedRemoteId(id) => {
                write!(f, "Unrecognized remote id of {}", id)
            }
            AttributeError::Unimplemented => write!(f, "Does not have an attribute implementation"),
            AttributeError::TooBigString(size) => write!(f, "Unexpected size for string: {}", size),
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum NetworkError {
    NotEnoughDataFor(&'static str),
    TimeOutOfRange(f32),
    TimeOutOfRangeUpdate(usize, usize, ActorId, StreamId, Attribute),
    TimeOutOfRangeNew(
        usize,
        usize,
        ActorId,
        Option<i32>,
        ObjectId,
        String,
        Trajectory,
    ),
    DeltaOutOfRange(f32),
    MaxStreamIdTooLarge(i32, ObjectId),
    ChannelsTooLarge(i32),
    ObjectIdOutOfRange(ObjectId),
    StreamTooLargeIndex(i32, i32),
    MissingParentClass(String, String),
    ParentHasNoAttributes(ObjectId, ObjectId),
    MissingActor(ActorId),
    MissingCache(ActorId, ObjectId, String),
    MissingAttribute(ActorId, ObjectId, String, StreamId, String),
    UnimplementedAttribute(ActorId, ObjectId, String, StreamId, String, String),
    AttributeError(AttributeError),
    TooManyFrames(i32),
}

impl Error for NetworkError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            NetworkError::AttributeError(attribute_error) => Some(attribute_error),
            _ => None,
        }
    }
}

impl Display for NetworkError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            NetworkError::NotEnoughDataFor(message) => write!(f, "Not enough data to decode {}", message),
            NetworkError::TimeOutOfRange(time) => write!(f, "Time is out of range: {}", time),
            NetworkError::TimeOutOfRangeUpdate(frame1, frame2, last_actor, stream_id, attribute) =>
                write!(f, "Time was out of range. Backtracking from frame: {} to {}, the last actor ({}) had a stream id of {}. This may mean that a new update of Rocket League updated this attribute. Decoded into {:?})",
                       frame1, frame2, last_actor, stream_id, attribute),
            NetworkError::TimeOutOfRangeNew(frame1, frame2, last_actor, name_id, object_id, id, trajectory) =>
                write!(f, "Time was out of range. Backtracking from frame: {} to {}, the last actor ({}) had a name id of {:?}, object id: {} ({}), and trajectory: {:?}. This may mean that a new update of Rocket League updated this object.",
                       frame1, frame2, last_actor, name_id, object_id, id, trajectory),
            NetworkError::DeltaOutOfRange(delta) => write!(f, "Delta is out of range: {}", delta),
             NetworkError::MaxStreamIdTooLarge(ids, object_id) => write!(f, "Too many stream ids ({}) for object id: {}", ids, object_id),
            NetworkError::ChannelsTooLarge(max) => write!(f, "Number of channels exceeds maximum: {}", max),
            NetworkError::ObjectIdOutOfRange(id) => write!(f, "Object Id of {} exceeds range", id),
            NetworkError::StreamTooLargeIndex(steam_id, object_index) => write!(f, "Stream id of {} references out of range object index: {}", steam_id, object_index),
            NetworkError::MissingParentClass(obj, parent) => write!(f, "Replay contained object: {} but not the parent class: {}", obj, parent),
            NetworkError::ParentHasNoAttributes(parent_id, object_id) => write!(f, "Parent id of {} for object id of {} was not recognized to have attributes", parent_id, object_id),
            NetworkError::MissingActor(actor_id) => write!(f, "Actor id: {} was not found", actor_id),
            NetworkError::MissingCache(actor_id, object_id, object) => write!(f, "Actor id: {} of object id: {} ({}) but no attributes found", actor_id, object_id, object),
            NetworkError::MissingAttribute(actor_id, object_id, object, stream_id, attributes) => write!(f, "Actor id: {} of object id: {} ({}) but stream id: {} not found in {}", actor_id, object_id, object, stream_id, attributes),
            NetworkError::UnimplementedAttribute(actor_id, object_id, object, steam_id, stream, attributes) =>
                write!(f, "Actor id: {} of object id: {} ({}) but stream id: {} ({}) was not implemented. Possible missing implementations for stream id {}\n{}", actor_id, object_id, object, steam_id, stream, object, attributes),
            NetworkError::AttributeError(attribute_error) => write!(f, "Attribute error: {}", attribute_error),
            NetworkError::TooManyFrames(size) => write!(f, "Too many frames to decode: {}", size),

        }
    }
}
