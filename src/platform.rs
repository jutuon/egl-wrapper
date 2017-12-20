

//! Types to support different EGL platform extensions.

use std::os::raw::c_void;

use x11;

use egl_sys::ffi::types::{ EGLenum, EGLint, EGLDisplay };
use egl_sys::ffi;
use egl_sys::extensions;

use display::{ Display, DisplayCreationError };
use utils::{ AttributeListBuilder };

use surface::window::{ WindowSurface, WindowSurfaceAttributeList };

use error::EGLError;

use config::client_api::*;

pub trait PlatformDisplay: Sized {}

/*
pub struct EXTPlatformWayland;

impl Platform for EXTPlatformWayland {

}
*/


/// EGL 1.4 default platform
pub struct DefaultPlatform<T: RawNativeDisplay<T=ffi::types::NativeDisplayType> + RawNativeWindow<T=ffi::types::NativeWindowType>> {
    native: T,
}

impl <T: RawNativeDisplay<T=ffi::types::NativeDisplayType> + RawNativeWindow<T=ffi::types::NativeWindowType>> DefaultPlatform<T> {
    pub(crate) fn get_display(native_display: T) -> Result<Display<Self>, DisplayCreationError> {
        let raw_display = unsafe {
           ffi::GetDisplay(native_display.raw_native_display())
        };

        if raw_display == ffi::NO_DISPLAY {
            return Err(DisplayCreationError::NoMatchingDisplay)
        }

        let platform = DefaultPlatform {
            native: native_display,
        };

        Ok(Display::new(raw_display, platform)?)
    }

    pub fn get_platform_window_surface(&self, config_window: ConfigWindow, attribute_list: WindowSurfaceAttributeList) -> Result<WindowSurface, WindowCreationError> {
        let raw_native_window = if let Some(native) = self.native.raw_native_window() {
            native
        } else {
            return Err(WindowCreationError::NativeWindowNotFound);
        };

        let raw_surface = unsafe {
            ffi::CreateWindowSurface(config_window.display_config().raw_display(), config_window.display_config().raw_config(), raw_native_window, attribute_list.ptr())
        };

        if raw_surface == ffi::NO_SURFACE {
            return Err(WindowCreationError::EGLError(EGLError::check_errors()));
        }

        Ok(WindowSurface::new(config_window, raw_surface))
    }

    pub fn native_mut(&mut self) -> &mut T {
        &mut self.native
    }

    pub fn native(&self) -> &T {
        &self.native
    }
}

impl <T: RawNativeDisplay<T=ffi::types::NativeDisplayType> + RawNativeWindow<T=ffi::types::NativeWindowType>> PlatformDisplay for DefaultPlatform<T> {}


// Extension EGL_EXT_platform_x11 support

pub struct EXTPlatformX11<T: RawNativeDisplay<T=*mut x11::xlib::Display> + RawNativeWindow<T=x11::xlib::Window>> {
    x11: T,
}

impl <T: RawNativeDisplay<T=*mut x11::xlib::Display> + RawNativeWindow<T=x11::xlib::Window>> EXTPlatformX11<T> {
    pub(crate) fn get_display(native_display: T, attribute_list: EXTPlatformX11AttributeListBuilder) -> Result<Display<Self>, DisplayCreationError> {
        let attribute_list = attribute_list.0.build();

        let raw_display = unsafe {
            extensions::GetPlatformDisplayEXT(extensions::PLATFORM_X11_EXT, native_display.raw_native_display() as *mut c_void, attribute_list.ptr())
        };

        if raw_display == ffi::NO_DISPLAY {
            return Err(DisplayCreationError::NoMatchingDisplay);
        }

        let x11_platform = EXTPlatformX11 {
            x11: native_display,
        };

        Ok(Display::new(raw_display, x11_platform)?)
    }

    pub fn get_platform_window_surface(&self, config_window: ConfigWindow, attribute_list: WindowSurfaceAttributeList) -> Result<WindowSurface, WindowCreationError> {
        let raw_native_window = if let Some(native) = self.x11.raw_native_window() {
            native
        } else {
            return Err(WindowCreationError::NativeWindowNotFound);
        };

        let raw_surface = unsafe {
            extensions::CreatePlatformWindowSurfaceEXT(config_window.display_config().raw_display(), config_window.display_config().raw_config(), raw_native_window as *mut c_void, attribute_list.ptr())
        };

        if raw_surface == ffi::NO_SURFACE {
            return Err(WindowCreationError::EGLError(EGLError::check_errors()));
        }

        Ok(WindowSurface::new(config_window, raw_surface))
    }

    pub fn x11_mut(&mut self) -> &mut T {
        &mut self.x11
    }

    pub fn x11(&self) -> &T {
        &self.x11
    }
}

pub struct EXTPlatformX11AttributeListBuilder(AttributeListBuilder);

impl EXTPlatformX11AttributeListBuilder {
    pub fn new() -> EXTPlatformX11AttributeListBuilder {
        EXTPlatformX11AttributeListBuilder(AttributeListBuilder::new())
    }
}


impl <T: RawNativeDisplay<T=*mut x11::xlib::Display> + RawNativeWindow<T=x11::xlib::Window>> PlatformDisplay for EXTPlatformX11<T> {}

pub unsafe trait RawNativeDisplay {
    type T;

    fn raw_native_display(&self) -> Self::T;
}

pub unsafe trait RawNativeWindow {
    type T;

    fn raw_native_window(&self) -> Option<Self::T>;
}


pub enum WindowCreationError {
    NativeWindowNotFound,
    EGLError(Option<EGLError>),
}