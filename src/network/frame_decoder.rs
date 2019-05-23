use crate::errors::{AttributeError, NetworkError};
use crate::hashes::ATTRIBUTES;
use crate::network::attributes::{AttributeDecoder, ProductValueDecoder};
use crate::network::models::{
    ActorId, Frame, NewActor, ObjectId, SpawnTrajectory, StreamId, Trajectory, UpdatedAttribute,
};
use crate::network::{CacheInfo, ObjectAttribute, VersionTriplet};
use crate::parser::ReplayBody;
use bitter::BitGet;
use failure::Error;
use fnv::FnvHashMap;
use std::collections::HashMap;
use std::ops::Deref;

pub(crate) struct FrameDecoder<'a, 'b: 'a> {
    pub frames_len: usize,
    pub product_decoder: ProductValueDecoder,
    pub channel_bits: i32,
    pub body: &'a ReplayBody<'b>,
    pub spawns: &'a Vec<SpawnTrajectory>,
    pub object_ind_attributes: FnvHashMap<ObjectId, CacheInfo>,
    pub object_ind_attrs: HashMap<ObjectId, HashMap<StreamId, ObjectAttribute>>,
    pub version: VersionTriplet,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ContextObjectAttribute {
    obj_id: ObjectId,
    obj_name: String,
    prop_id: ObjectId,
    prop_name: String,
}

impl<'a, 'b> FrameDecoder<'a, 'b> {
    fn object_ind_to_string(&self, object_id: ObjectId) -> String {
        String::from(
            self.body
                .objects
                .get(usize::from(object_id))
                .map(Deref::deref)
                .unwrap_or("Out of bounds"),
        )
    }

    fn missing_attribute(
        &self,
        cache_info: &CacheInfo,
        actor_id: ActorId,
        object_id: ObjectId,
        stream_id: StreamId,
    ) -> NetworkError {
        NetworkError::MissingAttribute(
            actor_id,
            object_id,
            self.object_ind_to_string(object_id),
            stream_id,
            cache_info
                .attributes
                .keys()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(","),
        )
    }

