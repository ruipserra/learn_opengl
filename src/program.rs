use gl;
use gl::types::*;

use std::ffi;
use std::ptr;

use gl_object::{GlObject, Handle};

#[derive(Debug)]
pub enum ProgramCreationError {
    LinkError(String),
    InvalidInfoLog,
}

pub struct Program {
    id: Handle,
}

impl Drop for Program {
    fn drop(&mut self) {
        self.deactivate();

        unsafe { gl::DeleteProgram(self.id); }
    }
}

impl GlObject for Program {
    #[inline]
    fn id(&self) -> Handle {
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

#[derive(Clone, Copy)]
pub enum ShaderType {
    Vertex,
    Fragment,
    Geometry,
    TessControl,
    TessEvaluation,
    Compute,
}

impl From<ShaderType> for GLenum {
    fn from(shader_type: ShaderType) -> Self {
        match shader_type {
            ShaderType::Vertex         => gl::VERTEX_SHADER,
            ShaderType::Fragment       => gl::FRAGMENT_SHADER,
            ShaderType::Geometry       => gl::GEOMETRY_SHADER,
            ShaderType::TessControl    => gl::TESS_CONTROL_SHADER,
            ShaderType::TessEvaluation => gl::TESS_EVALUATION_SHADER,
            ShaderType::Compute        => gl::COMPUTE_SHADER,

        }
    }
}

impl ShaderType {
    fn from_extension(ext: &str) -> Option<ShaderType> {
        match ext {
            ".vert" | ".vs.glsl" => Some(ShaderType::Vertex),
            ".frag" | ".fs.glsl" => Some(ShaderType::Fragment),
            ".geom" | ".gs.glsl" => Some(ShaderType::Geometry),
            ".tesc" | ".tc.glsl" => Some(ShaderType::TessControl),
            ".tese" | ".te.glsl" => Some(ShaderType::TessEvaluation),
            ".comp" | ".cs.glsl" => Some(ShaderType::Compute),
            _ => None
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
    id: Handle,
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe { gl::DeleteShader(self.id); }
    }
}

impl GlObject for Shader {
    #[inline]
    fn id(&self) -> Handle {
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

#[derive(Debug)]
pub enum SourceCompilerError {
    ShaderCreationError(ShaderCreationError),
    ProgramCreationError(ProgramCreationError),
}

pub struct SourceCompiler {}

impl SourceCompiler {
    pub fn compile(shader_sources: &[(ShaderType, &str)]) -> Result<Program, SourceCompilerError> {
        let mut shaders = Vec::new();

        for &(ty, source) in shader_sources {
            let shader = Shader::new(ty, source)
                .map_err(|e| SourceCompilerError::ShaderCreationError(e))?;

            shaders.push(shader);
        }

        Program::link(&shaders).map_err(|e| SourceCompilerError::ProgramCreationError(e))
    }
}

// TODO
// struct FileCompiler {}
// struct WatchingCompiler {}
