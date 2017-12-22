extern crate egl_sys;

#[macro_use]
extern crate bitflags;

pub mod utils;
pub mod config;
mod error;
pub mod display;
pub mod surface;
pub mod context;
pub mod platform;

use platform::{EXTPlatform, EXTPlatformType};
use platform::EXTPlatformAttributeList;
pub use egl_sys::ffi;

use egl_sys::extensions;

use ffi::types::EGLint;

use display::{Display, DisplayCreationError};
use error::EGLError;

use std::borrow::Cow;
use std::ffi::CStr;

use platform::DefaultPlatform;

use display::get_proc_address;
use std::os::raw::c_void;

fn load_extension(text: &str) -> *const c_void {
    get_proc_address(text).unwrap()
}

#[derive(Debug)]
struct ClientExtensions {
    ext_platform_x11: bool,
    ext_platform_wayland: bool,
}

impl ClientExtensions {
    fn parse(text: &str) -> ClientExtensions {
        let mut extensions = ClientExtensions::default();

        for ext in text.split_whitespace() {
            match ext {
                "EGL_EXT_platform_base" => {
                    extensions::GetPlatformDisplayEXT::load_with(load_extension);
                    extensions::CreatePlatformWindowSurfaceEXT::load_with(load_extension);
                    extensions::CreatePlatformPixmapSurfaceEXT::load_with(load_extension);

                    if !extensions::GetPlatformDisplayEXT::is_loaded() {
                        panic!()
                    }
                    if !extensions::CreatePlatformWindowSurfaceEXT::is_loaded() {
                        panic!()
                    }
                    if !extensions::CreatePlatformPixmapSurfaceEXT::is_loaded() {
                        panic!()
                    }
                }
                "EGL_EXT_platform_x11" => extensions.ext_platform_x11 = true,
                "EGL_EXT_platform_wayland" => extensions.ext_platform_wayland = true,
                _ => (),
            }
        }

        extensions
    }
}

impl Default for ClientExtensions {
    fn default() -> Self {
        ClientExtensions {
            ext_platform_x11: false,
            ext_platform_wayland: false,
        }
    }
}

#[derive(Debug)]
pub struct DisplayBuilder {
    client_extensions: Option<ClientExtensions>,
}

impl DisplayBuilder {
    pub fn new() -> Result<DisplayBuilder, ()> {
        let mut display_builder = DisplayBuilder {
            client_extensions: None,
        };

        display_builder.parse_client_extensions();

        Ok(display_builder)
    }

    fn parse_client_extensions(&mut self) {
        let extensions = if let Ok(extensions) = self.query_client_extensions() {
            ClientExtensions::parse(&extensions)
        } else {
            return;
        };

        self.client_extensions = Some(extensions);
    }

    pub fn query_client_extensions(&self) -> Result<Cow<str>, ()> {
        unsafe {
            let ptr = ffi::QueryString(ffi::NO_DISPLAY, ffi::EXTENSIONS as EGLint);

            if EGLError::check_errors().is_some() || ptr.is_null() {
                return Err(());
            }

            let cstr = CStr::from_ptr(ptr);

            Ok(cstr.to_string_lossy())
        }
    }

    pub fn build_default_platform_display<T>(
        self,
        native_display: ffi::types::NativeDisplayType,
        optional_native_display_handle: T,
    ) -> Result<Display<DefaultPlatform<T>>, (Self, DisplayCreationError)> {
        DefaultPlatform::get_display(native_display, optional_native_display_handle)
            .map_err(|e| (self, e))
    }

    pub fn ext_platform_x11(&self) -> bool {
        if let Some(ref extensions) = self.client_extensions {
            extensions.ext_platform_x11
        } else {
            false
        }
    }

    pub fn ext_platform_wayland(&self) -> bool {
        if let Some(ref extensions) = self.client_extensions {
            extensions.ext_platform_wayland
        } else {
            false
        }
    }

    pub unsafe fn build_ext_platform_base_display<T>(
        self,
        display_type: EXTPlatformType,
        native_display_ptr: *mut c_void,
        native: T,
        attributes: Option<EXTPlatformAttributeList>,
    ) -> Result<Display<EXTPlatform<T>>, (Self, DisplayCreationError)> {
        EXTPlatform::get_display(
            display_type,
            native_display_ptr,
            native,
            attributes.unwrap_or_default(),
        ).map_err(|e| (self, e))
    }
}
