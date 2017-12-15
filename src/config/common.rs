
//! Traits and data types for Config information.

use egl_sys::{ extensions, ffi };
use egl_sys::ffi::types::{ EGLint, EGLenum, EGLBoolean };


use utils::{ PositiveInteger, IntegerError, UnsignedInteger, AttributeList, AttributeListBuilder };
use display::{Display, DisplayHandle};

use super::DisplayConfig;

#[derive(Debug)]
pub enum ColorBuffer {
    RGB(PositiveInteger, PositiveInteger, PositiveInteger),
    RGBA(PositiveInteger, PositiveInteger, PositiveInteger, PositiveInteger),
    Luminance(PositiveInteger),
    LuminanceAndAlpha(PositiveInteger, PositiveInteger),
}

#[repr(u32)]
pub enum ConfigAttribute {
    // BufferSize          = ffi::BUFFER_SIZE,
    RedSize             = ffi::RED_SIZE,
    GreenSize           = ffi::GREEN_SIZE,
    BlueSize            = ffi::BLUE_SIZE,
    LuminanceSize       = ffi::LUMINANCE_SIZE,
    AlphaSize           = ffi::ALPHA_SIZE,
    AlphaMaskSize       = ffi::ALPHA_MASK_SIZE,
    // BindToTextureRGB    = ffi::BIND_TO_TEXTURE_RGB,
    // BindToTextureRGBA   = ffi::BIND_TO_TEXTURE_RGBA,
    ColorBufferType     = ffi::COLOR_BUFFER_TYPE,
    ConfigCaveat        = ffi::CONFIG_CAVEAT,
    // ConfigID            = ffi::CONFIG_ID,
    Conformant          = ffi::CONFORMANT,
    DepthSize           = ffi::DEPTH_SIZE,
    Level               = ffi::LEVEL,
    MaxPbufferWidth     = ffi::MAX_PBUFFER_WIDTH,
    MaxPbufferHeight    = ffi::MAX_PBUFFER_HEIGHT,
    MaxPbufferPixels    = ffi::MAX_PBUFFER_PIXELS,
    MaxSwapInterval     = ffi::MAX_SWAP_INTERVAL,
    MinSwapInterval     = ffi::MIN_SWAP_INTERVAL,
    NativeRenderable    = ffi::NATIVE_RENDERABLE,
    NativeVisualID      = ffi::NATIVE_VISUAL_ID,
    // NativeVisualType    = ffi::NATIVE_VISUAL_TYPE,
    RenderableType      = ffi::RENDERABLE_TYPE,
    // SampleBuffers       = ffi::SAMPLE_BUFFERS,
    Samples             = ffi::SAMPLES,
    StencilSize         = ffi::STENCIL_SIZE,
    SurfaceType         = ffi::SURFACE_TYPE,
    TransparentType     = ffi::TRANSPARENT_TYPE,
    TransparenRedValue  = ffi::TRANSPARENT_RED_VALUE,
    TransparentGreenValue = ffi::TRANSPARENT_GREEN_VALUE,
    TransparentBlueValue  = ffi::TRANSPARENT_BLUE_VALUE,
}

#[derive(Debug)]
pub enum ConfigError {
    QueryError,
    BooleanError,
    EnumError,
    IntegerError(IntegerError),
}

impl From<IntegerError> for ConfigError {
    fn from(error: IntegerError) -> Self {
        ConfigError::IntegerError(error)
    }
}

type ConfigResult<T> = Result<T, ConfigError>;


pub trait ConfigUtils: Sized {
    fn raw_config(&self) -> ffi::types::EGLConfig;

    fn display(&self) -> &Display;

    fn query_attrib(&self, attribute: ConfigAttribute) -> ConfigResult<EGLint> {
        let attribute = attribute as EGLint;

        let mut value = 0;

        let result = unsafe {
            ffi::GetConfigAttrib(self.display().raw(), self.raw_config(), attribute, &mut value)
        };

        if result == ffi::FALSE {
            return Err(ConfigError::QueryError)
        }

        Ok(value)
    }

    fn into_display_config(self) -> DisplayConfig {
        DisplayConfig {
            display_handle: self.display().display_handle().clone(),
            raw_config: self.raw_config(),
        }
    }

    fn query_positive_integer_or_zero(&self, attribute: ConfigAttribute) -> ConfigResult<Option<PositiveInteger>> {
        let value = self.query_attrib(attribute)?;

        match PositiveInteger::try_convert(value) {
            Ok(value) => Ok(Some(value)),
            Err(IntegerError::Zero) => Ok(None),
            Err(error) => Err(ConfigError::IntegerError(error)),
        }
    }

    fn query_unsigned_integer(&self, attribute: ConfigAttribute) -> ConfigResult<UnsignedInteger> {
        let value = self.query_attrib(attribute)?;
        Ok(UnsignedInteger::try_convert(value)?)
    }

    fn query_positive_integer(&self, attribute: ConfigAttribute) -> ConfigResult<PositiveInteger> {
        let value = self.query_attrib(attribute)?;
        Ok(PositiveInteger::try_convert(value)?)
    }
}

pub trait Color: ConfigUtils {
    fn color_buffer(&self) -> ConfigResult<ColorBuffer> {
        let color_buffer_type = self.query_attrib(ConfigAttribute::ColorBufferType)?;

        match color_buffer_type as EGLenum {
            ffi::RGB_BUFFER => {
                let r = self.query_positive_integer(ConfigAttribute::RedSize)?;
                let g = self.query_positive_integer(ConfigAttribute::GreenSize)?;
                let b = self.query_positive_integer(ConfigAttribute::BlueSize)?;

                let a = self.query_attrib(ConfigAttribute::AlphaSize)?;

                match PositiveInteger::try_convert(a) {
                    Ok(alpha) => Ok(ColorBuffer::RGBA(r, g, b, alpha)),
                    Err(IntegerError::Zero) => Ok(ColorBuffer::RGB(r, g, b)),
                    Err(error) => Err(
                        ConfigError::IntegerError(error)
                    ),
                }
            },
            ffi::LUMINANCE_BUFFER => {
                let l = self.query_positive_integer(ConfigAttribute::LuminanceSize)?;

                let a = self.query_attrib(ConfigAttribute::AlphaSize)?;

                match PositiveInteger::try_convert(a) {
                    Ok(alpha) => Ok(ColorBuffer::LuminanceAndAlpha(l, alpha)),
                    Err(IntegerError::Zero) => Ok(ColorBuffer::Luminance(l)),
                    Err(error) => Err(
                        ConfigError::IntegerError(error)
                    ),
                }
            }
            _ => Err(ConfigError::EnumError)
        }
    }
}

pub trait AlphaMaskBuffer: ConfigUtils {
    fn alpha_mask_buffer(&self) -> ConfigResult<Option<PositiveInteger>> {
        self.query_positive_integer_or_zero(ConfigAttribute::AlphaMaskSize)
    }
}