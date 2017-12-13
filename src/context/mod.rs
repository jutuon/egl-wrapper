


pub mod gl;
pub mod gles;
pub mod vg;

use egl_sys::ffi;

use config::Config;
use display::Display;
use error::EGLError;
use surface::Surface;

use self::gl::OpenGLContext;

/// Handle multiple contexts.
pub struct ContextManager {

}


/// Create only one `SingleContext` per Display
pub struct SingleContext<'a, T: Context<'a>> {
    display: &'a Display,
    context: T,
}

impl <'a, T: Context<'a>> SingleContext<'a, T> {
    pub(crate) fn create(config: &'a Config<'a>) -> Result<SingleContext<'a, T>, Option<EGLError>> {
        Ok(SingleContext {
            display: config.display(),
            context: T::create(config)?,
        })
    }

    pub fn context(&self) -> &T {
        &self.context
    }
}


pub trait Context<'a>: Drop + Sized {
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
    fn create(&'a Config<'a>) -> Result<Self, Option<EGLError>>;

    fn display(&self) -> &Display;

    fn raw(&self) -> ffi::types::EGLContext;
}

pub trait CurrentContext<'a, T: Surface>: Context<'a> {
    fn make_current(&self, surface: &T) -> Result<(), Option<EGLError>> {
        let result = unsafe {
            ffi::MakeCurrent(self.display().raw(), surface.raw(), surface.raw(), self.raw())
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