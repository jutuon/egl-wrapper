//! OpenGL ES context

use std::marker::PhantomData;

use egl_sys::ffi;
use egl_sys::ffi::types::EGLint;
use egl_sys::extensions;

use context::{Context, RawContextUtils};
use config::client_api::ConfigOpenGLES;
use utils::{AttributeListBuilder, UnsignedInteger};
use error::EGLError;

use super::attribute::{AttributeOpenGLESVersion, CommonAttributes, ContextAttributeUtils};

#[derive(Debug, PartialEq)]
#[repr(u32)]
pub enum EGL14OpenGLESVersion {
    Version1 = 1,
    Version2 = 2,
}

#[derive(Debug, PartialEq)]
#[repr(u32)]
/// EGL_KHR_create_context
pub enum OpenGLESMajorVersionEXT {
    Version1 = 1,
    Version2 = 2,
    Version3 = 3,
}

#[derive(Debug)]
pub struct OpenGLESContext {
    config_opengl: ConfigOpenGLES,
    raw_context: ffi::types::EGLContext,
    _marker: PhantomData<ffi::types::EGLContext>,
}

impl ContextAttributeUtils for OpenGLESContext {}
impl CommonAttributes for OpenGLESContext {}

impl AttributeOpenGLESVersion for OpenGLESContext {}

impl Drop for OpenGLESContext {
    fn drop(&mut self) {
        super::destroy_context(self.raw_display(), self.raw_context);
    }
}

impl RawContextUtils for OpenGLESContext {
    const API_TYPE: ffi::types::EGLenum = ffi::OPENGL_ES_API;
}

impl Context for OpenGLESContext {
    fn raw_display(&self) -> ffi::types::EGLDisplay {
        self.config_opengl.display_config().raw_display()
    }

    fn raw_context(&self) -> ffi::types::EGLContext {
        self.raw_context
    }
}

pub struct OpenGLESContextBuilder {
    config_opengl: ConfigOpenGLES,
    attributes: AttributeListBuilder,
}

impl OpenGLESContextBuilder {
    pub(crate) fn new(config_opengl: ConfigOpenGLES) -> OpenGLESContextBuilder {
        OpenGLESContextBuilder {
            config_opengl,
            attributes: AttributeListBuilder::new(),
        }
    }

    /// Default value: 1
    pub(crate) fn set_context_client_version(&mut self, version: EGL14OpenGLESVersion) {
        self.attributes
            .add(ffi::CONTEXT_CLIENT_VERSION as EGLint, version as EGLint);
    }

    /// This function calls `bind_api` before creating the context.
    pub(crate) fn build(self) -> Result<OpenGLESContext, Option<EGLError>> {
        let attribute_list = self.attributes.build();

        OpenGLESContext::bind_api()?;

        let raw_context = unsafe {
            ffi::CreateContext(
                self.config_opengl.display_config().raw_display(),
                self.config_opengl.display_config().raw_config(),
                ffi::NO_CONTEXT,
                attribute_list.ptr(),
            )
        };

        if raw_context == ffi::NO_CONTEXT {
            Err(EGLError::check_errors())
        } else {
            let context = OpenGLESContext {
                config_opengl: self.config_opengl,
                raw_context,
                _marker: PhantomData,
            };

            Ok(context)
        }
    }
}

// EGL_KHR_create_context extension implementation

pub struct OpenGLESContextBuilderEXT(OpenGLESContextBuilder);

impl OpenGLESContextBuilderEXT {
    pub(crate) fn new(config_opengl: ConfigOpenGLES) -> OpenGLESContextBuilderEXT {
        OpenGLESContextBuilderEXT(OpenGLESContextBuilder::new(config_opengl))
    }

    /// Default value: `OpenGLESMajorVersionEXT::Version1`
    pub(crate) fn set_major_version(&mut self, major: OpenGLESMajorVersionEXT) {
        self.0.attributes.add(
            extensions::CONTEXT_MAJOR_VERSION_KHR as EGLint,
            major as EGLint,
        );
    }

    /// Default value: 0
    pub fn set_minor_version(&mut self, minor: UnsignedInteger) {
        self.0.attributes.add(
            extensions::CONTEXT_MINOR_VERSION_KHR as EGLint,
            minor.value(),
        );
    }

    /// Default value: false
    pub fn enable_debug_context(&mut self, debug: bool) {
        let value = if debug {
            extensions::CONTEXT_OPENGL_DEBUG_BIT_KHR as EGLint
        } else {
            0
        };

        self.0
            .attributes
            .add(extensions::CONTEXT_FLAGS_KHR as EGLint, value);
    }

    /// This function calls `bind_api` before creating the context.
    pub(crate) fn build(self) -> Result<OpenGLESContext, Option<EGLError>> {
        self.0.build()
    }
}
