pub(crate) use self::attributes::*;
pub use self::models::*;

pub mod attributes;
mod frame_decoder;
mod models;

use crate::data::{ATTRIBUTES, PARENT_CLASSES, SPAWN_STATS};
use crate::errors::NetworkError;
use crate::header::Header;
use crate::models::*;
use crate::network::frame_decoder::FrameDecoder;
use crate::parser::ReplayBody;
use fnv::FnvHashMap;
use std::cmp;
use std::collections::hash_map::Entry;

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

    // The exact same name can appear multiple times in body.objects. Very annoying.
    // So we designate a primary index for each name, and reverse lookups.
    let mut name_index: FnvHashMap<&str, ObjectId> = FnvHashMap::default();
    let mut secondary_indices: FnvHashMap<ObjectId, Vec<ObjectId>> = FnvHashMap::default();
    let mut primary_ind: FnvHashMap<ObjectId, ObjectId> = FnvHashMap::default();

    for (i, name) in body.objects.iter().enumerate() {
        let val = ObjectId(i as i32);
        match name_index.entry(name) {
            Entry::Occupied(occupied_entry) => {
                primary_ind.insert(val, *occupied_entry.get());
                secondary_indices
                    .entry(*occupied_entry.get())
                    .or_default()
                    .push(val);
            }
            Entry::Vacant(vacant_entry) => {
                vacant_entry.insert(val);
            }
        };
    }

    // Create a parallel vector where we lookup how to decode an object's initial trajectory
    // when they spawn as a new actor
    let mut spawns: Vec<Option<SpawnTrajectory>> = vec![None; body.objects.len()];
    for (object_name, spawn) in SPAWN_STATS.iter() {
        let all_indices = name_index
            .get(object_name)
            .map(|ind| std::iter::once(ind).chain(secondary_indices.get(ind).into_iter().flatten()))
            .into_iter()
            .flatten();

        for i in all_indices {
            spawns[i.0 as usize] = Some(*spawn);
        }
    }

    let mut parent_stack = Vec::new();
    for (i, name) in body.objects.iter().enumerate() {
        if spawns[i].is_some() {
            continue;
        }

        parent_stack.clear();
        parent_stack.push(ObjectId(i as i32));
        spawn_traversal(
            name,
            &name_index,
            &secondary_indices,
            &mut spawns,
            &mut parent_stack,
        );
    }

    let mut net_properties: FnvHashMap<ObjectId, Vec<(_, _)>> = FnvHashMap::default();
    for cache in &body.net_cache {
        let properties = cache
            .properties
            .iter()
            .map(|x| {
                let attr = normalized_objects
                    .get(x.object_ind as usize)
                    .map(|x| {
                        ATTRIBUTES
                            .get(x)
                            .copied()
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
            .collect::<Result<Vec<(_, _)>, NetworkError>>()?;

        let key = ObjectId(cache.object_ind);
        let primary_object = primary_ind.get(&key).copied().unwrap_or(key);

        // The same primary object can occur multiple times, though it tends to
        // be just duplicates.
        net_properties
            .entry(primary_object)
            .or_default()
            .extend(properties);
    }

    let mut object_ind_attrs: FnvHashMap<ObjectId, FnvHashMap<StreamId, ObjectAttribute>> =
        Default::default();

    let mut acc_attrs = Vec::new();
    let mut parent_stack = Vec::new();
    for (name, normalized_name) in body.objects.iter().zip(normalized_objects) {
        let Some(obj_ind) = name_index.get(name.as_str()) else {
            continue;
        };

        if object_ind_attrs.contains_key(obj_ind) {
            continue;
        }

        net_traversal(
            name.as_str(),
            &mut acc_attrs,
            &mut parent_stack,
            &net_properties,
            obj_ind,
            &name_index,
            &secondary_indices,
            &mut object_ind_attrs,
        );

        if normalized_name != name {
            net_traversal(
                normalized_name,
                &mut acc_attrs,
                &mut parent_stack,
                &net_properties,
                obj_ind,
                &name_index,
                &secondary_indices,
                &mut object_ind_attrs,
            );
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

    let product_decoder = ProductValueDecoder::create(version, &name_index);

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

        let spawns = spawns
            .drain(..)
            .map(|x| x.unwrap_or(SpawnTrajectory::None))
            .collect();

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

fn net_traversal(
    mut object_name: &str,
    acc_attrs: &mut Vec<(StreamId, ObjectAttribute)>,
    parent_stack: &mut Vec<(ObjectId, Vec<(StreamId, ObjectAttribute)>)>,
    net_properties: &FnvHashMap<ObjectId, Vec<(StreamId, ObjectAttribute)>>,
    obj_ind: &ObjectId,
    name_index: &FnvHashMap<&str, ObjectId>,
    secondary_indices: &FnvHashMap<ObjectId, Vec<ObjectId>>,
    object_ind_attrs: &mut FnvHashMap<ObjectId, FnvHashMap<StreamId, ObjectAttribute>>,
) {
    acc_attrs.clear();
    parent_stack.clear();

    let self_attributes = net_properties.get(obj_ind).cloned().unwrap_or_default();
    parent_stack.push((*obj_ind, self_attributes));

    while let Some(parent) = PARENT_CLASSES.get(object_name) {
        object_name = parent;

        let Some(ind) = name_index.get(parent) else {
            continue;
        };

        let attrs = net_properties.get(ind).cloned().unwrap_or_default();
        parent_stack.push((*ind, attrs));

        let Some(parent_attributes) = object_ind_attrs.get(ind) else {
            let attrs = net_properties.get(ind).cloned().unwrap_or_default();
            parent_stack.push((*ind, attrs));
            continue;
        };

        acc_attrs.extend(parent_attributes.iter().map(|(x, y)| (*x, *y)));
        while let Some((ind, attrs)) = parent_stack.pop() {
            acc_attrs.extend(attrs);
            let equiv_parents =
                std::iter::once(&ind).chain(secondary_indices.get(&ind).into_iter().flatten());
            for parent in equiv_parents {
                object_ind_attrs.insert(*parent, acc_attrs.iter().cloned().collect());
            }
        }

        return;
    }

    while let Some((ind, attrs)) = parent_stack.pop() {
        acc_attrs.extend(attrs);
        if !acc_attrs.is_empty() {
            let equiv_parents =
                std::iter::once(&ind).chain(secondary_indices.get(&ind).into_iter().flatten());
            for parent in equiv_parents {
                object_ind_attrs.insert(*parent, acc_attrs.iter().cloned().collect());
            }
        }
    }
}

fn spawn_traversal(
    mut object_name: &str,
    name_index: &FnvHashMap<&str, ObjectId>,
    secondary_indices: &FnvHashMap<ObjectId, Vec<ObjectId>>,
    spawns: &mut [Option<SpawnTrajectory>],
    parent_stack: &mut Vec<ObjectId>,
) {
    while let Some(parent) = PARENT_CLASSES.get(object_name) {
        object_name = parent;

        let Some(ind) = name_index.get(parent) else {
            continue;
        };

        let Some(parent_spawn) = spawns[ind.0 as usize] else {
            parent_stack.push(*ind);
            continue;
        };

        while let Some(p_ind) = parent_stack.pop() {
            let equiv_parents =
                std::iter::once(&p_ind).chain(secondary_indices.get(&p_ind).into_iter().flatten());
            for i in equiv_parents {
                spawns[i.0 as usize] = Some(parent_spawn)
            }
        }

        break;
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
