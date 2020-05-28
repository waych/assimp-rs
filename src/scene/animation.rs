use ffi::aiAnimation;
use ffi::aiNodeAnim;
use ffi::aiQuatKey;
use ffi::aiVectorKey;

define_type_and_iterator_indirect! {
    /// Animation type (not yet implemented)
    struct Animation(&aiAnimation)
    /// Animation iterator type.
    struct AnimationIter
}

define_type_and_iterator_indirect! {
    /// NodeAnim type (not yet implemented)
    struct NodeAnim(&aiNodeAnim)
    /// NodeAnim iterator type.
    struct NodeAnimIter
}

define_type_and_iterator_indirect! {
    /// VectorKey type (not yet implemented)
    struct VectorKey(&aiVectorKey)
    /// VectorKey iterator type.
    struct VectorKeyIter
}

define_type_and_iterator_indirect! {
    /// QuatKey type (not yet implemented)
    struct QuatKey(&aiQuatKey)
    /// QuatKey iterator type.
    struct QuatKeyIter
}

impl NodeAnim {
    pub fn get_position_key(&self, id: usize) -> Option<&VectorKey> {
        if id < self.mNumPositionKeys as usize {
            unsafe { Some(VectorKey::from_raw(self.mPositionKeys.offset(id as isize))) }
        } else {
            None
        }
    }
    pub fn get_rotation_key(&self, id: usize) -> Option<&QuatKey> {
        if id < self.mNumRotationKeys as usize {
            unsafe { Some(QuatKey::from_raw(self.mRotationKeys.offset(id as isize))) }
        } else {
            None
        }
    }
    pub fn get_scaling_key(&self, id: usize) -> Option<&VectorKey> {
        if id < self.mNumScalingKeys as usize {
            unsafe { Some(VectorKey::from_raw(self.mScalingKeys.offset(id as isize))) }
        } else {
            None
        }
    }
}

impl Animation {
    pub fn get_node_anim(&self, id: usize) -> Option<&NodeAnim> {
        if id < self.mNumChannels as usize {
            unsafe { Some(NodeAnim::from_raw(*(self.mChannels.offset(id as isize)))) }
        } else {
            None
        }
    }
}
