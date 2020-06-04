use crate::math::{Color3D, Color4D, Vector3D};
use derive_more::{From, TryInto};
use ffi::{
    aiBlendMode_aiBlendMode_Additive, aiBlendMode_aiBlendMode_Default, aiGetMaterialColor,
    aiGetMaterialFloatArray, aiGetMaterialIntegerArray, aiGetMaterialString, aiGetMaterialTexture,
    aiGetMaterialTextureCount, aiGetMaterialUVTransform, aiMaterial, aiMaterialProperty,
    aiShadingMode_aiShadingMode_Blinn, aiShadingMode_aiShadingMode_CookTorrance,
    aiShadingMode_aiShadingMode_Flat, aiShadingMode_aiShadingMode_Fresnel,
    aiShadingMode_aiShadingMode_Gouraud, aiShadingMode_aiShadingMode_Minnaert,
    aiShadingMode_aiShadingMode_NoShading, aiShadingMode_aiShadingMode_OrenNayar,
    aiShadingMode_aiShadingMode_Phong, aiShadingMode_aiShadingMode_Toon,
    aiTextureFlags__aiTextureFlags_Force32Bit, aiTextureFlags_aiTextureFlags_IgnoreAlpha,
    aiTextureFlags_aiTextureFlags_Invert, aiTextureFlags_aiTextureFlags_UseAlpha,
    aiTextureMapMode_aiTextureMapMode_Clamp, aiTextureMapMode_aiTextureMapMode_Decal,
    aiTextureMapMode_aiTextureMapMode_Mirror, aiTextureMapMode_aiTextureMapMode_Wrap,
    aiTextureMapping_aiTextureMapping_BOX, aiTextureMapping_aiTextureMapping_CYLINDER,
    aiTextureMapping_aiTextureMapping_PLANE, aiTextureMapping_aiTextureMapping_SPHERE,
    aiTextureMapping_aiTextureMapping_UV, aiTextureOp_aiTextureOp_Add,
    aiTextureOp_aiTextureOp_Divide, aiTextureOp_aiTextureOp_Multiply,
    aiTextureOp_aiTextureOp_SignedAdd, aiTextureOp_aiTextureOp_SmoothAdd,
    aiTextureOp_aiTextureOp_Subtract, aiTextureType_aiTextureType_AMBIENT,
    aiTextureType_aiTextureType_DIFFUSE, aiTextureType_aiTextureType_DISPLACEMENT,
    aiTextureType_aiTextureType_EMISSIVE, aiTextureType_aiTextureType_LIGHTMAP,
    aiTextureType_aiTextureType_OPACITY, aiTextureType_aiTextureType_REFLECTION,
    aiTextureType_aiTextureType_SPECULAR, aiTextureType_aiTextureType_UNKNOWN,
    _AI_MATKEY_MAPPINGMODE_U_BASE, _AI_MATKEY_MAPPINGMODE_V_BASE, _AI_MATKEY_MAPPING_BASE,
    _AI_MATKEY_TEXBLEND_BASE, _AI_MATKEY_TEXFLAGS_BASE, _AI_MATKEY_TEXMAP_AXIS_BASE,
    _AI_MATKEY_TEXOP_BASE, _AI_MATKEY_TEXTURE_BASE, _AI_MATKEY_UVWSRC_BASE,
};
use std::convert::{TryFrom, TryInto};
use std::{any::Any, ffi::CStr};

define_type_and_iterator_indirect! {
    /// A single material. This is _not_ the same as a single texture, and in fact a
    /// single material can combine many textures, and even have multiple UV mapping methods
    /// even when applied to the same mesh.
    struct Material(&aiMaterial)
    /// Iterator over materials
    struct MaterialIter
}

define_type_and_iterator_indirect! {
    /// Material type (not yet implemented)
    struct MaterialProperty(&aiMaterialProperty)
    /// Material iterator type.
    struct MaterialPropertyIter
}

/// A dynamically-typed value of a material property.
#[derive(TryInto, From, PartialEq, Debug)]
pub enum MaterialValue {
    Color3D(Color3D),
    String(crate::InlineString),
    Float(f32),
    Int(u32),
    Vector3D(Vector3D),

    Bool(bool),
    ShadingModel(ShadingModel),
    MaterialBlendOp(MaterialBlendOp),
    BlendOp(BlendOp),
    Mapping(Mapping),
    WrappingMode(WrappingMode),
    TextureFlags(TextureFlags),
}

