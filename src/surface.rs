
use std::marker::PhantomData;

use egl_sys::ffi;

use display::EGLDisplay;
use config::Config;
use utils::{AttributeList, AttributeListBuilder};

use error::EGLError;


pub struct WindowSurface<'a> {
    config: Config<'a>,
    raw_surface: ffi::types::EGLSurface,
    _marker: PhantomData<ffi::types::EGLSurface>,
}


impl <'a> Surface for WindowSurface<'a> {
    fn raw(&self) -> ffi::types::EGLSurface {
        self.raw_surface
    }
}

impl <'a> Drop for WindowSurface<'a> {
    fn drop(&mut self) {
        let result = unsafe {
            ffi::DestroySurface(self.config.display().raw(), self.raw_surface)
        };

        if result == ffi::FALSE {
            let error = EGLError::check_errors();
            eprintln!("egl_wrapper: couldn't destroy surface, error: {:?}", error);
        }

        // TODO: eglReleaseThread
    }
}

pub struct WindowSurfaceBuilder<'a> {
    config: Config<'a>,
    attributes: AttributeListBuilder,
}

impl <'a> WindowSurfaceBuilder<'a> {
    pub(crate) fn new(config: Config<'a>) -> WindowSurfaceBuilder<'a> {
        WindowSurfaceBuilder {
            config,
            attributes: AttributeListBuilder::new(),
        }
    }

    // TODO: WindowSurface attributes


    pub fn build(self, native_window: ffi::types::EGLNativeWindowType) -> Result<WindowSurface<'a>, Option<EGLError>> {
        let attributes = self.attributes.build();

        let result = unsafe {
            ffi::CreateWindowSurface(self.config.display().raw(), self.config.raw(), native_window, attributes.ptr())
        };

        if result == ffi::NO_SURFACE {
            return Err(EGLError::check_errors());
        }

        Ok(WindowSurface {
            config: self.config,
            raw_surface: result,
            _marker: PhantomData,
        })
    }
}



pub trait Surface {
    fn raw(&self) -> ffi::types::EGLSurface;
}