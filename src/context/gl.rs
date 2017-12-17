
//! OpenGL context

use std::marker::PhantomData;


use egl_sys::ffi;

use context::{ Context, RawContextUtils };
use config::client_api::ConfigOpenGL;
use utils::{AttributeListBuilder};
use error::EGLError;

use super::attribute::{
    ContextAttributeUtils,
    CommonAttributes,
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