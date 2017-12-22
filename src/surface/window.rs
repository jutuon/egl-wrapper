use std::marker::PhantomData;

use egl_sys::ffi;
use egl_sys::ffi::types::EGLint;

use utils::{AttributeList, AttributeListBuilder, AttributeListTrait};
use config::client_api::ConfigWindow;
use platform::Platform;

use super::{destroy_surface, Surface};

use super::attribute::{CommonAttributes, MultisampleResolve, RenderBuffer, SurfaceAttributeUtils,
                       SwapBehavior, WindowAttributes};

#[derive(Debug)]
pub struct WindowSurface<T, P: Platform> {
    optional_native_window_handle: T,
    window_config: ConfigWindow<P>,
    raw_surface: ffi::types::EGLSurface,
    _marker: PhantomData<ffi::types::EGLSurface>,
}

impl<T, P: Platform> WindowSurface<T, P> {
    pub(crate) fn new(
        optional_native_window_handle: T,
        window_config: ConfigWindow<P>,
        raw_surface: ffi::types::EGLSurface,
    ) -> Self {
        WindowSurface {
            optional_native_window_handle,
            window_config,
            raw_surface,
            _marker: PhantomData,
        }
    }
}

impl<T, P: Platform> Surface for WindowSurface<T, P> {
    fn raw_surface(&self) -> ffi::types::EGLSurface {
        self.raw_surface
    }

    fn raw_display(&self) -> ffi::types::EGLDisplay {
        self.window_config.display_config().raw_display()
    }
}

impl<T, P: Platform> Drop for WindowSurface<T, P> {
    fn drop(&mut self) {
        destroy_surface(self)
    }
}

impl<T, P: Platform> SurfaceAttributeUtils for WindowSurface<T, P> {}
impl<T, P: Platform> CommonAttributes for WindowSurface<T, P> {}
impl<T, P: Platform> MultisampleResolve for WindowSurface<T, P> {}
impl<T, P: Platform> SwapBehavior for WindowSurface<T, P> {}

impl<T, P: Platform> WindowAttributes for WindowSurface<T, P> {}

pub struct WindowSurfaceAttributeListBuilder {
    attributes: AttributeListBuilder,
}

impl WindowSurfaceAttributeListBuilder {
    pub fn new() -> Self {
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
        self.attributes
            .add(ffi::RENDER_BUFFER as EGLint, render_buffer as EGLint);
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

impl Default for WindowSurfaceAttributeList {
    fn default() -> Self {
        WindowSurfaceAttributeList(AttributeList::empty())
    }
}
