pub(crate) use self::attributes::*;
pub use self::models::*;

pub mod attributes;
mod frame_decoder;
mod models;

use crate::data::{attributes, object_classes, parent_class, spawn_stats};
use crate::errors::NetworkError;
use crate::header::Header;
use crate::models::*;
use crate::network::frame_decoder::FrameDecoder;
use crate::parser::ReplayBody;
use crate::parsing_utils::log2;
use fnv::FnvHashMap;
use std::collections::HashMap;
use std::ops::Deref;

pub(crate) struct CacheInfo<'a> {
    max_prop_id: i32,
    prop_id_bits: i32,
    attributes: &'a FnvHashMap<StreamId, ObjectAttribute>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ObjectAttribute {
    attribute: AttributeTag,
    object_id: ObjectId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct VersionTriplet(pub i32, pub i32, pub i32);

impl VersionTriplet {
    pub fn net_version(&self) -> i32 {
        self.2
    }
}

pub(crate) fn parse<'a>(
    header: &Header,
    body: &ReplayBody<'a>,
) -> Result<NetworkFrames, NetworkError> {
    let version = VersionTriplet(
        header.major_version,
        header.minor_version,
        header.net_version.unwrap_or(0),
    );

    // Create a parallel vector where each object has it's name normalized
    let normalized_objects: Vec<&str> = body.objects.iter().map(|x| normalize_object(x)).collect();

    // Create a parallel vector where we lookup how to decode an object's initial trajectory
    // when they spawn as a new actor
    let spawns: Vec<SpawnTrajectory> = body
        .objects
        .iter()
        .map(|x| {
            spawn_stats(x.deref())
                .unwrap_or(SpawnTrajectory::None)
        })
        .collect();

    let attrs: Vec<_> = normalized_objects
        .iter()
        .map(|x| {
            attributes(x.deref())
                .unwrap_or(AttributeTag::NotImplemented)
        })
        .collect();

    // Create a map of an object's normalized name to a list of indices in the object
    // vector that have that same normalized name
    let mut normalized_name_obj_ind: HashMap<&str, Vec<ObjectId>> =
        HashMap::with_capacity(normalized_objects.len());
    for (i, name) in normalized_objects.iter().enumerate() {
        normalized_name_obj_ind
            .entry(*name)
            .or_insert_with(|| vec![])
            .push(ObjectId(i as i32));
    }

    // Map each object's name to it's index
    let name_obj_ind: HashMap<&str, ObjectId> = body
        .objects
        .iter()
        .enumerate()
        .map(|(i, name)| (name.deref(), ObjectId(i as i32)))
        .collect();

    let mut object_ind_attrs: FnvHashMap<ObjectId, FnvHashMap<StreamId, ObjectAttribute>> =
        Default::default();
    for cache in &body.net_cache {
        let mut all_props: FnvHashMap<StreamId, ObjectAttribute> = cache
            .properties
            .iter()
            .map(|x| {
                let attr = attrs
                    .get(x.object_ind as usize)
                    .ok_or_else(|| NetworkError::StreamTooLargeIndex(x.stream_id, x.object_ind))?;
                Ok((
                    StreamId(x.stream_id),
                    ObjectAttribute {
                        attribute: *attr,
                        object_id: ObjectId(x.object_ind),
                    },
                ))
            })
            .collect::<Result<FnvHashMap<_, _>, NetworkError>>()?;

        let mut had_parent = false;

        // We are going to recursively resolve an object's name to find their direct parent.
        // Parents have parents as well (etc), so we repeatedly walk up the chain picking up
        // attributes on parent objects until we reach an object with no parent (`Core.Object`)
        let mut object_name: &str = &*body
            .objects
            .get(cache.object_ind as usize)
            .ok_or_else(|| NetworkError::ObjectIdOutOfRange(ObjectId(cache.object_ind)))?;

        while let Some(parent_name) = parent_class(object_name) {
            had_parent = true;
            if let Some(parent_ind) = name_obj_ind.get(parent_name) {
                if let Some(parent_attrs) = object_ind_attrs.get(parent_ind) {
                    all_props.extend(parent_attrs.iter());
                }
            }

            object_name = parent_name;
        }

        // Sometimes our hierarchy set up in build.rs isn't perfect so if we don't find a
        // parent and a parent cache id is set, try and find this parent id and carry down
        // their props.
        if !had_parent && cache.parent_id != 0 {
            if let Some(parent) = body
                .net_cache
                .iter()
                .find(|x| x.cache_id == cache.parent_id)
            {
                if let Some(parent_attrs) = object_ind_attrs.get(&ObjectId(parent.object_ind)) {
                    all_props.extend(parent_attrs.iter());
                }
            }
        }

        object_ind_attrs.insert(ObjectId(cache.object_ind), all_props);
    }

    let clses = object_classes();
    for (obj, parent) in clses.iter() {
        // It's ok if an object class doesn't appear in our replay. For instance, basketball
        // objects don't appear in a soccer replay.
        if let Some(object_ids) = normalized_name_obj_ind.get(obj) {
            let parent_id = name_obj_ind.get(parent).ok_or_else(|| {
                NetworkError::MissingParentClass(String::from(*obj), String::from(*parent))
            })?;

            for i in object_ids {
                let parent_attrs: FnvHashMap<_, _> = object_ind_attrs
                    .get(parent_id)
                    .ok_or_else(|| NetworkError::ParentHasNoAttributes(*parent_id, *i))?
                    .clone();
                object_ind_attrs.insert(*i, parent_attrs);
            }
        }
    }

    let object_ind_attributes: FnvHashMap<ObjectId, CacheInfo> = object_ind_attrs
        .iter()
        .map(|(obj_id, attrs)| {
            let id = *obj_id;
            let max = attrs.keys().map(|&x| i32::from(x)).max().unwrap_or(2) + 1;
            let next_max = (max as u32)
                .checked_next_power_of_two()
                .ok_or_else(|| NetworkError::MaxStreamIdTooLarge(max, id))?;
            Ok((
                id,
                CacheInfo {
                    max_prop_id: max,
                    prop_id_bits: log2(next_max) as i32,
                    attributes: attrs,
                },
            ))
        })
        .collect::<Result<FnvHashMap<_, _>, NetworkError>>()?;

    let product_decoder = ProductValueDecoder::create(version, &name_obj_ind);

    // 1023 stolen from rattletrap
    let channels = header.max_channels().unwrap_or(1023);
    let channels = (channels as u32)
        .checked_next_power_of_two()
        .ok_or_else(|| NetworkError::ChannelsTooLarge(channels))?;
    let channel_bits = log2(channels as u32) as i32;
    let num_frames = header.num_frames();

    if let Some(frame_len) = num_frames {
        if frame_len as usize > body.network_data.len() {
            return Err(NetworkError::TooManyFrames(frame_len));
        }

        let frame_decoder = FrameDecoder {
            frames_len: frame_len as usize,
            product_decoder,
            channel_bits,
            body,
            spawns: &spawns,
            object_ind_attributes,
            version,
        };
        Ok(NetworkFrames {
            frames: frame_decoder.decode_frames()?,
        })
    } else {
        Ok(NetworkFrames { frames: Vec::new() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_triplets() {
        let version = VersionTriplet(18, 27, 1);
        assert_eq!(version.net_version(), 1);

        assert!(version < VersionTriplet(19, 27, 1));
        assert!(version < VersionTriplet(18, 28, 1));
        assert!(version < VersionTriplet(18, 27, 2));
        assert_eq!(version, VersionTriplet(18, 27, 1));
        assert!(version > VersionTriplet(17, 27, 1));
        assert!(version > VersionTriplet(18, 26, 1));
        assert!(version > VersionTriplet(18, 27, 0));
    }
}
