
extern crate egl_wrapper;

extern crate gl;
extern crate x11;

mod utils;

use x11::xlib;

use std::ptr::null;
use std::thread;
use std::time::Duration;
use std::mem;

use egl_wrapper::display::{ Display, DisplayType };
use egl_wrapper::surface::window::WindowSurfaceAttributeListBuilder;

use egl_wrapper::platform::{RawNativeDisplay, RawNativeWindow, DefaultPlatform};

use utils::{ print_opengl_info, search_configs };

#[link(name="X11")]
extern {}

fn main() {
    println!("{}", "Hello world");

    client_extensions();
    x11();
}


#[derive(Debug)]
struct X11 {
    raw_display: *mut x11::xlib::Display,
    raw_window: Option<x11::xlib::Window>,
}

impl X11 {
    fn new() -> Result<X11, ()> {

        let raw_display = unsafe {
            xlib::XOpenDisplay(null())
        };

        if raw_display.is_null() {
            println!("x11 display creation error");
            return Err(());
        }

        Ok(X11 {
            raw_display,
            raw_window: None,
        })
    }

    fn create_window(&mut self, visual_id: xlib::VisualID) -> Result<(), ()> {

        println!("visual id: {}", visual_id);

        unsafe {
            let mut visual_info_template: xlib::XVisualInfo = mem::zeroed();

            visual_info_template.visualid = visual_id;

            let mut visual_count = 0;

            let mut visual_info_ptr: *mut xlib::XVisualInfo = xlib::XGetVisualInfo(
                self.raw_display,
                xlib::VisualIDMask,
                &mut visual_info_template,
                &mut visual_count
            );

            println!("visual_count: {}", visual_count);

            if visual_info_ptr.is_null() {
                println!("error: visual info ptr is null");
                return Err(())
            }

            let mut colormap: xlib::Colormap = xlib::XCreateColormap(
                self.raw_display,
                xlib::XRootWindow(self.raw_display, 0),
                (*visual_info_ptr).visual,
                xlib::AllocNone,
            );

            if colormap == 0 {
                println!("error: colormap is null");
                return Err(())
            }

            println!("colormap id: {}", colormap);

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
                colormap,
                cursor: 0,
            };


            let window = xlib::XCreateWindow(
                self.raw_display,
                xlib::XDefaultRootWindow(self.raw_display),
                0,
                0,
                640,
                480,
                0,
                (*visual_info_ptr).depth,
                xlib::InputOutput as u32,
                (*visual_info_ptr).visual,
                xlib::CWColormap,
                &mut window_attributes
            );

            println!("window id: {}", window);

            xlib::XSelectInput(self.raw_display, window, xlib::StructureNotifyMask);

            xlib::XMapWindow(self.raw_display, window);

            let mut event: xlib::XEvent = mem::zeroed();

            loop {
                xlib::XNextEvent(self.raw_display, &mut event);
                if event.type_ == xlib::MapNotify {
                    break;
                }
            }

            self.raw_window = Some(window);
            Ok(())
        }
    }
}

impl Drop for X11 {
    fn drop(&mut self) {
        unsafe {
            if let Some(raw_window) = self.raw_window {
                xlib::XDestroyWindow(self.raw_display, raw_window);
            }

            let result = xlib::XCloseDisplay(self.raw_display);

            if result != 0 {
                println!("x11 display close error");
            }
        }
    }
}


unsafe impl <'a> RawNativeDisplay for &'a mut X11 {
    type T = egl_wrapper::ffi::types::NativeDisplayType;

    fn raw_native_display(&self) -> Self::T {
        self.raw_display
    }
}

unsafe impl <'a> RawNativeWindow for &'a mut X11 {
    type T = egl_wrapper::ffi::types::NativeWindowType;

    fn raw_native_window(&self) -> Option<Self::T> {
        self.raw_window
    }
}

fn x11() {
    unsafe {
        let mut x11 = X11::new().unwrap();

        let display_builder = egl_wrapper::DisplayBuilder::new().unwrap();

        // Create display with mutable reference to X11 so it is sure that X11 will be dropped last, if
        // WindowSurface will not be returned from this function.
        let mut display: Display<DefaultPlatform<&mut X11>> = display_builder.build_from_native_display(&mut x11).expect("error");

        print_display_info(&display);

        let client_api_support = display.client_api_support().unwrap();

        if !client_api_support.opengl && !client_api_support.opengl_es {
            println!("OpenGL or OpenGL ES support is required");

            return
        }

        let (config_window, opengl_context_builder, visual_id) = {
            use egl_wrapper::config::attribute::NativeRenderable;

            let config = search_configs(&display).into_iter().next().unwrap();
            let config_window = config.clone().window_surface().unwrap();
            let opengl_context_builder = config.clone().opengl_context_builder().unwrap();

            let visual_id = config.native_visual_id().unwrap().unwrap();
            (config_window, opengl_context_builder, visual_id)
        };

        display.platform_display_mut().native_mut().create_window(visual_id as xlib::XID).unwrap();

        let attributes = WindowSurfaceAttributeListBuilder::new().build();
        let egl_window_surface = display.platform_display().get_platform_window_surface(config_window, attributes).unwrap();
        let context = display.build_opengl_context(opengl_context_builder).unwrap();

        let mut current_context = context.make_current(egl_window_surface).unwrap();

        {
            let function_loader = current_context.context().display().function_loader().unwrap();
            gl::load_with(|s| {
                function_loader.get_proc_address(s).unwrap()
            });
        }

        gl::ClearColor(0.0, 0.5, 0.8, 0.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);

        print_opengl_info();

        current_context = current_context.swap_buffers().unwrap();

        thread::sleep(Duration::from_secs(2));
    }
}

fn print_display_info(display: &Display<DefaultPlatform<&mut X11>>) {
    use egl_wrapper::config::attribute::*;

    // Test querying version information

    println!("egl: version {:?}", display.egl_version());

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

        for config in configs {
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

        for config in configs {
            config.all().unwrap();
        }
    }

    println!();

    //thread::sleep(Duration::from_secs(2));
}

fn client_extensions() {
    let display_builder = egl_wrapper::DisplayBuilder::new().unwrap();

    match display_builder.client_extension_mode() {
        Ok(client_extensions_builder) => {

            let client_extensions = client_extensions_builder.client_extensions().unwrap();

            println!("client extensions: ");

            for ext in client_extensions.split_whitespace() {
                println!("{}", ext);
            }

        },
        Err(_) => println!("EGL extension EGL_EXT_client_extensions is not supported"),
    }
}