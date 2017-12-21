use std::ffi::{CStr, CString, NulError};
use std::ptr;
use std::borrow::Cow;
use std::marker::PhantomData;
use std::sync::Arc;
use std::mem;
use std::os;

use egl_sys::ffi;
use egl_sys::ffi::types::EGLint;

use config::Configs;
use config::search::{ConfigSearchOptions, ConfigSearchOptionsBuilder};
use context::gl::{OpenGLContext, OpenGLContextBuilder, OpenGLContextBuilderEXT};
use context::gles::{OpenGLESContext, OpenGLESContextBuilder, OpenGLESContextBuilderEXT};
use context::SingleContext;
use error::EGLError;
use platform::PlatformDisplay;

#[derive(Debug, Clone)]
pub struct DisplayExtensionSupport {
    get_all_proc_addresses: bool,
    create_context: bool,
}

impl DisplayExtensionSupport {
    fn new() -> DisplayExtensionSupport {
        DisplayExtensionSupport {
            get_all_proc_addresses: false,
            create_context: false,
        }
    }

    fn parse(extensions: &str) -> DisplayExtensionSupport {
        let mut extension_support = DisplayExtensionSupport::new();

        for ext in extensions.split_whitespace() {
            match ext {
                "EGL_KHR_get_all_proc_addresses" => extension_support.get_all_proc_addresses = true,
                "EGL_KHR_create_context" => extension_support.create_context = true,
                _ => (),
            }
        }

        extension_support
    }

    pub fn create_context(&self) -> bool {
        self.create_context
    }
}

#[derive(Debug)]
pub struct ClientApiSupport {
    pub opengl: bool,
    pub opengl_es: bool,
    pub openvg: bool,
}

impl ClientApiSupport {
    fn new() -> ClientApiSupport {
        ClientApiSupport {
            opengl: false,
            opengl_es: false,
            openvg: false,
        }
    }
    fn parse(client_apis: &str) -> ClientApiSupport {
        let mut api_support = ClientApiSupport::new();

        for api in client_apis.split_whitespace() {
            match api {
                "OpenGL" => api_support.opengl = true,
                "OpenGL_ES" => api_support.opengl_es = true,
                "OpenVG" => api_support.openvg = true,
                _ => (),
            }
        }

        api_support
    }
}

#[derive(Debug)]
pub enum DisplayCreationError {
    NoMatchingDisplay,
    EGLInitializationError,
    EGLVersionUnsupported,
    PlatformExtensionNotSupported,
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
                _ => None,
            },
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct DisplayHandle {
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

    pub fn raw_display(&self) -> ffi::types::EGLDisplay {
        self.raw_display
    }
}

impl Drop for DisplayHandle {
    fn drop(&mut self) {
        let result = unsafe { ffi::Terminate(self.raw_display) };

        if result == ffi::FALSE {
            eprintln!("egl_wrapper: eglTerminate returned false");
        }

        let result = unsafe { ffi::ReleaseThread() };

        if result == ffi::FALSE {
            eprintln!("egl_wrapper: eglReleaseThread returned false");
        }
    }
}

// TODO: multiple calls to GetDisplay will return same EGLDisplay handle

/// EGLDisplay with initialized EGL
#[derive(Debug)]
pub struct Display<P: PlatformDisplay> {
    platform: P,
    extension_support: DisplayExtensionSupport,
    egl_version: EGLVersion,
    display_handle: Arc<DisplayHandle>,
}

