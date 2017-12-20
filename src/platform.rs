

//! Types to support different EGL platform extensions.

use std::os::raw::c_void;

use egl_sys::ffi::types::{ EGLenum, EGLint, EGLDisplay };
use egl_sys::ffi;
use egl_sys::extensions;

use display::{ Display, DisplayHandle, DisplayCreationError };
use utils::AttributeListTrait;
use utils;

use surface::window::WindowSurface;
use surface::pixmap::PixmapSurface;

pub trait PlatformDisplay: Sized {

}

pub trait PlatformWindow: PlatformDisplay {
    type NativeWindow: RawNativeWindow;

    fn get_platform_window_surface(&self, &Display<Self>) -> Result<WindowSurface, ()>;
}

/*
pub trait PlatformPixmap: PlatformDisplay {
    type NativePixmap;

    fn get_platform_pixmap_surface(&self, &Display<Self>) -> Result<PixmapSurface, ()>;
}
*/
/*
pub struct EXTPlatformWayland;

impl Platform for EXTPlatformWayland {

}
*/


/// EGL 1.4 default platform
pub struct DefaultPlatform;

impl DefaultPlatform {
    pub fn get_display(native_display: ffi::types::NativeDisplayType) -> Result<Display<Self>, DisplayCreationError> {
        let raw_display = unsafe {
           ffi::GetDisplay(native_display.raw_native_display())
        };

        if raw_display == ffi::NO_DISPLAY {
            return Err(DisplayCreationError::NoMatchingDisplay)
        }

        Ok(Display::new(raw_display, DefaultPlatform)?)
    }
}

impl PlatformDisplay for DefaultPlatform {}

/*
impl PlatformWindow for DefaultPlatform {
    type NativeWindow = ffi::types::NativeWindowType;

    fn get_platform_window_surface(&self, display: &DisplayHandle) -> Result<WindowSurface, ()> {
        unimplemented!()
    }
}
*/

/*

pub struct EXTPlatformX11;

impl PlatformDisplay for EXTPlatformX11 {
    type AttributeList = utils::AttributeList;
    type NativeDisplay = ffi::types::NativeDisplayType;

    const PLATFORM_TYPE: EGLenum = extensions::PLATFORM_X11_EXT;

    fn get_display(native_display: Self::NativeDisplay, attribute_list: &Self::AttributeList) -> Result<Display<Self>, ()> {
        let raw_display = unsafe {
            extensions::GetPlatformDisplayEXT(Self::PLATFORM_TYPE, native_display.raw_native_display(), attribute_list.attribute_list_ptr())
        };

        if raw_display == ffi::NO_DISPLAY {
            return Err(());
        }

        Ok(())
    }
}

impl PlatformWindow for EXTPlatformX11 {
    type NativeWindow = ffi::types::NativeWindowType;

    fn get_platform_window_surface(&self, display: &DisplayHandle) -> Result<WindowSurface, ()> {
        unimplemented!()
    }
}
*/


pub trait RawNativeDisplay {
    type T;

    fn raw_native_display(&self) -> Self::T;
}

pub trait RawNativeWindow {
    type T;

    fn raw_native_window(&self) -> Self::T;
}

pub trait RawNativePixmap {
    type T;

    fn raw_native_pixmap(&self) -> Self::T;
}

impl RawNativeDisplay for ffi::types::NativeDisplayType {
    type T = Self;

    fn raw_native_display(&self) -> Self::T {
        *self
    }
}

impl RawNativeWindow for ffi::types::NativeWindowType {
    type T = Self;

    fn raw_native_window(&self) -> Self::T {
        *self
    }
}
