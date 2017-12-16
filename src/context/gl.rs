
//! OpenGL context

use std::marker::PhantomData;


use egl_sys::ffi;

use surface::window::WindowSurface;
use context::{ Context, MakeCurrentSurfaceAndContext, RawContextUtils, SingleContext };
use config::DisplayConfig;


#[derive(Debug)]
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

impl RawContextUtils for OpenGLContext {
    const API_TYPE: ffi::types::EGLenum = ffi::OPENGL_API;

    fn new(display_config: DisplayConfig, raw_context: ffi::types::EGLContext) -> Self {
        OpenGLContext {
            display_config,
            raw_context,
            _marker: PhantomData,
        }
    }
}

impl Context for OpenGLContext {
    fn raw_display(&self) -> ffi::types::EGLDisplay {
        self.display_config.raw_display()
    }

    fn raw_context(&self) -> ffi::types::EGLContext {
        self.raw_context
    }
}

impl MakeCurrentSurfaceAndContext<WindowSurface> for SingleContext<OpenGLContext> {}