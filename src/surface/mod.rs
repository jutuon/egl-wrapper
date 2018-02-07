pub mod window;
//pub mod pbuffer;
//pub mod pixmap;
pub mod attribute;

use egl_sys::ffi;

use EGLHandle;
use error::EGLError;

pub trait Surface {
    fn raw_surface(&self) -> ffi::types::EGLSurface;
    fn raw_display(&self) -> ffi::types::EGLDisplay;
    fn egl_handle(&self) -> &EGLHandle;
}

fn destroy_surface<S: Surface>(surface: &mut S) {
    let result =
        unsafe { egl_function!(surface.egl_handle(), DestroySurface(surface.raw_display(), surface.raw_surface())) };

    if result == ffi::FALSE {
        let error = EGLError::check_errors(surface.egl_handle());
        eprintln!("egl_wrapper: couldn't destroy surface, error: {:?}", error);
    }
}
