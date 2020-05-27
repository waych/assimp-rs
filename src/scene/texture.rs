use ffi::{AiTexel, AiTexture};
use std::{mem, slice};

define_type_and_iterator_indirect! {
    /// Texture type.
    struct Texture(&AiTexture)
    /// Texture iterator type.
    struct TextureIter
}

/// The actual pixel data for a texture, either as RGBA bytes or as `AiTexel`s
#[repr(transparent)]
pub struct TextureData([AiTexel]);

impl TextureData {
    /// Get the texture data as an array of raw bytes - each 4 bytes is a single texel in RGBA
    /// order
    pub fn bytes(&self) -> &[u8] {
        let texels = self.texels();
        let count = texels.len();

        let ptr = texels as *const [AiTexel] as *const AiTexel as *const u8;

        unsafe { slice::from_raw_parts(ptr, count * mem::size_of::<AiTexel>()) }
    }

    /// Get the texture data as an array of "texels" - red, green, blue, and alpha
    pub fn texels(&self) -> &[AiTexel] {
        unsafe { mem::transmute(self) }
    }
}

impl Texture {
    pub fn data(&self) -> &TextureData {
        let data: *mut AiTexel = self.data;
        let count = self.width * self.height;

        unsafe { mem::transmute(slice::from_raw_parts(data, count as usize)) }
    }
}
