use crate::{bits::RlBits, network::attributes::Attribute};
use bitter::{BitReader, LittleEndianReader};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Vector3f {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3f {
    pub fn decode(bits: &mut LittleEndianReader<'_>, net_version: i32) -> Option<Vector3f> {
        Vector3i::decode(bits, net_version).map(|vec| Vector3f {
            x: (vec.x as f32) / 100.0,
            y: (vec.y as f32) / 100.0,
            z: (vec.z as f32) / 100.0,
        })
    }
}

/// An object's current vector
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Vector3i {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Vector3i {
    pub fn decode(bits: &mut LittleEndianReader<'_>, net_version: i32) -> Option<Vector3i> {
        if bits.has_bits_remaining(128) {
            unsafe { bits.refill_lookahead_unchecked() }
            let size_bits = bits.peek_bits_max_computed(4, if net_version >= 7 { 22 } else { 20 });
            let bias = 1 << (size_bits + 1);
            let bit_limit = (size_bits + 2) as u32;

            let dx = bits.peek_and_consume(bit_limit) as u32;
            unsafe { bits.refill_lookahead_unchecked() }
            let dy = bits.peek_and_consume(bit_limit) as u32;
            let dz = bits.peek_and_consume(bit_limit) as u32;

            Some(Vector3i {
                x: (dx as i32) - bias,
                y: (dy as i32) - bias,
                z: (dz as i32) - bias,
            })
        } else {
            let len = bits.refill_lookahead();
            if len < 5 {
                return None;
            }

            let size_bits = bits.peek_bits_max_computed(4, if net_version >= 7 { 22 } else { 20 });
            let bias = 1 << (size_bits + 1);
            let bit_limit = (size_bits + 2) as u32;

            if !bits.has_bits_remaining(3 * bit_limit as usize) {
                return None;
            }

            let dx = bits.peek_and_consume(bit_limit) as u32;

            let len = bits.refill_lookahead();
            debug_assert!(len >= bit_limit * 2);

            let dy = bits.peek_and_consume(bit_limit) as u32;
            let dz = bits.peek_and_consume(bit_limit) as u32;
            Some(Vector3i {
                x: (dx as i32) - bias,
                y: (dy as i32) - bias,
                z: (dz as i32) - bias,
            })
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Quaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quaternion {
    #[inline]
    fn unpack(val: u32) -> f32 {
        let max_quat = 1.0 / std::f32::consts::SQRT_2;
        let max_value = (1 << 18) - 1;
        let pos_range = (val as f32) / (max_value as f32);
        let range = (pos_range - 0.5) * 2.0;
        range * max_quat
    }

    #[inline]
    fn compressed_f32(bits: &mut LittleEndianReader<'_>) -> f32 {
        // algorithm from jjbott/RocketLeagueReplayParser.
        // Note that this code is heavily adapted. I noticed that there were branches that should
        // never execute. Specifically in jjbott implementation:
        //
        // ```
        // br.ReadFixedCompressedFloat(1, 16);
        // ```
        //
        // These values are hardcoded and this function is only used in one place. There's a branch
        // that compares these two hard coded numbers. I've removed said branch from this
        // implementation.
        //
        // Bakkes copied jjbott. Rattletrap is more in line here
        let res = bits.peek_and_consume(16) as i32;
        ((res + i32::from(std::i16::MIN)) as f32) * (std::i16::MAX as f32).recip()
    }

    pub fn decode_compressed(bits: &mut LittleEndianReader<'_>) -> Option<Self> {
        let len = bits.refill_lookahead();
        if len >= 3 * 16 {
            let x = Quaternion::compressed_f32(bits);
            let y = Quaternion::compressed_f32(bits);
            let z = Quaternion::compressed_f32(bits);
            Some(Quaternion { x, y, z, w: 0.0 })
        } else {
            None
        }
    }

    pub fn decode(bits: &mut LittleEndianReader<'_>) -> Option<Self> {
        let len = bits.refill_lookahead();
        if len < 2 + 3 * 18 {
            return None;
        }

        let largest = bits.peek_and_consume(2) as u32;
        let a = Quaternion::unpack(bits.peek_and_consume(18) as u32);
        let b = Quaternion::unpack(bits.peek_and_consume(18) as u32);
        let c = Quaternion::unpack(bits.peek_and_consume(18) as u32);
        let extra = (1.0 - (a * a) - (b * b) - (c * c)).sqrt();
        match largest {
            0 => Some(Quaternion {
                x: extra,
                y: a,
                z: b,
                w: c,
            }),
            1 => Some(Quaternion {
                x: a,
                y: extra,
                z: b,
                w: c,
            }),
            2 => Some(Quaternion {
                x: a,
                y: b,
                z: extra,
                w: c,
            }),
            3 => Some(Quaternion {
                x: a,
                y: b,
                z: c,
                w: extra,
            }),
            _ => unreachable!(),
        }
    }
}

/// An object's current rotation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Rotation {
    pub yaw: Option<i8>,
    pub pitch: Option<i8>,
    pub roll: Option<i8>,
}

impl Rotation {
    pub fn decode(bits: &mut LittleEndianReader<'_>) -> Option<Rotation> {
        let len = bits.refill_lookahead();
        if len >= 3 * 9 {
            let yaw = if bits.peek_and_consume(1) != 0 {
                Some(bits.peek_and_consume(8) as i8)
            } else {
                None
            };

            let pitch = if bits.peek_and_consume(1) != 0 {
                Some(bits.peek_and_consume(8) as i8)
            } else {
                None
            };

            let roll = if bits.peek_and_consume(1) != 0 {
                Some(bits.peek_and_consume(8) as i8)
            } else {
                None
            };

            Some(Rotation { yaw, pitch, roll })
        } else {
            let yaw = bits.if_get(LittleEndianReader::read_i8)?;
            let pitch = bits.if_get(LittleEndianReader::read_i8)?;
            let roll = bits.if_get(LittleEndianReader::read_i8)?;
            Some(Rotation { yaw, pitch, roll })
        }
    }
}

/// When a new actor spawns in rocket league it will either have a location, location and rotation,
/// or none of the above
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpawnTrajectory {
    None,
    Location,
    LocationAndRotation,
}

/// Notifies that an actor has had one of their properties updated (most likely their rigid body
/// state (location / rotation) has changed)
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct UpdatedAttribute {
    /// The actor that had an attribute updated
    pub actor_id: ActorId,

    /// The attribute stream id that was decoded
    pub stream_id: StreamId,

    /// The attribute's object id
    pub object_id: ObjectId,

    /// The actual data from the decoded attribute
    pub attribute: Attribute,
}

/// Contains the time and any new information that occurred during a frame
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Frame {
    /// The time in seconds that the frame is recorded at
    pub time: f32,

    /// Time difference between previous frame
    pub delta: f32,

    /// List of new actors seen during the frame
    pub new_actors: Vec<NewActor>,

    /// List of actor id's that are deleted / destroyed
    pub deleted_actors: Vec<ActorId>,

    /// List of properties updated on the actors
    pub updated_actors: Vec<UpdatedAttribute>,
}

/// A replay encodes a list of objects that appear in the network data. The index of an object in
/// this list is used as a key in many places: reconstructing the attribute hierarchy and new
/// actors in the network data.
#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash, Serialize)]
pub struct ObjectId(pub i32);

impl From<ObjectId> for i32 {
    fn from(x: ObjectId) -> i32 {
        x.0
    }
}

impl From<ObjectId> for usize {
    fn from(x: ObjectId) -> usize {
        x.0 as usize
    }
}

impl fmt::Display for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A `StreamId` is an attribute's object id in the network data. It is a more compressed form of
/// the object id. Whereas the an object id might need to take up 9 bits, a stream id may only take
/// up 6 bits.
#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash, Serialize)]
pub struct StreamId(pub i32);

impl From<StreamId> for i32 {
    fn from(x: StreamId) -> i32 {
        x.0
    }
}

impl fmt::Display for StreamId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// An actor in the network data stream. Could identify a ball, car, etc. Ids are not unique
/// across a replay (eg. an actor that is destroyed may have its id repurposed).
#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash, Serialize)]
pub struct ActorId(pub i32);

impl From<ActorId> for i32 {
    fn from(x: ActorId) -> i32 {
        x.0
    }
}

impl fmt::Display for ActorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Information for a new actor that appears in the game
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct NewActor {
    /// The id given to the new actor
    pub actor_id: ActorId,

    /// An name id
    pub name_id: Option<i32>,

    /// The actor's object id.
    pub object_id: ObjectId,

    /// The initial trajectory of the new actor
    pub initial_trajectory: Trajectory,
}

/// Contains the optional location and rotation of an object when it spawns
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Trajectory {
    pub location: Option<Vector3i>,
    pub rotation: Option<Rotation>,
}

impl Trajectory {
    pub fn from_spawn(
        bits: &mut LittleEndianReader<'_>,
        sp: SpawnTrajectory,
        net_version: i32,
    ) -> Option<Trajectory> {
        match sp {
            SpawnTrajectory::None => Some(Trajectory {
                location: None,
                rotation: None,
            }),

            SpawnTrajectory::Location => Vector3i::decode(bits, net_version).map(|v| Trajectory {
                location: Some(v),
                rotation: None,
            }),

            SpawnTrajectory::LocationAndRotation => {
                let v = Vector3i::decode(bits, net_version)?;
                let r = Rotation::decode(bits)?;
                Some(Trajectory {
                    location: Some(v),
                    rotation: Some(r),
                })
            }
        }
    }
}

/// Oftentimes a replay contains many different objects of the same type. For instance, each rumble
/// pickup item is of the same type but has a different name. The name of:
/// `stadium_foggy_p.TheWorld:PersistentLevel.VehiclePickup_Boost_TA_30` should be normalized to
/// `TheWorld:PersistentLevel.VehiclePickup_Boost_TA` so that we don't have to work around each
/// stadium and pickup that is released.
pub(crate) fn normalize_object(name: &str) -> &str {
    if name.contains("TheWorld:PersistentLevel.CrowdActor_TA") {
        "TheWorld:PersistentLevel.CrowdActor_TA"
    } else if name.contains("TheWorld:PersistentLevel.CrowdManager_TA") {
        "TheWorld:PersistentLevel.CrowdManager_TA"
    } else if name.contains("TheWorld:PersistentLevel.VehiclePickup_Boost_TA") {
        "TheWorld:PersistentLevel.VehiclePickup_Boost_TA"
    } else if name.contains("TheWorld:PersistentLevel.InMapScoreboard_TA") {
        "TheWorld:PersistentLevel.InMapScoreboard_TA"
    } else if name.contains("TheWorld:PersistentLevel.BreakOutActor_Platform_TA") {
        "TheWorld:PersistentLevel.BreakOutActor_Platform_TA"
    } else {
        name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_vector() {
        let mut bitter =
            LittleEndianReader::new(&[0b0000_0110, 0b0000_1000, 0b1101_1000, 0b0000_1101]);
        let v = Vector3i::decode(&mut bitter, 5).unwrap();
        assert_eq!(v, Vector3i { x: 0, y: 0, z: 93 });
    }

    #[test]
    fn test_decode_rotation() {
        let mut bitter = LittleEndianReader::new(&[0b0000_0101, 0b0000_0000]);
        let v = Rotation::decode(&mut bitter).unwrap();
        assert_eq!(
            v,
            Rotation {
                yaw: Some(2),
                pitch: None,
                roll: None,
            }
        );
    }
}
