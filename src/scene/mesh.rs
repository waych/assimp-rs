use ffi::{AiMesh, AiVector3D};

use math::vector3::{Vector3D, Vector3DIter};
use super::face::{Face, FaceIter};

define_type_and_iterator_indirect! {
    /// Mesh type (incomplete)
    struct Mesh(&AiMesh)
    /// Mesh iterator type.
    struct MeshIter
}

impl<'a> Mesh<'a> {
    // TODO return as PrimitiveType enum
    pub fn primitive_types(&self) -> u32 {
        self.primitive_types
    }

    pub fn num_vertices(&self) -> u32 {
        self.num_vertices
    }

    pub fn vertex_iter(&self) -> Vector3DIter {
        Vector3DIter::new(self.vertices,
                          self.num_vertices as usize)
    }

    pub fn get_vertex(&self, id: u32) -> Option<Vector3D> {
        self.vertex_data(self.vertices, id)
    }

    pub fn normal_iter(&self) -> Vector3DIter {
        Vector3DIter::new(self.normals,
                          self.num_vertices as usize)
    }

    pub fn get_normal(&self, id: u32) -> Option<Vector3D> {
        self.vertex_data(self.normals, id)
    }

    pub fn tangent_iter(&self) -> Vector3DIter {
        Vector3DIter::new(self.tangents,
                          self.num_vertices as usize)
    }

    pub fn get_tangent(&self, id: u32) -> Option<Vector3D> {
        self.vertex_data(self.tangents, id)
    }

    pub fn bitangent_iter(&self) -> Vector3DIter {
        Vector3DIter::new(self.bitangents,
                          self.num_vertices as usize)
    }

    pub fn get_bitangent(&self, id: u32) -> Option<Vector3D> {
        self.vertex_data(self.bitangents, id)
    }

    pub fn num_faces(&self) -> u32 {
        self.num_faces
    }

    pub fn face_iter(&self) -> FaceIter {
        FaceIter::new(self.faces,
                      self.num_faces as usize)
    }

    #[inline]
    fn vertex_data(&self, array: *mut AiVector3D, id: u32) -> Option<Vector3D> {
        if id < self.num_vertices {
            unsafe { Some(Vector3D::from_raw(array.offset(id as isize))) }
        } else {
            None
        }
    }
}