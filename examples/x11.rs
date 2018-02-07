extern crate egl_wrapper;

extern crate gl;
extern crate x11_wrapper;

mod utils;

use std::sync::Arc;

use x11_wrapper::XlibHandle;
use x11_wrapper::core::window::input_output::{InputOutputWindowBuilder, TopLevelInputOutputWindow};
use x11_wrapper::core::window::{Window};
use x11_wrapper::core::event::{SimpleEvent, EventBuffer};
use x11_wrapper::protocol::Protocols;
use x11_wrapper::core::display::DisplayHandle;

use egl_wrapper::display::{Display, DisplayType};
use egl_wrapper::surface::window::{WindowSurfaceAttributeListBuilder, WindowSurface};
use egl_wrapper::platform::{DefaultPlatform, Platform};
use egl_wrapper::EGLHandle;

fn main() {
    println!("{}", "Hello world");

    let egl_handle = EGLHandle::load().unwrap();

    client_extensions(&egl_handle);
    x11(&egl_handle);
}

fn x11(egl_handle: &EGLHandle) {
    let xlib_handle = XlibHandle::initialize_xlib().unwrap();
    let mut x11_display = xlib_handle.create_display().unwrap();

    // Create EGLDisplay

    let display_builder = egl_handle.display_builder();

    // TODO: mark build_default_platform_display as unsafe
    let display: Display<DefaultPlatform<Arc<DisplayHandle>>> = unsafe {
        display_builder.build_default_platform_display(x11_display.raw_display(), x11_display.display_handle().clone())
            .expect("error")
    };

    print_display_info(&display);

    let client_api_support = display.client_api_support().unwrap();

    if !client_api_support.opengl && !client_api_support.opengl_es {
        println!("OpenGL or OpenGL ES support is required");

        return;
    }

    // Find EGLConfig

    let (config_window, opengl_context_builder, visual_id) = {
        use egl_wrapper::config::attribute::NativeRenderable;

        let config = utils::search_configs(&display).into_iter().next().unwrap();
        let config_window = display.window_surface(&config).unwrap().unwrap();
        let opengl_context_builder = display.opengl_context_builder(&config).unwrap().unwrap();

        let visual_id = config.native_visual_id().unwrap().unwrap();
        (config_window, opengl_context_builder, visual_id)
    };

    // Create X11 window

    let default_screen = x11_display.default_screen();
    let visual = x11_display.visual_from_id(visual_id as x11_wrapper::x11::xlib::XID).expect("unsupported X11 visual");

    let mut protocols = Protocols::new();
    let delete_window_handler = protocols.enable_delete_window(&x11_display).unwrap();

    let window = InputOutputWindowBuilder::new(&default_screen, visual)
        .unwrap()
        .build_input_output_window()
        .unwrap()
        .start_configuring_normal_hints()
        .unwrap()
        .set_min_window_size(640, 480)
        .end()
        .set_protocols(protocols.protocol_atom_list())
        .unwrap()
        .map_window();
    // TODO: map window with mutable reference

    x11_display.flush_output_buffer();

    // Create EGLSurface

    // TODO: implement Default for WindowSurfaceAttributeListBuilder
    let attributes = WindowSurfaceAttributeListBuilder::new().build();
    let egl_window_surface: WindowSurface<TopLevelInputOutputWindow, DefaultPlatform<Arc<DisplayHandle>>> = unsafe {
        let window_id = window.window_id();
        display.display_handle()
            .platform()
            .get_platform_window_surface(window, window_id, config_window, attributes)
            .unwrap()
    };

    // TODO: add methods WindowSurface to get reference to native window handle

    // Create OpenGL context

    let context = display
        .build_opengl_context(opengl_context_builder)
        .unwrap();

    let mut current_context = context.make_current(egl_window_surface).unwrap();

    // Load OpenGL functions

    {
        let function_loader = current_context
            .context()
            .display()
            .function_loader()
            .unwrap();
        gl::load_with(|s| function_loader.get_proc_address(s).unwrap());
    }

    utils::print_opengl_info();

    // Clear color buffer and swap buffers

    unsafe {
        gl::ClearColor(0.0, 0.5, 0.8, 0.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);
    }

    current_context = current_context.swap_buffers().unwrap();

    // Handle X11 events

    let mut event_buffer = EventBuffer::new();

    loop {
        if let Some(error) = x11_wrapper::check_error(&x11_display) {
            eprintln!("xlib error: {:?}", error);
            break;
        }

        let event = x11_display.read_event_blocking(&mut event_buffer).into_event().into_simple_event();

        println!("{:?}", &event);

        match &event {
            &SimpleEvent::ClientMessage(e) => {
                if delete_window_handler.check_event(e) {
                    break;
                }
            }
            _ => (),
        }
    }
}

fn print_display_info<P: Platform>(display: &Display<P>) {
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
        let configs = utils::search_configs(&display);
        println!("config search results count: {}", configs.count());

        for config in configs {
            config.all().unwrap();
        }
    }

    println!();

    //thread::sleep(Duration::from_secs(2));
}

fn client_extensions(egl_handle: &EGLHandle) {
    let display_builder = egl_handle.display_builder();

    match display_builder.query_client_extensions() {
        Ok(text) => {
            println!("client extensions: ");

            for ext in text.split_whitespace() {
                println!("{}", ext);
            }
        }
        _ => println!("EGL extension EGL_EXT_client_extensions is not supported"),
    }
}