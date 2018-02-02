use egl_sys::ffi;
use egl_sys::ffi::types::{EGLenum, EGLint};
use egl_sys::extensions;

use utils::{PositiveInteger, QueryError, QueryResult, UnsignedInteger};

use super::Context;

#[derive(Debug)]
#[repr(u32)]
pub enum QueryableAttribute {
    ConfigID = ffi::CONFIG_ID,
    ContextClientType = ffi::CONTEXT_CLIENT_TYPE,
    ContextClientVersion = ffi::CONTEXT_CLIENT_VERSION,
    RenderBuffer = ffi::RENDER_BUFFER,
}

#[derive(Debug)]
#[repr(u32)]
pub enum ContextAPIType {
    OpenGL = ffi::OPENGL_API,
    OpenGLES = ffi::OPENGL_ES_API,
    OpenVG = ffi::OPENVG_API,
}

pub trait ContextAttributeUtils: Context {
    fn query_attribute(&self, attribute: QueryableAttribute) -> QueryResult<EGLint> {
        let mut value = 0;
        let result = unsafe {
            egl_function!(
                self.egl_handle(),
                QueryContext(
                    self.raw_display(),
                    self.raw_context(),
                    attribute as EGLint,
                    &mut value
                )
            )
        };

        if result == ffi::EGL_TRUE {
            Ok(value)
        } else {
            Err(QueryError::QueryError)
        }
    }

    fn query_positive_integer(
        &self,
        attribute: QueryableAttribute,
    ) -> QueryResult<PositiveInteger> {
        Ok(PositiveInteger::try_convert(
            self.query_attribute(attribute)?,
        )?)
    }

    fn query_unsigned_integer(
        &self,
        attribute: QueryableAttribute,
    ) -> QueryResult<UnsignedInteger> {
        Ok(UnsignedInteger::try_convert(
            self.query_attribute(attribute)?,
        )?)
    }

    fn query_boolean(&self, attribute: QueryableAttribute) -> Result<bool, QueryError> {
        let value = self.query_attribute(attribute)?;

        if value == ffi::EGL_TRUE as EGLint {
            Ok(true)
        } else if value == ffi::EGL_FALSE as EGLint {
            Ok(false)
        } else {
            Err(QueryError::BooleanError)
        }
    }
}

pub trait CommonAttributes: ContextAttributeUtils {
    fn config_id(&self) -> QueryResult<PositiveInteger> {
        self.query_positive_integer(QueryableAttribute::ConfigID)
    }

    fn context_api_type(&self) -> QueryResult<ContextAPIType> {
        let value = self.query_attribute(QueryableAttribute::ContextClientType)?;

        match value as EGLenum {
            ffi::OPENGL_API => Ok(ContextAPIType::OpenGL),
            ffi::OPENGL_ES_API => Ok(ContextAPIType::OpenGLES),
            ffi::OPENVG_API => Ok(ContextAPIType::OpenVG),
            _ => Err(QueryError::EnumError),
        }
    }
}

pub trait AttributeOpenGLESVersion: ContextAttributeUtils {
    fn opengl_es_version(&self) -> QueryResult<UnsignedInteger> {
        self.query_unsigned_integer(QueryableAttribute::ContextClientVersion)
    }
}

// EGL_KHR_create_context extension implementation

/// EGL_KHR_create_context
#[derive(Debug)]
#[repr(u32)]
pub enum OpenGLContextProfile {
    Core = extensions::CONTEXT_OPENGL_CORE_PROFILE_BIT_KHR,
    Compability = extensions::CONTEXT_OPENGL_COMPATIBILITY_PROFILE_BIT_KHR,
}

bitflags! {
    /// EGL_KHR_create_context
    pub struct OpenGLContextFlags: EGLenum {
        const DEBUG              = extensions::CONTEXT_OPENGL_DEBUG_BIT_KHR;

        /// Only for OpenGL 3.0 or later.
        const FORWARD_COMPATIBLE = extensions::CONTEXT_OPENGL_FORWARD_COMPATIBLE_BIT_KHR;

        const ROBUST_ACCESS      = extensions::CONTEXT_OPENGL_ROBUST_ACCESS_BIT_KHR;
    }
}

/// EGL_KHR_create_context
#[derive(Debug)]
#[repr(u32)]
pub enum ResetNotificationStrategy {
    NoResetNotification = extensions::NO_RESET_NOTIFICATION_KHR,
    LoseContextOnReset = extensions::LOSE_CONTEXT_ON_RESET_KHR,
}
