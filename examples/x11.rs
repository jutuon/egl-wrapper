
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


        thread::sleep(Duration::from_secs(2));
    }
}