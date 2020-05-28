use std::{ffi::CStr, slice::from_raw_parts};

use ffi::{aiMetadata, aiMetadataEntry, aiNode, aiString, aiVector3D};

use crate::math::{Matrix4x4, Vector3D};

define_type_and_iterator_indirect! {
    /// The `Node` type represents a node in the imported scene hierarchy.
    struct Node(&aiNode)
    /// Node iterator type.
    struct NodeIter
}

impl Node {
    /// Returns the name of the node.
    pub fn name(&self) -> &str {
        unsafe { crate::aistring_to_cstr(&self.mName) }
            .to_str()
            .unwrap()
    }

    /// Returns the node's transformation matrix.
    pub fn transformation(&self) -> Matrix4x4 {
        Matrix4x4::from_raw(self.mTransformation)
    }

    /// Return the parent of this node. Returns `None` if this node is the root node.
    pub fn parent(&self) -> Option<&Node> {
        if !self.mParent.is_null() {
            unsafe { Some(Node::from_raw(self.mParent)) }
        } else {
            None
        }
    }

    /// Returns the number of child nodes.
    pub fn num_children(&self) -> u32 {
        self.mNumChildren
    }

    /// Returns a vector containing all of the child nodes under this node.
    pub fn child_iter(&self) -> NodeIter {
        NodeIter::new(
            self.mChildren as *const *const aiNode,
            self.mNumChildren as usize,
        )
    }

    /// Returns the number of meshes under this node.
    pub fn num_meshes(&self) -> u32 {
        self.mNumMeshes
    }

    /// Returns a vector containing all of the meshes under this node. These are indices into
    /// the meshes contained in the `Scene` struct.
    pub fn meshes(&self) -> &[u32] {
        let len = self.mNumMeshes as usize;
        unsafe { from_raw_parts(self.mMeshes, len) }
    }

    pub fn metadata(&self) -> Metadata<'_> {
        unsafe { Metadata::from_raw(self.mMetaData) }
    }
}

/// Metadata for a specific node. If you want this as a `HashMap`, you can easily just
/// do `let map: HashMap<_, _> = node.metadata().collect()`.
pub struct Metadata<'a> {
    meta: &'a aiMetadata,
    index: usize,
}

impl std::ops::Deref for Metadata<'_> {
    type Target = aiMetadata;

    fn deref(&self) -> &Self::Target {
        self.meta
    }
}

impl Metadata<'_> {
    /// Create a metadata iterator from a raw pointer.
    pub unsafe fn from_raw(meta: *const aiMetadata) -> Self {
        Metadata {
            meta: &*meta,
            index: 0,
        }
    }
}

impl ExactSizeIterator for Metadata<'_> {
    fn len(&self) -> usize {
        self.mNumProperties as usize - self.index
    }
}

impl<'a> Iterator for Metadata<'a> {
    type Item = (&'a CStr, &'a MetadataEntry);

    fn next(&mut self) -> Option<Self::Item> {
        if self.len() > 0 {
            let key =
                unsafe { crate::aistring_to_cstr(&*self.meta.mKeys.offset(self.index as isize)) };
            let value =
                unsafe { MetadataEntry::from_raw(self.meta.mValues.offset(self.index as isize)) };

            self.index += 1;

            Some((key, value))
        } else {
            None
        }
    }
}

define_type! {
    /// A single metadata entry value
    struct MetadataEntry(&aiMetadataEntry)
}

/// The value of a metadata item.
pub enum Value<'a> {
    /// A boolean
    Bool(bool),
    /// A signed int
    I32(i32),
    /// An unsigned int
    U64(u64),
    /// A single-precision float
    F32(f32),
    /// A double-precision float
    F64(f64),
    /// A string
    Str(&'a CStr),
    /// A vector
    Vector3D(Vector3D),
}

impl MetadataEntry {
    /// Get the value of this entry
    pub fn get(&self) -> Value<'_> {
        unsafe {
            match self.mType {
                ffi::aiMetadataType_AI_BOOL => Value::Bool(*(self.mData as *const bool)),
                ffi::aiMetadataType_AI_INT32 => Value::I32(*(self.mData as *const i32)),
                ffi::aiMetadataType_AI_UINT64 => Value::U64(*(self.mData as *const u64)),
                ffi::aiMetadataType_AI_FLOAT => Value::F32(*(self.mData as *const f32)),
                ffi::aiMetadataType_AI_DOUBLE => Value::F64(*(self.mData as *const f64)),
                ffi::aiMetadataType_AI_AISTRING => {
                    Value::Str(crate::aistring_to_cstr(&*(self.mData as *const aiString)))
                }
                ffi::aiMetadataType_AI_AIVECTOR3D => {
                    Value::Vector3D(Vector3D::from_raw(*(self.mData as *const aiVector3D)))
                }
                _ => unreachable!(),
            }
        }
    }
}
