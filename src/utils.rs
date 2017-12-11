
use egl_sys::ffi::types::EGLint;

#[derive(Debug, Copy, Clone)]
pub struct PositiveInteger(EGLint);

impl PositiveInteger {
    /// Panics if value is equal or less than zero.
    pub fn new(value: EGLint) -> PositiveInteger {
        match PositiveInteger::try_convert(value) {
            Ok(positive_integer) => positive_integer,
            Err(_) => panic!("egl_wrapper: {} is not a positive integer", value),
        }
    }

    pub fn value(&self) -> EGLint {
        self.0
    }

    pub fn try_convert(value: EGLint) -> Result<PositiveInteger, IntegerError> {
        if 0 < value {
            Ok(PositiveInteger(value))
        } else if value == 0 {
            Err(IntegerError::Zero)
        } else {
            Err(IntegerError::Negative)
        }
    }
}

// TODO: tests for PositiveInteger

#[derive(Debug)]
pub enum IntegerError {
    Zero,
    Negative,
}


#[derive(Debug, Copy, Clone)]
pub struct UnsignedInteger(EGLint);

impl UnsignedInteger {
    /// Panics if value is less than zero.
    pub fn new(value: EGLint) -> UnsignedInteger {
        match UnsignedInteger::try_convert(value) {
            Ok(unsigned_integer) => unsigned_integer,
            Err(_) => panic!("egl_wrapper: {} is not a unsigned integer", value),
        }
    }

    pub fn value(&self) -> EGLint {
        self.0
    }

    pub fn try_convert(value: EGLint) -> Result<UnsignedInteger, IntegerError> {
        if 0 <= value {
            Ok(UnsignedInteger(value))
        } else {
            Err(IntegerError::Negative)
        }
    }
}

// TODO: tests for UnsignedInteger