
use egl_sys::ffi;

#[derive(Debug)]
#[repr(u32)]
pub enum RenderBuffer {
    BackBuffer   = ffi::BACK_BUFFER,
    SingleBuffer = ffi::SINGLE_BUFFER,
}

#[derive(Debug)]
#[repr(u32)]
/// OpenGL ES
pub enum TextureFormat {
    RGB       = ffi::TEXTURE_RGB,
    RGBA      = ffi::TEXTURE_RGBA,
    NoTexture = ffi::NO_TEXTURE,
}

#[derive(Debug)]
#[repr(u32)]
/// OpenGL ES
pub enum TextureTarget {
    Texture2D = ffi::TEXTURE_2D,
    NoTexture = ffi::NO_TEXTURE,
}