impl Material {
    /// A single component of this material, see the documentation for `MaterialComponent` for more
    /// information.
    pub fn component(
        &self,
        type_: MaterialComponentType,
    ) -> Option<MaterialComponent<impl ExactSizeIterator<Item = TextureDefinition> + '_>> {
        use std::mem::MaybeUninit;

        let color: Option<Color3D> = self
            .get_value(MaterialKey::Color(type_))
            .and_then(|val| val.try_into().ok());

        let count = self.num_textures(type_);
        let color = if count == 0 {
            color?
        } else {
            color.unwrap_or_default()
        };

        let textures = (0..count).map(move |index| {
            let mut path = MaybeUninit::uninit();
            let mut mapping = MaybeUninit::uninit();
            let mut uvindex = MaybeUninit::zeroed();
            let mut blend = MaybeUninit::uninit();
            let mut op = MaybeUninit::uninit();
            let mut mapmode = MaybeUninit::uninit();
            let mut flags = MaybeUninit::uninit();

            crate::aireturn_to_result(unsafe {
                aiGetMaterialTexture(
                    &self.0,
                    type_ as u32,
                    index,
                    path.as_mut_ptr(),
                    mapping.as_mut_ptr(),
                    uvindex.as_mut_ptr(),
                    blend.as_mut_ptr(),
                    op.as_mut_ptr(),
                    mapmode.as_mut_ptr(),
                    flags.as_mut_ptr(),
                )
            })
            .ok()
            .expect(
                "Somehow getting the texture failed, even though \
                we used data that should be valid",
            );

            let mapping = unsafe { Mapping::try_from(mapping.assume_init()) }.ok();

            let axis = if mapping.is_none() || mapping == Some(Mapping::UV) {
                None
            } else {
                self.get_value(MaterialKey::TextureMapAxis(type_, index))
                    .and_then(|val| val.try_into().ok())
            };

            let wrap_u = self
                .get_value(MaterialKey::MappingModeU(type_, index))
                .and_then(|val| val.try_into().ok());

            let wrap_v = self
                .get_value(MaterialKey::MappingModeV(type_, index))
                .and_then(|val| val.try_into().ok());

            let blend_op = BlendOp::try_from(unsafe { op.assume_init() })
                .ok()
                .unwrap_or_default();

            unsafe {
                TextureDefinition {
                    path: crate::InlineString(path.assume_init()),
                    strength: if blend_op == BlendOp::Replace {
                        // This value isn't set when the blend operation is undefined.
                        1.0
                    } else {
                        blend.assume_init()
                    },
                    blend_op,
                    mapping,
                    axis,
                    channel: (uvindex.assume_init() as i32)
                        .try_into()
                        .ok()
                        .unwrap_or_default(),
                    wrap_u,
                    wrap_v,
                    flags: TextureFlags::from_bits(flags.assume_init()).unwrap_or_default(),
                }
            }
        });

