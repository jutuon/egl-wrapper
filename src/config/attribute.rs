
//! Traits and data types for config attributes.

use egl_sys::{ extensions, ffi };
use egl_sys::ffi::types::{ EGLint, EGLenum, EGLBoolean };


use utils::{ PositiveInteger, IntegerError, UnsignedInteger};
use display::{Display, EGLVersion};

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

bitflags! {
    pub struct SurfaceType: EGLenum {
        const WINDOW                  = ffi::WINDOW_BIT;
        const PIXMAP                  = ffi::PIXMAP_BIT;
        const PBUFFER                 = ffi::PBUFFER_BIT;
        const MULTISAMPLE_RESOLVE_BOX = ffi::MULTISAMPLE_RESOLVE_BOX_BIT;
        const SWAP_BEHAVIOR_PRESERVED = ffi::SWAP_BEHAVIOR_PRESERVED_BIT;
        const VG_COLORSPACE_LINEAR    = ffi::VG_COLORSPACE_LINEAR_BIT;
        const VG_ALPHA_FORMAT_PRE     = ffi::VG_ALPHA_FORMAT_PRE_BIT;
    }
}

bitflags! {
    pub struct ConfigClientAPI: EGLenum {
        const OPENGL     = ffi::OPENGL_BIT;
        const OPENGL_ES  = ffi::OPENGL_ES_BIT;
        const OPENGL_ES2 = ffi::OPENGL_ES2_BIT;
        /// Defined only for EGL 1.5
        const OPENGL_ES3 = extensions::OPENGL_ES3_BIT;
        const OPENVG     = ffi::OPENVG_BIT;
    }
}

#[derive(Debug)]
pub struct ConfigInfo {
    pub color_buffer: ColorBuffer,
    pub alpha_mask_buffer: Option<PositiveInteger>,
    pub depth_buffer: Option<PositiveInteger>,
    pub stencil_buffer: Option<PositiveInteger>,
    pub multisample_buffer_samples: Option<PositiveInteger>,
    pub surface_type: SurfaceType,
    pub client_api: ConfigClientAPI,
    pub native_renderable: bool,
    pub native_visual_id: Option<EGLint>,
    pub slow_config: bool,
    pub client_api_conformance: ConfigClientAPI,
    pub level: EGLint,
    pub transparent_rgb: Option<(UnsignedInteger, UnsignedInteger, UnsignedInteger)>,
    pub max_pbuffer_width_height: (UnsignedInteger, UnsignedInteger),
    pub max_pbuffer_pixels: UnsignedInteger,
    pub max_swap_interval: UnsignedInteger,
    pub min_swap_interval: UnsignedInteger,
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
            ffi::GetConfigAttrib(self.display().raw_display(), self.raw_config(), attribute, &mut value)
        };

        if result == ffi::FALSE {
            return Err(ConfigError::QueryError)
        }

        Ok(value)
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

pub trait DepthBuffer: ConfigUtils {
    fn depth_buffer(&self) -> ConfigResult<Option<PositiveInteger>> {
        self.query_positive_integer_or_zero(ConfigAttribute::DepthSize)
    }
}

pub trait StencilBuffer: ConfigUtils {
    fn stencil_buffer(&self) -> ConfigResult<Option<PositiveInteger>> {
        self.query_positive_integer_or_zero(ConfigAttribute::StencilSize)
    }
}

pub trait MultisampleBuffer: ConfigUtils {
    /// Returns Ok(Some(sample_count)) if multisample buffer exists.
    fn multisample_buffer_samples(&self) -> ConfigResult<Option<PositiveInteger>> {
        self.query_positive_integer_or_zero(ConfigAttribute::Samples)
    }
}

pub trait SwapInterval: ConfigUtils {
    fn max_swap_interval(&self) -> ConfigResult<UnsignedInteger> {
        self.query_unsigned_integer(ConfigAttribute::MaxSwapInterval)
    }

    fn min_swap_interval(&self) -> ConfigResult<UnsignedInteger> {
        self.query_unsigned_integer(ConfigAttribute::MinSwapInterval)
    }
}

pub trait Surface: ConfigUtils {
    fn surface_type(&self) -> ConfigResult<SurfaceType> {
        let value = self.query_attrib(ConfigAttribute::SurfaceType)?;

        Ok(SurfaceType::from_bits_truncate(value as EGLenum))
    }
}

pub trait SlowConfig: ConfigUtils {
    fn slow_config(&self) -> ConfigResult<bool> {
        let caveat = self.query_attrib(ConfigAttribute::ConfigCaveat)?;

        match caveat as EGLenum {
            ffi::SLOW_CONFIG => Ok(true),
            ffi::NONE | ffi::NON_CONFORMANT_CONFIG => Ok(false),
            _ => Err(ConfigError::EnumError),
        }
    }
}

