pub(crate) use self::attributes::*;
pub use self::models::*;

pub mod attributes;
mod frame_decoder;
mod models;

use crate::data::{object_classes, ATTRIBUTES, PARENT_CLASSES, SPAWN_STATS};
use crate::errors::NetworkError;
use crate::header::Header;
use crate::models::*;
use crate::network::frame_decoder::FrameDecoder;
use crate::parser::ReplayBody;
use fnv::FnvHashMap;
use std::cmp;
use std::collections::HashMap;
use std::ops::Deref;

#[derive(Debug)]
pub(crate) struct CacheInfo<'a> {
    max_prop_id: u32,
    prop_id_bits: u32,
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

pub(crate) fn parse(header: &Header, body: &ReplayBody) -> Result<NetworkFrames, NetworkError> {
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
            SPAWN_STATS
                .get(x.deref())
                .cloned()
                .unwrap_or(SpawnTrajectory::None)
        })
        .collect();

    // Create a map of an object's normalized name to a list of indices in the object
    // vector that have that same normalized name
    let mut normalized_name_obj_ind: HashMap<&str, Vec<ObjectId>> =
        HashMap::with_capacity(normalized_objects.len());
    for (i, name) in normalized_objects.iter().enumerate() {
        normalized_name_obj_ind
            .entry(*name)
            .or_default()
            .push(ObjectId(i as i32));
    }

    // Map each object's name to it's index
    let name_obj_ind: HashMap<&str, Vec<ObjectId>> = body
        .objects
        .iter()
        .map(|name| {
            (
                name.deref(),
                normalized_name_obj_ind
                    .get(name.deref())
                    .cloned()
                    .unwrap_or_default(),
            )
        })
        .collect();

    let mut object_ind_attrs: FnvHashMap<ObjectId, FnvHashMap<StreamId, ObjectAttribute>> =
        Default::default();
    for cache in &body.net_cache {
        let mut all_props: FnvHashMap<StreamId, ObjectAttribute> = cache
            .properties
            .iter()
            .map(|x| {
                let attr = normalized_objects
                    .get(x.object_ind as usize)
                    .map(|x| {
                        ATTRIBUTES
                            .get(x)
                            .cloned()
                            .unwrap_or(AttributeTag::NotImplemented)
                    })
                    .ok_or(NetworkError::StreamTooLargeIndex(x.stream_id, x.object_ind))?;
                Ok((
                    StreamId(x.stream_id),
                    ObjectAttribute {
                        attribute: attr,
                        object_id: ObjectId(x.object_ind),
                    },
                ))
            })
            .collect::<Result<FnvHashMap<_, _>, NetworkError>>()?;

        let mut had_parent = false;

        // We are going to recursively resolve an object's name to find their direct parent.
        // Parents have parents as well (etc), so we repeatedly walk up the chain picking up
        // attributes on parent objects until we reach an object with no parent (`Core.Object`)
        let mut object_name: &str = body
            .objects
            .get(cache.object_ind as usize)
            .ok_or(NetworkError::ObjectIdOutOfRange(ObjectId(cache.object_ind)))?;

        while let Some(parent_name) = PARENT_CLASSES.get(object_name) {
            had_parent = true;
            if let Some(parent_ids) = name_obj_ind.get(parent_name) {
                for parent_id in parent_ids {
                    if let Some(parent_attrs) = object_ind_attrs.get(parent_id) {
                        all_props.extend(parent_attrs.iter());
                    }
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

    for (obj, parent) in object_classes().iter() {
        // It's ok if an object class doesn't appear in our replay. For instance, basketball
        // objects don't appear in a soccer replay.
        if let Some(object_ids) = normalized_name_obj_ind.get(obj) {
            let parent_ids = name_obj_ind.get(parent).ok_or_else(|| {
                NetworkError::MissingParentClass(String::from(*obj), String::from(*parent))
            })?;

            for i in object_ids {
                for parent_id in parent_ids {
                    let parent_attrs: FnvHashMap<_, _> = object_ind_attrs
                        .get(parent_id)
                        .ok_or(NetworkError::ParentHasNoAttributes(*parent_id, *i))?
                        .clone();

                    object_ind_attrs
                        .entry(*i)
                        .and_modify(|e| e.extend(parent_attrs.iter()))
                        .or_insert(parent_attrs);
                }
            }
        }
    }

    let object_ind_attributes: FnvHashMap<ObjectId, CacheInfo> = object_ind_attrs
        .iter()
        .map(|(obj_id, attrs)| {
            let id = *obj_id;
            let max = attrs
                .keys()
                .map(|&x| i32::from(x))
                .max()
                .unwrap_or(2)
                .saturating_add(1);

            let max_bit_width = crate::bits::bit_width(max as u64);
            Ok((
                id,
                CacheInfo {
                    max_prop_id: max as u32,
                    prop_id_bits: cmp::max(max_bit_width, 1) - 1,
                    attributes: attrs,
                },
            ))
        })
        .collect::<Result<FnvHashMap<_, _>, NetworkError>>()?;

    let product_decoder = ProductValueDecoder::create(version, &name_obj_ind);

    // 1023 stolen from rattletrap
    let max_channels = header.max_channels().unwrap_or(1023) as u32;
    let channel_width = crate::bits::bit_width(u64::from(max_channels)).saturating_sub(1);
    let channel_bits = cmp::max(channel_width, 0);
    let num_frames = header.num_frames();
    let is_lan = header.match_type().map(|x| x == "Lan").unwrap_or(false);
    let is_rl_223 = matches!(header.build_version(), Some(x) if x >= "221120.42953.406184");

    if let Some(frame_len) = num_frames {
        if frame_len as usize > body.network_data.len() {
            return Err(NetworkError::TooManyFrames(frame_len));
        }

        let frame_decoder = FrameDecoder {
            frames_len: frame_len as usize,
            product_decoder,
            max_channels,
            channel_bits,
            body,
            spawns: &spawns,
            object_ind_attributes,
            version,
            is_lan,
            is_rl_223,
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
