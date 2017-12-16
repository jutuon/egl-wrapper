

use std::marker::PhantomData;

use egl_sys::ffi;
use egl_sys::ffi::types::EGLint;

use config::DisplayConfig;
use utils::{AttributeListBuilder};

use error::EGLError;

use super::{
    Surface,
    destroy_surface,
};

use super::attribute::{ RenderBuffer };

#[derive(Debug)]
pub struct WindowSurface {
    display_config: DisplayConfig,
    raw_surface: ffi::types::EGLSurface,
    _marker: PhantomData<ffi::types::EGLSurface>,
}


impl Surface for WindowSurface {
    fn raw_surface(&self) -> ffi::types::EGLSurface {
        self.raw_surface
    }

    fn display_config(&self) -> &DisplayConfig {
        &self.display_config
    }
}

impl Drop for WindowSurface {
    fn drop(&mut self) {
       destroy_surface(self)
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

    // TODO: WindowSurface OpenVG attributes

    /// Set preferred rendering buffer.
    ///
    /// Default value: `RenderBuffer::BackBuffer`.
    ///
    /// For more information see EGL 1.4 specification page 28.
    pub fn render_buffer(&mut self, render_buffer: RenderBuffer) -> &mut Self {
        self.attributes.add(ffi::RENDER_BUFFER as EGLint, render_buffer as EGLint);
        self
    }

    pub fn build(self, native_window: ffi::types::EGLNativeWindowType) -> Result<WindowSurface, Option<EGLError>> {
        let attributes = self.attributes.build();

        let result = unsafe {
            ffi::CreateWindowSurface(self.display_config.raw_display(), self.display_config.raw_config(), native_window, attributes.ptr())
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