        Some(MaterialComponent { color, textures })
    }

    pub fn num_textures(&self, type_: MaterialComponentType) -> u32 {
        unsafe { aiGetMaterialTextureCount(&self.0, type_ as u32) }
    }

    pub fn get_value(&self, key: MaterialKey) -> Option<MaterialValue> {
        use std::mem::MaybeUninit;

        let (base, type_, index) = key.triple()?;

        let value_type = key.type_();

        Some(match value_type {
            // These are the types that Assimp understands natively
            ValueType::Color3D => {
                let mut out = MaybeUninit::uninit();

                crate::aireturn_to_result(unsafe {
                    aiGetMaterialColor(&self.0, base.as_ptr(), type_, index, out.as_mut_ptr())
                })
                .ok()?;

                MaterialValue::Color3D(Color4D(unsafe { out.assume_init() }).into())
            }
            ValueType::String => {
                let mut out = MaybeUninit::uninit();

                crate::aireturn_to_result(unsafe {
                    aiGetMaterialString(&self.0, base.as_ptr(), type_, index, out.as_mut_ptr())
                })
                .ok()?;

                MaterialValue::String(crate::InlineString(unsafe { out.assume_init() }))
            }
            ValueType::Float => {
                let mut out = MaybeUninit::uninit();

                crate::aireturn_to_result(unsafe {
                    aiGetMaterialFloatArray(
                        &self.0,
                        base.as_ptr(),
                        type_,
                        index,
                        out.as_mut_ptr(),
                        std::ptr::null_mut(),
                    )
                })
                .ok()?;

                MaterialValue::Float(unsafe { out.assume_init() })
            }
            ValueType::Int
            | ValueType::Bool
            | ValueType::ShadingModel
            | ValueType::MaterialBlendOp
            | ValueType::BlendOp
            | ValueType::Mapping
            | ValueType::WrappingMode
            | ValueType::TextureFlags => {
                let mut out = MaybeUninit::uninit();

                crate::aireturn_to_result(unsafe {
                    aiGetMaterialIntegerArray(
                        &self.0,
                        base.as_ptr(),
                        type_,
                        index,
                        out.as_mut_ptr(),
                        std::ptr::null_mut(),
                    )
                })
                .ok()?;

                let out = unsafe { out.assume_init() } as u32;

                match value_type {
                    ValueType::Int => MaterialValue::Int(out),
                    ValueType::Bool => MaterialValue::Bool(out != 0),
                    ValueType::ShadingModel => MaterialValue::ShadingModel(out.try_into().ok()?),
                    ValueType::MaterialBlendOp => {
                        MaterialValue::MaterialBlendOp(out.try_into().ok()?)
                    }
                    ValueType::BlendOp => MaterialValue::BlendOp(out.try_into().ok()?),
                    ValueType::Mapping => MaterialValue::Mapping(out.try_into().ok()?),
                    ValueType::WrappingMode => MaterialValue::WrappingMode(out.try_into().ok()?),
                    ValueType::TextureFlags => {
                        MaterialValue::TextureFlags(TextureFlags::from_bits(out)?)
                    }
                    _ => unreachable!(),
                }
            }
            ValueType::Vector3D => todo!(
                "Getting vector properties from materials currently unimplemented: \
                    The documentation has some pretty weird stuff here, it confusingly says that \
                    we should use the `pMax` parameter to specify the size in floats, but the \
                    example code (NOT tests, this is only in the documentation and therefore \
                    may be wrong or out of date) passes the requested size in _bytes_."
            ),
        })
    }

    /// The "diffuse" component of the material - this is likely to be rendered using gourard shading.
    pub fn diffuse(
        &self,
    ) -> Option<MaterialComponent<impl ExactSizeIterator<Item = TextureDefinition> + '_>> {
        self.component(MaterialComponentType::Diffuse)
    }

    /// The "specular" component of the material - this is likely to be rendered using phong shading.
    pub fn specular(
        &self,
    ) -> Option<MaterialComponent<impl ExactSizeIterator<Item = TextureDefinition> + '_>> {
        self.component(MaterialComponentType::Specular)
    }

    /// The "ambient" component of the material - this is likely to be rendered using flat shading,
    /// multiplied by ambient light.
    pub fn ambient(
        &self,
    ) -> Option<MaterialComponent<impl ExactSizeIterator<Item = TextureDefinition> + '_>> {
        self.component(MaterialComponentType::Ambient)
    }

    /// The "emissive" component of the material - this is likely to be rendered using flat shading,
    /// without being affected by light.
    pub fn emissive(
        &self,
    ) -> Option<MaterialComponent<impl ExactSizeIterator<Item = TextureDefinition> + '_>> {
        self.component(MaterialComponentType::Emissive)
    }

    /// The transparency of the material. According to the sample code in the documentation, it appears
    /// as if the recommended way to treat this is as per-component.
    pub fn opacity(
        &self,
    ) -> Option<MaterialComponent<impl ExactSizeIterator<Item = TextureDefinition> + '_>> {
        self.component(MaterialComponentType::Opacity)
    }
}

/// A component of a material - see `MaterialComponentType` for what the different components can be.
/// For each component, the resultant texture is created by combining many textures together,
pub struct MaterialComponent<I> {
    /// The base color of this material component, which the rest of the "stack" will be based on
    pub color: Color3D,
    /// The iterator of textures, each of which may be `None` (which represents an invalid texture)
    pub textures: I,
}

