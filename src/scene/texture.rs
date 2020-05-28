use ffi::{aiTexel, aiTexture};
use std::{mem, slice};

define_type_and_iterator_indirect! {
    /// Texture type.
    struct Texture(&aiTexture)
    /// Texture iterator type.
    struct TextureIter
}

/// The actual pixel data for a texture, either as RGBA bytes or as `aiTexel`s
#[repr(transparent)]
pub struct TextureData([aiTexel]);

impl TextureData {
    /// Get the texture data as an array of raw bytes - each 4 bytes is a single texel in RGBA
    /// order
    pub fn bytes(&self) -> &[u8] {
        let texels = self.texels();
        let count = texels.len();

        let ptr = texels as *const [aiTexel] as *const aiTexel as *const u8;

        unsafe { slice::from_raw_parts(ptr, count * mem::size_of::<aiTexel>()) }
    }

    /// Get the texture data as an array of "texels" - red, green, blue, and alpha
    pub fn texels(&self) -> &[aiTexel] {
        unsafe { mem::transmute(self) }
    }
}

impl Texture {
    pub fn filename(&self) -> &str {
        unsafe { crate::aistring_to_cstr(&self.mFilename).to_str().unwrap() }
    }

    pub fn data(&self) -> Option<&TextureData> {
        let data: *mut aiTexel = self.pcData;

        if data.is_null() {
            return None;
        }

        let count = self.mWidth * self.mHeight;

        Some(unsafe { mem::transmute(slice::from_raw_parts(data, count as usize)) })
    }
}
