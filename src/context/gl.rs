
//! OpenGL context

use std::marker::PhantomData;
use std::ptr;

use egl_sys::ffi;

use surface::WindowSurface;
use context::{ Context, CurrentContext };
use display::Display;
use error::EGLError;
use config::Config;


pub struct OpenGLContext<'a> {
    display: &'a Display,
    raw_context: ffi::types::EGLContext,
    _marker: PhantomData<ffi::types::EGLContext>,
}


impl <'a> Drop for OpenGLContext<'a> {
    fn drop(&mut self) {
        super::destroy_context(self.display().raw(), self.raw_context);
    }
}

impl <'a> Context<'a> for OpenGLContext<'a> {
    const API_TYPE: ffi::types::EGLenum = ffi::OPENGL_API;

    fn create(config: &'a Config<'a>) -> Result<OpenGLContext<'a>, Option<EGLError>> {
        Self::bind_api()?;

        let raw_context = unsafe {
            ffi::CreateContext(config.display().raw(), config.raw(), ffi::NO_CONTEXT, ptr::null())
        };

        if raw_context == ffi::NO_CONTEXT {
            Err(EGLError::check_errors())
        } else {
            Ok(OpenGLContext {
                display: config.display(),
                raw_context,
                _marker: PhantomData,
            })
        }
    }

    fn display(&self) -> &Display {
        self.display
    }

    fn raw(&self) -> ffi::types::EGLContext {
        self.raw_context
    }
}

impl <'a> CurrentContext<'a, WindowSurface<'a>> for OpenGLContext<'a> {}