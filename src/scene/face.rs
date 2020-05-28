use std::ops::Index;

use ffi::aiFace;

define_type_and_iterator! {
    /// Face type (not yet implemented)
    struct Face(&aiFace)
    /// Face iterator type.
    struct FaceIter
}

impl Index<usize> for Face {
    type Output = u32;

    fn index(&self, index: usize) -> &u32 {
        assert!(index < self.mNumIndices as usize);

        unsafe { &*self.mIndices.offset(index as isize) }
    }
}
