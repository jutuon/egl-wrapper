pub mod client_api;
pub mod search;
pub mod attribute;

use std::vec;
use std::sync::Arc;

use egl_sys::ffi;

use platform::Platform;
use display::{DisplayExtensionSupport, DisplayHandle, DisplayType};
use EGLHandle;

use self::attribute::*;

use utils::QueryResult;

#[derive(Debug, Clone)]
/// Config with reference counted handle to `Display`.
pub struct DisplayConfig<P: Platform> {
    display_handle: Arc<DisplayHandle<P>>,
    raw_config: ffi::types::EGLConfig,
}

impl<P: Platform> DisplayConfig<P> {
    pub(crate) fn new(
        display_handle: Arc<DisplayHandle<P>>,
        raw_config: ffi::types::EGLConfig,
    ) -> DisplayConfig<P> {
        DisplayConfig {
            display_handle,
            raw_config,
        }
    }

    pub fn raw_display(&self) -> ffi::types::EGLDisplay {
        self.display_handle.raw_display()
    }

    pub fn raw_config(&self) -> ffi::types::EGLConfig {
        self.raw_config
    }

    pub fn egl_handle(&self) -> &EGLHandle {
        self.display_handle.egl_handle()
    }
}

/// Config query results.
pub struct Configs<'a, D: DisplayType + 'a> {
    display: &'a D,
    raw_configs: Vec<ffi::types::EGLConfig>,
}

impl<'a, D: DisplayType + 'a> Configs<'a, D> {
    pub(crate) fn new(display: &'a D, raw_configs: Vec<ffi::types::EGLConfig>) -> Self {
        Configs {
            display,
            raw_configs,
        }
    }

    /// Query result count.
    pub fn count(&self) -> usize {
        self.raw_configs.len()
    }
}

/// Iterate config query results.
pub struct IntoIter<'a, D: DisplayType + 'a> {
    display: &'a D,
    raw_configs_iter: vec::IntoIter<ffi::types::EGLConfig>,
}

impl<'a, D: DisplayType + 'a> IntoIter<'a, D> {
    fn new(display: &'a D, raw_configs_iter: vec::IntoIter<ffi::types::EGLConfig>) -> Self {
        IntoIter {
            display,
            raw_configs_iter,
        }
    }
}

impl<'a, D: DisplayType + 'a> Iterator for IntoIter<'a, D> {
    type Item = Config<'a, D>;

    fn next(&mut self) -> Option<Self::Item> {
        self.raw_configs_iter.next().map(|raw_config| Config {
            display: self.display,
            raw_config,
        })
    }
}

impl<'a, D: DisplayType + 'a> IntoIterator for Configs<'a, D> {
    type Item = Config<'a, D>;
    type IntoIter = IntoIter<'a, D>;

    fn into_iter(self) -> IntoIter<'a, D> {
        IntoIter::new(self.display, self.raw_configs.into_iter())
    }
}

#[derive(Debug)]
pub struct Config<'a, D: DisplayType + 'a> {
    display: &'a D,
    raw_config: ffi::types::EGLConfig,
}

impl<'a, D: DisplayType + 'a> Config<'a, D> {
    pub fn window_config(&self) -> QueryResult<bool> {
        self.surface_type()
            .map(|flags| flags.contains(SurfaceType::WINDOW))
    }

    pub fn opengl_config(&self) -> QueryResult<bool> {
        self.client_api()
            .map(|flags| flags.contains(ConfigClientAPI::OPENGL))
    }

    pub fn opengl_es_1_config(&self) -> QueryResult<bool> {
        self.client_api()
            .map(|flags| flags.contains(ConfigClientAPI::OPENGL_ES))
    }

    pub fn opengl_es_2_config(&self) -> QueryResult<bool> {
        self.client_api()
            .map(|flags| flags.contains(ConfigClientAPI::OPENGL_ES2))
    }

    /// EGL_KHR_create_context
    pub fn opengl_es_3_config(&self) -> QueryResult<bool> {
        self.client_api()
            .map(|flags| flags.contains(ConfigClientAPI::OPENGL_ES3_KHR))
    }
}

impl<'a, D: DisplayType + 'a> ConfigUtils for Config<'a, D> {
    fn raw_config(&self) -> ffi::types::EGLConfig {
        self.raw_config
    }

    fn raw_display(&self) -> ffi::types::EGLDisplay {
        self.display.raw_display()
    }

    fn display_extensions(&self) -> &DisplayExtensionSupport {
        self.display.display_extensions()
    }

    fn egl_handle(&self) -> &EGLHandle {
        self.display.egl_handle()
    }
}

impl<'a, D: DisplayType + 'a> Color for Config<'a, D> {}
impl<'a, D: DisplayType + 'a> AlphaMaskBuffer for Config<'a, D> {}
impl<'a, D: DisplayType + 'a> Pbuffer for Config<'a, D> {}
impl<'a, D: DisplayType + 'a> FramebufferLevel for Config<'a, D> {}
impl<'a, D: DisplayType + 'a> ClientAPI for Config<'a, D> {}
impl<'a, D: DisplayType + 'a> NativeRenderable for Config<'a, D> {}
impl<'a, D: DisplayType + 'a> SlowConfig for Config<'a, D> {}
impl<'a, D: DisplayType + 'a> Surface for Config<'a, D> {}
impl<'a, D: DisplayType + 'a> SwapInterval for Config<'a, D> {}
impl<'a, D: DisplayType + 'a> MultisampleBuffer for Config<'a, D> {}
impl<'a, D: DisplayType + 'a> DepthBuffer for Config<'a, D> {}
impl<'a, D: DisplayType + 'a> StencilBuffer for Config<'a, D> {}
impl<'a, D: DisplayType + 'a> TransparentColor for Config<'a, D> {}

impl<'a, D: DisplayType + 'a> AllAttributes for Config<'a, D> {}
