use egl_sys::ffi;
use egl_sys::ffi::types::{EGLenum, EGLint};

use error::EGLError;
use utils::{PositiveInteger, QueryError, QueryResult, UnsignedInteger};

use super::Surface;

#[derive(Debug)]
#[repr(u32)]
pub enum RenderBuffer {
    BackBuffer = ffi::BACK_BUFFER,
    SingleBuffer = ffi::SINGLE_BUFFER,
}

#[derive(Debug)]
#[repr(u32)]
/// OpenGL ES
pub enum TextureFormat {
    RGB = ffi::TEXTURE_RGB,
    RGBA = ffi::TEXTURE_RGBA,
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
    MipmapLevel = ffi::MIPMAP_LEVEL,

    MultisampleResolve = ffi::MULTISAMPLE_RESOLVE,
    SwapBehavior = ffi::SWAP_BEHAVIOR,
}

#[derive(Debug)]
#[repr(u32)]
pub enum MultisampleResolveValue {
    ResolveDefault = ffi::MULTISAMPLE_RESOLVE_DEFAULT,
    ResolveBox = ffi::MULTISAMPLE_RESOLVE_BOX,
}

#[derive(Debug)]
#[repr(u32)]
pub enum SwapBehaviorValue {
    BufferPreserved = ffi::BUFFER_PRESERVED,
    BufferDestroyed = ffi::BUFFER_DESTROYED,
}

#[derive(Debug)]
#[repr(u32)]
pub enum QueryableAttribute {
    VgAlphaFormat = ffi::VG_ALPHA_FORMAT,
    VgColorSpace = ffi::VG_COLORSPACE,
    ConfigID = ffi::CONFIG_ID,
    Height = ffi::HEIGHT,
    HorizontalResolution = ffi::HORIZONTAL_RESOLUTION,
    LargestPbuffer = ffi::LARGEST_PBUFFER,
    MipmapTexture = ffi::MIPMAP_TEXTURE,
    MipmapLevel = ffi::MIPMAP_LEVEL,
    MultisampleResolve = ffi::MULTISAMPLE_RESOLVE,
    PixelAspectRatio = ffi::PIXEL_ASPECT_RATIO,
    RenderBuffer = ffi::RENDER_BUFFER,
    SwapBehavior = ffi::SWAP_BEHAVIOR,
    TextureFormat = ffi::TEXTURE_FORMAT,
    TextureTarget = ffi::TEXTURE_TARGET,
    VerticalResolution = ffi::VERTICAL_RESOLUTION,
    Width = ffi::WIDTH,
}

pub trait SurfaceAttributeUtils: Surface {
    fn set_surface_attribute(
        &mut self,
        attribute: SetSurfaceAttribute,
        value: EGLint,
    ) -> Result<(), Option<EGLError>> {
        let result = unsafe {
            ffi::SurfaceAttrib(
                self.raw_display(),
                self.raw_surface(),
                attribute as EGLint,
                value,
            )
        };

        if result == ffi::EGL_TRUE {
            Ok(())
        } else {
            Err(EGLError::check_errors())
        }
    }

    fn query_attribute(&self, attribute: QueryableAttribute) -> QueryResult<EGLint> {
        let mut value = 0;
        let result = unsafe {
            ffi::QuerySurface(
                self.raw_display(),
                self.raw_surface(),
                attribute as EGLint,
                &mut value,
            )
        };

        if result == ffi::EGL_TRUE {
            Ok(value)
        } else {
            Err(QueryError::QueryError)
        }
    }

    fn query_positive_integer(
        &self,
        attribute: QueryableAttribute,
    ) -> QueryResult<PositiveInteger> {
        Ok(PositiveInteger::try_convert(
            self.query_attribute(attribute)?,
        )?)
    }

    fn query_unsigned_integer(
        &self,
        attribute: QueryableAttribute,
    ) -> QueryResult<UnsignedInteger> {
        Ok(UnsignedInteger::try_convert(
            self.query_attribute(attribute)?,
        )?)
    }

    fn query_boolean(&self, attribute: QueryableAttribute) -> Result<bool, QueryError> {
        let value = self.query_attribute(attribute)?;

        if value == ffi::EGL_TRUE as EGLint {
            Ok(true)
        } else if value == ffi::EGL_FALSE as EGLint {
            Ok(false)
        } else {
            Err(QueryError::BooleanError)
        }
    }
}

pub trait CommonAttributes: SurfaceAttributeUtils {
    fn config_id(&self) -> QueryResult<PositiveInteger> {
        self.query_positive_integer(QueryableAttribute::ConfigID)
    }

    fn width(&self) -> QueryResult<UnsignedInteger> {
        self.query_unsigned_integer(QueryableAttribute::Width)
    }

    fn height(&self) -> QueryResult<UnsignedInteger> {
        self.query_unsigned_integer(QueryableAttribute::Height)
    }
}

/// OpenGL ES texture PbufferSurface attributes.
pub trait Texture: SurfaceAttributeUtils {
    // TODO: check that surface config supports OpenGL ES
    //       before setting mipmap attribute

