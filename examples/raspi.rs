

// Raspberry Pi Broadcom Dispmanx, X11, EGL and OpenGL ES 2.0 example.
// Build and run with `cargo run --example raspi --features raspberry-pi-broadcom`


extern crate egl_wrapper;

extern crate opengles;
extern crate x11_wrapper;
extern crate videocore_wrapper;

use std::sync::Arc;

use opengles::glesv2;

use videocore_wrapper::BCMHostHandle;
use videocore_wrapper::display as videocore_display;
use videocore_wrapper::videocore::image::Rect;
use videocore_wrapper::videocore::dispmanx::Transform;

use x11_wrapper::XlibHandle;
use x11_wrapper::core::window::input_output::{InputOutputWindowBuilder};
use x11_wrapper::core::event::{SimpleEvent, EventBuffer, EventMask};
use x11_wrapper::protocol::Protocols;
use x11_wrapper::core::window::attribute::CommonAttributes;
use x11_wrapper::core::window::attribute::InputOutputWindowAttributes;

use egl_wrapper::display::{Display, DisplayType};
use egl_wrapper::surface::window::{WindowSurfaceAttributeListBuilder, WindowSurface};
use egl_wrapper::platform::{DefaultPlatform, Platform};
use egl_wrapper::EGLHandle;
use egl_wrapper::config::Configs;
use egl_wrapper::context::gles::EGL14OpenGLESVersion;

fn main() {
    let egl_handle = EGLHandle::load().unwrap();

    client_extensions(&egl_handle);
    x11(&egl_handle);
}

const DEFAULT_WINDOW_WIDTH: i32 = 640;
const DEFAULT_WINDOW_HEIGHT: i32 = 480;


