use crate::types::type_id::TypeId;

use std::collections::BTreeMap;

use thiserror::Error;

/// A mapping between assignments of TypeId's
///
/// Used to preserve consistency of associations from TypeId to ElemType when
/// updating multiple TypeId's
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TypeIdMap {
    map: BTreeMap<TypeId, TypeId>,
}

impl Default for TypeIdMap {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeIdMap {
    /// New empty TypeIdMap
    pub fn new() -> Self {
        TypeIdMap {
            map: BTreeMap::new(),
        }
    }

    /// Add a mapping to the TypeIdMap, failing if the "from" TypeId" already
    /// exists in the map
    pub fn push(&mut self, from: TypeId, to: TypeId) -> Result<(), TypeIdMapError> {
        if let std::collections::btree_map::Entry::Vacant(e) = self.map.entry(from) {
            e.insert(to);
            Ok(())
        } else {
            Err(TypeIdMapError::PushExists {
                from,
                to,
                map: self.clone(),
            })
        }
    }

    /// Resolve the map on a single TypeId
    pub fn get(&self, index: &TypeId, location: usize) -> Result<&TypeId, TypeIdMapError> {
        self.map.get(index)
            .ok_or_else(|| TypeIdMapError::GetUnknownTypeId {
                index: *index,
                location,
                type_map: self.clone(),
            })
    }

    /// Resolve the map on a Vec of TypeId's
    pub fn run(&self, type_vars: Vec<TypeId>) -> Result<Vec<TypeId>, TypeIdMapError> {
        type_vars.iter().enumerate().map(|(i, x)| Ok(*self.get(x, i)?)).collect()
    }
}

/// TypeIdMap trait errors
#[derive(Clone, Debug, PartialEq, Error)]
pub enum TypeIdMapError {
    /// "TypeIdMap::get attempted to get a TypeId: {index:?}, not in the map: {type_map:?}; at location in TypeIdMap::run {location:?}"
    #[error("TypeIdMap::get attempted to get a TypeId: {index:?}, not in the map: {type_map:?}; at location in TypeIdMap::run {location:?}")]
    GetUnknownTypeId {
        /// Missing TypeId
        index: TypeId,

        /// TypeIdMap::run location
        location: usize,

        /// index missing from this TypeIdMap
        type_map: TypeIdMap,
    },

    /// "TypeIdMap::push already exists: mapping from: {from:?}, to: {to:?}, in TypeIdMap {map:?}"
    #[error("TypeIdMap::push already exists: mapping from: {from:?}, to: {to:?}, in TypeIdMap {map:?}")]
    PushExists {
        /// _.push(from, _)
        from: TypeId,

        /// _.push(_, to)
        to: TypeId,

        /// TypeId "from" already present in this TypeIdMap
        map: TypeIdMap,
    },
}

