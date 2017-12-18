
//! OpenGL context

use std::marker::PhantomData;

use egl_sys::ffi::types::EGLint;
use egl_sys::ffi;
use egl_sys::extensions;

use context::{ Context, RawContextUtils };
use config::client_api::ConfigOpenGL;
use utils::{AttributeListBuilder, PositiveInteger, UnsignedInteger};
use error::EGLError;

use super::attribute::{
    ContextAttributeUtils,
    CommonAttributes,
    OpenGLContextFlags,
    OpenGLContextProfile,
    ResetNotificationStrategy,
};

#[derive(Debug)]
pub struct OpenGLContext {
    config_opengl: ConfigOpenGL,
    raw_context: ffi::types::EGLContext,
    _marker: PhantomData<ffi::types::EGLContext>,
}

impl ContextAttributeUtils for OpenGLContext {}
impl CommonAttributes      for OpenGLContext {}

impl Drop for OpenGLContext {
    fn drop(&mut self) {
        super::destroy_context(self.raw_display(), self.raw_context);
    }
}

impl RawContextUtils for OpenGLContext {
    const API_TYPE: ffi::types::EGLenum = ffi::OPENGL_API;
}

impl Context for OpenGLContext {
    fn raw_display(&self) -> ffi::types::EGLDisplay {
        self.config_opengl.display_config().raw_display()
    }

    fn raw_context(&self) -> ffi::types::EGLContext {
        self.raw_context
    }
}

pub struct OpenGLContextBuilder {
    config_opengl: ConfigOpenGL,
    attributes: AttributeListBuilder,
}

impl OpenGLContextBuilder {
    pub(crate) fn new(config_opengl: ConfigOpenGL) -> OpenGLContextBuilder {
        OpenGLContextBuilder {
            config_opengl,
            attributes: AttributeListBuilder::new(),
        }
    }

    /// This function calls `bind_api` before creating the context.
    pub(crate) fn build(self) -> Result<OpenGLContext, Option<EGLError>> {
        let attribute_list = self.attributes.build();

        OpenGLContext::bind_api()?;

        let raw_context = unsafe {
            ffi::CreateContext(
                self.config_opengl.display_config().raw_display(),
                self.config_opengl.display_config().raw_config(),
                ffi::NO_CONTEXT,
                attribute_list.ptr()
            )
        };

        if raw_context == ffi::NO_CONTEXT {
            Err(EGLError::check_errors())
        } else {
            let context = OpenGLContext {
                config_opengl: self.config_opengl,
                raw_context,
                _marker: PhantomData,
            };

            Ok(context)
        }
    }
}


// EGL_KHR_create_context extension implementation

/// OpenGL context with EGL_KHR_create_context extension attributes.
#[derive(Debug)]
pub struct OpenGLContextEXT(OpenGLContext);

impl Context for OpenGLContextEXT {
    fn raw_display(&self) -> ffi::types::EGLDisplay {
        self.0.raw_display()
    }

    fn raw_context(&self) -> ffi::types::EGLContext {
        self.0.raw_context()
    }
}

impl ContextAttributeUtils for OpenGLContextEXT {}
impl CommonAttributes      for OpenGLContextEXT {}

/// OpenGL context builder with EGL_KHR_create_context extension attributes.
pub struct OpenGLContextBuilderEXT {
    builder: OpenGLContextBuilder,
}

impl OpenGLContextBuilderEXT {

    pub(crate) fn new(config_opengl: ConfigOpenGL) -> OpenGLContextBuilderEXT {
        OpenGLContextBuilderEXT {
            builder:OpenGLContextBuilder::new(config_opengl),
        }
    }

    /// Default value: 1
    pub fn set_major_version(&mut self, major: PositiveInteger) {
        self.builder.attributes.add(extensions::CONTEXT_MAJOR_VERSION_KHR as EGLint, major.value());
    }

    /// Default value: 0
    pub fn set_minor_version(&mut self, minor: UnsignedInteger) {
        self.builder.attributes.add(extensions::CONTEXT_MINOR_VERSION_KHR as EGLint, minor.value());
    }

    /// Default value: `OpenGLContextProfile::Core`
    pub fn set_profile(&mut self, value: OpenGLContextProfile) {
        self.builder.attributes.add(extensions::CONTEXT_OPENGL_PROFILE_MASK_KHR as EGLint, value as EGLint);
    }

    /// Default value: 0
    ///
    /// `OpenGLContextFlags::FORWARD_COMPATIBLE` is supported only with
    /// OpenGL 3.0 or later.
    pub fn set_context_flags(&mut self, value: OpenGLContextFlags) {
        self.builder.attributes.add(extensions::CONTEXT_FLAGS_KHR as EGLint, value.bits() as EGLint);
    }

    /// Default value: `ResetNotificationStrategy::NoResetNotification`
    pub fn set_reset_notification_strategy(&mut self, strategy: ResetNotificationStrategy) {
        self.builder.attributes.add(extensions::CONTEXT_OPENGL_RESET_NOTIFICATION_STRATEGY_KHR as EGLint, strategy as EGLint);
    }

    /// This function calls `bind_api` before creating the context.
    pub(crate) fn build(self) -> Result<OpenGLContextEXT, Option<EGLError>> {
        Ok(OpenGLContextEXT(self.builder.build()?))
    }
}