
pub mod client_api;
pub mod search;
pub mod attribute;

use std::vec;
use std::sync::Arc;

use egl_sys::{ ffi };

use display::{DisplayHandle, DisplayType, DisplayExtensionSupport};
use context::gl::{ OpenGLContextBuilder, OpenGLContextBuilderEXT };
use context::gles::{ OpenGLESContextBuilder, OpenGLESContextBuilderEXT, EGL14OpenGLESVersion, OpenGLESMajorVersionEXT };

use self::attribute::*;
use self::client_api::*;

#[derive(Debug, Clone)]
/// Config with reference counted handle to `Display`.
pub struct DisplayConfig {
    display_handle: Arc<DisplayHandle>,
    raw_config: ffi::types::EGLConfig,
}

impl DisplayConfig {
    pub fn raw_display(&self) -> ffi::types::EGLDisplay {
        self.display_handle.raw_display()
    }

    pub fn raw_config(&self) -> ffi::types::EGLConfig {
        self.raw_config
    }
}

/// Config query results.
pub struct Configs<'a, D: DisplayType + 'a> {
    display: &'a D,
    raw_configs: Vec<ffi::types::EGLConfig>,
}

impl <'a, D: DisplayType + 'a> Configs<'a, D> {
    pub(crate) fn new(display: &'a D, raw_configs: Vec<ffi::types::EGLConfig>) -> Configs<'a, D> {
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

impl <'a, D: DisplayType + 'a> IntoIter<'a, D> {
    fn new(display: &'a D, raw_configs_iter: vec::IntoIter<ffi::types::EGLConfig>) -> IntoIter<'a, D> {
        IntoIter {
            display,
            raw_configs_iter,
        }
    }
}

impl <'a, D: DisplayType + 'a> Iterator for IntoIter<'a, D> {
    type Item = Config<'a, D>;

    fn next(&mut self) -> Option<Self::Item> {
        self.raw_configs_iter.next().map(|raw_config| {
            Config {
                display: self.display,
                raw_config,
            }
        })
    }
}

impl <'a, D: DisplayType + 'a> IntoIterator for Configs<'a, D> {
    type Item = Config<'a, D>;
    type IntoIter = IntoIter<'a, D>;

    fn into_iter(self) -> IntoIter<'a, D> {
        IntoIter::new(self.display, self.raw_configs.into_iter())
    }
}

#[derive(Debug, Clone)]
pub struct Config<'a, D: DisplayType + 'a> {
    display: &'a D,
    raw_config: ffi::types::EGLConfig,
}

impl <'a, D: DisplayType + 'a> Config<'a, D> {
    fn into_display_config(self) -> DisplayConfig {
        DisplayConfig {
            display_handle: self.display.display_handle().clone(),
            raw_config: self.raw_config(),
        }
    }

    pub fn window_surface(self) -> Option<ConfigWindow> {
        match self.surface_type() {
            Ok(surface_type) if surface_type.contains(SurfaceType::WINDOW) => {
                Some(ConfigWindow::new(self.into_display_config()))
            }
            _ => None
        }
    }

    pub fn opengl_context_builder(self) -> Option<OpenGLContextBuilder> {
        match self.client_api() {
            Ok(client_api) if client_api.contains(ConfigClientAPI::OPENGL) => {
                Some(OpenGLContextBuilder::new(ConfigOpenGL::new(self.into_display_config())))
            }
            _ => None
        }
    }

    /// Returns None if extension EGL_KHR_create_context is not supported or
    /// config does not support OpengGL.
    pub fn opengl_context_builder_ext(self) -> Option<OpenGLContextBuilderEXT> {
        if !self.display_extensions().create_context() {
            return None;
        }

        match self.client_api() {
            Ok(client_api) if client_api.contains(ConfigClientAPI::OPENGL) => {
                Some(OpenGLContextBuilderEXT::new(ConfigOpenGL::new(self.into_display_config())))
            }
            _ => None
        }
    }

    pub fn opengl_es_context_builder(self, version: EGL14OpenGLESVersion) -> Option<OpenGLESContextBuilder> {
        let mut builder = match self.client_api() {
            Ok(client_api) if client_api.contains(ConfigClientAPI::OPENGL_ES) && version == EGL14OpenGLESVersion::Version1 => {
                OpenGLESContextBuilder::new(ConfigOpenGLES::new(self.into_display_config()))
            },
            Ok(client_api) if client_api.contains(ConfigClientAPI::OPENGL_ES2) && version == EGL14OpenGLESVersion::Version2 => {
                OpenGLESContextBuilder::new(ConfigOpenGLES::new(self.into_display_config()))
            }
            _ => return None,
        };

        builder.set_context_client_version(version);
        Some(builder)
    }

    /// EGL_KHR_create_context
    pub fn opengl_es_context_builder_ext(self, version: OpenGLESMajorVersionEXT) -> Option<OpenGLESContextBuilderEXT> {
        if !self.display_extensions().create_context() {
            return None;
        }

        let mut builder = match self.client_api() {
            Ok(client_api) if client_api.contains(ConfigClientAPI::OPENGL_ES) && version == OpenGLESMajorVersionEXT::Version1 => {
                OpenGLESContextBuilderEXT::new(ConfigOpenGLES::new(self.into_display_config()))
            },
            Ok(client_api) if client_api.contains(ConfigClientAPI::OPENGL_ES2) && version == OpenGLESMajorVersionEXT::Version2 => {
                OpenGLESContextBuilderEXT::new(ConfigOpenGLES::new(self.into_display_config()))
            },
            Ok(client_api) if client_api.contains(ConfigClientAPI::OPENGL_ES3_KHR) && version == OpenGLESMajorVersionEXT::Version3 => {
                OpenGLESContextBuilderEXT::new(ConfigOpenGLES::new(self.into_display_config()))
            }
            _ => return None,
        };

        builder.set_major_version(version);
        Some(builder)
    }
}

impl <'a, D: DisplayType + 'a> ConfigUtils for Config<'a, D> {
    fn raw_config(&self) -> ffi::types::EGLConfig {
        self.raw_config
    }

    fn raw_display(&self) -> ffi::types::EGLDisplay {
        self.display.display_handle().raw_display()
    }

    fn display_extensions(&self) -> &DisplayExtensionSupport {
        self.display.display_extensions()
    }
}

impl <'a, D: DisplayType + 'a> Color             for Config<'a, D> {}
impl <'a, D: DisplayType + 'a> AlphaMaskBuffer   for Config<'a, D> {}
impl <'a, D: DisplayType + 'a> Pbuffer           for Config<'a, D> {}
impl <'a, D: DisplayType + 'a> FramebufferLevel  for Config<'a, D> {}
impl <'a, D: DisplayType + 'a> ClientAPI         for Config<'a, D> {}
impl <'a, D: DisplayType + 'a> NativeRenderable  for Config<'a, D> {}
impl <'a, D: DisplayType + 'a> SlowConfig        for Config<'a, D> {}
impl <'a, D: DisplayType + 'a> Surface           for Config<'a, D> {}
impl <'a, D: DisplayType + 'a> SwapInterval      for Config<'a, D> {}
impl <'a, D: DisplayType + 'a> MultisampleBuffer for Config<'a, D> {}
impl <'a, D: DisplayType + 'a> DepthBuffer       for Config<'a, D> {}
impl <'a, D: DisplayType + 'a> StencilBuffer     for Config<'a, D> {}
impl <'a, D: DisplayType + 'a> TransparentColor  for Config<'a, D> {}

impl <'a, D: DisplayType + 'a> AllAttributes     for Config<'a, D> {}

