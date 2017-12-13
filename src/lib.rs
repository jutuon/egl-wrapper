
extern crate egl_sys;

#[macro_use]
extern crate bitflags;

#[cfg(unix)]
extern crate x11;

pub mod utils;
pub mod config;
mod error;
pub mod display;
pub mod surface;
pub mod context;


pub use egl_sys::ffi;