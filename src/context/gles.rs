


//! OpenGL ES context

use std::marker::PhantomData;


use egl_sys::ffi;
use egl_sys::ffi::types::EGLint;

use context::{ Context, RawContextUtils };
use config::client_api::ConfigOpenGLES;
use utils::{AttributeListBuilder};
use error::EGLError;

use super::attribute::{
    ContextAttributeUtils,
    CommonAttributes,
    AttributeOpenGLESVersion,
};

pub trait ContextVersionGLES {
    fn version_number_attribute() -> EGLint;
}

pub struct Version1;
pub struct Version2;
pub struct Version3;

impl ContextVersionGLES for Version1 {
    fn version_number_attribute() -> EGLint {
        1
    }
}

impl ContextVersionGLES for Version2 {
    fn version_number_attribute() -> EGLint {
        2
    }
}

impl ContextVersionGLES for Version3 {
    fn version_number_attribute() -> EGLint {
        3
    }
}


#[derive(Debug)]
pub struct OpenGLESContext {
    config_opengl: ConfigOpenGLES,
    raw_context: ffi::types::EGLContext,
    _marker: PhantomData<ffi::types::EGLContext>,
}

impl ContextAttributeUtils for OpenGLESContext {}
impl CommonAttributes      for OpenGLESContext {}

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
    pub(crate) fn new<T: ContextVersionGLES>(config_opengl: ConfigOpenGLES) -> OpenGLESContextBuilder {
        let mut builder = OpenGLESContextBuilder {
            config_opengl,
            attributes: AttributeListBuilder::new(),
        };

        builder.attributes.add(ffi::CONTEXT_CLIENT_VERSION as EGLint, T::version_number_attribute());

        builder
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
                attribute_list.ptr()
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


// TODO: EGL_KHR_create_context extension, OpenGL ES 1.1 context request