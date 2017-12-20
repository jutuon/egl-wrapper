

use std::marker::PhantomData;

use egl_sys::ffi;
use egl_sys::ffi::types::EGLint;

use config::DisplayConfig;
use utils::{AttributeListBuilder, AttributeList, AttributeListTrait};

use config::client_api::ConfigWindow;

use super::{
    Surface,
    destroy_surface,
};

use super::attribute::{
    RenderBuffer,
    CommonAttributes,
    SurfaceAttributeUtils,
    WindowAttributes,
    MultisampleResolve,
    SwapBehavior,
};

#[derive(Debug)]
pub struct WindowSurface {
    window_config: ConfigWindow,
    raw_surface: ffi::types::EGLSurface,
    _marker: PhantomData<ffi::types::EGLSurface>,
}

impl WindowSurface {
    pub(crate) fn new(window_config: ConfigWindow, raw_surface: ffi::types::EGLSurface) -> WindowSurface {
        WindowSurface {
            window_config,
            raw_surface,
            _marker: PhantomData,
        }
    }
}

impl Surface for WindowSurface {
    fn raw_surface(&self) -> ffi::types::EGLSurface {
        self.raw_surface
    }

    fn display_config(&self) -> &DisplayConfig {
        self.window_config.display_config()
    }
}

impl Drop for WindowSurface {
    fn drop(&mut self) {
       destroy_surface(self)
    }
}

impl SurfaceAttributeUtils  for WindowSurface {}
impl CommonAttributes       for WindowSurface {}
impl MultisampleResolve     for WindowSurface {}
impl SwapBehavior           for WindowSurface {}

impl WindowAttributes       for WindowSurface {}

pub struct WindowSurfaceAttributeListBuilder {
    attributes: AttributeListBuilder,
}

impl WindowSurfaceAttributeListBuilder {
    pub fn new() -> WindowSurfaceAttributeListBuilder {
        WindowSurfaceAttributeListBuilder {
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

    pub fn build(self) -> WindowSurfaceAttributeList {
        WindowSurfaceAttributeList(self.attributes.build())
    }
}

pub struct WindowSurfaceAttributeList(AttributeList);

impl WindowSurfaceAttributeList {
    pub fn ptr(&self) -> *const EGLint {
        self.0.attribute_list_ptr()
    }
}