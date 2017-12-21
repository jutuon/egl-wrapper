use std::marker::PhantomData;

use egl_sys::ffi;
use egl_sys::ffi::types::EGLint;

use config::DisplayConfig;
use utils::{AttributeListBuilder, UnsignedInteger};

use error::EGLError;

use super::{destroy_surface, Surface};

use super::attribute::{CommonAttributes, LargestPbuffer, MultisampleResolve,
                       SurfaceAttributeUtils, SwapBehavior, Texture, TextureFormat, TextureTarget};

#[derive(Debug)]
pub struct PbufferSurface {
    display_config: DisplayConfig,
    raw_surface: ffi::types::EGLSurface,
    _marker: PhantomData<ffi::types::EGLSurface>,
}

impl Surface for PbufferSurface {
    fn raw_surface(&self) -> ffi::types::EGLSurface {
        self.raw_surface
    }

    fn display_config(&self) -> &DisplayConfig {
        &self.display_config
    }
}

impl Drop for PbufferSurface {
    fn drop(&mut self) {
        destroy_surface(self)
    }
}

impl SurfaceAttributeUtils for PbufferSurface {}
impl CommonAttributes for PbufferSurface {}
impl MultisampleResolve for PbufferSurface {}
impl SwapBehavior for PbufferSurface {}

impl Texture for PbufferSurface {}
impl LargestPbuffer for PbufferSurface {}

pub struct PbufferSurfaceBuilder {
    display_config: DisplayConfig,
    attributes: AttributeListBuilder,
}

impl PbufferSurfaceBuilder {
    pub(crate) fn new(display_config: DisplayConfig) -> PbufferSurfaceBuilder {
        PbufferSurfaceBuilder {
            display_config,
            attributes: AttributeListBuilder::new(),
        }
    }

    // TODO: PbufferSurface OpenVG attributes

    /// Default value: zero
    pub fn width(&mut self, width: UnsignedInteger) -> &mut Self {
        self.attributes.add(ffi::WIDTH as EGLint, width.value());
        self
    }

    /// Default value: zero
    pub fn height(&mut self, height: UnsignedInteger) -> &mut Self {
        self.attributes.add(ffi::HEIGHT as EGLint, height.value());
        self
    }

    // TODO: Before setting pbuffer texture attributes check
    //       that config supports OpenGL ES

    /// Default value: `TextureFormat::NoTexture`
    pub fn texture_format(&mut self, format: TextureFormat) -> &mut Self {
        self.attributes
            .add(ffi::TEXTURE_FORMAT as EGLint, format as EGLint);
        self
    }

    /// Default value: `TextureTarget::NoTexture`
    pub fn texture_target(&mut self, target: TextureTarget) -> &mut Self {
        self.attributes
            .add(ffi::TEXTURE_TARGET as EGLint, target as EGLint);
        self
    }

    /// Default value: false
    pub fn mipmap_texture(&mut self, value: bool) -> &mut Self {
        let value = if value { ffi::TRUE } else { ffi::FALSE };

        self.attributes
            .add(ffi::MIPMAP_TEXTURE as EGLint, value as EGLint);
        self
    }

    /// Default value: false
    pub fn largest_pbuffer(&mut self, value: bool) -> &mut Self {
        let value = if value { ffi::TRUE } else { ffi::FALSE };

        self.attributes
            .add(ffi::LARGEST_PBUFFER as EGLint, value as EGLint);
        self
    }

    pub fn build(self) -> Result<PbufferSurface, Option<EGLError>> {
        let attributes = self.attributes.build();

        let result = unsafe {
            ffi::CreatePbufferSurface(
                self.display_config.raw_display(),
                self.display_config.raw_config(),
                attributes.ptr(),
            )
        };

        if result == ffi::NO_SURFACE {
            return Err(EGLError::check_errors());
        }

        Ok(PbufferSurface {
            display_config: self.display_config,
            raw_surface: result,
            _marker: PhantomData,
        })
    }
}
