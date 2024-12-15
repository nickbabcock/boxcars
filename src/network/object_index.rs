use crate::ObjectId;
use fnv::FnvHashMap;
use std::collections::hash_map::Entry;

/// A lookup of an object's ID (its index in body.objects) from its name.
///
/// The exact same name can appear multiple times in body.objects, so we
/// designate these additional occurrences as "secondary IDs", and an
/// `ObjectIndex` is a bidirectional map of primary to secondary IDs.
pub(crate) struct ObjectIndex<'a> {
    name_index: FnvHashMap<&'a str, ObjectId>,
    secondary_indices: FnvHashMap<ObjectId, Vec<ObjectId>>,
    primary_ind: FnvHashMap<ObjectId, ObjectId>,
}

impl<'a> ObjectIndex<'a> {
    pub(crate) fn new(objects: &'a [String]) -> Self {
        let mut name_index: FnvHashMap<&str, ObjectId> = FnvHashMap::default();
        let mut secondary_indices: FnvHashMap<ObjectId, Vec<ObjectId>> = FnvHashMap::default();
        let mut primary_ind: FnvHashMap<ObjectId, ObjectId> = FnvHashMap::default();

        for (i, name) in objects.iter().enumerate() {
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

        Self {
            name_index,
            secondary_indices,
            primary_ind,
        }
    }

    /// Return primary `ObjectId` given the object name
    pub(crate) fn primary_by_name(&self, name: &str) -> Option<ObjectId> {
        self.name_index.get(name).copied()
    }

    /// Return the primary `ObjectId` given either a primary or secondary `ObjectId`
    pub(crate) fn primary_by_index(&self, id: ObjectId) -> ObjectId {
        self.primary_ind.get(&id).copied().unwrap_or(id)
    }

    /// Returns a list of equivalent `ObjectId` as the primary id passed in.
    /// Includes self.
    pub(crate) fn all_indices(&self, id: ObjectId) -> impl Iterator<Item = ObjectId> + '_ {
        std::iter::once(id).chain(
            self.secondary_indices
                .get(&id)
                .into_iter()
                .flatten()
                .copied(),
        )
    }
}
