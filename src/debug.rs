use gl;
use gl::types::GLuint;

use std::fmt;
use std::error::Error;

#[macro_export]
macro_rules! check_gl_error {
    () => {
        match $crate::GlError::check() {
            Some(error) => println!("{}:{}: {}", file!(), line!(), error),
            _ => (),
        }
    }
}

#[derive(Debug)]
pub enum GlError {
    InvalidEnum,
    InvalidValue,
    InvalidOperation,
    StackOverflow,
    StackUnderflow,
    OutOfMemory,
    InvalidFramebufferOperation,
    Unrecognized(GLuint),
}

impl GlError {
    pub fn from_error_code(error_code: GLuint) -> Option<GlError> {
        match error_code {
            gl::NO_ERROR                      => None,
            gl::INVALID_ENUM                  => Some(GlError::InvalidEnum),
            gl::INVALID_VALUE                 => Some(GlError::InvalidValue),
            gl::INVALID_OPERATION             => Some(GlError::InvalidOperation),
            gl::STACK_OVERFLOW                => Some(GlError::StackOverflow),
            gl::STACK_UNDERFLOW               => Some(GlError::StackUnderflow),
            gl::OUT_OF_MEMORY                 => Some(GlError::OutOfMemory),
            gl::INVALID_FRAMEBUFFER_OPERATION => Some(GlError::InvalidFramebufferOperation),
            _                                 => Some(GlError::Unrecognized(error_code)),
        }
    }

    pub fn check() -> Option<GlError> {
        let error_code = unsafe { gl::GetError() };

        GlError::from_error_code(error_code)
    }
}

impl fmt::Display for GlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            GlError::Unrecognized(error_code) => write!(f, "{}: {}", self.description(), error_code),
            _                                 => write!(f, "{}", self.description()),
        }
    }
}

impl Error for GlError {
    fn description(&self) -> &str {
        match *self {
            GlError::InvalidEnum                 => "Invalid enum error",
            GlError::InvalidValue                => "Invalid value error",
            GlError::InvalidOperation            => "Invalid operation error",
            GlError::StackOverflow               => "Stack overflow error",
            GlError::StackUnderflow              => "Stack underflow error",
            GlError::OutOfMemory                 => "Out of memory error",
            GlError::InvalidFramebufferOperation => "Invalid framebuffer operation error",
            GlError::Unrecognized(_)             => "Unrecognized error code",
        }
    }
}
