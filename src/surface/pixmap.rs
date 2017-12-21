use std::marker::PhantomData;

use egl_sys::ffi;

use config::DisplayConfig;
use utils::AttributeListBuilder;

use error::EGLError;

use super::{destroy_surface, Surface};

use super::attribute::{CommonAttributes, MultisampleResolve, SurfaceAttributeUtils, SwapBehavior};

#[derive(Debug)]
pub struct PixmapSurface {
    display_config: DisplayConfig,
    raw_surface: ffi::types::EGLSurface,
    _marker: PhantomData<ffi::types::EGLSurface>,
}

impl Surface for PixmapSurface {
    fn raw_surface(&self) -> ffi::types::EGLSurface {
        self.raw_surface
    }

    fn display_config(&self) -> &DisplayConfig {
        &self.display_config
    }
}

impl Drop for PixmapSurface {
    fn drop(&mut self) {
        destroy_surface(self)
    }
}

impl SurfaceAttributeUtils for PixmapSurface {}
impl CommonAttributes for PixmapSurface {}
impl MultisampleResolve for PixmapSurface {}
impl SwapBehavior for PixmapSurface {}

pub struct PixmapSurfaceBuilder {
    display_config: DisplayConfig,
    attributes: AttributeListBuilder,
    native_pixmap: ffi::types::EGLNativePixmapType,
}

impl PixmapSurfaceBuilder {
    pub(crate) fn new(
        display_config: DisplayConfig,
        native_pixmap: ffi::types::EGLNativePixmapType,
    ) -> PixmapSurfaceBuilder {
        PixmapSurfaceBuilder {
            display_config,
            attributes: AttributeListBuilder::new(),
            native_pixmap,
        }
    }

    // TODO: search configs with MatchNativePixmap if creating pixmap surface
    // TODO: PixmapSurface OpenVG attributes

    pub fn build(self) -> Result<PixmapSurface, Option<EGLError>> {
        let attributes = self.attributes.build();

        let result = unsafe {
            ffi::CreatePixmapSurface(
                self.display_config.raw_display(),
                self.display_config.raw_config(),
                self.native_pixmap,
                attributes.ptr(),
            )
        };

        if result == ffi::NO_SURFACE {
            return Err(EGLError::check_errors());
        }

        Ok(PixmapSurface {
            display_config: self.display_config,
            raw_surface: result,
            _marker: PhantomData,
        })
    }
}
