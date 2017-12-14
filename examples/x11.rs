
extern crate egl_wrapper;

extern crate gl;
extern crate x11;

use x11::xlib;

use std::os::raw;
use std::ffi::{CString, CStr};
use std::ptr::null;
use std::thread;
use std::time::Duration;
use std::mem;

use egl_wrapper::config::Configs;
use egl_wrapper::display::Display;
use egl_wrapper::ffi;
use egl_wrapper::context::CurrentContext;


use egl_wrapper::surface::Surface;

#[link(name="X11")]
extern {}

fn main() {
    println!("{}", "Hello world");

    default();
    x11();
}

fn x11() {
    unsafe {

        let display_ptr = xlib::XOpenDisplay(null());

        if display_ptr.is_null() {
            println!("x11 display creation error");
            return;
        }


        let mut display = egl_wrapper::display::Display::from_native_display_type(display_ptr).expect("error");

        println!("egl: version {:?}", display.version());

        let mut window_attributes = xlib::XSetWindowAttributes {
            background_pixmap: 0,
            background_pixel: 0,
            border_pixmap: 0,
            border_pixel: 0,
            bit_gravity: 0,
            win_gravity: 0,
            backing_store: 0,
            backing_planes: 0,
            backing_pixel: 0,
            save_under: 0,
            event_mask: 0,
            do_not_propagate_mask: 0,
            override_redirect: 0,
            colormap: 0,
            cursor: 0,
        };

        let window = xlib::XCreateWindow(
            display_ptr,
            xlib::XDefaultRootWindow(display_ptr),
            0,
            0,
            640,
            480,
            0,
            xlib::CopyFromParent,
            xlib::CopyFromParent as u32,
            xlib::CopyFromParent as *mut xlib::Visual,
            0,
            &mut window_attributes
        );

        xlib::XSelectInput(display_ptr, window, xlib::StructureNotifyMask);

        xlib::XMapWindow(display_ptr, window);

        let mut event: xlib::XEvent = mem::zeroed();

        while true {
            xlib::XNextEvent(display_ptr, &mut event);
            if event.type_ == xlib::MapNotify {
                break;
            }
        }

        let config = {
            let configs = search_configs(&display);
            configs.into_iter().next().unwrap().into_display_config()
        };

        let egl_window_surface = display.window_surface_builder(config.clone()).build(window).unwrap();

        let context = display.opengl_context(config).unwrap().unwrap();

        context.context().make_current(&egl_window_surface).unwrap();

        gl::load_with(|s| {
            let c_string = CString::new(s);
            ffi::GetProcAddress(c_string.unwrap().as_ptr()) as *const raw::c_void
        });

        gl::ClearColor(0.0, 0.5, 0.8, 0.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);

        print_opengl_info();

        egl_wrapper::ffi::SwapBuffers(display.raw(), egl_window_surface.raw());

        thread::sleep(Duration::from_secs(2));


        xlib::XDestroyWindow(display_ptr, window);


        let result = xlib::XCloseDisplay(display_ptr);

        if result != 0 {
            println!("x11 display close error");
        }
    }
}

fn default() {
    unsafe {

        let mut display = egl_wrapper::display::Display::default_display().expect("error");


        // Test querying version information

        println!("egl: version {:?}", display.version());

        println!("vendor: {:?}", display.vendor().unwrap());
        println!("client_apis: {:?}", display.client_apis().unwrap());
        println!("version: {:?}", display.version_string().unwrap());

        {
            let extensions = display.extensions().unwrap();

            println!("extensions: ");

            for ext in extensions.split_whitespace() {
                println!("{}", ext);
            }
        }

        // Test querying all configs

        {
            let configs = display.configs().unwrap();
            println!("config count: {}", configs.count());

            for config in configs.iter() {
                match config.color_buffer() {
                    Err(error) => println!("{:?}", error),
                    _ => (),
                }

                //println!();

                config.all().unwrap();

            }
        }

        // Test searching configs

        {
            let configs = search_configs(&display);
            println!("config search results count: {}", configs.count());

            for config in configs.iter() {
                config.all().unwrap();
            }
        }

        println!();

        //thread::sleep(Duration::from_secs(2));
    }
}


fn search_configs<'a>(display: &'a Display) -> Configs<'a> {
    use egl_wrapper::display::EGLVersion;
    use egl_wrapper::config::{
        UnsignedIntegerSearchAttributes,
        SurfaceType,
        RenderableType,
        EGL14ConfigClientAPI,
        EGL15ConfigClientAPI,
        ClientApiConformance,
    };
    use egl_wrapper::utils::UnsignedInteger;

    let mut options = display.config_search_options_builder();

    options.add_unsigned_integer_attribute(
        UnsignedIntegerSearchAttributes::AlphaSize,
        Some(UnsignedInteger::new(8))
    );

    let renderable_type = match display.version() {
        EGLVersion::EGL_1_4 => RenderableType::EGL14(EGL14ConfigClientAPI::OPENGL),
        EGLVersion::EGL_1_5 => RenderableType::EGL15(EGL15ConfigClientAPI::OPENGL),
    };

    let client_api_conformance = match display.version() {
        EGLVersion::EGL_1_4 => ClientApiConformance::EGL14(EGL14ConfigClientAPI::OPENGL),
        EGLVersion::EGL_1_5 => ClientApiConformance::EGL15(EGL15ConfigClientAPI::OPENGL),
    };

    options.client_api_conformance(client_api_conformance).unwrap();
    options.renderable_type(renderable_type).unwrap();
    options.surface_type(SurfaceType::WINDOW);

    let configs = display.config_search(options.build()).unwrap();

    configs
}

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