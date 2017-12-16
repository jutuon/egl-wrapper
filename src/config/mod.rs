
pub mod client_api;
pub mod search;
pub mod attribute;

use std::vec;
use std::sync::Arc;

use egl_sys::{ ffi };

use display::{Display, DisplayHandle};

use surface::WindowSurfaceBuilder;
use self::attribute::*;

use self::client_api::*;


#[derive(Debug)]
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

    pub fn count(&self) -> usize {
        self.raw_configs.len()
    }
}

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

    pub fn window_surface(self) -> Option<WindowSurfaceBuilder> {
        // TODO: check config

        Some(WindowSurfaceBuilder::new(self.into_display_config()))
    }

    pub fn opengl_context(self) -> Option<ConfigOpenGL> {
        // TODO: check config

        Some(ConfigOpenGL::new(self.into_display_config()))
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
impl <'a> ClientApi         for Config<'a> {}
impl <'a> NativeRenderable  for Config<'a> {}
impl <'a> SlowConfig        for Config<'a> {}
impl <'a> Surface           for Config<'a> {}
impl <'a> SwapInterval      for Config<'a> {}
impl <'a> MultisampleBuffer for Config<'a> {}
impl <'a> DepthBuffer       for Config<'a> {}
impl <'a> StencilBuffer     for Config<'a> {}
impl <'a> TransparentColor  for Config<'a> {}

impl <'a> AllAttributes     for Config<'a> {}

