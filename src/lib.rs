
extern crate egl_sys;

#[macro_use]
extern crate bitflags;

#[cfg(unix)]
extern crate x11;

pub mod utils;
pub mod config;
mod error;
pub mod display;
pub mod surface;
pub mod context;
pub mod platform;

pub use egl_sys::ffi;

use ffi::types::EGLint;

use display::{ Display, DisplayCreationError };
use error::EGLError;

use std::borrow::Cow;
use std::ffi::CStr;

use platform::{ DefaultPlatform };

#[derive(Debug)]
struct ClientExtensions {

}

impl ClientExtensions {
    fn parse(text: &str) -> ClientExtensions {
        ClientExtensions {}
    }
}

#[derive(Debug)]
pub struct DisplayBuilder {

}

impl DisplayBuilder {
    pub fn new() -> Result<DisplayBuilder, ()> {
        Ok(DisplayBuilder {})
    }

    pub fn client_extension_mode(self) -> Result<DisplayBuilderWithClientExtensions, Self> {
        let ptr = unsafe {
            ffi::QueryString(ffi::NO_DISPLAY, ffi::EXTENSIONS as EGLint)
        };

        if let Some(_) = EGLError::check_errors() {
            return Err(self)
        }

        if ptr.is_null() {
            return Err(self);
        }

        let cstr = unsafe {
            CStr::from_ptr(ptr)
        };

        let client_extensions = ClientExtensions::parse(&cstr.to_string_lossy());

        Ok(DisplayBuilderWithClientExtensions {
            client_extensions,
        })
    }

    pub fn build_from_native_display(self, display_id: ffi::types::EGLNativeDisplayType) -> Result<Display<DefaultPlatform>, (Self, DisplayCreationError)> {
        DefaultPlatform::get_display(display_id).map_err(|e| (self, e))
    }
/*
    pub fn build_default_display(self) -> Result<Display<DefaultPlatform>, (Self, DisplayCreationError)> {
        Display::default_display().map_err(|e| (self, e))
    }
*/
}

#[derive(Debug)]
pub struct DisplayBuilderWithClientExtensions {
    client_extensions: ClientExtensions
}

impl DisplayBuilderWithClientExtensions {
    pub fn client_extensions(&self) -> Result<Cow<str>, ()> {
        unsafe {
            let ptr = ffi::QueryString(ffi::NO_DISPLAY, ffi::EXTENSIONS as EGLint);

            if ptr.is_null() {
                return Err(());
            }

            let cstr = CStr::from_ptr(ptr);

            Ok(cstr.to_string_lossy())
        }
    }
}