
extern crate egl_sys;

#[macro_use]
extern crate bitflags;

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

use platform::{ DefaultPlatform, EXTPlatformX11, RawNativeDisplay, RawNativeWindow, EXTPlatformX11AttributeListBuilder };

#[derive(Debug)]
struct ClientExtensions {
    ext_platform_x11: bool,
    ext_platform_wayland: bool,
}

impl ClientExtensions {
    fn new() -> ClientExtensions {
        ClientExtensions {
            ext_platform_x11: false,
            ext_platform_wayland: false,
        }
    }

    fn parse(text: &str) -> ClientExtensions {
        let mut extensions = ClientExtensions::new();

        for ext in text.split_whitespace() {
            match ext {
                "EGL_EXT_platform_x11" => extensions.ext_platform_x11 = true,
                "EGL_EXT_platform_wayland" => extensions.ext_platform_wayland = true,
                _ => (),
            }
        }

        extensions
    }
}

#[derive(Debug)]
pub struct DisplayBuilder {}

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

    pub fn build_from_native_display<T: RawNativeDisplay<T=ffi::types::NativeDisplayType> + RawNativeWindow<T=ffi::types::NativeWindowType>> (self, native: T) -> Result<Display<DefaultPlatform<T>>, (Self, DisplayCreationError)> {
        DefaultPlatform::get_display(native).map_err(|e| (self, e))
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

    pub fn to_display_builder(self) -> DisplayBuilder {
        DisplayBuilder {}
    }

    pub fn build_ext_platform_x11<T: RawNativeDisplay<T=*mut x11::xlib::Display> + RawNativeWindow<T=x11::xlib::Window>>(self, native: T, list: EXTPlatformX11AttributeListBuilder) -> Result<Display<EXTPlatformX11<T>>, (Self, DisplayCreationError)> {
        if !self.client_extensions.ext_platform_x11 {
            return Err((self,DisplayCreationError::PlatformExtensionNotSupported));
        }

        EXTPlatformX11::get_display(native, list).map_err(|e| (self, e))
    }
}