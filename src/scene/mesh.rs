use ffi::{aiBone, aiColor4D, aiMesh, aiVector3D, aiVertexWeight};

use std::ptr::NonNull;

use super::face::{Face, FaceIter};
use crate::import::structs::PrimitiveTypes;
use crate::math::color4::{Color4D, Color4DIter};
use crate::math::vector3::{Vector3D, Vector3DIter};
use crate::math::Matrix4x4;

define_type_and_iterator_indirect! {
    /// Mesh type (incomplete)
    struct Mesh(&aiMesh)
    /// Mesh iterator type.
    struct MeshIter
}

define_type_and_iterator_indirect! {
    /// Bone type
    struct Bone(&aiBone)
    /// Bone iterator type.
    struct BoneIter
}

define_type_and_iterator_indirect! {
    /// Vertex weight type
    struct VertexWeight(&aiVertexWeight)
    /// Vertex weight iterator type.
    struct VertexWeightIter
}

impl Mesh {
    /// Returns a bitset of all the primitive types in use in this mesh.
    pub fn primitive_types(&self) -> PrimitiveTypes {
        PrimitiveTypes::from_bits(self.mPrimitiveTypes).unwrap()
    }

    /// The number of unique vertices in this mesh. This is _not_ the same as the total
    /// number of vertices in this mesh, you need to iterate through the faces to get
    /// the indices to the vertices in `vertex_iter`.
    pub fn num_vertices(&self) -> u32 {
        self.mNumVertices
    }

    /// Iterator over the unique vertex positions in the mesh. You need to iterate through
    /// faces to get the indices into this iterator in order to render the mesh.
    pub fn vertex_iter(&self) -> Vector3DIter {
        // Every format should at least provide vertex positions, so we `unwrap` here instead
        // of returning `None`.
        Vector3DIter::new(NonNull::new(self.mVertices), self.mNumVertices as usize)
    }

    /// Get the nth unique vertex position.
    pub fn get_vertex(&self, id: u32) -> Option<Vector3D> {
        self.vertex_data(self.mVertices, id)
    }

    /// Iterator over the unique vertex normals.
    pub fn normal_iter(&self) -> Vector3DIter {
        Vector3DIter::new(NonNull::new(self.mNormals), self.mNumVertices as usize)
    }

    /// Get the nth unique vertex normal.
    pub fn get_normal(&self, id: u32) -> Option<Vector3D> {
        self.vertex_data(self.mNormals, id)
    }

    /// Iterator over the unique vertex tangents, if available. Not all formats provide tangents,
    pub fn tangent_iter(&self) -> Vector3DIter {
        Vector3DIter::new(NonNull::new(self.mTangents), self.mNumVertices as usize)
    }

    pub fn get_tangent(&self, id: u32) -> Option<Vector3D> {
        self.vertex_data(self.mTangents, id)
    }

    pub fn bitangent_iter(&self) -> Vector3DIter {
        Vector3DIter::new(NonNull::new(self.mBitangents), self.mNumVertices as usize)
    }

    pub fn get_bitangent(&self, id: u32) -> Option<Vector3D> {
        self.vertex_data(self.mBitangents, id)
    }

    pub fn vertex_color_iter(&self, set_id: usize) -> Color4DIter {
        Color4DIter::new(
            NonNull::new(self.mColors[set_id]),
            self.mNumVertices as usize,
        )
    }

    pub fn get_vertex_color(&self, set_id: usize, id: u32) -> Option<Color4D> {
        self.color_data(self.mColors[set_id], id)
    }

    pub fn texture_coords_iter(&self, channel_id: usize) -> Vector3DIter {
        Vector3DIter::new(
            NonNull::new(self.mTextureCoords[channel_id]),
            self.mNumVertices as usize,
        )
    }

    pub fn get_texture_coord(&self, channel_id: usize, id: u32) -> Option<Vector3D> {
        self.vertex_data(self.mTextureCoords[channel_id], id)
    }

    pub fn num_faces(&self) -> u32 {
        self.mNumFaces
    }

    pub fn face_iter(&self) -> FaceIter {
        FaceIter::new(NonNull::new(self.mFaces), self.mNumFaces as usize)
    }

    pub fn get_face(&self, id: u32) -> Option<&Face> {
        if id < self.mNumFaces {
            unsafe {
                Some(Face::from_raw(NonNull::new(
                    NonNull::new(self.mFaces)?.as_ptr().offset(id as isize),
                )?))
            }
        } else {
            None
        }
    }

    pub fn num_bones(&self) -> u32 {
        self.mNumBones
    }

    pub fn bone_iter(&self) -> BoneIter {
        BoneIter::new(
            NonNull::new(self.mBones as *mut *const aiBone),
            self.mNumBones as usize,
        )
    }

    pub fn get_bone(&self, id: u32) -> Option<&Bone> {
        if id < self.mNumBones {
            unsafe {
                Some(Bone::from_raw(NonNull::new(
                    *(NonNull::new(self.mBones)?.as_ptr().offset(id as isize)),
                )?))
            }
        } else {
            None
        }
    }

    #[inline]
    fn vertex_data(&self, array: *mut aiVector3D, id: u32) -> Option<Vector3D> {
        if id < self.mNumVertices {
            unsafe { Some(Vector3D::from_raw(*array.offset(id as isize))) }
        } else {
            None
        }
    }

    #[inline]
    fn color_data(&self, array: *mut aiColor4D, id: u32) -> Option<Color4D> {
        if id < self.mNumVertices {
            unsafe { Some(Color4D::from_raw(*array.offset(id as isize))) }
        } else {
            None
        }
    }
}

impl Bone {
    /// Returns the name of the bone.
    pub fn name(&self) -> &str {
        unsafe { crate::aistring_to_cstr(&self.mName) }
            .to_str()
            .unwrap()
    }

    /// Returns the bones's offset transformation matrix.
    pub fn offset_matrix(&self) -> Matrix4x4 {
        Matrix4x4::from_raw(self.mOffsetMatrix)
    }

    /// Get the number of vertex weights
    pub fn num_weights(&self) -> u32 {
        self.mNumWeights
    }

    /// Get an iterator over the vertex weights for this bone
    pub fn weight_iter(&self) -> VertexWeightIter {
        VertexWeightIter::new(
            NonNull::new(self.mWeights as *mut *const aiVertexWeight),
            self.mNumWeights as usize,
        )
    }

    /// Get the nth vertex weight
    pub fn get_weight(&self, id: u32) -> Option<&VertexWeight> {
        if id < self.mNumWeights {
            unsafe {
                Some(VertexWeight::from_raw(NonNull::new(
                    NonNull::new(self.mWeights)?.as_ptr().offset(id as isize),
                )?))
            }
        } else {
            None
        }
    }
}
