use crate::{data::PARENT_CLASSES, ObjectId};
use fnv::FnvHashMap;

use super::normalize_object;

/// A lookup of an object's ID (its index in body.objects) from its name.
///
/// The exact same name can appear multiple times in body.objects. In that
/// case, the first occurrence is used for hierarchy lookups.
pub(crate) struct ObjectIndex<'a> {
    name_index: FnvHashMap<&'a str, ObjectId>,
}

impl<'a> ObjectIndex<'a> {
    pub(crate) fn new(objects: &'a [String]) -> Self {
        let mut name_index: FnvHashMap<&str, ObjectId> = FnvHashMap::default();

        for (i, name) in objects.iter().enumerate() {
            let val = ObjectId(i as i32);
            name_index.entry(name.as_str()).or_insert(val);
        }

        Self { name_index }
    }

    /// Return primary `ObjectId` given the object name
    pub(crate) fn by_name(&self, name: &str) -> Option<ObjectId> {
        self.name_index.get(name).copied()
    }

    /// Returns the inheritance hierarchy `ObjectId` starting with self
    pub(crate) fn hierarchy<'b>(&'b self, name: &'b str) -> AncestorIterator<'a, 'b> {
        AncestorIterator { name, index: self }
    }
}

pub(crate) struct AncestorIterator<'a, 'b> {
    name: &'b str,
    index: &'a ObjectIndex<'b>,
}

impl Iterator for AncestorIterator<'_, '_> {
    type Item = ObjectId;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let current = self.name;
            self.name = PARENT_CLASSES.get(normalize_object(self.name))?;
            if let sme @ Some(_) = self.index.by_name(current) {
                return sme;
            }
        }
    }
}