pub trait NativeRenderable: ConfigUtils {
    fn native_renderable(&self) -> ConfigResult<bool> {
        let value = self.query_attrib(ConfigAttribute::NativeRenderable)?;

        match value as EGLBoolean {
            ffi::TRUE => Ok(true),
            ffi::FALSE => Ok(false),
            _ => Err(ConfigError::BooleanError),
        }
    }

    fn native_visual_id(&self) -> ConfigResult<Option<EGLint>> {
        let id = self.query_attrib(ConfigAttribute::NativeVisualID)?;

        match id {
            0 => Ok(None),
            id => Ok(Some(id)),
        }
    }

    // TODO: EGL_NATIVE_VISUAL_TYPE
}

pub trait ClientAPI: ConfigUtils {
    fn client_api(&self) -> ConfigResult<ConfigClientAPI> {
        let value = self.query_attrib(ConfigAttribute::RenderableType)?;

        let mut client_api = ConfigClientAPI::from_bits_truncate(value as EGLenum);

        if let EGLVersion::EGL_1_4 = self.display().version() {
            client_api -= ConfigClientAPI::OPENGL_ES3;
        }

        Ok(client_api)
    }

    fn client_api_conformance(&self) -> ConfigResult<ConfigClientAPI> {
        let value = self.query_attrib(ConfigAttribute::Conformant)?;

        let mut client_api = ConfigClientAPI::from_bits_truncate(value as EGLenum);

        if let EGLVersion::EGL_1_4 = self.display().version() {
            client_api -= ConfigClientAPI::OPENGL_ES3;
        }

        Ok(client_api)
    }
}

pub trait FramebufferLevel: ConfigUtils {
    /// Framebuffer overlay or underlay level. Zero is default level.
    fn level(&self) -> ConfigResult<EGLint> {
        Ok(self.query_attrib(ConfigAttribute::Level)?)
    }
}

pub trait Pbuffer: ConfigUtils {
    fn max_pbuffer_width_height(&self) -> ConfigResult<(UnsignedInteger, UnsignedInteger)> {
        let width = self.query_unsigned_integer(ConfigAttribute::MaxPbufferWidth)?;
        let height = self.query_unsigned_integer(ConfigAttribute::MaxPbufferHeight)?;

        Ok((width, height))
    }

    fn max_pbuffer_pixels(&self) -> ConfigResult<UnsignedInteger> {
        self.query_unsigned_integer(ConfigAttribute::MaxPbufferPixels)
    }
}

pub trait TransparentColor: ConfigUtils {
    fn transparent_rgb(&self) -> Result<Option<(UnsignedInteger, UnsignedInteger, UnsignedInteger)>, ConfigError> {
        let transparent_type = self.query_attrib(ConfigAttribute::TransparentType)?;

        match transparent_type as EGLenum {
            ffi::TRANSPARENT_RGB => {
                let r = self.query_unsigned_integer(ConfigAttribute::TransparenRedValue)?;
                let g = self.query_unsigned_integer(ConfigAttribute::TransparentGreenValue)?;
                let b = self.query_unsigned_integer(ConfigAttribute::TransparentBlueValue)?;

                // TODO: check other end of the value range

                Ok(Some((r, g, b)))
            },
            ffi::NONE => Ok(None),
            _ => Err(ConfigError::EnumError),
        }
    }
}

pub trait AllAttributes where
    Self:   ConfigUtils +
            Color +
            Pbuffer +
            FramebufferLevel +
            ClientAPI +
            NativeRenderable +
            SlowConfig +
            Surface +
            SwapInterval +
            MultisampleBuffer +
            DepthBuffer +
            AlphaMaskBuffer +
            StencilBuffer +
            TransparentColor {

    fn all(&self) -> Result<ConfigInfo, ConfigError> {
        Ok(ConfigInfo {
            color_buffer: self.color_buffer()?,
            alpha_mask_buffer: self.alpha_mask_buffer()?,
            depth_buffer: self.depth_buffer()?,
            stencil_buffer: self.stencil_buffer()?,
            multisample_buffer_samples: self.multisample_buffer_samples()?,
            surface_type: self.surface_type()?,
            client_api: self.client_api()?,
            native_renderable: self.native_renderable()?,
            native_visual_id: self.native_visual_id()?,
            slow_config: self.slow_config()?,
            client_api_conformance: self.client_api_conformance()?,
            level: self.level()?,
            transparent_rgb: self.transparent_rgb()?,
            max_pbuffer_width_height: self.max_pbuffer_width_height()?,
            max_pbuffer_pixels: self.max_pbuffer_pixels()?,
            max_swap_interval: self.max_swap_interval()?,
            min_swap_interval: self.min_swap_interval()?,
        })
    }
}