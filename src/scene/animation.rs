use crate::math::{Quaternion, Vector3D};
use ffi::{aiAnimation, aiNodeAnim, aiQuatKey, aiVectorKey};
use std::ptr::NonNull;

define_type_and_iterator_indirect! {
    /// Animation type (not yet implemented)
    struct Animation(&aiAnimation)
    /// Animation iterator type.
    struct AnimationIter
}

impl Animation {
    pub fn fps(&self) -> f64 {
        self.mTicksPerSecond
    }

    pub fn duration(&self) -> f64 {
        self.mDuration
    }

    pub fn node_anims(&self) -> NodeAnimIter {
        NodeAnimIter::new(
            NonNull::new(self.mChannels as *mut *const _),
            self.mNumChannels as usize,
        )
    }

    pub fn get_node_anim(&self, id: usize) -> Option<&NodeAnim> {
        if id < self.mNumChannels as usize {
            unsafe {
                Some(NodeAnim::from_raw(NonNull::new(
                    *(NonNull::new(self.mChannels)?.as_ptr().offset(id as isize)),
                )?))
            }
        } else {
            None
        }
    }
}

define_type_and_iterator_indirect! {
    /// NodeAnim type (not yet implemented)
    struct NodeAnim(&aiNodeAnim)
    /// NodeAnim iterator type.
    struct NodeAnimIter
}

define_type_and_iterator! {
    /// VectorKey type (not yet implemented)
    struct VectorKey(&aiVectorKey)
    /// VectorKey iterator type.
    struct VectorKeyIter
}

define_type_and_iterator! {
    /// QuatKey type (not yet implemented)
    struct QuatKey(&aiQuatKey)
    /// QuatKey iterator type.
    struct QuatKeyIter
}

impl VectorKey {
    pub fn time(&self) -> f64 {
        self.mTime
    }

    pub fn value(&self) -> Vector3D {
        Vector3D(self.mValue)
    }
}

impl QuatKey {
    pub fn time(&self) -> f64 {
        self.mTime
    }

    pub fn value(&self) -> Quaternion {
        Quaternion(self.mValue)
    }
}

impl NodeAnim {
    pub fn node_name(&self) -> &str {
        unsafe { crate::aistring_to_cstr(&self.mNodeName) }
            .to_str()
            .unwrap()
    }

    pub fn get_position_key(&self, id: usize) -> Option<&VectorKey> {
        if id < self.mNumPositionKeys as usize {
            unsafe {
                Some(VectorKey::from_raw(NonNull::new(
                    NonNull::new(self.mPositionKeys)?
                        .as_ptr()
                        .offset(id as isize),
                )?))
            }
        } else {
            None
        }
    }

    pub fn position_keys(&self) -> VectorKeyIter {
        VectorKeyIter::new(
            NonNull::new(self.mPositionKeys),
            self.mNumPositionKeys as usize,
        )
    }

    pub fn rotation_keys(&self) -> QuatKeyIter {
        QuatKeyIter::new(
            NonNull::new(self.mRotationKeys),
            self.mNumRotationKeys as usize,
        )
    }

    pub fn scaling_keys(&self) -> VectorKeyIter {
        VectorKeyIter::new(
            NonNull::new(self.mScalingKeys),
            self.mNumScalingKeys as usize,
        )
    }

    pub fn get_rotation_key(&self, id: usize) -> Option<&QuatKey> {
        if id < self.mNumRotationKeys as usize {
            unsafe {
                Some(QuatKey::from_raw(NonNull::new(
                    NonNull::new(self.mRotationKeys)?
                        .as_ptr()
                        .offset(id as isize),
                )?))
            }
        } else {
            None
        }
    }
    pub fn get_scaling_key(&self, id: usize) -> Option<&VectorKey> {
        if id < self.mNumScalingKeys as usize {
            unsafe {
                Some(VectorKey::from_raw(NonNull::new(
                    NonNull::new(self.mScalingKeys)?
                        .as_ptr()
                        .offset(id as isize),
                )?))
            }
        } else {
            None
        }
    }
}
