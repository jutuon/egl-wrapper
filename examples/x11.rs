
extern crate egl_wrapper;

extern crate x11;

use x11::xlib;

use std::ptr::null;
use std::thread;
use std::time::Duration;

//#[link(name="X11")]
//extern {}

fn main() {
    println!("{}", "Hello world");

    default();

}

fn x11() {
    unsafe {

        let display_ptr = x11::xlib::XOpenDisplay(null());

        if display_ptr.is_null() {
            println!("x11 display creation error");
            return;
        }


        let display = egl_wrapper::display::EGLDisplay::from_native_display_type(display_ptr).expect("error");

        println!("egl: version {:?}", display.version());

        thread::sleep(Duration::from_secs(2));


        let result = x11::xlib::XCloseDisplay(display_ptr);

        if result != 0 {
            println!("x11 display close error");
        }
    }
}

fn default() {
    unsafe {

        let mut display = egl_wrapper::display::EGLDisplay::default_display().expect("error");


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

            println!("config search results count: {}", configs.count());

            for config in configs.iter() {
                config.all().unwrap();
            }
        }

        println!();

        thread::sleep(Duration::from_secs(2));
    }
}