impl<P: PlatformDisplay> Display<P> {
    pub(crate) fn new(
        raw_display: ffi::types::EGLDisplay,
        platform: P,
    ) -> Result<Display<P>, DisplayCreationError> {
        let mut version_major = 0;
        let mut version_minor = 0;

        let result =
            unsafe { ffi::Initialize(raw_display, &mut version_major, &mut version_minor) };

        if result == ffi::FALSE {
            return Err(DisplayCreationError::EGLInitializationError);
        }

        let version = EGLVersion::parse(version_major, version_minor);
        let extension_support = DisplayExtensionSupport::new();

        match version {
            Some(version) => {
                let mut display = Display {
                    platform,
                    extension_support,
                    egl_version: version,
                    display_handle: DisplayHandle::new_in_arc(raw_display),
                };

                let parsed_extensions = match display.extensions() {
                    Ok(text) => Some(DisplayExtensionSupport::parse(&text)),
                    Err(()) => None,
                };

                if let Some(ext) = parsed_extensions {
                    mem::replace(&mut display.extension_support, ext);
                }

                Ok(display)
            }
            None => {
                // Could not parse version so lets just destroy EGLDisplay and
                // return error.

                let display = Display {
                    platform,
                    extension_support,
                    egl_version: EGLVersion::EGL_1_4,
                    display_handle: DisplayHandle::new_in_arc(raw_display),
                };

                drop(display);

                Err(DisplayCreationError::EGLVersionUnsupported)
            }
        }
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
            let ptr = ffi::QueryString(self.display_handle().raw_display(), name);

            if ptr.is_null() {
                return Err(());
            }

            let cstr = CStr::from_ptr(ptr);

            Ok(cstr.to_string_lossy())
        }
    }

    pub fn configs<'a>(&'a self) -> Result<Configs<'a, Self>, ()> {
        let buf_config_count = self.config_count();
        let mut vec: Vec<ffi::types::EGLConfig> = Vec::with_capacity(buf_config_count as usize);

        let mut new_count = 0;

        unsafe {
            let result = ffi::GetConfigs(
                self.display_handle().raw_display(),
                vec.as_mut_slice().as_mut_ptr(),
                buf_config_count,
                &mut new_count,
            );

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
            let result = ffi::GetConfigs(
                self.display_handle().raw_display(),
                ptr::null_mut(),
                0,
                &mut count,
            );

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
        ConfigSearchOptionsBuilder::new(self.egl_version, self.extension_support.clone())
    }

    pub fn config_search<'a>(
        &'a self,
        options: ConfigSearchOptions,
    ) -> Result<Configs<'a, Self>, ()> {
        let mut count = 0;

        unsafe {
            let result = ffi::ChooseConfig(
                self.display_handle().raw_display(),
                options.attribute_list().ptr(),
                ptr::null_mut(),
                0,
                &mut count,
            );

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
                self.display_handle().raw_display(),
                options.attribute_list().ptr(),
                vec.as_mut_slice().as_mut_ptr(),
                count,
                &mut new_count,
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

    pub fn build_opengl_context(
        self,
        builder: OpenGLContextBuilder,
    ) -> Result<SingleContext<OpenGLContext, Self>, DisplayError<P, Option<EGLError>>> {
        match builder.build() {
            Ok(context) => Ok(SingleContext::new(context, self)),
            Err(error) => Err(DisplayError::new(self, error)),
        }
    }

    /// Extension EGL_KHR_create_context
    pub fn build_opengl_context_ext(
        self,
        builder: OpenGLContextBuilderEXT,
    ) -> Result<SingleContext<OpenGLContext, Self>, DisplayError<P, Option<EGLError>>> {
        match builder.build() {
            Ok(context) => Ok(SingleContext::new(context, self)),
            Err(error) => Err(DisplayError::new(self, error)),
        }
    }

    pub fn build_opengl_es_context(
        self,
        builder: OpenGLESContextBuilder,
    ) -> Result<SingleContext<OpenGLESContext, Self>, DisplayError<P, Option<EGLError>>> {
        match builder.build() {
            Ok(context) => Ok(SingleContext::new(context, self)),
            Err(error) => Err(DisplayError::new(self, error)),
        }
    }

    /// Extension EGL_KHR_create_context
    pub fn build_opengl_es_context_ext(
        self,
        builder: OpenGLESContextBuilderEXT,
    ) -> Result<SingleContext<OpenGLESContext, Self>, DisplayError<P, Option<EGLError>>> {
        match builder.build() {
            Ok(context) => Ok(SingleContext::new(context, self)),
            Err(error) => Err(DisplayError::new(self, error)),
        }
    }

    pub(crate) fn display_handle(&self) -> &Arc<DisplayHandle> {
        &self.display_handle
    }

    pub fn extension_function_loader(&self) -> ExtensionFunctionLoader<P> {
        ExtensionFunctionLoader { _display: self }
    }

    /// Returns `Some(function_loader)` if EGL extension
    /// `EGL_KHR_get_all_proc_addresses` is supported.
    pub fn function_loader(&self) -> Option<FunctionLoader<P>> {
        match self.extension_support.get_all_proc_addresses {
            true => Some(FunctionLoader { _display: self }),
            false => None,
        }
    }

    pub fn client_api_support(&self) -> Result<ClientApiSupport, ()> {
        Ok(ClientApiSupport::parse(&self.client_apis()?))
    }

    pub fn platform_display(&self) -> &P {
        &self.platform
    }

    pub fn platform_display_mut(&mut self) -> &mut P {
        &mut self.platform
    }
}

/// Return ownership of Display back if error occurred.
#[derive(Debug)]
pub struct DisplayError<P: PlatformDisplay, E> {
    pub display: Display<P>,
    pub error: E,
}

impl<P: PlatformDisplay, E> DisplayError<P, E> {
    fn new(display: Display<P>, error: E) -> DisplayError<P, E> {
        DisplayError { display, error }
    }
}

/// Load client API and EGL extension function pointers
pub struct ExtensionFunctionLoader<'a, P: PlatformDisplay + 'a> {
    _display: &'a Display<P>,
}

impl<'a, P: PlatformDisplay + 'a> ExtensionFunctionLoader<'a, P> {
    /// If null is returned the function does not exists.
    /// A non null value does not guarantee existence of the extension function.
    pub fn get_proc_address(&self, name: &str) -> Result<*const os::raw::c_void, NulError> {
        get_proc_address(name)
    }
}

/// Load client API and EGL function pointers.
/// Supports all functions, not only extensions functions.
pub struct FunctionLoader<'a, P: PlatformDisplay + 'a> {
    _display: &'a Display<P>,
}

impl<'a, P: PlatformDisplay + 'a> FunctionLoader<'a, P> {
    /// If null is returned the function does not exists.
    /// A non null value does not guarantee existence of the function.
    pub fn get_proc_address(&self, name: &str) -> Result<*const os::raw::c_void, NulError> {
        get_proc_address(name)
    }
}

pub(crate) fn get_proc_address(name: &str) -> Result<*const os::raw::c_void, NulError> {
    let c_string = match CString::new(name) {
        Ok(s) => s,
        Err(error) => return Err(error),
    };

    unsafe {
        Ok(ffi::GetProcAddress(c_string.as_ptr())
            as *const os::raw::c_void)
    }
}

impl<P: PlatformDisplay> DisplayType for Display<P> {
    fn display_handle(&self) -> &Arc<DisplayHandle> {
        &self.display_handle
    }

    fn egl_version(&self) -> EGLVersion {
        self.egl_version
    }

    fn display_extensions(&self) -> &DisplayExtensionSupport {
        &self.extension_support
    }
}

pub trait DisplayType {
    fn display_handle(&self) -> &Arc<DisplayHandle>;
    fn display_extensions(&self) -> &DisplayExtensionSupport;
    fn egl_version(&self) -> EGLVersion;
}
