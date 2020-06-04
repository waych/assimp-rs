use ffi::{aiBone, aiColor4D, aiMesh, aiVector3D, aiVertexWeight};

use arrayvec::ArrayVec;

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

/// A single vertex
#[derive(Debug, Clone, PartialEq)]
pub struct Vertex {
    /// The position of this vertex, local to the mesh (which may be transformed because of its place
    /// in the node heirarchy)
    pub pos: Vector3D,
    /// The normal. This should be pointing away from the mesh's center but unless you call
    /// `Importer::fix_infacing_normals(true)` before importing the model it's not guaranteed.
    pub normal: Option<Vector3D>,
    /// This vertex's tangent
    pub tangent: Option<Vector3D>,
    /// This vertex's bitangent (see [this StackOverflow answer](https://gamedev.stackexchange.com/a/51402)
    /// for a quick explanation of what this means with respect to computer graphics).
    pub bitangent: Option<Vector3D>,
}

impl Mesh {
    /// This mesh's name (may be empty)
    pub fn name(&self) -> &str {
        unsafe { crate::aistring_to_cstr(&self.mName) }
            .to_str()
            .unwrap()
    }

    /// Returns a bitset of all the primitive types in use in this mesh.
    pub fn primitive_types(&self) -> PrimitiveTypes {
        PrimitiveTypes::from_bits(self.mPrimitiveTypes).unwrap()
    }

    /// The index of this mesh's material in the parent `Model`'s `materials` array.
    pub fn material_id(&self) -> u32 {
        self.mMaterialIndex
    }

    /// The number of unique vertices in this mesh. This is _not_ the same as the total
    /// number of vertices in this mesh, you need to iterate through the faces to get
    /// the indices to the vertices in `vertex`.
    pub fn num_vertices(&self) -> u32 {
        self.mNumVertices
    }

    /// Convenience iterator over the vertices, including positions, normals, tangents and
    /// bitangents. Because of how materials work in Assimp, it doesn't make sense to include
    /// UVs or colors here, as each vertex may have multiple of each.
    pub fn vertices(&self) -> impl Iterator<Item = Vertex> + '_ {
        let mut positions = self.positions();
        let mut normals = self.normals();
        let mut tangents = self.tangents();
        let mut bitangents = self.bitangents();

        std::iter::from_fn(move || {
            let pos = positions.next()?;
            let normal = normals.next();
            let tangent = tangents.next();
            let bitangent = bitangents.next();

            Some(Vertex {
                pos,
                normal,
                tangent,
                bitangent,
            })
        })
    }

    /// Iterator over the unique vertex positions in the mesh. You need to iterate through
    /// faces to get the indices into this iterator in order to render the mesh.
    pub fn positions(&self) -> Vector3DIter {
        // Every format should at least provide vertex positions, so we `unwrap` here instead
        // of returning `None`.
        Vector3DIter::new(NonNull::new(self.mVertices), self.mNumVertices as usize)
    }

    /// Get the position of the nth unique vertex .
    pub fn position(&self, id: u32) -> Option<Vector3D> {
        self.vertex_data(self.mVertices, id)
    }

    /// Iterator over the vertex normals.
    pub fn normals(&self) -> Vector3DIter {
        Vector3DIter::new(NonNull::new(self.mNormals), self.mNumVertices as usize)
    }

    /// Get the normal of the nth unique vertex .
    pub fn normal(&self, id: u32) -> Option<Vector3D> {
        self.vertex_data(self.mNormals, id)
    }

    /// Iterator over the vertex tangents, if available. Not all formats provide tangents,
    pub fn tangents(&self) -> Vector3DIter {
        Vector3DIter::new(NonNull::new(self.mTangents), self.mNumVertices as usize)
    }

    /// Get the tangent of the nth unique vertex.
    pub fn tangent(&self, id: u32) -> Option<Vector3D> {
        self.vertex_data(self.mTangents, id)
    }

    /// Iterator over the vertex bitangents, if available. Not all formats provide bitangents,
    pub fn bitangents(&self) -> Vector3DIter {
        Vector3DIter::new(NonNull::new(self.mBitangents), self.mNumVertices as usize)
    }

    /// Get the bitangent of the nth unique vertex.
    pub fn bitangent(&self, id: u32) -> Option<Vector3D> {
        self.vertex_data(self.mBitangents, id)
    }

    /// Iterator over the vertex colors, if available. Not all formats provide colors,
    pub fn vertex_colors(&self, set_id: u32) -> Color4DIter {
        Color4DIter::new(
            NonNull::new(self.mColors[set_id as usize]),
            self.mNumVertices as usize,
        )
    }

    /// Get the color of the nth unique vertex .
    pub fn vertex_color(&self, set_id: u32, id: u32) -> Option<Color4D> {
        self.color_data(self.mColors[set_id as usize], id)
    }

    /// Iterator over the vertex UVs, if available. Not all formats provide UVs, and even if this
    /// mesh has a material it may be mapped in a way that doesn't require UVs,
    pub fn texture_coords(&self, channel_id: u32) -> Vector3DIter {
        Vector3DIter::new(
            NonNull::new(self.mTextureCoords[channel_id as usize]),
            self.mNumVertices as usize,
        )
    }

    /// Get the UV of the nth unique vertex
    pub fn texture_coord(&self, channel_id: u32, id: u32) -> Option<Vector3D> {
        self.vertex_data(self.mTextureCoords[channel_id as usize], id)
    }

    /// The number of faces in this mesh
    pub fn num_faces(&self) -> u32 {
        self.mNumFaces
    }

    /// Iterator over the faces in this mesh. Each face is described as a list of indices into the vertex
    /// array
    pub fn faces(&self) -> FaceIter {
        FaceIter::new(NonNull::new(self.mFaces), self.mNumFaces as usize)
    }

    /// Get the nth face.
    pub fn face(&self, id: u32) -> Option<&Face> {
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

    pub fn bones(&self) -> BoneIter {
        BoneIter::new(
            NonNull::new(self.mBones as *mut *const aiBone),
            self.mNumBones as usize,
        )
    }

    pub fn bone(&self, id: u32) -> Option<&Bone> {
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
            unsafe {
                Some(Vector3D::from_raw(
                    *NonNull::new(array)?.as_ptr().offset(id as isize),
                ))
            }
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
    pub fn weights(&self) -> VertexWeightIter {
        VertexWeightIter::new(
            NonNull::new(self.mWeights as *mut *const aiVertexWeight),
            self.mNumWeights as usize,
        )
    }

    /// Get the nth vertex weight
    pub fn weight(&self, id: u32) -> Option<&VertexWeight> {
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
