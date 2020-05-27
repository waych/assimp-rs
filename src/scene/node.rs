use std::{ffi::CStr, slice::from_raw_parts};

use ffi::{AiMetadata, AiMetadataEntry, AiNode, AiString, AiVector3D};

use math::{Matrix4x4, Vector3D};

define_type_and_iterator_indirect! {
    /// The `Node` type represents a node in the imported scene hierarchy.
    struct Node(&AiNode)
    /// Node iterator type.
    struct NodeIter
}

impl Node {
    /// Returns the name of the node.
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    /// Returns the node's transformation matrix.
    pub fn transformation(&self) -> Matrix4x4 {
        Matrix4x4::from_raw(self.transformation)
    }

    /// Return the parent of this node. Returns `None` if this node is the root node.
    pub fn parent(&self) -> Option<&Node> {
        if !self.parent.is_null() {
            unsafe { Some(Node::from_raw(self.parent)) }
        } else {
            None
        }
    }

    /// Returns the number of child nodes.
    pub fn num_children(&self) -> u32 {
        self.num_children
    }

    /// Returns a vector containing all of the child nodes under this node.
    pub fn child_iter(&self) -> NodeIter {
        NodeIter::new(
            self.children as *const *const AiNode,
            self.num_children as usize,
        )
    }

    /// Returns the number of meshes under this node.
    pub fn num_meshes(&self) -> u32 {
        self.num_meshes
    }

    /// Returns a vector containing all of the meshes under this node. These are indices into
    /// the meshes contained in the `Scene` struct.
    pub fn meshes(&self) -> &[u32] {
        let len = self.num_meshes as usize;
        unsafe { from_raw_parts(self.meshes, len) }
    }

    pub fn metadata(&self) -> Metadata<'_> {
        unsafe { Metadata::from_raw(self.metadata) }
    }
}

/// Metadata for a specific node. If you want this as a `HashMap`, you can easily just
/// do `let map: HashMap<_, _> = node.metadata().collect()`.
pub struct Metadata<'a> {
    meta: &'a AiMetadata,
    index: usize,
}

impl std::ops::Deref for Metadata<'_> {
    type Target = AiMetadata;

    fn deref(&self) -> &Self::Target {
        self.meta
    }
}

impl Metadata<'_> {
    /// Create a metadata iterator from a raw pointer.
    pub unsafe fn from_raw(meta: *const AiMetadata) -> Self {
        Metadata {
            meta: &*meta,
            index: 0,
        }
    }
}

unsafe fn aistring_to_cstr(aistring: &AiString) -> &CStr {
    CStr::from_bytes_with_nul_unchecked(&aistring.data[..aistring.length])
}

impl ExactSizeIterator for Metadata<'_> {
    fn len(&self) -> usize {
        self.num_properties as usize - self.index
    }
}

impl<'a> Iterator for Metadata<'a> {
    type Item = (&'a CStr, &'a MetadataEntry);

    fn next(&mut self) -> Option<Self::Item> {
        if self.len() > 0 {
            let key = unsafe { aistring_to_cstr(&*self.meta.keys.offset(self.index as isize)) };
            let value =
                unsafe { MetadataEntry::from_raw(self.meta.values.offset(self.index as isize)) };

            self.index += 1;

            Some((key, value))
        } else {
            None
        }
    }
}

define_type! {
    /// A single metadata entry value
    struct MetadataEntry(&AiMetadataEntry)
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
        use ffi::AiMetadataType;

        unsafe {
            match self.data_type {
                AiMetadataType::Bool => Value::Bool(*(self.data as *const bool)),
                AiMetadataType::Int32 => Value::I32(*(self.data as *const i32)),
                AiMetadataType::Uint64 => Value::U64(*(self.data as *const u64)),
                AiMetadataType::Float => Value::F32(*(self.data as *const f32)),
                AiMetadataType::Double => Value::F64(*(self.data as *const f64)),
                AiMetadataType::AiString => {
                    Value::Str(aistring_to_cstr(&*(self.data as *const AiString)))
                }
                AiMetadataType::AiVector3D => {
                    Value::Vector3D(Vector3D::from_raw(*(self.data as *const AiVector3D)))
                }
            }
        }
    }
}