    fn properties_with_stream_id(&self, stream_id: StreamId) -> Vec<ContextObjectAttribute> {
        self.body
            .net_cache
            .iter()
            .map(|x| {
                x.properties
                    .iter()
                    .map(|prop| (x.object_ind, prop.object_ind, prop.stream_id))
                    .collect::<Vec<(i32, i32, i32)>>()
            })
            .flat_map(|x| x)
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

    fn unimplemented_attribute(
        &self,
        actor_id: ActorId,
        object_id: ObjectId,
        stream_id: StreamId,
    ) -> NetworkError {
        let fm = self
            .properties_with_stream_id(stream_id)
            .into_iter()
            .map(|x| {
                format!(
                    "\tobject {} ({}) has property {} ({})",
                    x.obj_id, x.obj_name, x.prop_id, x.prop_name
                )
            })
            .collect::<Vec<String>>()
            .join("\n");

        NetworkError::UnimplementedAttribute(
            actor_id,
            object_id,
            self.object_ind_to_string(object_id),
            stream_id,
            self.object_ind_attrs
                .get(&object_id)
                .and_then(|x| x.get(&stream_id))
                .map(|x| self.object_ind_to_string(x.object_id))
                .unwrap_or_else(|| "type id not recognized".to_string()),
            fm,
        )
    }

    fn parse_new_actor(
        &self,
        mut bits: &mut BitGet<'_>,
        actor_id: ActorId,
    ) -> Result<NewActor, NetworkError> {
        if_chain! {
            if let Some(name_id) =
                if self.version >= VersionTriplet(868, 14, 0) {
                    bits.read_i32().map(Some)
                } else {
                    Some(None)
                };

            if let Some(_) = bits.read_bit();
            if let Some(object_id) = bits.read_i32().map(ObjectId);
            let spawn = self.spawns.get(usize::from(object_id))
                .ok_or_else(|| NetworkError::ObjectIdOutOfRange(object_id))?;

            if let Some(traj) = Trajectory::from_spawn(&mut bits, *spawn, self.version.net_version());
            then {
                Ok(NewActor {
                    actor_id,
                    name_id,
                    object_id,
                    initial_trajectory: traj
                })
            } else {
                Err(NetworkError::NotEnoughDataFor("New Actor"))
            }
        }
    }

    fn decode_frame(
        &self,
        attr_decoder: &AttributeDecoder,
        mut bits: &mut BitGet<'_>,
        actors: &mut FnvHashMap<ActorId, ObjectId>,
        time: f32,
        delta: f32,
    ) -> Result<Frame, NetworkError> {
        let mut new_actors = Vec::new();
        let mut updated_actors = Vec::new();
        let mut deleted_actors = Vec::new();

        while bits
            .read_bit()
            .ok_or_else(|| NetworkError::NotEnoughDataFor("Actor data"))?
        {
            let actor_id = bits
                .read_i32_bits(self.channel_bits)
                .map(ActorId)
                .ok_or_else(|| NetworkError::NotEnoughDataFor("Actor Id"))?;

            // alive
            if bits
                .read_bit()
                .ok_or_else(|| NetworkError::NotEnoughDataFor("Is actor alive"))?
            {
                // new
                if bits
                    .read_bit()
                    .ok_or_else(|| NetworkError::NotEnoughDataFor("Is new actor"))?
                {
                    let actor = self.parse_new_actor(&mut bits, actor_id)?;

                    // Insert the new actor so we can keep track of it for attribute
                    // updates. It's common for an actor id to already exist, so we
                    // overwrite it.
                    actors.insert(actor.actor_id, actor.object_id);
                    new_actors.push(actor);
                } else {
                    // We'll be updating an existing actor with some attributes so we need
                    // to track down what the actor's type is
                    let object_id = actors
                        .get(&actor_id)
                        .ok_or_else(|| NetworkError::MissingActor(actor_id))?;

                    // Once we have the type we need to look up what attributes are
                    // available for said type
                    let cache_info =
                        self.object_ind_attributes.get(object_id).ok_or_else(|| {
                            NetworkError::MissingCache(
                                actor_id,
                                *object_id,
                                self.object_ind_to_string(*object_id),
                            )
                        })?;

                    // While there are more attributes to update for our actor:
                    while bits
                        .read_bit()
                        .ok_or_else(|| NetworkError::NotEnoughDataFor("Is prop present"))?
                    {
                        // We've previously calculated the max the stream id can be for a
                        // given type and how many bits that it encompasses so use those
                        // values now
                        let stream_id = bits
                            .read_bits_max(cache_info.prop_id_bits, cache_info.max_prop_id)
                            .map(|x| StreamId(x as i32))
                            .ok_or_else(|| NetworkError::NotEnoughDataFor("Prop id"))?;

                        // Look the stream id up and find the corresponding attribute
                        // decoding function. Experience has told me replays that fail to
                        // parse, fail to do so here, so a large chunk is dedicated to
                        // generating an error message with context
                        let attr = cache_info.attributes.get(&stream_id).ok_or_else(|| {
                            self.missing_attribute(cache_info, actor_id, *object_id, stream_id)
                        })?;

                        let attribute =
                            attr_decoder.decode(*attr, &mut bits).map_err(|e| match e {
                                AttributeError::Unimplemented => {
                                    self.unimplemented_attribute(actor_id, *object_id, stream_id)
                                }
                                _ => NetworkError::AttributeError(e),
                            })?;

                        updated_actors.push(UpdatedAttribute {
                            actor_id,
                            stream_id,
                            attribute,
                        });
                    }
                }
            } else {
                deleted_actors.push(actor_id);
                actors.remove(&actor_id);
            }
        }

        Ok(Frame {
            time,
            delta,
            new_actors,
            deleted_actors,
            updated_actors,
        })
    }

    pub fn decode_frames(&self) -> Result<Vec<Frame>, Error> {
        let attr_decoder = AttributeDecoder::new(self.version, self.product_decoder);
        let mut frames: Vec<Frame> = Vec::with_capacity(self.frames_len);
        let mut actors = FnvHashMap::default();
        let mut bits = BitGet::new(self.body.network_data);
        while !bits.is_empty() && frames.len() < self.frames_len {
            let time = bits
                .read_f32()
                .ok_or_else(|| NetworkError::NotEnoughDataFor("Time"))?;

            if time < 0.0 || (time > 0.0 && time < 1e-10) {
                let mut frame_index = frames.len() - 1;
                while let Some(frame) = frames.get(frame_index) {
                    if let Some(last_update) = frame.updated_actors.last() {
                        return Err(NetworkError::TimeOutOfRangeUpdate(
                            frames.len(),
                            frame_index,
                            last_update.actor_id,
                            last_update.stream_id,
                            last_update.attribute.clone(),
                        ))?;
                    }
                    frame_index -= 1;
                }

                return Err(NetworkError::TimeOutOfRange(time))?;
            }

            let delta = bits
                .read_f32()
                .ok_or_else(|| NetworkError::NotEnoughDataFor("Delta"))?;

            if delta < 0.0 || (delta > 0.0 && delta < 1e-10) {
                return Err(NetworkError::DeltaOutOfRange(delta))?;
            }

            if time == 0.0 && delta == 0.0 {
                break;
            }

            let frame = self.decode_frame(&attr_decoder, &mut bits, &mut actors, time, delta)?;
            frames.push(frame);
        }

        if self.version >= VersionTriplet(868, 24, 10) {
            bits.read_u32()
                .ok_or_else(|| NetworkError::NotEnoughDataFor("Trailer"))?;
        }

        Ok(frames)
    }
}
