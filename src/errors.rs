use crate::network::{ActorId, Attribute, Frame, ObjectId, StreamId, Trajectory, UpdatedAttribute, NewActor};
use crate::data::ATTRIBUTES;
use crate::models::ClassNetCache;
use std::ops::Deref;
use fnv::FnvHashMap;
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct ContextObjectAttribute {
    obj_id: ObjectId,
    obj_name: String,
    prop_id: ObjectId,
    prop_name: String,
}

#[derive(PartialEq, Debug, Clone)]
pub struct FrameContext {
    pub objects: Vec<String>,
    pub net_cache: Vec<ClassNetCache>,
    pub frames: Vec<Frame>,
    pub actors: FnvHashMap<ActorId, ObjectId>,
    pub new_actors: Vec<NewActor>,
    pub updated_actors: Vec<UpdatedAttribute>,
}

impl FrameContext {
    fn object_ind_to_string(&self, object_id: ObjectId) -> String {
        String::from(
            self.objects
                .get(usize::from(object_id))
                .map(Deref::deref)
                .unwrap_or("Out of bounds"),
        )
    }

    fn properties_with_stream_id(&self, stream_id: StreamId) -> Vec<ContextObjectAttribute> {
        self.net_cache
            .iter()
            .map(|x| {
                x.properties
                    .iter()
                    .map(|prop| (x.object_ind, prop.object_ind, prop.stream_id))
                    .collect::<Vec<(i32, i32, i32)>>()
            })
            .flatten()
            .filter(|&(_obj_id, _prop_id, prop_stream_id)| StreamId(prop_stream_id) == stream_id)
            .map(|(obj_id, prop_id, _prop_stream_id)| {
                let obj_id = ObjectId(obj_id);
                let prop_id = ObjectId(prop_id);
                ContextObjectAttribute {
                    obj_id,
                    prop_id,
                    obj_name: self.object_ind_to_string(obj_id),
                    prop_name: self.object_ind_to_string(prop_id),
                }
            })
            .filter(|x| !ATTRIBUTES.contains_key(x.prop_name.as_str()))
            .collect()
    }

    fn display_new_actor(&self, f: &mut fmt::Formatter<'_>, actor: &NewActor) -> fmt::Result {
        write!(
            f,
            "(id: {}, nameId: {}, objId: {}, objName: {}, initial trajectory: {:?})",
            actor.actor_id,
            actor.name_id.map(|x| x.to_string()).unwrap_or_else(|| String::from("<none>")),
            actor.object_id,
            self.object_ind_to_string(actor.object_id),
            actor.initial_trajectory
        )
    }

    fn display_update(&self, f: &mut fmt::Formatter<'_>, attr: &UpdatedAttribute) -> fmt::Result {
        let actor_entry = self.actors.get(&attr.actor_id);
        let actor_obj_name = actor_entry.and_then(|x| self.objects.get(usize::from(*x)));
        let stream_obj_name = self.objects.get(usize::from(attr.object_id));

        write!(f, "(actor stream id / object id / name: {} / ", attr.actor_id)?;
        if let Some(actor_id) = actor_entry {
            write!(f, "{} / ", actor_id)
        } else {
            write!(f, "{} / ", "<none>")
        }?;

        if let Some(name) = actor_obj_name {
            write!(f, "{}, ", name)
        } else {
            write!(f, "{}, ", "<none>")
        }?;

        write!(f, "attribute stream id / object id / name: {} / {} / ", attr.stream_id, attr.object_id)?;

        if let Some(name) = stream_obj_name {
            write!(f, "{}", name)
        } else {
            write!(f, "{}", "<none>")
        }?;

        write!(f, ", attribute: {:?})", attr.attribute)
    }

    fn most_recent_frame_with_data(&self) -> Option<(usize, &Frame)> {
        self.frames
            .iter()
            .enumerate()
            .rev()
            .find(|(_, frame)| !frame.updated_actors.is_empty() || !frame.new_actors.is_empty())
    }
}

impl fmt::Display for FrameContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "on frame: {}, ", self.frames.len())?;
        if let Some(updated) = self.updated_actors.last() {
            write!(f, "last updated actor: ")?;
            self.display_update(f, updated)
        } else if let Some(new) = self.new_actors.last() {
            write!(f, "last new actor: ")?;
            self.display_new_actor(f, new)
        } else if let Some((frame_idx, frame)) = self.most_recent_frame_with_data() {
            write!(f, "backtracking to frame {}, ", frame_idx)?;
            if let Some(updated) = frame.updated_actors.last() {
                write!(f, "last updated actor: ")?;
                self.display_update(f, updated)
            } else if let Some(new) = frame.new_actors.last() {
                write!(f, "last new actor: ")?;
                self.display_new_actor(f, new)
            } else {
                write!(f, "it didn't decode anything")
            }
        } else {
            write!(f, "it didn't decode anything")
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum FrameError {
    NotEnoughDataFor(&'static str),
    TimeOutOfRange { time: f32 },
    DeltaOutOfRange { delta: f32 },
    ObjectIdOutOfRange { obj: ObjectId },
    MissingActor { actor: ActorId },
    MissingCache { actor: ActorId, actor_object: ObjectId },
    MissingAttribute { actor: ActorId, actor_object: ObjectId, attribute_stream: StreamId },
    AttributeError { actor: ActorId, actor_object: ObjectId, attribute_stream: StreamId, error: AttributeError },
}

impl FrameError {
    fn contextualize(&self, f: &mut fmt::Formatter<'_>, context: &FrameContext) -> fmt::Result {
        unimplemented!()
    }

}

impl Error for FrameError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            FrameError::AttributeError { error, .. } => Some(error),
            _ => None,
        }
    }
}

impl Display for FrameError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "frame error")
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
    FrameError(FrameError, FrameContext),
    TooManyFrames(i32),
}

impl Error for NetworkError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            NetworkError::AttributeError(attribute_error) => Some(attribute_error),
            NetworkError::FrameError(err, _) => Some(err),
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
            NetworkError::FrameError(err, context) => unimplemented!(),

        }
    }
}
