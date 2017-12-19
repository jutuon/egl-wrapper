
pub mod client_api;
pub mod search;
pub mod attribute;

use std::vec;
use std::sync::Arc;

use egl_sys::{ ffi };

use display::{Display, DisplayHandle};
use surface::window::WindowSurfaceBuilder;
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
        self.display_handle.raw()
    }

    pub fn raw_config(&self) -> ffi::types::EGLConfig {
        self.raw_config
    }
}

/// Config query results.
pub struct Configs<'a> {
    display: &'a Display,
    raw_configs: Vec<ffi::types::EGLConfig>,
}

impl <'a> Configs<'a> {
    pub(crate) fn new(display: &Display, raw_configs: Vec<ffi::types::EGLConfig>) -> Configs {
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
pub struct IntoIter<'a> {
    display: &'a Display,
    raw_configs_iter: vec::IntoIter<ffi::types::EGLConfig>,
}

impl <'a> IntoIter<'a> {
    fn new(display: &'a Display, raw_configs_iter: vec::IntoIter<ffi::types::EGLConfig>) -> IntoIter<'a> {
        IntoIter {
            display,
            raw_configs_iter,
        }
    }
}

impl <'a> Iterator for IntoIter<'a> {
    type Item = Config<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.raw_configs_iter.next().map(|raw_config| {
            Config {
                display: self.display,
                raw_config,
            }
        })
    }
}

impl <'a> IntoIterator for Configs<'a> {
    type Item = Config<'a>;
    type IntoIter = IntoIter<'a>;

    fn into_iter(self) -> IntoIter<'a> {
        IntoIter::new(self.display, self.raw_configs.into_iter())
    }
}

#[derive(Debug, Clone)]
pub struct Config<'a> {
    display: &'a Display,
    raw_config: ffi::types::EGLConfig,
}

impl <'a> Config<'a> {
    fn into_display_config(self) -> DisplayConfig {
        DisplayConfig {
            display_handle: self.display().display_handle().clone(),
            raw_config: self.raw_config(),
        }
    }

    pub fn window_surface_builder(self) -> Option<WindowSurfaceBuilder> {
        match self.surface_type() {
            Ok(surface_type) if surface_type.contains(SurfaceType::WINDOW) => {
                Some(WindowSurfaceBuilder::new(self.into_display_config()))
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
        if !self.display.extension_support().create_context() {
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
        if !self.display().extension_support().create_context() {
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

impl <'a> ConfigUtils for Config<'a> {
    fn raw_config(&self) -> ffi::types::EGLConfig {
        self.raw_config
    }

    fn display(&self) -> &Display {
        self.display
    }
}

impl <'a> Color             for Config<'a> {}
impl <'a> AlphaMaskBuffer   for Config<'a> {}
impl <'a> Pbuffer           for Config<'a> {}
impl <'a> FramebufferLevel  for Config<'a> {}
impl <'a> ClientAPI         for Config<'a> {}
impl <'a> NativeRenderable  for Config<'a> {}
impl <'a> SlowConfig        for Config<'a> {}
impl <'a> Surface           for Config<'a> {}
impl <'a> SwapInterval      for Config<'a> {}
impl <'a> MultisampleBuffer for Config<'a> {}
impl <'a> DepthBuffer       for Config<'a> {}
impl <'a> StencilBuffer     for Config<'a> {}
impl <'a> TransparentColor  for Config<'a> {}

impl <'a> AllAttributes     for Config<'a> {}