    /// Default value: zero
    ///
    /// This attribute is only supported by OpenGL ES.
    fn set_mipmap_level(&mut self, mipmap_level: UnsignedInteger) -> Result<(), Option<EGLError>> {
        self.set_surface_attribute(SetSurfaceAttribute::MipmapLevel, mipmap_level.value())
    }

    fn texture_format(&self) -> QueryResult<TextureFormat> {
        let value = self.query_attribute(QueryableAttribute::TextureFormat)?;

        match value as EGLenum {
            ffi::TEXTURE_RGB => Ok(TextureFormat::RGB),
            ffi::TEXTURE_RGBA => Ok(TextureFormat::RGBA),
            ffi::NO_TEXTURE => Ok(TextureFormat::NoTexture),
            _ => Err(QueryError::EnumError),
        }
    }

    fn texture_target(&self) -> QueryResult<TextureTarget> {
        let value = self.query_attribute(QueryableAttribute::TextureTarget)?;

        match value as EGLenum {
            ffi::TEXTURE_2D => Ok(TextureTarget::Texture2D),
            ffi::NO_TEXTURE => Ok(TextureTarget::NoTexture),
            _ => Err(QueryError::EnumError),
        }
    }

    fn mipmap_texture(&self) -> QueryResult<bool> {
        self.query_boolean(QueryableAttribute::MipmapTexture)
    }

    fn mipmap_level(&self) -> QueryResult<UnsignedInteger> {
        self.query_unsigned_integer(QueryableAttribute::MipmapLevel)
    }
}

pub trait MultisampleResolve: SurfaceAttributeUtils {
    // TODO: check that surface config supports `MultisampleResolveValue::ResolveBox`

    /// Default value: `MultisampleResolveValue::ResolveDefault`
    fn set_multisample_resolve(
        &mut self,
        multisample_resolve: MultisampleResolveValue,
    ) -> Result<(), Option<EGLError>> {
        self.set_surface_attribute(
            SetSurfaceAttribute::MultisampleResolve,
            multisample_resolve as EGLint,
        )
    }

    fn multisample_resolve(&self) -> QueryResult<MultisampleResolveValue> {
        let value = self.query_attribute(QueryableAttribute::MultisampleResolve)?;

        match value as EGLenum {
            ffi::MULTISAMPLE_RESOLVE_DEFAULT => Ok(MultisampleResolveValue::ResolveDefault),
            ffi::MULTISAMPLE_RESOLVE_BOX => Ok(MultisampleResolveValue::ResolveBox),
            _ => Err(QueryError::EnumError),
        }
    }
}

pub trait SwapBehavior: SurfaceAttributeUtils {
    // TODO: check that surface config supports `SwapBuffersValue::BufferPreserved`

    /// Default value is EGL implementation defined.
    fn set_swap_behavior(
        &mut self,
        swap_behavior: SwapBehaviorValue,
    ) -> Result<(), Option<EGLError>> {
        self.set_surface_attribute(SetSurfaceAttribute::SwapBehavior, swap_behavior as EGLint)
    }

    fn swap_behavior(&self) -> QueryResult<SwapBehaviorValue> {
        let value = self.query_attribute(QueryableAttribute::SwapBehavior)?;

        match value as EGLenum {
            ffi::BUFFER_PRESERVED => Ok(SwapBehaviorValue::BufferPreserved),
            ffi::BUFFER_DESTROYED => Ok(SwapBehaviorValue::BufferDestroyed),
            _ => Err(QueryError::EnumError),
        }
    }
}

/// Attribute exists only for PbufferSurface.
pub trait LargestPbuffer: SurfaceAttributeUtils {
    fn largest_pbuffer(&self) -> QueryResult<bool> {
        self.query_boolean(QueryableAttribute::LargestPbuffer)
    }
}

/// Attributes exist only for WindowSurface.
pub trait WindowAttributes: SurfaceAttributeUtils {
    /// Display dot pitch in pixels/meter.
    fn horizontal_resolution(&self) -> QueryResult<UnsignedInteger> {
        self.query_unsigned_integer(QueryableAttribute::HorizontalResolution)
    }

    /// Display dot pitch in pixels/meter.
    fn vertical_resolution(&self) -> QueryResult<UnsignedInteger> {
        self.query_unsigned_integer(QueryableAttribute::VerticalResolution)
    }

    /// Pixel aspect ratio multiplied with constant `ffi::DISPLAY_SCALING`.
    fn pixel_aspect_ratio(&self) -> QueryResult<UnsignedInteger> {
        self.query_unsigned_integer(QueryableAttribute::PixelAspectRatio)
    }

    fn render_buffer(&self) -> QueryResult<RenderBuffer> {
        let value = self.query_attribute(QueryableAttribute::RenderBuffer)?;

        match value as EGLenum {
            ffi::BACK_BUFFER => Ok(RenderBuffer::BackBuffer),
            ffi::SINGLE_BUFFER => Ok(RenderBuffer::SingleBuffer),
            _ => Err(QueryError::EnumError),
        }
    }
}

// TODO: OpenVG surface attribute querying