/// The component of this material - these affect how the supplied textures interact with light.
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum MaterialComponentType {
    /// The "diffuse" component of the material - this is likely to be rendered using gourard shading.
    Diffuse = aiTextureType_aiTextureType_DIFFUSE,
    /// The "specular" component of the material - this is likely to be rendered using phong shading.
    Specular = aiTextureType_aiTextureType_SPECULAR,
    /// The "ambient" component of the material - this is likely to be rendered using flat shading,
    /// multiplied by ambient light.
    Ambient = aiTextureType_aiTextureType_AMBIENT,
    /// The "emissive" component of the material - this is likely to be rendered using flat shading,
    /// without being affected by light.
    Emissive = aiTextureType_aiTextureType_EMISSIVE,
    /// The transparency of the material. According to the sample code in the documentation, it appears
    /// as if the recommended way to treat this is as per-component.
    Opacity = aiTextureType_aiTextureType_OPACITY,
    /// The displacement of the material. The exact way to interpret this is application-dependent.
    Displacement = aiTextureType_aiTextureType_DISPLACEMENT,
    /// The lightmap for the material. As light is affected by an element's placement in a larger scene,
    /// for many materials this will just be used for ambient occlusion.
    Lightmap = aiTextureType_aiTextureType_LIGHTMAP,
    /// Reflectivity map - this is likely application-dependent, and real-time applications probably
    /// don't need to worry about it.
    Reflection = aiTextureType_aiTextureType_REFLECTION,
    /// Unknown material component - accessible but not processed in any way by Assimp.
    Unknown = aiTextureType_aiTextureType_UNKNOWN,
}

/// The shading model that meshes with this material applied will use - this is just a hint. The shading
/// models here map roughly to shading models in Blender, and are only meant as a way to roughly
/// approximate the intended shading method. Most applications can ignore this.
///
/// Even though many of these shading models are not sufficient for most materials alone (most renderers
/// will use a combination of these models depending on the material component), it should give a rough
/// idea of the original intent.
#[repr(u32)]
#[derive(Debug, PartialEq, Hash)]
pub enum ShadingModel {
    /// Flat shading.
    ///
    /// Shading is done on per-face base, diffuse only. Also known as 'faceted shading'.
    Flat = aiShadingMode_aiShadingMode_Flat,
    /// Simple Gouraud shading (i.e. diffuse shading).
    Gouraud = aiShadingMode_aiShadingMode_Gouraud,
    /// Phong shading (i.e. uses the specular map).
    Phong = aiShadingMode_aiShadingMode_Phong,
    /// Phong-Blinn shading, or "modified Phong shading".
    Blinn = aiShadingMode_aiShadingMode_Blinn,
    /// Per-pixel Toon shading, or "comic" shading. The precise meaning of this will likely be
    /// application-dependent
    Toon = aiShadingMode_aiShadingMode_Toon,
    /// OrenNayar-Shading per pixel.
    ///
    /// Extension to standard Lambertian shading, taking the roughness of the material into account
    OrenNayar = aiShadingMode_aiShadingMode_OrenNayar,
    /// Minnaert-Shading per pixel.
    ///
    /// Extension to standard Lambertian shading, taking the "darkness" of the material into account
    Minnaert = aiShadingMode_aiShadingMode_Minnaert,
    /// Cook-Torrance per pixel.
    ///
    /// Special shader for metallic surfaces.
    CookTorrance = aiShadingMode_aiShadingMode_CookTorrance,
    /// No shading at all (does not take light into account)
    NoShading = aiShadingMode_aiShadingMode_NoShading,
    /// Fresnel shading (i.e. glancing reflections)
    Fresnel = aiShadingMode_aiShadingMode_Fresnel,
}

impl TryFrom<u32> for ShadingModel {
    type Error = ();

    fn try_from(other: u32) -> Result<Self, Self::Error> {
        match other {
            ffi::aiShadingMode_aiShadingMode_Flat => Ok(Self::Flat),
            ffi::aiShadingMode_aiShadingMode_Gouraud => Ok(Self::Gouraud),
            ffi::aiShadingMode_aiShadingMode_Phong => Ok(Self::Phong),
            ffi::aiShadingMode_aiShadingMode_Blinn => Ok(Self::Blinn),
            ffi::aiShadingMode_aiShadingMode_Toon => Ok(Self::Toon),
            ffi::aiShadingMode_aiShadingMode_OrenNayar => Ok(Self::OrenNayar),
            ffi::aiShadingMode_aiShadingMode_Minnaert => Ok(Self::Minnaert),
            ffi::aiShadingMode_aiShadingMode_CookTorrance => Ok(Self::CookTorrance),
            ffi::aiShadingMode_aiShadingMode_NoShading => Ok(Self::NoShading),
            ffi::aiShadingMode_aiShadingMode_Fresnel => Ok(Self::Fresnel),
            _ => Err(()),
        }
    }
}

