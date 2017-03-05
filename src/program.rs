use gl;
use gl::types::*;

use std::ffi;
use std::ptr;

use GlObject;

#[derive(Debug)]
pub enum ProgramCreationError {
    LinkError(String),
    InvalidInfoLog,
}

pub struct Program {
    id: GLuint,
}

impl Drop for Program {
    fn drop(&mut self) {
        self.deactivate();

        unsafe { gl::DeleteProgram(self.id); }
    }
}

impl GlObject for Program {
    fn id(&self) -> GLuint {
        self.id
    }
}

impl Program {
    pub fn link(shaders: &[Shader]) -> Result<Program, ProgramCreationError> {
        let mut link_status = gl::FALSE as GLint;
        let program;

        unsafe {
            // 1. Create a program object.
            program = Program {
                id: gl::CreateProgram()
            };

            // 2. Attach the shaders. Notice we don't need to specify their type, as OpenGL already has
            // that information.
            for shader in shaders {
                gl::AttachShader(program.id(), shader.id());
            }

            // 3. Link the program.
            gl::LinkProgram(program.id());

            // 4. Retrieve the link status.
            gl::GetProgramiv(program.id(), gl::LINK_STATUS, &mut link_status);
        }

        if link_status == (gl::TRUE as GLint) {
            Ok(program)
        } else {
            unsafe {
                // Getting the program info log is similar to getting the shader info log. We just need
                // to call different functions, eg. GetProgramiv instead of GetShaderiv.

                let mut buffer_len = 0;
                gl::GetProgramiv(program.id(), gl::INFO_LOG_LENGTH, &mut buffer_len);

                let mut buffer = vec![0u8; buffer_len as usize];
                gl::GetProgramInfoLog(program.id(), buffer_len, ptr::null_mut(), buffer.as_mut_ptr() as *mut GLchar);

                let log = ffi::CStr::from_bytes_with_nul(&buffer)
                    .map_err(|_| ProgramCreationError::InvalidInfoLog)?
                    .to_string_lossy()
                    .to_string();

                Err(ProgramCreationError::LinkError(log))
            }
        }
    }

    pub fn activate(&self) {
        unsafe { gl::UseProgram(self.id); }
    }

    pub fn deactivate(&self) {
        unsafe { gl::UseProgram(0); }
    }
}

pub enum ShaderType {
    Vertex,
    Fragment,
    // TODO
}

impl From<ShaderType> for GLenum {
    fn from(shader_type: ShaderType) -> Self {
        match shader_type {
            ShaderType::Vertex   => gl::VERTEX_SHADER,
            ShaderType::Fragment => gl::FRAGMENT_SHADER,
        }
    }
}

#[derive(Debug)]
pub enum ShaderCreationError {
    InvalidSource,
    CompileError(String),
    InvalidInfoLog,
}

pub struct Shader {
    id: GLuint,
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe { gl::DeleteShader(self.id); }
    }
}

impl GlObject for Shader {
    fn id(&self) -> GLuint {
        self.id
    }
}

impl Shader {
    pub fn new(ty: ShaderType, source: &str) -> Result<Shader, ShaderCreationError> {
        let c_source = ffi::CString::new(source).map_err(|_| ShaderCreationError::InvalidSource)?;

        let shader = Shader {
            id: unsafe { gl::CreateShader(ty.into()) },
        };

        let mut compile_status = gl::FALSE as GLint;

        unsafe {
            gl::ShaderSource(shader.id, 1, &c_source.as_ptr(), ptr::null());
            gl::CompileShader(shader.id);
            gl::GetShaderiv(shader.id, gl::COMPILE_STATUS, &mut compile_status);
        }

        if compile_status == (gl::TRUE as GLint) {
            Ok(shader)
        } else {
            let mut len = 0;
            unsafe { gl::GetShaderiv(shader.id, gl::INFO_LOG_LENGTH, &mut len); }

            let mut buf = vec![0u8; len as usize];
            unsafe { gl::GetShaderInfoLog(shader.id, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar); }

            let log = ffi::CStr::from_bytes_with_nul(&buf)
                .map_err(|_| ShaderCreationError::InvalidInfoLog)?
                .to_string_lossy()
                .to_string();


            Err(ShaderCreationError::CompileError(log))
        }
    }
}

// I'd like to support the following:
// 1. Compile from source (string).
// 2. Compile from files.
// 3. Compile from files with live reload.
struct SourceCompiler {}

impl SourceCompiler {

}


struct FileCompiler {}
struct WatchingCompiler {}
