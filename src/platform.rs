//! Types to support different EGL platform extensions.

use utils::AttributeList;
use std::os::raw::c_void;

use egl_sys::ffi::types::{EGLenum, EGLint, NativeDisplayType, NativeWindowType};
use egl_sys::ffi;
use egl_sys::extensions;

use display::{Display, DisplayCreationError};
use utils::AttributeListBuilder;

use surface::window::{WindowSurface, WindowSurfaceAttributeList};

use error::EGLError;

use config::client_api::*;

pub trait Platform: Sized {}

#[derive(Debug)]
/// EGL implementation default platform.
pub struct DefaultPlatform<T> {
    optional_native_display_handle: T,
}

impl<T> DefaultPlatform<T> {
    pub(crate) fn get_display(native_display: NativeDisplayType, optional_native_display_handle: T) -> Result<Display<Self>, DisplayCreationError> {
        let raw_display = unsafe { ffi::GetDisplay(native_display) };

        if raw_display == ffi::NO_DISPLAY {
            return Err(DisplayCreationError::NoMatchingDisplay);
        }

        let platform = DefaultPlatform {
            optional_native_display_handle,
        };

        Ok(Display::new(raw_display, platform)?)
    }

    pub unsafe fn get_platform_window_surface<W>(
        &self,
        optional_native_window_handle: W,
        raw_native_window: NativeWindowType,
        config_window: ConfigWindow<Self>,
        attribute_list: WindowSurfaceAttributeList,
    ) -> Result<WindowSurface<W, Self>, WindowCreationError> {
        let raw_surface = ffi::CreateWindowSurface(
            config_window.display_config().raw_display(),
            config_window.display_config().raw_config(),
            raw_native_window,
            attribute_list.ptr(),
        );

        if raw_surface == ffi::NO_SURFACE {
            return Err(WindowCreationError::EGLError(EGLError::check_errors()));
        }

        Ok(WindowSurface::new(optional_native_window_handle, config_window, raw_surface))
    }

    pub fn optional_native_display(&self) -> &T {
        &self.optional_native_display_handle
    }

    pub fn optional_native_display_mut(&mut self) -> &mut T {
        &mut self.optional_native_display_handle
    }
}

impl<T> Platform for DefaultPlatform<T> {}


#[derive(Debug)]
/// EGL extension EGL_EXT_platform_base platforms.
pub struct EXTPlatform<T> {
    optional_native_display_handle: T,
}

#[derive(Debug)]
#[repr(u32)]
pub enum EXTPlatformType {
    X11 = extensions::PLATFORM_X11_EXT,
    Wayland = extensions::PLATFORM_WAYLAND_EXT,
}


impl<T> EXTPlatform<T> {
    pub(crate) fn get_display(
        platform_type: EXTPlatformType,
        ptr_to_native_display: *mut c_void,
        optional_native_display_handle: T,
        attribute_list: EXTPlatformAttributeList,
    ) -> Result<Display<Self>, DisplayCreationError> {

        let raw_display = unsafe {
            extensions::GetPlatformDisplayEXT(
                platform_type as EGLenum,
                ptr_to_native_display,
                attribute_list.ptr(),
            )
        };

        if raw_display == ffi::NO_DISPLAY {
            return Err(DisplayCreationError::NoMatchingDisplay);
        }

        let platform = EXTPlatform {
            optional_native_display_handle,
        };

        Ok(Display::new(raw_display, platform)?)
    }

    pub unsafe fn get_platform_window_surface<W>(
        &self,
        optional_native_window_handle: W,
        raw_native_window: *mut c_void,
        config_window: ConfigWindow<Self>,
        attribute_list: WindowSurfaceAttributeList,
    ) -> Result<WindowSurface<W, Self>, WindowCreationError> {
        let raw_surface = extensions::CreatePlatformWindowSurfaceEXT(
            config_window.display_config().raw_display(),
            config_window.display_config().raw_config(),
            raw_native_window,
            attribute_list.ptr(),
        );

        if raw_surface == ffi::NO_SURFACE {
            return Err(WindowCreationError::EGLError(EGLError::check_errors()));
        }

        Ok(WindowSurface::new(optional_native_window_handle, config_window, raw_surface))
    }

    pub fn optional_native_display(&self) -> &T {
        &self.optional_native_display_handle
    }

    pub fn optional_native_display_mut(&mut self) -> &mut T {
        &mut self.optional_native_display_handle
    }
}

pub struct EXTPlatformAttributeListBuilder(AttributeListBuilder);

impl EXTPlatformAttributeListBuilder {
    pub fn new() -> Self {
        EXTPlatformAttributeListBuilder(AttributeListBuilder::new())
    }
}

pub struct EXTPlatformAttributeList(AttributeList);

impl EXTPlatformAttributeList {
    // TODO: X11 attribute
}

impl EXTPlatformAttributeList {
    pub fn ptr(&self) -> *const EGLint {
        self.0.ptr()
    }
}

impl Default for EXTPlatformAttributeList {
    fn default() -> Self {
        EXTPlatformAttributeList(AttributeList::empty())
    }
}

impl<T> Platform for EXTPlatform<T> {}

#[derive(Debug)]
pub enum WindowCreationError {
    NativeWindowNotFound,
    EGLError(Option<EGLError>),
}
