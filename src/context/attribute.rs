
use egl_sys::ffi;
use egl_sys::ffi::types::{ EGLint, EGLenum };

use utils::{UnsignedInteger, PositiveInteger, QueryError, QueryResult};

use super::Context;

#[derive(Debug)]
#[repr(u32)]
pub enum QueryableAttribute {
    ConfigID                = ffi::CONFIG_ID,
    ContextClientType       = ffi::CONTEXT_CLIENT_TYPE,
    ContextClientVersion    = ffi::CONTEXT_CLIENT_VERSION,
    RenderBuffer            = ffi::RENDER_BUFFER,
}

#[derive(Debug)]
#[repr(u32)]
pub enum ContextAPIType {
    OpenGL    = ffi::OPENGL_API,
    OpenGLES  = ffi::OPENGL_ES_API,
    OpenVG    = ffi::OPENVG_API,
}


pub trait ContextAttributeUtils: Context {
    fn query_attribute(&self, attribute: QueryableAttribute) -> QueryResult<EGLint> {
        let mut value = 0;
        let result = unsafe {
            ffi::QueryContext(self.raw_display(), self.raw_context(), attribute as EGLint, &mut value)
        };

        if result == ffi::TRUE {
            Ok(value)
        } else {
            Err(QueryError::QueryError)
        }
    }

    fn query_positive_integer(&self, attribute: QueryableAttribute) -> QueryResult<PositiveInteger> {
        Ok(PositiveInteger::try_convert(self.query_attribute(attribute)?)?)
    }

    fn query_unsigned_integer(&self, attribute: QueryableAttribute) -> QueryResult<UnsignedInteger> {
        Ok(UnsignedInteger::try_convert(self.query_attribute(attribute)?)?)
    }

    fn query_boolean(&self, attribute: QueryableAttribute) -> Result<bool, QueryError> {
        let value = self.query_attribute(attribute)?;

        if value == ffi::TRUE as EGLint {
            Ok(true)
        } else if value == ffi::FALSE as EGLint {
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
            ffi::OPENGL_API     => Ok(ContextAPIType::OpenGL),
            ffi::OPENGL_ES_API  => Ok(ContextAPIType::OpenGLES),
            ffi::OPENVG_API     => Ok(ContextAPIType::OpenVG),
            _                   => Err(QueryError::EnumError),
        }
    }
}

pub trait AttributeOpenGLESVersion: ContextAttributeUtils {
    fn opengl_es_version(&self) -> QueryResult<UnsignedInteger> {
        self.query_unsigned_integer(QueryableAttribute::ContextClientVersion)
    }
}

