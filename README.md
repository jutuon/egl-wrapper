# egl-wrapper

Safe Rust wrapper for EGL. Targets EGL versions 1.4 and later.

Uses `egl-sys` crate for EGL bindings.

## Status

Currently not usable.

## Features

### General

- [ ] Runtime linking to EGL

### Platforms

- [x] X11
- [ ] Wayland
- [ ] Windows
- [ ] Android

### EGL 1.4

- [ ] Surfaces
    - [x] Window
    - [ ] Pbuffer
    - [ ] Pixmap
- [ ] Contexts
    - [x] OpenGL
    - [x] OpenGL ES
    - [ ] OpenVG

### EGL 1.5

Features are listed like in EGL 1.5 specification Appendix F.

- [ ] Platform support
    - [ ] `EGL_EXT_client_extensions`
    - [ ] `EGL_EXT_platform_base`
- [ ] Client API interoperability
    - [ ] `EGL_KHR_fence_sync`
    - [ ] `EGL_KHR_cl_event2`
    - [ ] `EGL_KHR_wait_sync`
- [ ] Image sharing
    - [ ] `EGL_KHR_image_base`
    - [ ] `EGL_KHR_gl_texture_2D_image`
    - [ ] `EGL_KHR_gl_texture_3D_image`
    - [ ] `EGL_KHR_gl_texture_cubemap_image`
    - [ ] `EGL_KHR_gl_renderbuffer_image`
- [ ] General API cleanup
    - [ ] `EGL_KHR_create_context`
    - [ ] `EGL_EXT_create_context_robustness`
    - [ ] `EGL_KHR_get_all_proc_addresses`
    - [ ] `EGL_KHR_client_get_all_proc_addresses`
    - [ ] `EGL_KHR_gl_colorspace`
    - [ ] `EGL_KHR_surfaceless_context`

## Unsupported EGL features

I'm not currently planing to support these features:

* Rendering to textures
* Shared state between contexts

## License

This crate is licensed under terms of

* Apache 2.0 license or
* MIT license

at your opinion.

## Contributions

Contributions will be licensed as stated in License section
of this file unless otherwise specified.
