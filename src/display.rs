

use std::ffi::CStr;
use std::ptr;
use std::borrow::Cow;
use std::marker::PhantomData;
use std::sync::Arc;

use egl_sys::ffi;
use egl_sys::ffi::types::EGLint;

use config::{Configs};
use config::client_api::ConfigOpenGL;
use config::search::{ ConfigSearchOptions, ConfigSearchOptionsBuilder};

use context::gl::OpenGLContext;
use context::SingleContext;
use error::EGLError;

#[derive(Debug)]
pub enum DisplayCreationError {
    NoMatchingDisplay,
    EGLInitializationError,
    EGLVersionUnsupported,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone)]
pub enum EGLVersion {
    EGL_1_4,
    EGL_1_5,
}

impl EGLVersion {
    fn parse(version_major: EGLint, version_minor: EGLint) -> Option<EGLVersion> {
        match version_major {
            1 => match version_minor {
                4 => Some(EGLVersion::EGL_1_4),
                5 => Some(EGLVersion::EGL_1_5),
                _ => None
            },
            _ => None
        }
    }
}

#[derive(Debug)]
pub(crate) struct DisplayHandle {
    raw_display: ffi::types::EGLDisplay,
    _marker: PhantomData<ffi::types::EGLDisplay>,
}

impl DisplayHandle {
    fn new_in_arc(raw_display: ffi::types::EGLDisplay) -> Arc<DisplayHandle> {
        let display_handle = DisplayHandle {
            raw_display,
            _marker: PhantomData,
        };

        Arc::new(display_handle)
    }

    pub fn raw(&self) -> ffi::types::EGLDisplay {
        self.raw_display
    }
}


impl Drop for DisplayHandle {
    fn drop(&mut self) {
        let result = unsafe {
            ffi::Terminate(self.raw_display)
        };

        if result == ffi::FALSE {
            eprintln!("egl_wrapper: eglTerminate returned false");
        }

        // TODO: call eglReleaseThread

        // TODO: Make sure that there is no current contexts when Display is
        //       dropped.
    }
}


// TODO: multiple calls to GetDisplay will return same EGLDisplay handle

/// EGLDisplay with initialized EGL
#[derive(Debug)]
pub struct Display {
    egl_version: EGLVersion,
    display_handle: Arc<DisplayHandle>,
}


impl Display {
    fn new(display_id: ffi::types::EGLNativeDisplayType) -> Result<Display, DisplayCreationError> {
        let raw_display = unsafe {
            ffi::GetDisplay(display_id)
        };

        if raw_display == ffi::NO_DISPLAY {
            return Err(DisplayCreationError::NoMatchingDisplay)
        }

        let mut version_major = 0;
        let mut version_minor = 0;

        let result = unsafe {
            ffi::Initialize(raw_display, &mut version_major, &mut version_minor)
        };

        if result == ffi::FALSE {
            return Err(DisplayCreationError::EGLInitializationError);
        }

        let version = EGLVersion::parse(version_major, version_minor);

        match version {
            Some(version) => {
                Ok(Display {
                    egl_version: version,
                    display_handle: DisplayHandle::new_in_arc(raw_display),
                })
            },
            None => {
                let display = Display {
                    egl_version: EGLVersion::EGL_1_4,
                    display_handle: DisplayHandle::new_in_arc(raw_display),
                };

                drop(display);

                Err(DisplayCreationError::EGLVersionUnsupported)
            }
        }
    }

    pub fn default_display() -> Result<Display, DisplayCreationError> {
        Display::new(ffi::DEFAULT_DISPLAY)
    }

    pub fn from_native_display_type(display_id: ffi::types::EGLNativeDisplayType) -> Result<Display, DisplayCreationError> {
        Display::new(display_id)
    }

    pub fn raw(&self) -> ffi::types::EGLDisplay {
        self.display_handle.raw()
    }

    pub fn version(&self) -> EGLVersion {
        self.egl_version
    }

