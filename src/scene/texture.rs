use ffi::{aiTexel, aiTexture};
use std::{ffi::CStr, mem, slice};

define_type_and_iterator_indirect! {
    /// Texture type.
    struct Texture(&aiTexture)
    /// Texture iterator type.
    struct TextureIter
}

/// The actual pixel data for a texture, either as BGRA bytes or as `aiTexel`s
#[repr(transparent)]
pub struct TextureData([aiTexel]);

impl TextureData {
    /// Get the texture data as an array of raw bytes - each 4 bytes is a single texel in ARGB8888
    /// format
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
    /// This texture's width in pixels
    pub fn width(&self) -> u32 {
        self.mWidth
    }

    /// This texture's height in pixels
    pub fn height(&self) -> u32 {
        self.mHeight
    }

    /// The width and height as a tuple
    pub fn size(&self) -> (u32, u32) {
        (self.width(), self.height())
    }

    /// A "format hint" intended to give an idea of how to interpret the texture data,
    /// which will either be `None` for raw image data or otherwise the file extension
    /// of the image.
    pub fn format_hint(&self) -> Option<&str> {
        let out =
            unsafe { CStr::from_bytes_with_nul_unchecked(mem::transmute(&self.achFormatHint[..])) }
                .to_str()
                .unwrap();

        if out.len() == 0 {
            None
        } else {
            Some(out)
        }
    }

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
