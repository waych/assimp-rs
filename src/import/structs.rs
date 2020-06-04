//! Argument structs for `Importer` post-processing configuration.

use ffi::*;

use std::convert::TryFrom;

use crate::math::Matrix4x4;

bitflags::bitflags! {
    /// Enumerates components of the Scene and Mesh data structures that can be excluded from the import
    /// using the `remove_component` step.
    ///
    /// See `Importer::remove_component` for more details.
    #[derive(Default)]
    pub struct ComponentTypes: aiComponent {
        const NORMALS                 = aiComponent_aiComponent_NORMALS;
        const TANGENTS_AND_BITANGENTS = aiComponent_aiComponent_TANGENTS_AND_BITANGENTS;
        const COLORS                  = aiComponent_aiComponent_COLORS;
        const TEX_COORDS              = aiComponent_aiComponent_TEXCOORDS;
        const BONE_WEIGHTS            = aiComponent_aiComponent_BONEWEIGHTS;
        const ANIMATIONS              = aiComponent_aiComponent_ANIMATIONS;
        const TEXTURES                = aiComponent_aiComponent_TEXTURES;
        const LIGHTS                  = aiComponent_aiComponent_LIGHTS;
        const CAMERAS                 = aiComponent_aiComponent_CAMERAS;
        const MESHES                  = aiComponent_aiComponent_MESHES;
        const MATERIALS               = aiComponent_aiComponent_MATERIALS;
    }
}

bitflags::bitflags! {
    #[derive(Default)]
    pub struct UVTransformFlags: u32 {
        const SCALING     = AI_UVTRAFO_SCALING;
        const ROTATION    = AI_UVTRAFO_ROTATION;
        const TRANSLATION = AI_UVTRAFO_TRANSLATION;
        const ALL         = AI_UVTRAFO_ALL;
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u32)]
pub enum PrimitiveType {
    Point = aiPrimitiveType_aiPrimitiveType_POINT,
    Line = aiPrimitiveType_aiPrimitiveType_LINE,
    Triangle = aiPrimitiveType_aiPrimitiveType_TRIANGLE,
    Polygon = aiPrimitiveType_aiPrimitiveType_POLYGON,
}

impl TryFrom<u32> for PrimitiveType {
    type Error = ();

    fn try_from(other: u32) -> Result<Self, ()> {
        match other {
            ffi::aiPrimitiveType_aiPrimitiveType_POINT => Ok(Self::Point),
            ffi::aiPrimitiveType_aiPrimitiveType_LINE => Ok(Self::Line),
            ffi::aiPrimitiveType_aiPrimitiveType_TRIANGLE => Ok(Self::Triangle),
            ffi::aiPrimitiveType_aiPrimitiveType_POLYGON => Ok(Self::Polygon),
            _ => Err(()),
        }
    }
}

bitflags::bitflags! {
    /// A bitset of all of the primitive types that are used for the faces of a mesh.
    #[derive(Default)]
    pub struct PrimitiveTypes: aiPrimitiveType {
        /// Just a single point - a `POINT`-type face contains precisely one index
        const POINT    = aiPrimitiveType_aiPrimitiveType_POINT;
        /// A line - a `LINE`-type face contains precisely two indices
        const LINE     = aiPrimitiveType_aiPrimitiveType_LINE;
        /// A triangle - by default the winding order is counter-clockwise (formats
        /// that wind clockwise will be converted on import) but this can be changed
        /// in the `Importer`. A `TRIANGLE`-type face contains precisely three
        /// indices.
        const TRIANGLE = aiPrimitiveType_aiPrimitiveType_TRIANGLE;
        /// A polygon - like `TRIANGLE`, by default the winding order is
        /// counter-clockwise. A `POLYGON`-type face can have any number of vertices,
        /// and although the documentation isn't fully clear, it seems like it's
        /// possible for polygons to be concave or otherwise degenerate. You can
        /// convert all `POLYGON`-type faces to `TRIANGLE`-type faces by enabling
        /// `triangulate` in the importer.
        const POLYGON  = aiPrimitiveType_aiPrimitiveType_POLYGON;
    }
}

