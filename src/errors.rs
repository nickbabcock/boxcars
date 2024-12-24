use crate::data::ATTRIBUTES;
use crate::network::{ActorId, Frame, NewActor, ObjectId, StreamId, UpdatedAttribute};
use crate::{AttributeTag, CacheInfo};
use fnv::FnvHashMap;
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::str;

#[derive(PartialEq, Debug, Clone)]
pub enum ParseError {
    ParseError(&'static str, i32, Box<ParseError>),
    Utf8Error(str::Utf8Error),
    TextTooLarge(i32),
    InsufficientData(i32, i32),
    UnexpectedProperty(String),
    CrcMismatch(u32, u32),
    CorruptReplay(String, Box<ParseError>),
    ListTooLarge(usize),
    NetworkError(Box<NetworkError>),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
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
pub struct FrameContext {
    pub(crate) objects: Vec<String>,
    pub(crate) object_attributes: Vec<Option<CacheInfo>>,
    pub(crate) frames: Vec<Frame>,
    pub(crate) actors: FnvHashMap<ActorId, ObjectId>,
    pub(crate) new_actors: Vec<NewActor>,
    pub(crate) updated_actors: Vec<UpdatedAttribute>,
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

    fn display_new_actor(&self, f: &mut fmt::Formatter<'_>, actor: &NewActor) -> fmt::Result {
        writeln!(f)?;
        writeln!(f, "Last new actor:")?;
        writeln!(f, "{}", "-".repeat("Last new actor:".len()))?;
        writeln!(f, "actor id: {}", actor.actor_id)?;
        writeln!(
            f,
            "name id: {}",
            actor
                .name_id
                .map(|x| x.to_string())
                .unwrap_or_else(|| String::from("<none>"))
        )?;
        writeln!(f, "object id: {}", actor.object_id)?;
        writeln!(
            f,
            "object name: {}",
            self.object_ind_to_string(actor.object_id)
        )?;
        writeln!(f, "location: {:?}", actor.initial_trajectory.location)?;
        writeln!(f, "rotation: {:?}", actor.initial_trajectory.rotation)
    }

    fn display_update(&self, f: &mut fmt::Formatter<'_>, attr: &UpdatedAttribute) -> fmt::Result {
        writeln!(f)?;
        writeln!(f, "Last actor update:")?;
        writeln!(f, "{}", "-".repeat("Last actor update:".len()))?;

        writeln!(f, "actor id: {}", attr.actor_id)?;
        if let Some(object_id) = self.actors.get(&attr.actor_id) {
            writeln!(f, "object id: {}", object_id)?;
            writeln!(f, "object name: {}", self.object_ind_to_string(*object_id))?;
        } else {
            writeln!(f, "object id: <none>")?;
        };
        writeln!(f, "attribute stream id: {}", attr.stream_id)?;
        writeln!(f, "attribute object id: {}", attr.object_id)?;
        writeln!(
            f,
            "attribute object name: {}",
            self.object_ind_to_string(attr.object_id)
        )?;
        writeln!(f, "attribute: {:?}", attr.attribute)
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
        let last_frame = self.frames.last();
        writeln!(
            f,
            "Current frame: {} (time: {} delta: {})",
            self.frames.len(),
            last_frame.map(|x| x.time).unwrap_or_default(),
            last_frame.map(|x| x.delta).unwrap_or_default()
        )?;
        if let Some(updated) = self.updated_actors.last() {
            return self.display_update(f, updated);
        } else if let Some(new) = self.new_actors.last() {
            return self.display_new_actor(f, new);
        }

        writeln!(f, "- No actor information decoded")?;

        let Some((frame_idx, frame)) = self.most_recent_frame_with_data() else {
            return Ok(());
        };

        writeln!(
            f,
            "Retrace frame: {} (time: {}, delta: {})",
            frame_idx, frame.time, frame.delta
        )?;

        if let Some(updated) = frame.updated_actors.last() {
            self.display_update(f, updated)
        } else if let Some(new) = frame.new_actors.last() {
            self.display_new_actor(f, new)
        } else {
            writeln!(f, "- No actor information decoded")
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum FrameError {
    NotEnoughDataFor(&'static str),
    TimeOutOfRange {
        time: f32,
    },
    DeltaOutOfRange {
        delta: f32,
    },
    ObjectIdOutOfRange {
        obj: ObjectId,
    },
    MissingActor {
        actor: ActorId,
    },
    MissingCache {
        actor: ActorId,
        actor_object: ObjectId,
    },
    MissingAttribute {
        actor: ActorId,
        actor_object: ObjectId,
        attribute_stream: StreamId,
    },
    AttributeError {
        actor: ActorId,
        actor_object: ObjectId,
        attribute_stream: StreamId,
        error: AttributeError,
    },
}

impl FrameError {
    fn contextualize(&self, f: &mut fmt::Formatter<'_>, context: &FrameContext) -> fmt::Result {
        match self {
            FrameError::MissingCache { actor_object, .. } => {
                if let Some(name) = context.objects.get(usize::from(*actor_object)) {
                    write!(f, "({})", name)
                } else {
                    Ok(())
                }
            }
            FrameError::MissingAttribute {
                actor_object,
                attribute_stream,
                ..
            }
            | FrameError::AttributeError {
                actor_object,
                attribute_stream,
                ..
            } => {
                let actor_obj_name = context
                    .objects
                    .get(usize::from(*actor_object))
                    .map(|x| x.to_string())
                    .unwrap_or_else(|| String::from("<none>"));

                let objs_with_attr = context
                    .object_attributes
                    .iter()
                    .filter_map(|x| x.as_ref())
                    .filter_map(|c| c.attributes.get(*attribute_stream))
                    .collect::<Vec<_>>();

                let obj = context
                    .object_attributes
                    .get(usize::from(*actor_object))
                    .and_then(|x| x.as_ref())
                    .and_then(|x| x.attributes.get(*attribute_stream));

                if let Some(attr_obj) = obj {
                    if attr_obj.attribute != AttributeTag::NotImplemented {
                        write!(
                            f,
                            "found attribute ({:?}) on {} in network cache data. ",
                            attr_obj.attribute, actor_obj_name
                        )?;
                    } else {
                        writeln!(f, "attribute: {}", context.objects.get(usize::from(attr_obj.object_id)).map_or("<unknown>", |v| v))?;

                        // No need for further context so we return early.
                        return Ok(());
                    }
                } else {
                    write!(
                        f,
                        "did not find attribute id ({}) on {} in object hierarchy: ",
                        attribute_stream, actor_obj_name
                    )?;
                }

                let mut obj_attr_names = objs_with_attr
                    .iter()
                    .filter_map(|attr| context.objects.get(usize::from(attr.object_id)))
                    .collect::<Vec<_>>();

                obj_attr_names.sort();
                obj_attr_names.dedup();

                let mut unknown_attributes = obj_attr_names
                    .iter()
                    .filter(|x| !ATTRIBUTES.contains_key(x.as_str()))
                    .cloned()
                    .cloned()
                    .collect::<Vec<_>>();

                unknown_attributes.sort();
                unknown_attributes.dedup();

                let stringify_names = obj_attr_names
                    .iter()
                    .map(|attr_name| format!("({})", attr_name))
                    .collect::<Vec<_>>();

                write!(f, "searching all attributes with the same stream id, ")?;
                write!(
                    f,
                    "unknown attributes: [{}], ",
                    unknown_attributes.join(", ")
                )?;
                write!(
                    f,
                    "all attributes with that stream id: [{}]. ",
                    stringify_names.join(", ")
                )?;
                Ok(())
            }
            _ => Ok(()),
        }
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
        match self {
            FrameError::NotEnoughDataFor(message) => write!(f, "not enough data to decode {}", message),
            FrameError::TimeOutOfRange {time} => write!(f, "time is out of range: {}", time),
            FrameError::DeltaOutOfRange {delta} => write!(f, "delta is out of range: {}", delta),
            FrameError::ObjectIdOutOfRange {obj} => write!(f, "new actor object id out of range: {}", obj),
            FrameError::MissingActor {actor} => write!(f, "attribute update references unknown actor: {}", actor),
            FrameError::MissingCache {actor, actor_object} => write!(f, "no known attributes found for actor id / object id: {} / {}", actor, actor_object),
            FrameError::MissingAttribute {actor, actor_object, attribute_stream} =>{
                writeln!(f, "attribute unknown or not implemented:")?;
                writeln!(f, "{}", "-".repeat(10))?;
                writeln!(f, "actor id: {}", actor)?;
                writeln!(f, "actor object id: {}", actor_object)?;
                writeln!(f, "attribute stream id: {}", attribute_stream)
            },
            FrameError::AttributeError {actor, actor_object, attribute_stream, error} => write!(f, "attribute decoding error encountered: {} for actor id / actor object id / attribute id: {} / {} / {}", error, actor, actor_object, attribute_stream),
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum NetworkError {
    NotEnoughDataFor(&'static str),
    ObjectIdOutOfRange(ObjectId),
    StreamTooLargeIndex(i32, i32),
    MissingParentClass(String, String),
    ParentHasNoAttributes(ObjectId, ObjectId),
    FrameError(FrameError, Box<FrameContext>),
    TooManyFrames(i32),
}

impl Error for NetworkError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            NetworkError::FrameError(err, _) => Some(err),
            _ => None,
        }
    }
}

impl Display for NetworkError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            NetworkError::NotEnoughDataFor(message) => {
                write!(f, "Not enough data to decode {}", message)
            }
            NetworkError::ObjectIdOutOfRange(id) => write!(f, "Object Id of {} exceeds range", id),
            NetworkError::StreamTooLargeIndex(steam_id, object_index) => write!(
                f,
                "Stream id of {} references out of range object index: {}",
                steam_id, object_index
            ),
            NetworkError::MissingParentClass(obj, parent) => write!(
                f,
                "Replay contained object: {} but not the parent class: {}",
                obj, parent
            ),
            NetworkError::ParentHasNoAttributes(parent_id, object_id) => write!(
                f,
                "Parent id of {} for object id of {} was not recognized to have attributes",
                parent_id, object_id
            ),
            NetworkError::TooManyFrames(size) => write!(f, "Too many frames to decode: {}", size),
            NetworkError::FrameError(err, context) => {
                write!(f, "Error decoding frame: {}", err)?;
                if !matches!(err, FrameError::MissingAttribute { .. }) {
                    write!(f, ". ")?;
                }

                err.contextualize(f, context)?;
                writeln!(f)?;
                write!(f, "{}", context)
            }
        }
    }
}
