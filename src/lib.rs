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

use std::fmt;
use std::borrow::Cow;
use std::ffi::CStr;
use std::sync::Arc;

use platform::DefaultPlatform;

use display::get_proc_address;
use std::os::raw::c_void;

fn load_extension(text: &str) -> *const c_void {
    get_proc_address(text).unwrap()
}

#[derive(Clone)]
pub struct EGLHandle {
    pub(crate) extension_functions: Arc<extensions::Egl>,
}

impl fmt::Debug for EGLHandle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "EGLHandle")
    }
}

impl EGLHandle {
    pub fn load() -> Result<Self, ()> {
        let extension_functions = extensions::Egl::load_with(load_extension);

        Ok(EGLHandle {
            extension_functions: Arc::new(extension_functions)
        })
    }

    pub fn display_builder(&self) -> DisplayBuilder {
        DisplayBuilder::new(self.clone())
    }
}


#[derive(Debug)]
struct ClientExtensions {
    ext_platform_x11: bool,
    ext_platform_wayland: bool,
}

impl ClientExtensions {
    fn parse(text: &str, functions: &EGLHandle) -> Result<ClientExtensions, ()> {
        let mut extensions = ClientExtensions::default();

        for ext in text.split_whitespace() {
            match ext {
                "EGL_EXT_platform_base" => {
                    if !functions.extension_functions.GetPlatformDisplayEXT.is_loaded() ||
                        !functions.extension_functions.CreatePlatformWindowSurfaceEXT.is_loaded() ||
                        !functions.extension_functions.CreatePlatformPixmapSurfaceEXT.is_loaded() {
                            return Err(())
                    }
                }
                "EGL_EXT_platform_x11" => extensions.ext_platform_x11 = true,
                "EGL_EXT_platform_wayland" => extensions.ext_platform_wayland = true,
                _ => (),
            }
        }

        // TODO: If platform extension is found, require EGL_EXT_platform_base.

        Ok(extensions)
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
    egl_handle: EGLHandle,
}

impl DisplayBuilder {
    fn new(egl_handle: EGLHandle) -> DisplayBuilder {
        let mut display_builder = DisplayBuilder {
            client_extensions: None,
            egl_handle
        };

        display_builder.parse_client_extensions();

        display_builder
    }

    fn parse_client_extensions(&mut self) {
        let extensions = if let Ok(extensions) = self.query_client_extensions() {
            match ClientExtensions::parse(&extensions, &self.egl_handle) {
                Ok(extensions) => extensions,
                Err(()) => return,
            }
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
        // TODO: check client extension support

        EXTPlatform::get_display(
            display_type,
            native_display_ptr,
            native,
            attributes.unwrap_or_default(),
            self.egl_handle.clone()
        ).map_err(|e| (self, e))
    }
}
