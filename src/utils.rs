
use std::fmt;

use egl_sys::ffi::types::EGLint;
use egl_sys::ffi;

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

impl fmt::Display for PositiveInteger {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.value())
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


impl fmt::Display for UnsignedInteger {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.value())
    }
}

// TODO: tests for UnsignedInteger



pub(crate) struct AttributeListBuilder(Vec<EGLint>);

impl AttributeListBuilder {
    pub fn new() -> AttributeListBuilder {
        AttributeListBuilder(Vec::new())
    }

    pub fn add(&mut self, attribute: EGLint, value: EGLint) {
        self.0.push(attribute);
        self.0.push(value);
    }

    pub fn build(mut self) -> AttributeList {
        self.0.push(ffi::NONE as EGLint);
        AttributeList(self.0)
    }
}


pub(crate) struct AttributeList(Vec<EGLint>);

impl AttributeList {
    pub fn ptr(&self) -> *const EGLint {
        self.0.as_slice().as_ptr()
    }
}


#[derive(Debug)]
pub enum QueryError {
    QueryError,
    BooleanError,
    EnumError,
    IntegerError(IntegerError),
}

impl From<IntegerError> for QueryError {
    fn from(error: IntegerError) -> Self {
        QueryError::IntegerError(error)
    }
}

pub type QueryResult<T> = Result<T, QueryError>;