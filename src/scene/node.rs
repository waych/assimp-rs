use std::{ffi::CStr, ptr::NonNull, slice::from_raw_parts};

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
    pub fn transform(&self) -> Matrix4x4 {
        Matrix4x4::from_raw(self.mTransformation)
    }

    /// Return the parent of this node. Returns `None` if this node is the root node.
    pub fn parent(&self) -> Option<&Node> {
        unsafe { Some(Node::from_raw(NonNull::new(self.mParent)?)) }
    }

    /// Returns the number of child nodes.
    pub fn num_children(&self) -> u32 {
        self.mNumChildren
    }

    /// Returns a vector containing all of the child nodes under this node.
    pub fn children(&self) -> NodeIter {
        NodeIter::new(
            NonNull::new(self.mChildren as *mut *const aiNode),
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

    /// Any custom metadata for this node - for example, the importer for HL1 `.mdl` files
    /// will store hitbox information here.
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
            let key = unsafe {
                crate::aistring_to_cstr(
                    &*NonNull::new(
                        NonNull::new(self.meta.mKeys)?
                            .as_ptr()
                            .offset(self.index as isize),
                    )?
                    .as_ptr(),
                )
            };
            let value = unsafe {
                MetadataEntry::from_raw(NonNull::new(
                    NonNull::new(self.meta.mValues)?
                        .as_ptr()
                        .offset(self.index as isize),
                )?)
            };

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
pub enum MetadataValue<'a> {
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
    pub fn get(&self) -> MetadataValue<'_> {
        unsafe {
            match self.mType {
                ffi::aiMetadataType_AI_BOOL => MetadataValue::Bool(*(self.mData as *const bool)),
                ffi::aiMetadataType_AI_INT32 => MetadataValue::I32(*(self.mData as *const i32)),
                ffi::aiMetadataType_AI_UINT64 => MetadataValue::U64(*(self.mData as *const u64)),
                ffi::aiMetadataType_AI_FLOAT => MetadataValue::F32(*(self.mData as *const f32)),
                ffi::aiMetadataType_AI_DOUBLE => MetadataValue::F64(*(self.mData as *const f64)),
                ffi::aiMetadataType_AI_AISTRING => {
                    MetadataValue::Str(crate::aistring_to_cstr(&*(self.mData as *const aiString)))
                }
                ffi::aiMetadataType_AI_AIVECTOR3D => {
                    MetadataValue::Vector3D(Vector3D::from_raw(*(self.mData as *const aiVector3D)))
                }
                _ => unreachable!(),
            }
        }
    }
}
