extern crate egl_wrapper;

extern crate gl;
extern crate winit;
extern crate x11;

mod utils;

use utils::{print_opengl_info, search_configs};

use x11::xlib;

use std::thread;
use std::time::Duration;
use std::fmt;

use egl_wrapper::display::{Display, DisplayType};
use egl_wrapper::surface::window::WindowSurfaceAttributeListBuilder;
use egl_wrapper::platform::{EXTPlatformX11, EXTPlatformX11AttributeListBuilder, RawNativeDisplay,
                            RawNativeWindow};
use egl_wrapper::config::attribute::NativeRenderable;

use winit::{EventsLoop, Window, WindowBuilder};
use winit::os::unix::{EventsLoopExt, WindowExt};

struct WinitWindowX11 {
    events: EventsLoop,
    window: Option<Window>,
    window_xid: xlib::Window,
}

impl fmt::Debug for WinitWindowX11 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "WinitWindowX11")
    }
}

impl WinitWindowX11 {
    fn create_window(&mut self, x11_visual_id: egl_wrapper::ffi::types::EGLint) -> Result<(), ()> {
        // TODO: create window with X11 Visual

        let window = WindowBuilder::new()
            .with_dimensions(640, 480)
            .build(&self.events)
            .unwrap();

        self.window_xid = window.get_xlib_window().unwrap() as xlib::Window;
        self.window = Some(window);

        Ok(())
    }
}

unsafe impl<'a> RawNativeDisplay for &'a mut WinitWindowX11 {
    type T = *mut x11::xlib::Display;

    fn raw_native_display(&self) -> Self::T {
        self.events.get_xlib_xconnection().unwrap().display as *mut x11::xlib::Display
    }
}

unsafe impl<'a> RawNativeWindow for &'a mut WinitWindowX11 {
    type T = *mut x11::xlib::Window;

    fn raw_native_window(&self) -> Option<Self::T> {
        let window_ptr: *const _ = &self.window_xid;
        Some(window_ptr as *mut xlib::Window)
    }
}

fn main() {
    let events_loop = EventsLoop::new();

    if events_loop.is_wayland() {
        unimplemented!()
    } else if events_loop.is_x11() {
        x11_platform_extension(events_loop);
    } else {
        panic!("events loop did not indicated either x11 or wayland support");
    }
}

fn x11_platform_extension(events_loop: EventsLoop) {
    let mut winit_window = WinitWindowX11 {
        events: events_loop,
        window: None,
        window_xid: 0,
    };

    let display_builder = egl_wrapper::DisplayBuilder::new()
        .unwrap()
        .client_extension_mode()
        .unwrap();

    let mut display: Display<EXTPlatformX11<&mut WinitWindowX11>> = display_builder
        .build_ext_platform_x11(&mut winit_window, EXTPlatformX11AttributeListBuilder::new())
        .expect("error");

    println!("display: {:?}", display.egl_version());

    let (config_window, opengl_context_builder, x11_visual) = {
        let config = search_configs(&display).into_iter().next().unwrap();
        let config_window = config.clone().window_surface().unwrap();
        let opengl_context_builder = config.clone().opengl_context_builder().unwrap();
        let x11_visual = config.native_visual_id().unwrap().unwrap();
        (config_window, opengl_context_builder, x11_visual)
    };

    display
        .platform_display_mut()
        .x11_mut()
        .create_window(x11_visual)
        .unwrap();

    let attributes = WindowSurfaceAttributeListBuilder::new().build();
    let egl_window_surface = display
        .platform_display()
        .get_platform_window_surface(config_window, attributes)
        .unwrap();
    let context = display
        .build_opengl_context(opengl_context_builder)
        .unwrap();

    let mut current_context = context.make_current(egl_window_surface).unwrap();

    {
        let function_loader = current_context
            .context()
            .display()
            .function_loader()
            .unwrap();
        gl::load_with(|s| function_loader.get_proc_address(s).unwrap());
    }

    unsafe {
        gl::ClearColor(0.0, 0.5, 0.8, 0.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);
    }

    print_opengl_info();

    current_context = current_context.swap_buffers().unwrap();

    thread::sleep(Duration::from_secs(2));
}
