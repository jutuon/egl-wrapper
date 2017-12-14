
use std::marker::PhantomData;

use egl_sys::ffi;

use config::DisplayConfig;
use utils::{AttributeListBuilder};

use error::EGLError;


pub struct WindowSurface {
    display_config: DisplayConfig,
    raw_surface: ffi::types::EGLSurface,
    _marker: PhantomData<ffi::types::EGLSurface>,
}


impl Surface for WindowSurface {
    fn raw(&self) -> ffi::types::EGLSurface {
        self.raw_surface
    }
}

impl Drop for WindowSurface {
    fn drop(&mut self) {
        let result = unsafe {
            ffi::DestroySurface(self.display_config.raw_display(), self.raw_surface)
        };

        if result == ffi::FALSE {
            let error = EGLError::check_errors();
            eprintln!("egl_wrapper: couldn't destroy surface, error: {:?}", error);
        }

        // TODO: eglReleaseThread
    }
}

pub struct WindowSurfaceBuilder {
    display_config: DisplayConfig,
    attributes: AttributeListBuilder,
}

impl WindowSurfaceBuilder {
    pub(crate) fn new(display_config: DisplayConfig) -> WindowSurfaceBuilder {
        WindowSurfaceBuilder {
            display_config,
            attributes: AttributeListBuilder::new(),
        }
    }

    // TODO: WindowSurface attributes


    pub fn build(self, native_window: ffi::types::EGLNativeWindowType) -> Result<WindowSurface, Option<EGLError>> {
        let attributes = self.attributes.build();

        let result = unsafe {
            ffi::CreateWindowSurface(self.display_config.raw_display(), self.display_config.raw(), native_window, attributes.ptr())
        };

        if result == ffi::NO_SURFACE {
            return Err(EGLError::check_errors());
        }

        Ok(WindowSurface {
            display_config: self.display_config,
            raw_surface: result,
            _marker: PhantomData,
        })
    }
}



pub trait Surface {
    fn raw(&self) -> ffi::types::EGLSurface;
}