/// The "key" for each field of the material, which can be used to extract single fields of the material.
/// However, it is likely easier to use the helper methods on the `Material` struct.
#[derive(Debug, PartialEq, Eq, Hash)]
pub enum MaterialKey {
    /// The material's name, if applicable
    Name,
    /// The base color for the given component.
    Color(MaterialComponentType),
    /// Whether faces with this material applied should be rendered using wireframe mode (whatever that
    /// means for your renderer).
    Wireframe,
    /// Whether faces with this material applied will have backface culling.
    TwoSided,
    /// The (real-time) shading model, i.e. how to render this material. Depending on this, different
    /// components will be available and may have different meanings. This is just a hint, and treating
    /// the diffuse component using gouraud, specular using phong, and ambient/emissive using flat
    /// shading will do the right thing in most circumstances.
    ShadingModel,
    /// The blend method for this material - i.e., when rendering a face with this material, how to combine
    /// this material's calculated color with the objects behind it.
    BlendFunc,
    /// The opacity of the material, the amount to multiply the alpha component by.
    Opacity,
    /// The "shininess", which is the exponent for phong shading.
    Shininess,
    /// Amount to multiply the specular component of the material by before using it for phong calculation.
    ShininessStrength,
    /// The "index of refraction" for this material. Has some advanced usecases but not even
    /// available in the majority of formats and most renderers can ignore it.
    RefractionIndex,
    Texture(MaterialComponentType, u32),
    TextureBlend(MaterialComponentType, u32),
    TextureOp(MaterialComponentType, u32),
    Mapping(MaterialComponentType, u32),
    UVWSource(MaterialComponentType, u32),
    MappingModeU(MaterialComponentType, u32),
    MappingModeV(MaterialComponentType, u32),
    TextureMapAxis(MaterialComponentType, u32),
    Flags(MaterialComponentType, u32),
}

enum ValueType {
    // These are the types that Assimp understands natively
    Color3D,
    String,
    Float,
    Int,
    Vector3D,

    // These are the types which we convert from native Assimp types for ergonomics purposes
    /// Assimp only deals in ints, so we convert to a boolean for appropriate properties.
    Bool,
    /// We convert an int to this type.
    ShadingModel,
    /// We convert an int to this type.
    MaterialBlendOp,
    /// We convert an int to this type.
    BlendOp,
    /// We convert an int to this type.
    Mapping,
    /// We convert an int to this type.
    WrappingMode,
    /// We convert an int to this type.
    TextureFlags,
}

