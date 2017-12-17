


pub mod gl;
pub mod gles;
pub mod vg;


use egl_sys::ffi;

use error::EGLError;
use surface::Surface;

/// Handle multiple contexts.
pub struct ContextManager {

}


/// Create only one `SingleContext` per Display
#[derive(Debug)]
pub struct SingleContext<C: Context> {
    context: C,
}

impl <C: Context> SingleContext<C> {
    pub(crate) fn new(context: C) -> SingleContext<C> {
        SingleContext {
            context,
        }
    }
}

impl <C: Context> Context for SingleContext<C> {
    fn raw_display(&self) -> ffi::types::EGLDisplay {
        self.context.raw_display()
    }

    fn raw_context(&self) -> ffi::types::EGLContext {
        self.context.raw_context()
    }
}

impl <S: Surface, C: Context + MakeCurrentSurfaceAndContext<S>> MakeCurrentSurfaceAndContext<S> for SingleContext<C> {}

pub(crate) trait RawContextUtils: Context {
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
}


pub trait Context: Sized {
    fn raw_display(&self) -> ffi::types::EGLDisplay;
    fn raw_context(&self) -> ffi::types::EGLContext;
}

pub trait MakeCurrentSurfaceAndContext<S: Surface>: Context {
    fn make_current(self, surface: S) -> Result<CurrentSurfaceAndContext<S, Self>, MakeCurrentError<S, Self, Option<EGLError>>> {
        let result = unsafe {
            ffi::MakeCurrent(self.raw_display(), surface.raw_surface(), surface.raw_surface(), self.raw_context())
        };

        if result == ffi::TRUE {
            Ok(CurrentSurfaceAndContext {
                surface,
                context: self,
            })
        } else {
            Err(MakeCurrentError::new(surface, self, EGLError::check_errors()))
        }
    }
}

pub struct CurrentSurfaceAndContext<S: Surface, C: Context> {
    surface: S,
    context: C,
}

impl <S: Surface, C: Context> CurrentSurfaceAndContext<S, C> {
    pub fn swap_buffers(&mut self) -> Result<(), Option<EGLError>> {
        let result = unsafe {
            ffi::SwapBuffers(self.context.raw_display(), self.surface.raw_surface())
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

/// Return ownership of context and surface if there is an error.
#[derive(Debug)]
pub struct MakeCurrentError<S: Surface, C: Context, E> {
    pub surface: S,
    pub context: C,
    pub error: E,
}

impl <S: Surface, C: Context, E> MakeCurrentError<S, C, E> {
    fn new(surface: S, context: C, error: E) -> MakeCurrentError<S, C, E> {
        MakeCurrentError {
            surface,
            context,
            error,
        }
    }
}