    pub fn client_apis(&self) -> Result<Cow<str>, ()> {
        self.query_string(ffi::CLIENT_APIS as EGLint)
    }

    pub fn extensions(&self) -> Result<Cow<str>, ()> {
        self.query_string(ffi::EXTENSIONS as EGLint)
    }

    pub fn vendor(&self) -> Result<Cow<str>, ()> {
        self.query_string(ffi::VENDOR as EGLint)
    }

    pub fn version_string(&self) -> Result<Cow<str>, ()> {
        self.query_string(ffi::VERSION as EGLint)
    }

    fn query_string(&self, name: EGLint) -> Result<Cow<str>, ()> {
        unsafe {
            let ptr = ffi::QueryString(self.raw(), name);

            if ptr.is_null() {
                return Err(());
            }

            let cstr = CStr::from_ptr(ptr);

            Ok(cstr.to_string_lossy())
        }
    }

    pub fn configs<'a>(&'a self) -> Result<Configs<'a>, ()> {
        let buf_config_count = self.config_count();
        let mut vec: Vec<ffi::types::EGLConfig> = Vec::with_capacity(buf_config_count as usize);

        let mut new_count = 0;

        unsafe {
            let result = ffi::GetConfigs(self.raw(), vec.as_mut_slice().as_mut_ptr(), buf_config_count, &mut new_count);

            if result == ffi::FALSE {
                return Err(());
            }

            if new_count < 0 || buf_config_count < new_count {
                return Err(());
            }

            vec.set_len(new_count as usize);
        }

        Ok(Configs::new(self, vec))
    }

    fn config_count(&self) -> EGLint {
        let mut count = 0;

        unsafe {
            let result = ffi::GetConfigs(self.raw(), ptr::null_mut(), 0, &mut count);

            if result == ffi::FALSE {
                return 0;
            }
        }

        if count >= 0 {
            return count;
        } else {
            return 0;
        }
    }

    pub fn config_search_options_builder(&self) -> ConfigSearchOptionsBuilder {
        ConfigSearchOptionsBuilder::new(self.egl_version)
    }

    pub fn config_search<'a>(&'a self, options: ConfigSearchOptions) -> Result<Configs<'a>, ()> {
        let mut count = 0;

        unsafe {
            let result = ffi::ChooseConfig(self.raw(), options.attribute_list().ptr(), ptr::null_mut(), 0, &mut count);

            if result == ffi::FALSE {
                return Err(());
            }
        }

        if count < 0 {
            return Err(());
        } else if count == 0 {
            return Ok(Configs::new(self, Vec::new()));
        }

        let mut vec: Vec<ffi::types::EGLConfig> = Vec::with_capacity(count as usize);

        let mut new_count = 0;

        unsafe {
            let result = ffi::ChooseConfig(
                self.raw(),
                options.attribute_list().ptr(),
                vec.as_mut_slice().as_mut_ptr(),
                count,
                &mut new_count
            );

            if result == ffi::FALSE {
                return Err(());
            }
        }

        if count != new_count {
            return Err(());
        }

        unsafe {
            vec.set_len(new_count as usize);
        }

        Ok(Configs::new(self, vec))
    }

    pub fn opengl_context(self, config: ConfigOpenGL) -> Result<SingleContext<OpenGLContext>, DisplayError<Option<EGLError>>> {
        SingleContext::opengl_context(config.into_display_config()).map_err(|e| DisplayError::new(self, e))
    }

    pub(crate) fn display_handle(&self) -> &Arc<DisplayHandle> {
        &self.display_handle
    }
}

/// Return ownership of Display back if error occurred.
#[derive(Debug)]
pub struct DisplayError<E> {
    pub display: Display,
    pub error: E,
}

impl <E> DisplayError<E>  {
    fn new(display: Display, error: E) -> DisplayError<E> {
        DisplayError {
            display,
            error,
        }
    }
}