impl MaterialKey {
    fn triple(&self) -> Option<(&'static CStr, u32, u32)> {
        // We have to copy some strings from `include/assimp/material.h` since
        // they're macros that expand to multiple arguments(!)
        let (name, type_, index): (&[u8], _, _) = match self {
            // I think the name starts with `?` because it's optional, but this is really inconsistent.
            // Old C++ codebases, I guess.
            MaterialKey::Name => (b"?mat.name\0", 0, 0),
            MaterialKey::Color(MaterialComponentType::Diffuse) => (b"$clr.diffuse\0", 0, 0),
            MaterialKey::Color(MaterialComponentType::Ambient) => (b"$clr.ambient\0", 0, 0),
            MaterialKey::Color(MaterialComponentType::Specular) => (b"$clr.specular\0", 0, 0),
            MaterialKey::Color(MaterialComponentType::Emissive) => (b"$clr.emissive\0", 0, 0),
            MaterialKey::Color(MaterialComponentType::Opacity) => (b"$clr.transparent\0", 0, 0),
            MaterialKey::Color(MaterialComponentType::Reflection) => (b"$clr.reflective\0", 0, 0),
            MaterialKey::Color(_) => return None,
            MaterialKey::Wireframe => (b"$mat.wireframe\0", 0, 0),
            MaterialKey::TwoSided => (b"$mat.twosided\0", 0, 0),
            MaterialKey::ShadingModel => (b"$mat.shadingm\0", 0, 0),
            MaterialKey::BlendFunc => (b"$mat.blend\0", 0, 0),
            MaterialKey::Opacity => (b"$mat.opacity\0", 0, 0),
            MaterialKey::Shininess => (b"$mat.shininess\0", 0, 0),
            MaterialKey::ShininessStrength => (b"$mat.shinpercent\0", 0, 0),
            MaterialKey::RefractionIndex => (b"$mat.refracti\0", 0, 0),
            MaterialKey::Texture(comp, index) => (_AI_MATKEY_TEXTURE_BASE, *comp as u32, *index),
            MaterialKey::TextureBlend(comp, index) => {
                (_AI_MATKEY_TEXBLEND_BASE, *comp as u32, *index)
            }
            MaterialKey::TextureOp(comp, index) => (_AI_MATKEY_TEXOP_BASE, *comp as u32, *index),
            MaterialKey::Mapping(comp, index) => (_AI_MATKEY_MAPPING_BASE, *comp as u32, *index),
            MaterialKey::UVWSource(comp, index) => (_AI_MATKEY_UVWSRC_BASE, *comp as u32, *index),
            MaterialKey::MappingModeU(comp, index) => {
                (_AI_MATKEY_MAPPINGMODE_U_BASE, *comp as u32, *index)
            }
            MaterialKey::MappingModeV(comp, index) => {
                (_AI_MATKEY_MAPPINGMODE_V_BASE, *comp as u32, *index)
            }
            MaterialKey::TextureMapAxis(comp, index) => {
                (_AI_MATKEY_TEXMAP_AXIS_BASE, *comp as u32, *index)
            }
            MaterialKey::Flags(comp, index) => (_AI_MATKEY_TEXFLAGS_BASE, *comp as u32, *index),
        };

        Some((CStr::from_bytes_with_nul(name).unwrap(), type_, index))
    }

    fn type_(&self) -> ValueType {
        match self {
            MaterialKey::Name => ValueType::String,
            MaterialKey::Color(..) => ValueType::Color3D,
            MaterialKey::Wireframe => ValueType::Bool,
            MaterialKey::TwoSided => ValueType::Bool,
            MaterialKey::ShadingModel => ValueType::ShadingModel,
            MaterialKey::BlendFunc => ValueType::MaterialBlendOp,
            MaterialKey::Opacity => ValueType::Float,
            MaterialKey::Shininess => ValueType::Float,
            MaterialKey::ShininessStrength => ValueType::Float,
            MaterialKey::RefractionIndex => ValueType::Float,
            MaterialKey::Texture(..) => ValueType::String,
            MaterialKey::TextureBlend(..) => ValueType::Float,
            MaterialKey::TextureOp(..) => ValueType::BlendOp,
            MaterialKey::Mapping(..) => ValueType::Mapping,
            MaterialKey::UVWSource(..) => ValueType::Int,
            MaterialKey::MappingModeU(..) => ValueType::WrappingMode,
            MaterialKey::MappingModeV(..) => ValueType::WrappingMode,
            MaterialKey::TextureMapAxis(..) => ValueType::Vector3D,
            MaterialKey::Flags(..) => ValueType::TextureFlags,
        }
    }
}

/// The blend operation - i.e., how each pixel of the current texture on the "stack" (for the relevant
/// component of the material) should be blended with the combination of all textures before this one.
#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum BlendOp {
    /// For each pixel, `out = prev * cur`
    Multiply = aiTextureOp_aiTextureOp_Multiply,
    /// For each pixel, `out = prev + cur`
    Add = aiTextureOp_aiTextureOp_Add,
    /// For each pixel, `out = prev - cur`
    Subtract = aiTextureOp_aiTextureOp_Subtract,
    /// For each pixel, `out = prev / cur`
    Divide = aiTextureOp_aiTextureOp_Divide,
    /// For each pixel, `out = (prev + cur) - (prev * cur)`
    SmoothAdd = aiTextureOp_aiTextureOp_SmoothAdd,
    /// For each pixel, `out = prev + (cur - 0.5)`
    SignedAdd = aiTextureOp_aiTextureOp_SignedAdd,
    /// No blending, simply replace the previous color.
    Replace,
}

impl Default for BlendOp {
    fn default() -> Self {
        BlendOp::Replace
    }
}

impl TryFrom<u32> for BlendOp {
    type Error = ();

