
use egl_sys::{ ffi };
use egl_sys::ffi::types::{ EGLint };


use utils::{ UnsignedInteger, AttributeList, AttributeListBuilder };
use display::{ EGLVersion};



use super::attribute::{
    RenderableType,
    SurfaceType,
    ClientApiConformance,
};

pub struct ConfigSearchOptionsBuilder {
    egl_version: EGLVersion,
    list_builder: AttributeListBuilder,
}

// Note: Attributes EGL_LEVEL and EGL_MATCH_NATIVE_PIXMAP
//       value can't be set to EGL_DONT_CARE

impl ConfigSearchOptionsBuilder {
    pub(crate) fn new(egl_version: EGLVersion) -> ConfigSearchOptionsBuilder {
        ConfigSearchOptionsBuilder {
            egl_version,
            list_builder: AttributeListBuilder::new(),
        }
    }

    /// If value is None, sets attributes value to `EGL_DONT_CARE`.
    pub fn add_unsigned_integer_attribute(&mut self, attribute: UnsignedIntegerSearchAttributes, value: Option<UnsignedInteger>) {
        match value {
            Some(value) => self.list_builder.add(attribute as EGLint, value.value()),
            None => self.list_builder.add(attribute as EGLint, ffi::DONT_CARE),
        }
    }

    pub fn ignore_attribute(&mut self, attribute: IgnoreAttribute) {
        self.list_builder.add(attribute as EGLint, 0)
    }

    /// If surface doesn't have `Window` bit enabled, then attribute
    /// `EGL_NATIVE_VISUAL_TYPE` is ignored.
    pub fn surface_type(&mut self, surface_type: SurfaceType) {
        self.list_builder.add(ffi::SURFACE_TYPE as EGLint, surface_type.bits() as EGLint);
    }

    /// Returns EGLVersionError if RenderableType does not match current EGL version.
    pub fn renderable_type(&mut self, renderable_type: RenderableType) -> Result<(), EGLVersionError> {
        match self.egl_version {
            EGLVersion::EGL_1_4 => {
                match renderable_type {
                    RenderableType::EGL14(value) => {
                        self.list_builder.add(ffi::RENDERABLE_TYPE as EGLint, value.bits() as EGLint);
                        Ok(())
                    }
                    _ => Err(EGLVersionError),
                }
            },
            EGLVersion::EGL_1_5 => {
                match renderable_type {
                    RenderableType::EGL15(value) => {
                        self.list_builder.add(ffi::RENDERABLE_TYPE as EGLint, value.bits() as EGLint);
                        Ok(())
                    }
                    _ => Err(EGLVersionError),
                }
            }
        }
    }

    /// Returns EGLVersionError if ClientApiConformance does not match current EGL version.
    pub fn client_api_conformance(&mut self, client_api_conformance: ClientApiConformance) -> Result<(), EGLVersionError> {
        match self.egl_version {
            EGLVersion::EGL_1_4 => {
                match client_api_conformance {
                    ClientApiConformance::EGL14(value) => {
                        self.list_builder.add(ffi::CONFORMANT as EGLint, value.bits() as EGLint);
                        Ok(())
                    }
                    _ => Err(EGLVersionError),
                }
            },
            EGLVersion::EGL_1_5 => {
                match client_api_conformance {
                    ClientApiConformance::EGL15(value) => {
                        self.list_builder.add(ffi::CONFORMANT as EGLint, value.bits() as EGLint);
                        Ok(())
                    }
                    _ => Err(EGLVersionError),
                }
            }
        }
    }

    // TODO: Implement rest of the EGLConfig searching options

    pub fn build(self) -> ConfigSearchOptions {
        ConfigSearchOptions {
            egl_version: self.egl_version,
            attribute_list: self.list_builder.build(),
        }
    }
}

pub struct ConfigSearchOptions {
    egl_version: EGLVersion,
    attribute_list: AttributeList,
}

impl ConfigSearchOptions {
    pub fn version(&self) -> EGLVersion {
        self.egl_version
    }

    pub(crate) fn attribute_list(&self) -> &AttributeList {
        &self.attribute_list
    }
}


#[repr(u32)]
pub enum UnsignedIntegerSearchAttributes {
    // BufferSize          = ffi::BUFFER_SIZE,
    RedSize             = ffi::RED_SIZE,
    GreenSize           = ffi::GREEN_SIZE,
    BlueSize            = ffi::BLUE_SIZE,
    LuminanceSize       = ffi::LUMINANCE_SIZE,
    AlphaSize           = ffi::ALPHA_SIZE,
    AlphaMaskSize       = ffi::ALPHA_MASK_SIZE,
    DepthSize           = ffi::DEPTH_SIZE,
    Level               = ffi::LEVEL,
    // SampleBuffers       = ffi::SAMPLE_BUFFERS,
    Samples             = ffi::SAMPLES,
    StencilSize         = ffi::STENCIL_SIZE,
}

#[repr(u32)]
pub enum IgnoreAttribute {
    MaxPbufferWidth     = ffi::MAX_PBUFFER_WIDTH,
    MaxPbufferHeight    = ffi::MAX_PBUFFER_HEIGHT,
    MaxPbufferPixels    = ffi::MAX_PBUFFER_PIXELS,
    NativeVisualID      = ffi::NATIVE_VISUAL_ID,
}

#[derive(Debug)]
pub struct EGLVersionError;