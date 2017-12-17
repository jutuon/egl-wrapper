
pub mod window;
pub mod pbuffer;
pub mod pixmap;
pub mod attribute;

use egl_sys::ffi;

use config::DisplayConfig;
use error::EGLError;

pub trait Surface {
    fn raw_surface(&self) -> ffi::types::EGLSurface;
    fn display_config(&self) -> &DisplayConfig;
}

fn destroy_surface<S: Surface>(surface: &mut S) {
    let result = unsafe {
        ffi::DestroySurface(surface.display_config().raw_display(), surface.raw_surface())
    };

    if result == ffi::FALSE {
        let error = EGLError::check_errors();
        eprintln!("egl_wrapper: couldn't destroy surface, error: {:?}", error);
    }
}