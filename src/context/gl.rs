//! OpenGL context

use std::marker::PhantomData;

use egl_sys::ffi::types::EGLint;
use egl_sys::ffi;
use egl_sys::extensions;

use platform::Platform;
use context::{Context, RawContextUtils};
use config::client_api::ConfigOpenGL;
use utils::{AttributeListBuilder, PositiveInteger, UnsignedInteger};
use error::EGLError;

use super::attribute::{CommonAttributes, ContextAttributeUtils, OpenGLContextFlags,
                       OpenGLContextProfile, ResetNotificationStrategy};

#[derive(Debug)]
pub struct OpenGLContext<P: Platform> {
    config_opengl: ConfigOpenGL<P>,
    raw_context: ffi::types::EGLContext,
    _marker: PhantomData<ffi::types::EGLContext>,
}

impl<P: Platform> ContextAttributeUtils for OpenGLContext<P> {}
impl<P: Platform> CommonAttributes for OpenGLContext<P> {}

impl<P: Platform> Drop for OpenGLContext<P> {
    fn drop(&mut self) {
        super::destroy_context(self.raw_display(), self.raw_context);
    }
}

impl<P: Platform> RawContextUtils for OpenGLContext<P> {
    const API_TYPE: ffi::types::EGLenum = ffi::OPENGL_API;
}

impl<P: Platform> Context for OpenGLContext<P> {
    fn raw_display(&self) -> ffi::types::EGLDisplay {
        self.config_opengl.display_config().raw_display()
    }

    fn raw_context(&self) -> ffi::types::EGLContext {
        self.raw_context
    }
}

pub struct OpenGLContextBuilder<P: Platform> {
    config_opengl: ConfigOpenGL<P>,
    attributes: AttributeListBuilder,
}

impl<P: Platform> OpenGLContextBuilder<P> {
    pub(crate) fn new(config_opengl: ConfigOpenGL<P>) -> Self {
        OpenGLContextBuilder {
            config_opengl,
            attributes: AttributeListBuilder::new(),
        }
    }

    /// This function calls `bind_api` before creating the context.
    pub(crate) fn build(self) -> Result<OpenGLContext<P>, Option<EGLError>> {
        let attribute_list = self.attributes.build();

        OpenGLContext::<P>::bind_api()?;

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

/// OpenGL context builder with EGL_KHR_create_context extension attributes.
pub struct OpenGLContextBuilderEXT<P: Platform> {
    builder: OpenGLContextBuilder<P>,
}

impl<P: Platform> OpenGLContextBuilderEXT<P> {
    pub(crate) fn new(config_opengl: ConfigOpenGL<P>) -> Self {
        OpenGLContextBuilderEXT {
            builder: OpenGLContextBuilder::new(config_opengl),
        }
    }

    /// Default value: 1
    pub fn set_major_version(&mut self, major: PositiveInteger) {
        self.builder.attributes.add(
            extensions::CONTEXT_MAJOR_VERSION_KHR as EGLint,
            major.value(),
        );
    }

    /// Default value: 0
    pub fn set_minor_version(&mut self, minor: UnsignedInteger) {
        self.builder.attributes.add(
            extensions::CONTEXT_MINOR_VERSION_KHR as EGLint,
            minor.value(),
        );
    }

    /// Default value: `OpenGLContextProfile::Core`
    pub fn set_profile(&mut self, value: OpenGLContextProfile) {
        self.builder.attributes.add(
            extensions::CONTEXT_OPENGL_PROFILE_MASK_KHR as EGLint,
            value as EGLint,
        );
    }

    /// Default value: 0
    ///
    /// `OpenGLContextFlags::FORWARD_COMPATIBLE` is supported only with
    /// OpenGL 3.0 or later.
    pub fn set_context_flags(&mut self, value: OpenGLContextFlags) {
        self.builder.attributes.add(
            extensions::CONTEXT_FLAGS_KHR as EGLint,
            value.bits() as EGLint,
        );
    }

    /// Default value: `ResetNotificationStrategy::NoResetNotification`
    pub fn set_reset_notification_strategy(&mut self, strategy: ResetNotificationStrategy) {
        self.builder.attributes.add(
            extensions::CONTEXT_OPENGL_RESET_NOTIFICATION_STRATEGY_KHR as EGLint,
            strategy as EGLint,
        );
    }

    /// This function calls `bind_api` before creating the context.
    pub(crate) fn build(self) -> Result<OpenGLContext<P>, Option<EGLError>> {
        self.builder.build()
    }
}
