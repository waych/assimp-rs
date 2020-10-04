use crate::import::structs::PrimitiveType;
use std::{
    borrow::Borrow,
    convert::AsRef,
    fmt,
    ops::Index,
};

use ffi::aiFace;

define_type_and_iterator! {
    /// Face type (not yet implemented)
    struct Face(&aiFace)
    /// Face iterator type.
    struct FaceIter
}

impl Face {
    /// The "kind" of this face - each mesh contains a bitset of all the primitive types that this mesh
    /// contains. For most applications you will want to call `Importer::triangulate(true)`, which will
    /// make Assimp automatically convert all faces to triangles.
    pub fn primitive_type(&self) -> PrimitiveType {
        match self.indices().len() {
            0 => unreachable!(),
            1 => PrimitiveType::Point,
            2 => PrimitiveType::Line,
            3 => PrimitiveType::Triangle,
            _ => PrimitiveType::Polygon,
        }
    }

    /// The list of indices into the parent mesh's vertex list used by this face.
    pub fn indices(&self) -> &[u32] {
        if self.mIndices.is_null() {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(self.mIndices, self.mNumIndices as usize) }
        }
    }
}

impl fmt::Debug for Face {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        #[derive(Debug)]
        struct Face<'a>(&'a [u32]);

        Face(self.indices()).fmt(f)
    }
}

impl AsRef<[u32]> for Face {
    fn as_ref(&self) -> &[u32] {
        self.indices()
    }
}

impl Borrow<[u32]> for Face {
    fn borrow(&self) -> &[u32] {
        self.indices()
    }
}

impl Index<usize> for Face {
    type Output = u32;

    fn index(&self, index: usize) -> &u32 {
        assert!(index < self.mNumIndices as usize);

        unsafe { &*self.mIndices.offset(index as isize) }
    }
}
