extern crate glutin;
extern crate gl;

use std::mem;
use std::ptr;

use glutin::{Window, Event};
use gl::types::*;

const VERTEX_SHADER_SRC: &'static str = r#"
#version 330 core

layout (location = 0) in vec3 position;
layout (location = 1) in vec3 color;

out vec3 our_color;

void main() {
    gl_Position = vec4(position, 1.0);
    our_color = color;
}
"#;

const FRAGMENT_SHADER_SRC: &'static str = r#"
#version 330 core

in vec3 our_color;

out vec4 color;

void main() {
    color = vec4(our_color, 1.0);
}
"#;

const VALUES_PER_VERTEX: usize = 6;
const VERTEX_DATA: [GLfloat; 3 * VALUES_PER_VERTEX] = [
    // Positions        // Colors
    -0.5, -0.5, 0.0,    1.0, 0.0, 0.0, // Bottom right
     0.5, -0.5, 0.0,    0.0, 1.0, 0.0, // Bottom left
     0.0,  0.5, 0.0,    0.0, 0.0, 1.0, // Top
];

fn main() {
    // In this demo, we'll upload a different color for each vertex in the Vertex Buffer Object.
    // The vertex shader has changed to include a new input variable (color) and an output variable
    // (our_color), so that we can pass the color to the fragment shader.

    let window  = create_window("Colors in vertex data");
    let program = create_program(VERTEX_SHADER_SRC, FRAGMENT_SHADER_SRC);

    let mut vao = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);
    }

    let vbo = create_vbo(&VERTEX_DATA);

    unsafe {
        let gl_float_size = mem::size_of::<GLfloat>();
        let stride = (VALUES_PER_VERTEX * gl_float_size) as GLsizei;

        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, 0 as *const GLvoid);
        gl::EnableVertexAttribArray(0);

        // As with the position vertex attribute, we need to configure the color attribute. We
        // specified the attribute location manually, so we don't need to query it.
        gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, stride, (3 * gl_float_size) as *const GLvoid);
        gl::EnableVertexAttribArray(1);

        gl::BindVertexArray(0)
    }

    for event in window.wait_events() {
        if let Event::Closed = event {
            break;
        }

        unsafe {
            gl::UseProgram(program);

            gl::ClearColor(0.3, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::BindVertexArray(vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
            gl::BindVertexArray(0);
        }

        window.swap_buffers().unwrap();
    }

    unsafe {
        gl::DeleteProgram(program);
        gl::DeleteBuffers(1, &vbo);
        gl::DeleteVertexArrays(1, &vao);
    }
}

fn create_window(title: &str) -> Window {
    use glutin::{Api, GlProfile, GlRequest, WindowBuilder};

    let window = WindowBuilder::new()
        .with_gl(GlRequest::Specific(Api::OpenGl, (3, 3)))
        .with_gl_profile(GlProfile::Core)
        .with_title(title)
        .build()
        .unwrap();

    unsafe { window.make_current().unwrap() };

    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    window
}

fn create_program(vertex_shader: &str, fragment_shader: &str) -> GLuint {
    // Yes, I know I'm ignoring these errors. This is just a demo, not production-grade stuff.
    let vertex_shader   = compile_shader(vertex_shader, gl::VERTEX_SHADER).unwrap();
    let fragment_shader = compile_shader(fragment_shader, gl::FRAGMENT_SHADER).unwrap();
    let program         = link_program(vertex_shader, fragment_shader).unwrap();

    cleanup_shader(vertex_shader, program);
    cleanup_shader(fragment_shader, program);

    program
}

fn compile_shader(src: &str, shader_type: GLenum) -> Result<GLuint, String> {
    use std::ffi::CString;

    let src_as_cstring = CString::new(src.as_bytes()).unwrap();
    let mut compile_status = gl::FALSE as GLint;
    let shader;

    unsafe {
        shader = gl::CreateShader(shader_type);
        gl::ShaderSource(shader, 1,&src_as_cstring.as_ptr(), ptr::null());
        gl::CompileShader(shader);
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut compile_status);
    }

    if compile_status == (gl::TRUE as GLint) {
        Ok(shader)
    } else {
        unsafe {
            let mut buffer_len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut buffer_len);

            let mut buffer: Vec<u8> = Vec::with_capacity(buffer_len as usize);
            gl::GetShaderInfoLog(shader, buffer_len, &mut buffer_len, buffer.as_mut_ptr() as *mut GLchar);
            buffer.set_len(buffer_len as usize);

            Err(String::from_utf8(buffer).unwrap())
        }
    }
}

fn cleanup_shader(shader: GLuint, program: GLuint) {
    unsafe {
        gl::DetachShader(program, shader);
        gl::DeleteShader(shader);
    }
}

fn link_program(vertex_shader_id: GLuint, fragment_shader_id: GLuint) -> Result<GLuint, String> {
    let mut link_status = gl::FALSE as GLint;
    let program;

    unsafe {
        program = gl::CreateProgram();
        gl::AttachShader(program, vertex_shader_id);
        gl::AttachShader(program, fragment_shader_id);
        gl::LinkProgram(program);
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut link_status);
    }

    if link_status == (gl::TRUE as GLint) {
        Ok(program)
    } else {
        unsafe {
            let mut buffer_len = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut buffer_len);

            let mut buffer: Vec<u8> = Vec::with_capacity(buffer_len as usize);
            gl::GetProgramInfoLog(program, buffer_len, &mut buffer_len, buffer.as_mut_ptr() as *mut GLchar);
            buffer.set_len(buffer_len as usize);

            Err(String::from_utf8(buffer).unwrap())
        }
    }
}

fn create_vbo(vertices: &[GLfloat]) -> GLuint {
    unsafe {
        let mut vbo = 0;
        gl::GenBuffers(1, &mut vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            mem::size_of_val(vertices) as GLsizeiptr,
            vertices.as_ptr() as *const GLvoid,
            gl::STATIC_DRAW
        );

        vbo
    }
}
