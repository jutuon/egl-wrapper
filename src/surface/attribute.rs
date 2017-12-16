
use egl_sys::ffi;
use egl_sys::ffi::types::EGLint;

use error::EGLError;
use utils::UnsignedInteger;

use super::Surface;

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

#[derive(Debug)]
#[repr(u32)]
pub enum SetSurfaceAttribute {
    /// OpenGL ES
    MipmapLevel        = ffi::MIPMAP_LEVEL,

    MultisampleResolve = ffi::MULTISAMPLE_RESOLVE,
    SwapBehavior       = ffi::SWAP_BEHAVIOR,
}

#[derive(Debug)]
#[repr(u32)]
pub enum MultisampleResolveValue {
    ResolveDefault = ffi::MULTISAMPLE_RESOLVE_DEFAULT,
    ResolveBox     = ffi::MULTISAMPLE_RESOLVE_BOX,
}

#[derive(Debug)]
#[repr(u32)]
pub enum SwapBehaviorValue {
    BufferPreserved = ffi::BUFFER_PRESERVED,
    BufferDestroyed = ffi::BUFFER_DESTROYED,
}


pub trait SurfaceAttributeUtils: Surface {
    fn set_surface_attribute(&mut self, attribute: SetSurfaceAttribute, value: EGLint) -> Result<(), Option<EGLError>> {
        let result = unsafe {
            ffi::SurfaceAttrib(self.display_config().raw_display(), self.raw_surface(), attribute as EGLint, value)
        };

        if result == ffi::TRUE {
            Ok(())
        } else {
            Err(EGLError::check_errors())
        }
    }
}

pub trait MipmapLevel: SurfaceAttributeUtils {
    // TODO: check that surface config supports OpenGL ES
    //       before setting mipmap attribute

    /// Default value: zero
    ///
    /// This attribute is only supported by OpenGL ES.
    fn set_mipmap_level(&mut self, mipmap_level: UnsignedInteger) -> Result<(), Option<EGLError>> {
        self.set_surface_attribute(SetSurfaceAttribute::MipmapLevel, mipmap_level.value())
    }
}

pub trait MultisampleResolve: SurfaceAttributeUtils {
    // TODO: check that surface config supports `MultisampleResolveValue::ResolveBox`

    /// Default value: `MultisampleResolveValue::ResolveDefault`
    fn set_multisample_resolve(&mut self, multisample_resolve: MultisampleResolveValue) -> Result<(), Option<EGLError>> {
        self.set_surface_attribute(SetSurfaceAttribute::MultisampleResolve, multisample_resolve as EGLint)
    }
}

pub trait SwapBehavior: SurfaceAttributeUtils {
    // TODO: check that surface config supports `SwapBuffersValue::BufferPreserved`

    /// Default value is EGL implementation defined.
    fn set_swap_behavior(&mut self, swap_behavior: SwapBehaviorValue) -> Result<(), Option<EGLError>> {
        self.set_surface_attribute(SetSurfaceAttribute::SwapBehavior, swap_behavior as EGLint)
    }
}