// Macro to simplify defining and structs and implementing Default trait
// NOTE: pub keyword in field definition is to workaround rust issue #24189
macro_rules! struct_with_defaults {
    ($(#[$struct_attr:meta])* struct $i:ident {
        $($(#[$field_attr:meta])* pub $n:ident: $t:ty = $v:expr),*
    }) => (
        $(#[$struct_attr])*
        pub struct $i {
            /// Whether to enable the step. Default: false
            pub enable: bool,
            $($(#[$field_attr])* pub $n: $t),*
        }

        impl Default for $i {
            fn default() -> $i {
                $i {
                    enable: false,
                    $($n: $v),*
                }
            }
        }
    )
}

struct_with_defaults! {
    /// Arguments for `calc_tangent_space` post-process step.
    struct CalcTangentSpace {
        /// Maximum angle between two vertex tangents used for smoothing. Default: 45.0
        pub max_smoothing_angle: f32 = 45.0,
        /// Source UV channel for tangent space computation. Default: 0
        pub texture_channel: i32 = 0
    }
}

struct_with_defaults! {
    /// Arguments for `remove_component` post-process step.
    struct RemoveComponent {
        /// Specify which components to remove. Default: none
        pub components: ComponentTypes = Default::default()
    }
}

struct_with_defaults! {
    /// Arguments for `generate_normals` post-process step.
    struct GenerateNormals {
        /// Whether the generated normals are smoothed or not. Default: false
        pub smooth: bool = false,
        /// Maximum angle between two vertex normals used for smoothing. Default: 175.0
        /// Only applies if `smooth` is `true`.
        pub max_smoothing_angle: f32 = 175.0
    }
}

struct_with_defaults! {
    /// Arguments for `split_large_meshes` post-process step.
    struct SplitLargeMeshes {
        /// Maximum number of vertices per mesh. Default: 1000000
        pub vertex_limit: u32 = AI_SLM_DEFAULT_MAX_VERTICES,
        /// Maximum number of triangles per mesh. Default: 1000000
        pub triangle_limit: u32 = AI_SLM_DEFAULT_MAX_TRIANGLES
    }
}

struct_with_defaults! {
    /// Arguments for `pre_transform_vertices` post-process step.
    struct PreTransformVertices {
        /// Whether to keep the existing scene hierarchy. Default: false
        pub keep_hierarchy: bool = false,
        /// Whether to normalize all vertices into the [-1, 1] range. Default: false
        pub normalize: bool = false,
        /// Whether to pre-transform all vertices using the matrix specified in the
        /// `root_transformation` field. Default: false
        pub add_root_transformation: bool = false,
        /// Transformation matrix to use.
        pub root_transformation: Matrix4x4 = Matrix4x4::new(1.0, 0.0, 0.0, 0.0,
                                                            0.0, 1.0, 0.0, 0.0,
                                                            0.0, 0.0, 1.0, 0.0,
                                                            0.0, 0.0, 0.0, 1.0)
    }
}

struct_with_defaults! {
    /// Arguments for `limit_bone_weights` post-process step.
    struct LimitBoneWeights {
        /// Maximum number of bones that affect a single vertex. Default: 4
        pub max_weights: u32 = AI_LMW_MAX_WEIGHTS
    }
}

struct_with_defaults! {
    /// Arguments for `improve_cache_locality` post-process step.
    struct ImproveCacheLocality {
        /// Set the size of the post-transform vertex cache. Default: 12
        pub cache_size: u32 = PP_ICL_PTCACHE_SIZE
    }
}

struct_with_defaults! {
    /// Arguments for `remove_redundant_materials` post-process step.
    struct RemoveRedundantMaterials {
        /// Space-delimited list of materials to keep. Identifiers containing whitespace must be
        /// enclosed in single quotes. e.g. `material1 'material 2' material3`.
        pub exclude_list: String = String::new()
    }
}

struct_with_defaults! {
    /// Arguments for `sort_by_primitive_type` post-process step.
    struct SortByPrimitiveType {
        /// List of primitive types to remove. Default: none
        pub remove: PrimitiveTypes = Default::default()
    }
}

struct_with_defaults! {
    /// Arguments for `find_degenerates` post-process step.
    struct FindDegenerates {
        /// Whether to remove any found degenerates. Default: true
        pub remove: bool = false
    }
}

struct_with_defaults! {
    /// Arguments for `find_invalid_data` post-process step.
    struct FindInvalidData {
        /// Specify the accuracy for considering animation values as invalid. Default: 0
        pub accuracy: f32 = 0.0
    }
}

struct_with_defaults! {
    /// Arguments for `transform_uv_coords` post-process step.
    struct TransformUVCoords {
        /// Specify which UV transforms to evaluate. Default: all
        pub flags: UVTransformFlags = UVTransformFlags::ALL
    }
}

struct_with_defaults! {
    /// Arguments for `optimize_graph` post-process step.
    struct OptimizeGraph {
        /// Space-delimited list of nodes to keep. Identifiers containing whitespace must be
        /// enclosed in single quotes. e.g. `node1 'node 2' node3`.
        pub exclude_list: String = String::new()
    }
}

struct_with_defaults! {
    /// Arguments for `split_by_bone_count` post-process step.
    struct SplitByBoneCount {
        /// Maximum number of bones per mesh. Default: 60
        pub max_bones: u32 = AI_SBBC_DEFAULT_MAX_BONES
    }
}

struct_with_defaults! {
    /// Arguments for `debone` post-process step.
    struct Debone {
        /// Threshold for considering bone necessary. Default: 1.0
        pub threshold: f64 = AI_DEBONE_THRESHOLD,
        /// Whether to require all bones to meet the threshold before removing any. Default: false
        pub all_or_none: bool = false
    }
}
