extern crate egl_sys;

#[cfg(feature = "runtime-linking")]
extern crate libloading;

macro_rules! egl_function {
    ( $egl_handle:expr, $function:tt ( $( $function_argument:expr ),*) ) => {
        {
            #[cfg(not(feature = "runtime-linking"))]
            {
                (::egl_sys::ffi::$function)( $( $function_argument ,)* )
            }

            #[cfg(feature = "runtime-linking")]
            {
                $egl_handle.functions.functions.$function( $( $function_argument ,)* )
            }
        }
    };
}

#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate lazy_static;

pub mod utils;
pub mod config;
mod error;
pub mod display;
pub mod surface;
pub mod context;
pub mod platform;

pub use egl_sys::ffi;

use std::fmt;
use std::borrow::Cow;
use std::ffi::CStr;
use std::sync::{Arc, Mutex};
use std::io;
use std::os::raw::c_void;

use egl_sys::extensions;
use egl_sys::ffi::types::EGLint;

use display::{Display, DisplayCreationError};
use error::EGLError;
use platform::{EXTPlatform, EXTPlatformType, DefaultPlatform, EXTPlatformAttributeList};

lazy_static! {
    static ref INIT_FLAG: Mutex<bool> = Mutex::new(false);
}

#[derive(Debug)]
/// Initialization error
pub enum EGLInitError {
    AlreadyInitialized,
    /// This error can only happen if runtime library
    /// loading feature is enabled.
    LibraryLoadingError(io::Error),
    /// This error can only happen if runtime library
    /// loading feature is enabled.
    SymbolNotFound(io::Error),
}


#[cfg(not(feature = "runtime-linking"))]
pub(crate) struct EGLFunctions {
    pub(crate) extensions: extensions::Egl,
}

#[cfg(feature = "runtime-linking")]
pub(crate) struct EGLFunctions {
    _egl_library: libloading::Library,
    pub(crate) functions: ffi::Egl,
    pub(crate) extensions: extensions::Egl,
}

#[derive(Clone)]
pub struct EGLHandle {
    pub(crate) functions: Arc<EGLFunctions>,
}

impl fmt::Debug for EGLHandle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "EGLHandle")
    }
}

impl EGLHandle {
    #[cfg(not(feature = "runtime-linking"))]
    /// EGLHandle can only be created once.
    pub fn load() -> Result<Self, EGLInitError> {
        let mut init_flag_guard = INIT_FLAG.lock().unwrap();

        if *init_flag_guard {
            Err(EGLInitError::AlreadyInitialized)
        } else {
            let extensions = extensions::Egl::load_with(|name| {
                // TODO: add function name null error
                let c_string = match std::ffi::CString::new(name) {
                    Ok(s) => s,
                    Err(_) => return std::ptr::null(),
                };

                unsafe {
                    ffi::GetProcAddress(c_string.as_ptr())
                        as *const std::os::raw::c_void
                }
            });

            *init_flag_guard = true;

            Ok(EGLHandle {
                functions: Arc::new(EGLFunctions {
                    extensions
                })
            })
        }
    }

    #[cfg(feature = "runtime-linking")]
    pub fn load() -> Result<Self, EGLInitError> {
        let mut init_flag_guard = INIT_FLAG.lock().unwrap();

        if *init_flag_guard {
            Err(EGLInitError::AlreadyInitialized)
        } else {
            let egl_library = libloading::Library::new("EGL").map_err(|e| EGLInitError::LibraryLoadingError(e))?;

            let mut loading_error: Option<io::Error> = None;

            let functions = ffi::Egl::load_with(|name| {
                let function_pointer: libloading::Symbol<*const c_void> = unsafe {
                    // TODO: Does libloading return error if symbol is not found?
                    match egl_library.get(name.as_bytes()) {
                        Ok(function_pointer) => function_pointer,
                        Err(error) => {
                            loading_error = Some(error);
                            return std::ptr::null();
                        }
                    }
                };

                *function_pointer
            });

            if let Some(error) = loading_error {
                return Err(EGLInitError::SymbolNotFound(error));
            }

            let extensions = extensions::Egl::load_with(|name| {
                // TODO: add function name null error
                let c_string = match std::ffi::CString::new(name) {
                    Ok(s) => s,
                    Err(_) => return std::ptr::null(),
                };

                unsafe {
                    functions.GetProcAddress(c_string.as_ptr())
                        as *const std::os::raw::c_void
                }
            });

            let egl_functions = EGLFunctions {
                _egl_library: egl_library,
                functions,
                extensions
            };

            *init_flag_guard = true;

            Ok(EGLHandle {
                functions: Arc::new(egl_functions),
            })
        }
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
    fn parse(text: &str, egl_handle: &EGLHandle) -> Result<ClientExtensions, ()> {
        let mut extensions = ClientExtensions::default();

        for ext in text.split_whitespace() {
            match ext {
                "EGL_EXT_platform_base" => {
                    if !egl_handle.functions.extensions.GetPlatformDisplayEXT.is_loaded() ||
                        !egl_handle.functions.extensions.CreatePlatformWindowSurfaceEXT.is_loaded() ||
                        !egl_handle.functions.extensions.CreatePlatformPixmapSurfaceEXT.is_loaded() {
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
            let ptr = egl_function!(self.egl_handle, QueryString(ffi::NO_DISPLAY, ffi::EXTENSIONS as EGLint));

            if EGLError::check_errors(&self.egl_handle).is_some() || ptr.is_null() {
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
        DefaultPlatform::get_display(self.egl_handle.clone(), native_display, optional_native_display_handle)
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
