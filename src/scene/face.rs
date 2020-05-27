use std::ops::Index;

use ffi::AiFace;

define_type_and_iterator! {
    /// Face type (not yet implemented)
    struct Face(&AiFace)
    /// Face iterator type.
    struct FaceIter
}

impl Index<usize> for Face {
    type Output = u32;

    fn index(&self, index: usize) -> &u32 {
        assert!(index < self.num_indices as usize);

        unsafe { &*self.indices.offset(index as isize) }
    }
}
