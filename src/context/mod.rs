


pub mod gl;
pub mod gles;
pub mod vg;
pub mod attribute;

use egl_sys::ffi;

use error::EGLError;
use surface::Surface;
use surface::attribute::RenderBuffer;
use utils::{ UnsignedInteger, QueryResult, QueryError};
use display::{ DisplayType };

/// Create only one `SingleContext` per Display
#[derive(Debug)]
pub struct SingleContext<C: Context, D: DisplayType> {
    display: D,
    context: C,
}

impl <C: Context, D: DisplayType> SingleContext<C, D> {
    pub(crate) fn new(context: C, display: D) -> Self {
        SingleContext {
            display,
            context,
        }
    }

    pub fn context(&self) -> &C {
        &self.context
    }

    pub fn display(&self) -> &D {
        &self.display
    }

    pub fn destroy(self) -> D {
        self.display
    }

    // TODO: check that surface will match with context before MakeCurrent function call

    /// This method call also completes deletion of previously dropped Contexts and Surfaces.
    pub fn make_current<S: Surface>(self, surface: S) -> Result<CurrentSurfaceAndContext<S, C, D>, ContextOrSurfaceError<S, C, D>> {
        let result = unsafe {
            ffi::MakeCurrent(self.context.raw_display(), surface.raw_surface(), surface.raw_surface(), self.context.raw_context())
        };

        if result == ffi::TRUE {
            Ok(CurrentSurfaceAndContext {
                surface,
                context: self,
            })
        } else {
            let error = EGLError::check_errors();

            match error {
                Some(EGLError::ContextLost)     => Err(ContextOrSurfaceError::ContextLost(self.display, surface)),
                Some(EGLError::BadNativeWindow) => Err(ContextOrSurfaceError::BadNativeWindow(self)),
                other_error                     => Err(ContextOrSurfaceError::OtherError(self.display, other_error)),
            }
        }
    }
}

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


pub struct CurrentSurfaceAndContext<S: Surface, C: Context, D: DisplayType> {
    surface: S,
    context: SingleContext<C, D>,
}

impl <S: Surface, C: Context, D: DisplayType> CurrentSurfaceAndContext<S, C, D> {
    pub fn swap_buffers(self) -> Result<Self, ContextOrSurfaceError<S, C, D>> {
        let result = unsafe {
            ffi::SwapBuffers(self.context.context().raw_display(), self.surface.raw_surface())
        };

        if result == ffi::TRUE {
            Ok(self)
        } else {
            let error = EGLError::check_errors();

            match error {
                Some(EGLError::ContextLost)     => Err(ContextOrSurfaceError::ContextLost(self.context.display, self.surface)),
                Some(EGLError::BadNativeWindow) => Err(ContextOrSurfaceError::BadNativeWindow(self.context)),
                other_error                     => Err(ContextOrSurfaceError::OtherError(self.context.display, other_error)),
            }
        }
    }

    /// Default value: 1
    ///
    /// Interval value will be clamped between min and max value defined by Config.
    pub fn swap_interval(&mut self, interval: UnsignedInteger) -> Result<(), Option<EGLError>> {
        let result = unsafe {
            ffi::SwapInterval(self.context.context().raw_display(), interval.value())
        };

        if result == ffi::TRUE {
            Ok(())
        } else {
            Err(EGLError::check_errors())
        }

    }

    pub fn context(&self) -> &SingleContext<C, D> {
        &self.context
    }
}

impl <S: Surface, C: Context + attribute::ContextAttributeUtils, D: DisplayType> CurrentSurfaceAndContext<S, C, D> {
    pub fn render_buffer(&self) -> QueryResult<RenderBuffer> {
        let value = self.context.context().query_attribute(attribute::QueryableAttribute::RenderBuffer)?;

        match value as ffi::types::EGLenum {
            ffi::BACK_BUFFER   => Ok(RenderBuffer::BackBuffer),
            ffi::SINGLE_BUFFER => Ok(RenderBuffer::SingleBuffer),
            _ => Err(QueryError::EnumError),
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
}

#[derive(Debug)]
pub enum ContextOrSurfaceError<S: Surface, C: Context, D: DisplayType> {
    ContextLost(D, S),
    BadNativeWindow(SingleContext<C, D>),
    OtherError(D, Option<EGLError>),
}


// TODO: extension KHR_create_context current contexts without default framebuffer