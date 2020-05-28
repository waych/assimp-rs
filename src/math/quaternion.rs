#[cfg(feature = "cgmath")]
use cgmath::Quaternion as CgQuaternion;
use ffi::aiQuaternion;

define_type! {
    /// Quaternion docs
    #[derive(Clone, Copy, Debug, PartialEq)]
    struct Quaternion(aiQuaternion)
}

impl Quaternion {
    pub fn new(w: f32, x: f32, y: f32, z: f32) -> Quaternion {
        Quaternion(aiQuaternion {
            w: w,
            x: x,
            y: y,
            z: z,
        })
    }
}

#[cfg(feature = "cgmath")]
impl From<CgQuaternion<f32>> for Quaternion {
    fn from(q: CgQuaternion<f32>) -> Quaternion {
        Quaternion::new(q[0], q[1], q[2], q[3])
    }
}

#[cfg(feature = "cgmath")]
impl From<Quaternion> for CgQuaternion<f32> {
    fn from(q: Quaternion) -> CgQuaternion<f32> {
        CgQuaternion::new(q.w, q.x, q.y, q.z)
    }
}