    fn try_from(other: u32) -> Result<Self, Self::Error> {
        match other {
            ffi::aiTextureOp_aiTextureOp_Multiply => Ok(Self::Multiply),
            ffi::aiTextureOp_aiTextureOp_Add => Ok(Self::Add),
            ffi::aiTextureOp_aiTextureOp_Subtract => Ok(Self::Subtract),
            ffi::aiTextureOp_aiTextureOp_Divide => Ok(Self::Divide),
            ffi::aiTextureOp_aiTextureOp_SmoothAdd => Ok(Self::SmoothAdd),
            ffi::aiTextureOp_aiTextureOp_SignedAdd => Ok(Self::SignedAdd),
            _ => Err(()),
        }
    }
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum MaterialBlendOp {
    Additive = aiBlendMode_aiBlendMode_Additive,
    Default = aiBlendMode_aiBlendMode_Default,
}

impl TryFrom<u32> for MaterialBlendOp {
    type Error = ();

    fn try_from(other: u32) -> Result<Self, Self::Error> {
        match other {
            ffi::aiBlendMode_aiBlendMode_Additive => Ok(Self::Additive),
            ffi::aiBlendMode_aiBlendMode_Default => Ok(Self::Default),
            _ => Err(()),
        }
    }
}

/// The way the texture is mapped - it is _highly_ recommended to use `.gen_uv_coords(true)` on the
/// importer, which will convert all mappings to `UV`, which is the easiest to render. The Assimp
/// documentation is unclear as to precisely how the `axis` field of each of the other kinds of
/// mapping should be interpreted, and so it's almost certainly best to just let the library do
/// the conversion for you.
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Mapping {
    /// Each vertex has a unique `U` and `V` coordinate, which maps directly to
    /// a position in the texture.
    UV = aiTextureMapping_aiTextureMapping_UV,
    /// Each pixel's UV coordinates are derived from a sphere with the supplied
    /// axis, where U is the longitude of the pixel on the sphere, and V is the
    /// latitude of the pixel on the sphere.
    ///
    /// > TODO: Is the center of the sphere always the mesh's origin?
    Sphere = aiTextureMapping_aiTextureMapping_SPHERE,
    /// > TODO: The documentation doesn't state exactly how this works.
    /// >
    /// > My assumption is that each vertex has either their U or V defined, with the other
    /// > being derived from cylindrical mapping with the supplied `axis` being the normal.
    Cylinder = aiTextureMapping_aiTextureMapping_CYLINDER,
    /// > TODO: The documentation doesn't state exactly how this works.
    /// >
    /// > My assumption is that the extent of the `axis` somehow defines the size of the box,
    /// > or that each vertex has UV coordinates that relate to this somehow.
    Box = aiTextureMapping_aiTextureMapping_BOX,
    /// > TODO: The documentation doesn't state exactly how this works.
    /// >
    /// > My assumption is that the extent of this axis somehow defines the size of this plane,
    /// > or that each vertex's UV coordinates should be interpreted as being on this plane instead
    /// > of being in relation to the texture itself.
    Plane = aiTextureMapping_aiTextureMapping_PLANE,
}

impl TryFrom<u32> for Mapping {
    type Error = ();

    fn try_from(other: u32) -> Result<Self, Self::Error> {
        match other {
            ffi::aiTextureMapping_aiTextureMapping_UV => Ok(Self::UV),
            ffi::aiTextureMapping_aiTextureMapping_SPHERE => Ok(Self::Sphere),
            ffi::aiTextureMapping_aiTextureMapping_CYLINDER => Ok(Self::Cylinder),
            ffi::aiTextureMapping_aiTextureMapping_BOX => Ok(Self::Box),
            ffi::aiTextureMapping_aiTextureMapping_PLANE => Ok(Self::Plane),
            _ => Err(()),
        }
    }
}

/// Behavior when a texture's U or V coordinates are outside the bounds of 0..1. These (mostly)
/// map directly to sampler methods in most hardware graphics libraries.
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum WrappingMode {
    /// A texture coordinate `(u, v)` is translated to `(u mod 1, v mod 1)`, where `mod` is not
    /// quite the same as `%` in C/Rust, which is the remainder operation. The behavior is
    /// different for negative numbers, where `-0.5 rem 1` (which is `-0.5 % 1` in C/Rust) is
    /// -0.5, but `-0.5 mod 1` would be 0.5.
    Repeat = aiTextureMapMode_aiTextureMapMode_Wrap,
    /// Texture coordinates below 0 are treated as 0, texture coordinates above 1 are treated
    /// as 1.
    Clamp = aiTextureMapMode_aiTextureMapMode_Clamp,
    /// Pixels with U or V coordinates outside 0..1 have no texture applied (this usually means
    /// not rendering these pixels at all, but it may depend on the rendering method).
    Decal = aiTextureMapMode_aiTextureMapMode_Decal,
    /// Like repeat, except that every second repeat on both sides is reversed - so coordinates
    /// in range 0..1 are treated as 0..1, coordinates in 1..2 are treated as 1..0, coordinates
    MirrorRepeat = aiTextureMapMode_aiTextureMapMode_Mirror,
}

impl TryFrom<u32> for WrappingMode {
    type Error = ();

