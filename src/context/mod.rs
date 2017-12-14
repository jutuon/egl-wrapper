


pub mod gl;
pub mod gles;
pub mod vg;

use egl_sys::ffi;

use config::DisplayConfig;
use error::EGLError;
use surface::Surface;

/// Handle multiple contexts.
pub struct ContextManager {

}


/// Create only one `SingleContext` per Display
pub struct SingleContext<T: Context> {
    context: T,
}

impl <T: Context> SingleContext<T> {
    pub(crate) fn create(config: DisplayConfig) -> Result<SingleContext<T>, Option<EGLError>> {
        Ok(SingleContext {
            context: T::create(config)?,
        })
    }

    pub fn context(&self) -> &T {
        &self.context
    }
}


pub trait Context: Drop + Sized {
    const API_TYPE: ffi::types::EGLenum;

    fn bind_api() -> Result<(), Option<EGLError>> {
        let result = unsafe {
            ffi::BindAPI(Self::API_TYPE)
        };

        if result == ffi::TRUE {
            Ok(())
        } else {
            Err(EGLError::check_errors())
        }
    }

    /// This function calls `bind_api` before creating the context.
    fn create(DisplayConfig) -> Result<Self, Option<EGLError>>;

    fn raw_display(&self) -> ffi::types::EGLDisplay;

    fn raw(&self) -> ffi::types::EGLContext;
}

pub trait CurrentContext<T: Surface>: Context {
    fn make_current(&self, surface: &T) -> Result<(), Option<EGLError>> {
        let result = unsafe {
            ffi::MakeCurrent(self.raw_display(), surface.raw(), surface.raw(), self.raw())
        };

        if result == ffi::TRUE {
            Ok(())
        } else {
            Err(EGLError::check_errors())
        }
    }
}


pub(self) fn destroy_context(raw_display: ffi::types::EGLDisplay, raw_context: ffi::types::EGLContext) {
    let result = unsafe {
        ffi::DestroyContext(raw_display, raw_context)
    };

    if result == ffi::FALSE {
        eprintln!("egl_wrapper: couldn't destroy context");
    }

    // TODO: eglReleaseThread
}