fn x11(egl_handle: &EGLHandle) {

    // Create X11 window

    let xlib_handle = XlibHandle::initialize_xlib().unwrap();
    let mut x11_display = xlib_handle.create_display().unwrap();

    let default_screen = x11_display.default_screen();
    let default_visual = default_screen.default_visual().unwrap();

    let mut protocols = Protocols::new();
    let delete_window_handler = protocols.enable_delete_window(&x11_display).unwrap();

    let event_mask = EventMask::KEY_PRESS | EventMask::KEY_RELEASE | EventMask::BUTTON_PRESS
        | EventMask::BUTTON_RELEASE | EventMask::POINTER_MOTION
        | EventMask::ENTER_WINDOW | EventMask::LEAVE_WINDOW
        | EventMask::FOCUS_CHANGE | EventMask::STRUCTURE_NOTIFY;

    let window = InputOutputWindowBuilder::new(&default_screen, default_visual)
        .unwrap()
        .set_event_mask(event_mask)
        .set_background_pixel(0x000000)
        .build_input_output_window()
        .unwrap()
        .start_configuring_normal_hints()
        .unwrap()
        .set_min_window_size(DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT)
        .end()
        .set_protocols(protocols.protocol_atom_list())
        .unwrap()
        .map_window();

    x11_display.flush_output_buffer();

    // Initialize Broadcom libraries

    let bcm_host = BCMHostHandle::init().unwrap();

    // Create dispmanx display. Its not used for creating EGLDisplay
    // but lets use it as optional native display handle.

    let dispmanx_display = bcm_host.dispmanx_display(videocore_display::DisplayID::MainLCD);

    // Create EGLDisplay

    let display_builder = egl_handle.display_builder();

    let display: Display<DefaultPlatform<Arc<videocore_display::DisplayHandle>>> =
        display_builder.build_default_platform_default_display(dispmanx_display.display_handle().clone()).unwrap();


    print_display_info(&display);

    let client_api_support = display.client_api_support().unwrap();

    if !client_api_support.opengl_es {
        println!("OpenGL ES support is required");

        return;
    }

    // Find EGLConfig

    let (config_window, opengl_es_context_builder) = {
        let config = search_configs(&display).into_iter().next().unwrap();
        let config_window = display.window_surface(&config).unwrap().unwrap();
        let opengl_es_context_builder = display.opengl_es_context_builder(EGL14OpenGLESVersion::Version2, &config).unwrap().unwrap();

        // TODO: add build method to opengl context builders

        (config_window, opengl_es_context_builder)
    };

    // Create Dispmanx Element

    let mut dest_rect = Rect {
        x: 0,
        y: 0,
        width: DEFAULT_WINDOW_WIDTH,
        height: DEFAULT_WINDOW_HEIGHT,
    };

    let mut src_rect = Rect {
        x: 0,
        y: 0,
        width: 0,
        height: 0,
    };

    let element = bcm_host.dispmanx_update_builder(0).unwrap().element_add(
        &dispmanx_display,
        100,
        &mut dest_rect,
        &mut src_rect,
        videocore_display::Protection::None,
        Transform::NO_ROTATE,
    ).unwrap();

    let mut window = element.into_window();

    // Create EGLSurface

    let attributes = WindowSurfaceAttributeListBuilder::new().build();
    let egl_window_surface: WindowSurface<&mut videocore_display::Window, DefaultPlatform<Arc<videocore_display::DisplayHandle>>> = unsafe {
        let raw_window_ptr = window.raw_window();
        display.display_handle()
            .platform()
            .get_platform_window_surface(&mut window, raw_window_ptr as *mut _, config_window, attributes)
            .unwrap()
    };

    // Create OpenGL ES context

    let context = display
        .build_opengl_es_context(opengl_es_context_builder)
        .unwrap();

    let mut current_context = context.make_current(egl_window_surface).unwrap();

    // Print OpenGL info

    print_opengl_info();

    // Clear color buffer and swap buffers

    glesv2::clear_color(0.0, 0.5, 0.8, 0.0);
    glesv2::clear(glesv2::GL_COLOR_BUFFER_BIT);

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
            // Key Q
            &SimpleEvent::KeyRelease { keycode: 24 } => {
                let dest_rect = Rect {
                    x: 100,
                    y: 100,
                    width: DEFAULT_WINDOW_WIDTH,
                    height: DEFAULT_WINDOW_HEIGHT,
                };

                current_context.surface_mut().optional_native_window_handle_mut()
                    .change_element_attributes(None, None, Some(&dest_rect), None)
                    .unwrap();
            }
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
        let configs = search_configs(&display);
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

// Functions form utils.rs module modified to work with this example

pub fn print_opengl_info() {
    println!("OpenGL context information:");
    println!("  Version:  {:?}", glesv2::get_string(glesv2::GL_VERSION));
    println!("  Vendor:   {:?}", glesv2::get_string(glesv2::GL_VENDOR));
    println!("  Renderer: {:?}", glesv2::get_string(glesv2::GL_RENDERER));
}

pub fn search_configs<'a, P: Platform>(display: &'a Display<P>) -> Configs<'a, Display<P>> {
    use egl_wrapper::config::attribute::{ConfigClientAPI, SurfaceType};

    use egl_wrapper::config::search::UnsignedIntegerSearchAttributes;

    use egl_wrapper::utils::UnsignedInteger;

    let mut builder = display.config_search_options_builder();

    builder
        .add_unsigned_integer_attribute(
            UnsignedIntegerSearchAttributes::RedSize,
            Some(UnsignedInteger::new(8)),
        )
        .add_unsigned_integer_attribute(
            UnsignedIntegerSearchAttributes::GreenSize,
            Some(UnsignedInteger::new(8)),
        )
        .add_unsigned_integer_attribute(
            UnsignedIntegerSearchAttributes::BlueSize,
            Some(UnsignedInteger::new(8)),
        )
        .add_unsigned_integer_attribute(
            UnsignedIntegerSearchAttributes::AlphaSize,
            Some(UnsignedInteger::new(8)),
        )
        .client_api_conformance(ConfigClientAPI::OPENGL_ES2)
        .client_api(ConfigClientAPI::OPENGL_ES2)
        .surface_type(SurfaceType::WINDOW);

    let configs = display.config_search(builder.build()).unwrap();

    configs
}