    fn try_from(other: u32) -> Result<Self, Self::Error> {
        match other {
            ffi::aiTextureMapMode_aiTextureMapMode_Wrap => Ok(Self::Repeat),
            ffi::aiTextureMapMode_aiTextureMapMode_Clamp => Ok(Self::Clamp),
            ffi::aiTextureMapMode_aiTextureMapMode_Decal => Ok(Self::Decal),
            ffi::aiTextureMapMode_aiTextureMapMode_Mirror => Ok(Self::MirrorRepeat),
            _ => Err(()),
        }
    }
}

bitflags::bitflags! {
    /// Flags for how this texture's data should be interpreted.
    #[derive(Default)]
    pub struct TextureFlags: u32 {
        /// Invert the texture's color values componentwise
        const INVERT       = aiTextureFlags_aiTextureFlags_Invert;
        /// Override default alpha handling to force usage of this texture's alpha channel
        const FORCE_ALPHA  = aiTextureFlags_aiTextureFlags_UseAlpha;
        /// Override default alpha handling to force treating this texture as if it had no alpha channel
        const IGNORE_ALPHA = aiTextureFlags_aiTextureFlags_IgnoreAlpha;
        /// Override default texture data handling to force 32-bit
        const FORCE_32BIT = aiTextureFlags__aiTextureFlags_Force32Bit;
    }
}

/// A definition of a single texture within a material, this does not fully describe the material
/// as a material may include many individual textures combined with a specified blend mode.
#[derive(Clone, Debug, PartialEq)]
pub struct TextureDefinition {
    /// The path to this texture
    pub path: crate::InlineString,
    /// The blend factor for this texture (i.e. each component should be multiplied by this)
    pub strength: f32,
    /// The blend operation, i.e. how the pixels of this texture should be combined with the pixels
    /// of the existing state of this component. The base is the component's color, if applicable, and
    /// from then on you evaluate each texture definition in sequence, where each operation takes only
    /// the existing state and the current definition.
    pub blend_op: BlendOp,
    /// The mapping method for this texture - according to Assimp docs it's possible for a mesh not to
    /// supply a mapping mode and for that format not to have a default mapping mode. In this case you're
    /// likely best off defaulting to UV, but if this doesn't seem to help then it may be best to just
    /// error/refuse to load the model/replace the material or material component with your renderer's
    /// missing texture.
    pub mapping: Option<Mapping>,
    /// This is used for spherical, cylindrical, box and plane mapping, although the documentation isn't
    /// particularly clear on its precise meaning. If you use `Importer::gen_uv_coords(true)` Assimp will
    /// convert all mappings to `Mapping::UV`, which is the easiest to handle anyway.
    pub axis: Option<Vector3D>,
    /// The UV channel that this texture uses - each mesh can have up to 8 UV channels, so up to 8
    /// sets of UV coordinates per-vertex. Assimp recommends in [their documentation on this property](http://assimp.sourceforge.net/lib_html/materials.html#uvwsrc),
    /// there called `MatKey_UVWSRC`, that if a texture does not define a channel, it should be
    /// either 0 if there is only a single channel or allocated in ascending order otherwise.
    pub channel: u32,
    /// This texture's u-space wrapping mode - i.e. the behavior when u > 1 or u < 0
    pub wrap_u: Option<WrappingMode>,
    /// This texture's v-space wrapping mode - i.e. the behavior when v > 1 or v < 0
    pub wrap_v: Option<WrappingMode>,
    /// Any flags for this texture - this is going to be 0 in most cases and is usually unlikely to badly
    /// affect rendering if ignored.
    pub flags: TextureFlags,
}
