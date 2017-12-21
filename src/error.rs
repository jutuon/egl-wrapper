use egl_sys::ffi::types::EGLenum;
use egl_sys::ffi;

#[derive(Debug)]
pub enum EGLError {
    NotInitialized,
    BadAccess,
    BadAlloc,
    BadAttribute,
    BadContext,
    BadConfig,
    BadCurrentSurface,
    BadDisplay,
    BadSurface,
    BadMatch,
    BadParameter,
    BadNativePixmap,
    BadNativeWindow,
    ContextLost,
    UnknownError,
}

impl EGLError {
    /// Returns `Some(error)` if there is an error.
    pub(crate) fn check_errors() -> Option<EGLError> {
        let result = unsafe { ffi::GetError() };

        if result < 0 {
            eprintln!("egl_wrapper: unknown EGL error value: {}", result);
            return Some(EGLError::UnknownError);
        }

        let result = result as EGLenum;

        #[cfg_attr(rustfmt, rustfmt_skip)]
        let error = match result {
            // TODO: panic if there is no errors?
            ffi::SUCCESS             => return None,
            ffi::CONTEXT_LOST        => EGLError::ContextLost,
            ffi::NOT_INITIALIZED     => EGLError::NotInitialized,
            ffi::BAD_ACCESS          => EGLError::BadAccess,
            ffi::BAD_ALLOC           => EGLError::BadAlloc,
            ffi::BAD_ATTRIBUTE       => EGLError::BadAttribute,
            ffi::BAD_CONTEXT         => EGLError::BadContext,
            ffi::BAD_CONFIG          => EGLError::BadConfig,
            ffi::BAD_CURRENT_SURFACE => EGLError::BadCurrentSurface,
            ffi::BAD_DISPLAY         => EGLError::BadDisplay,
            ffi::BAD_SURFACE         => EGLError::BadSurface,
            ffi::BAD_MATCH           => EGLError::BadMatch,
            ffi::BAD_PARAMETER       => EGLError::BadParameter,
            ffi::BAD_NATIVE_PIXMAP   => EGLError::BadNativePixmap,
            ffi::BAD_NATIVE_WINDOW   => EGLError::BadNativeWindow,
            unknown_error  => {
                eprintln!("egl_wrapper: unknown EGL error value: {}", unknown_error);
                EGLError::UnknownError
            }
        };

        return Some(error);
    }
}
