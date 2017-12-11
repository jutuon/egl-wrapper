
use std::slice;


use egl_sys::{ extensions, ffi };
use egl_sys::ffi::types::{ EGLint, EGLenum, EGLBoolean };

use utils::{ PositiveInteger, IntegerError, UnsignedInteger };
use display::EGLDisplay;
use display::EGLVersion;

#[derive(Debug)]
pub enum ColorBuffer {
    RGB(PositiveInteger, PositiveInteger, PositiveInteger),
    RGBA(PositiveInteger, PositiveInteger, PositiveInteger, PositiveInteger),
    Luminance(PositiveInteger),
    LuminanceAndAlpha(PositiveInteger, PositiveInteger),
}




pub struct Configs<'a> {
    display: &'a EGLDisplay,
    raw_configs: Vec<ffi::types::EGLConfig>,
}

impl <'a> Configs<'a> {
    pub(crate) fn new(display: &EGLDisplay, raw_configs: Vec<ffi::types::EGLConfig>) -> Configs {
        Configs {
            display,
            raw_configs,
        }
    }

    pub fn count(&self) -> usize {
        self.raw_configs.len()
    }

    pub fn iter(&'a self) -> Iter<'a> {
        Iter::new(self.display, self.raw_configs.iter())
    }
}

pub struct Iter<'a> {
    display: &'a EGLDisplay,
    raw_configs_iter: slice::Iter<'a, ffi::types::EGLConfig>,
}

impl <'a> Iter<'a> {
    fn new(display: &'a EGLDisplay, raw_configs_iter: slice::Iter<'a, ffi::types::EGLConfig>) -> Iter<'a> {
        Iter {
            display,
            raw_configs_iter,
        }
    }
}

impl <'a> Iterator for Iter<'a> {
    type Item = Config<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.raw_configs_iter.next().map(|raw_config| {
            Config {
                display: self.display,
                raw_config: *raw_config,
            }
        })
    }
}


pub struct Config<'a> {
    display: &'a EGLDisplay,
    raw_config: ffi::types::EGLConfig,
}

impl <'a> Config<'a> {
    fn query_attrib(&self, attribute: ConfigAttribute) -> Result<EGLint, ConfigError> {
        let attribute = attribute as EGLint;

        let mut value = 0;

        let result = unsafe {
            ffi::GetConfigAttrib(self.display.raw(), self.raw_config, attribute, &mut value)
        };

        if result == ffi::FALSE {
            return Err(ConfigError::QueryError)
        }

        Ok(value)
    }


    fn query_positive_integer_or_zero(&self, attribute: ConfigAttribute) -> Result<Option<PositiveInteger>, ConfigError> {
        let value = self.query_attrib(attribute)?;

        match PositiveInteger::try_convert(value) {
            Ok(value) => Ok(Some(value)),
            Err(IntegerError::Zero) => Ok(None),
            Err(error) => Err(ConfigError::IntegerError(error)),
        }
    }

