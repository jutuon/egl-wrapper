
//! OpenGL context

use std::marker::PhantomData;
use std::ptr;

use egl_sys::ffi;

use surface::WindowSurface;
use context::{ Context, CurrentContext };
use error::EGLError;
use config::DisplayConfig;


pub struct OpenGLContext {
    display_config: DisplayConfig,
    raw_context: ffi::types::EGLContext,
    _marker: PhantomData<ffi::types::EGLContext>,
}


impl Drop for OpenGLContext {
    fn drop(&mut self) {
        super::destroy_context(self.raw_display(), self.raw_context);
    }
}

impl Context for OpenGLContext {
    const API_TYPE: ffi::types::EGLenum = ffi::OPENGL_API;

    fn create(config: DisplayConfig) -> Result<OpenGLContext, Option<EGLError>> {
        Self::bind_api()?;

        let raw_context = unsafe {
            ffi::CreateContext(config.raw_display(), config.raw(), ffi::NO_CONTEXT, ptr::null())
        };

        if raw_context == ffi::NO_CONTEXT {
            Err(EGLError::check_errors())
        } else {
            Ok(OpenGLContext {
                display_config: config,
                raw_context,
                _marker: PhantomData,
            })
        }
    }

    fn raw_display(&self) -> ffi::types::EGLDisplay {
        self.display_config.raw_display()
    }

    fn raw(&self) -> ffi::types::EGLContext {
        self.raw_context
    }
}

impl CurrentContext<WindowSurface> for OpenGLContext {}