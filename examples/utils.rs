use gl;

use std::ffi::CStr;
use std::os::raw;

use egl_wrapper::platform::PlatformDisplay;
use egl_wrapper::display::Display;
use egl_wrapper::config::Configs;

pub fn print_opengl_info() {
    println!("OpenGL context information:");
    println!("  Version:  {:?}", get_version_string());
    println!("  Vendor:   {:?}", get_vendor_string());
    println!("  Renderer: {:?}", get_renderer_string());
}

/// Return OpenGL version string.
pub fn get_version_string<'a>() -> &'a CStr {
    unsafe {
        let ptr_to_str = gl::GetString(gl::VERSION) as *const raw::c_char;
        CStr::from_ptr(ptr_to_str)
    }
}

/// Return OpenGL vendor string.
pub fn get_vendor_string<'a>() -> &'a CStr {
    unsafe {
        let ptr_to_str = gl::GetString(gl::VENDOR) as *const raw::c_char;
        CStr::from_ptr(ptr_to_str)
    }
}

/// Return OpenGL renderer string.
pub fn get_renderer_string<'a>() -> &'a CStr {
    unsafe {
        let ptr_to_str = gl::GetString(gl::RENDERER) as *const raw::c_char;
        CStr::from_ptr(ptr_to_str)
    }
}

pub fn search_configs<'a, P: PlatformDisplay>(display: &'a Display<P>) -> Configs<'a, Display<P>> {
    use egl_wrapper::config::attribute::{ConfigClientAPI, SurfaceType};

    use egl_wrapper::config::search::UnsignedIntegerSearchAttributes;

    use egl_wrapper::utils::UnsignedInteger;

    let mut builder = display.config_search_options_builder();

    builder
        .add_unsigned_integer_attribute(
            UnsignedIntegerSearchAttributes::AlphaSize,
            Some(UnsignedInteger::new(8)),
        )
        .client_api_conformance(ConfigClientAPI::OPENGL | ConfigClientAPI::OPENGL_ES2)
        .client_api(ConfigClientAPI::OPENGL | ConfigClientAPI::OPENGL_ES2)
        .surface_type(SurfaceType::WINDOW);

    let configs = display.config_search(builder.build()).unwrap();

    configs
}