    pub fn color_buffer(&self) -> Result<ColorBuffer, ConfigError> {
        let color_buffer_type = self.query_attrib(ConfigAttribute::ColorBufferType)?;

        match color_buffer_type as EGLenum {
            ffi::RGB_BUFFER => {
                let r = self.query_attrib(ConfigAttribute::RedSize)?;
                let g = self.query_attrib(ConfigAttribute::GreenSize)?;
                let b = self.query_attrib(ConfigAttribute::BlueSize)?;

                let r = PositiveInteger::try_convert(r)?;
                let g = PositiveInteger::try_convert(g)?;
                let b = PositiveInteger::try_convert(b)?;

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
                let l = self.query_attrib(ConfigAttribute::LuminanceSize)?;
                let l = PositiveInteger::try_convert(l)?;

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

    pub fn alpha_mask_buffer(&self) -> Result<Option<PositiveInteger>, ConfigError> {
        self.query_positive_integer_or_zero(ConfigAttribute::AlphaMaskSize)
    }

    pub fn depth_buffer(&self) -> Result<Option<PositiveInteger>, ConfigError> {
        self.query_positive_integer_or_zero(ConfigAttribute::DepthSize)
    }

    pub fn stencil_buffer(&self) -> Result<Option<PositiveInteger>, ConfigError> {
        self.query_positive_integer_or_zero(ConfigAttribute::StencilSize)
    }

    /// Returns Ok(Some(sample_count)) if multisample buffer exists.
    pub fn multisample_buffer_samples(&self) -> Result<Option<PositiveInteger>, ConfigError> {
        self.query_positive_integer_or_zero(ConfigAttribute::Samples)
    }

    pub fn surface_type(&self) -> Result<SurfaceType, ConfigError> {
        let value = self.query_attrib(ConfigAttribute::SurfaceType)?;

        Ok(SurfaceType::from_bits_truncate(value as EGLenum))
    }

    pub fn renderable_type(&self) -> Result<RenderableType, ConfigError> {
        let value = self.query_attrib(ConfigAttribute::RenderableType)?;

        match self.display.version() {
            EGLVersion::EGL_1_4 => {
                Ok(RenderableType::EGL14(EGL14ConfigClientAPI::from_bits_truncate(value as EGLenum)))
            },
            EGLVersion::EGL_1_5 => {
                Ok(RenderableType::EGL15(EGL15ConfigClientAPI::from_bits_truncate(value as EGLenum)))
            },
        }
    }

    pub fn native_renderable(&self) -> Result<bool, ConfigError> {
        let value = self.query_attrib(ConfigAttribute::NativeRenderable)?;

        match value as EGLBoolean {
            ffi::TRUE => Ok(true),
            ffi::FALSE => Ok(false),
            _ => Err(ConfigError::BooleanError),
        }
    }

    pub fn native_visual_id(&self) -> Result<Option<EGLint>, ConfigError> {
        let id = self.query_attrib(ConfigAttribute::NativeVisualID)?;

        match id {
            0 => Ok(None),
            id => Ok(Some(id)),
        }
    }

    // TODO: EGL_NATIVE_VISUAL_TYPE

    pub fn slow_config(&self) -> Result<bool, ConfigError> {
        let caveat = self.query_attrib(ConfigAttribute::ConfigCaveat)?;

        match caveat as EGLenum {
            ffi::SLOW_CONFIG => Ok(true),
            ffi::NONE | ffi::NON_CONFORMANT_CONFIG => Ok(false),
            _ => Err(ConfigError::EnumError),
        }
    }

    pub fn client_api_conformance(&self) -> Result<ClientApiConformance, ConfigError> {
        let value = self.query_attrib(ConfigAttribute::Conformant)?;

        match self.display.version() {
            EGLVersion::EGL_1_4 => {
                Ok(ClientApiConformance::EGL14(EGL14ConfigClientAPI::from_bits_truncate(value as EGLenum)))
            },
            EGLVersion::EGL_1_5 => {
                Ok(ClientApiConformance::EGL15(EGL15ConfigClientAPI::from_bits_truncate(value as EGLenum)))
            },
        }
    }

    /// Framebuffer overlay or underlay level. Zero is default level.
    pub fn level(&self) -> Result<EGLint, ConfigError> {
        Ok(self.query_attrib(ConfigAttribute::Level)?)
    }

    pub fn transparent_rgb(&self) -> Result<Option<(UnsignedInteger, UnsignedInteger, UnsignedInteger)>, ConfigError> {
        let transparent_type = self.query_attrib(ConfigAttribute::TransparentType)?;

        match transparent_type as EGLenum {
            ffi::TRANSPARENT_RGB => {
                let r = self.query_attrib(ConfigAttribute::TransparenRedValue)?;
                let g = self.query_attrib(ConfigAttribute::TransparentGreenValue)?;
                let b = self.query_attrib(ConfigAttribute::TransparentBlueValue)?;

                let r = UnsignedInteger::try_convert(r)?;
                let g = UnsignedInteger::try_convert(g)?;
                let b = UnsignedInteger::try_convert(b)?;

                // TODO: check other end of the value range

                Ok(Some((r, g, b)))
            },
            ffi::NONE => Ok(None),
            _ => Err(ConfigError::EnumError),
        }
    }

    pub fn max_pbuffer_width_height(&self) -> Result<(UnsignedInteger, UnsignedInteger), ConfigError> {
        let width = self.query_attrib(ConfigAttribute::MaxPbufferWidth)?;
        let height = self.query_attrib(ConfigAttribute::MaxPbufferHeight)?;

        let width = UnsignedInteger::try_convert(width)?;
        let height = UnsignedInteger::try_convert(height)?;

        Ok((width, height))
    }

    pub fn max_pbuffer_pixels(&self) -> Result<UnsignedInteger, ConfigError> {
        let pixels = self.query_attrib(ConfigAttribute::MaxPbufferPixels)?;
        let pixels = UnsignedInteger::try_convert(pixels)?;

        Ok(pixels)
    }

    pub fn max_swap_interval(&self) -> Result<UnsignedInteger, ConfigError> {
        let value = self.query_attrib(ConfigAttribute::MaxSwapInterval)?;
        let value = UnsignedInteger::try_convert(value)?;

        Ok(value)
    }

    pub fn min_swap_interval(&self) -> Result<UnsignedInteger, ConfigError> {
        let value = self.query_attrib(ConfigAttribute::MinSwapInterval)?;
        let value = UnsignedInteger::try_convert(value)?;

        Ok(value)
    }

    pub fn all(&self) -> Result<ConfigInfo, ConfigError> {
        Ok(ConfigInfo {
            color_buffer: self.color_buffer()?,
            alpha_mask_buffer: self.alpha_mask_buffer()?,
            depth_buffer: self.depth_buffer()?,
            stencil_buffer: self.stencil_buffer()?,
            multisample_buffer_samples: self.multisample_buffer_samples()?,
            surface_type: self.surface_type()?,
            renderable_type: self.renderable_type()?,
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

#[derive(Debug)]
pub struct ConfigInfo {
    pub color_buffer: ColorBuffer,
    pub alpha_mask_buffer: Option<PositiveInteger>,
    pub depth_buffer: Option<PositiveInteger>,
    pub stencil_buffer: Option<PositiveInteger>,
    pub multisample_buffer_samples: Option<PositiveInteger>,
    pub surface_type: SurfaceType,
    pub renderable_type: RenderableType,
    pub native_renderable: bool,
    pub native_visual_id: Option<EGLint>,
    pub slow_config: bool,
    pub client_api_conformance: ClientApiConformance,
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


#[repr(u32)]
enum ConfigAttribute {
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


#[derive(Debug)]
pub enum RenderableType {
    EGL14(EGL14ConfigClientAPI),
    EGL15(EGL15ConfigClientAPI),
}

bitflags! {
    pub struct EGL14ConfigClientAPI: EGLenum {
        const OPENGL     = ffi::OPENGL_BIT;
        const OPENGL_ES  = ffi::OPENGL_ES_BIT;
        const OPENGL_ES2 = ffi::OPENGL_ES2_BIT;
        const OPENVG_BIT = ffi::OPENVG_BIT;
    }
}

bitflags! {
    pub struct EGL15ConfigClientAPI: EGLenum {
        const OPENGL     = ffi::OPENGL_BIT;
        const OPENGL_ES  = ffi::OPENGL_ES_BIT;
        const OPENGL_ES2 = ffi::OPENGL_ES2_BIT;
        const OPENGL_ES3 = extensions::OPENGL_ES3_BIT;
        const OPENVG     = ffi::OPENVG_BIT;
    }
}

#[derive(Debug)]
pub enum ClientApiConformance {
    EGL14(EGL14ConfigClientAPI),
    EGL15(EGL15ConfigClientAPI),
}