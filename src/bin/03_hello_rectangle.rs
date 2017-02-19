extern crate glutin;
extern crate gl;

use std::mem;
use std::ptr;

use glutin::{Window, Event};

use gl::types::*;

const VERTICES: [GLfloat; 12] = [
     0.5,  0.5, 0.0, // Top Right
     0.5, -0.5, 0.0, // Bottom Right
    -0.5, -0.5, 0.0, // Bottom Left
    -0.5,  0.5, 0.0, // Top Left
];

const INDICES: [GLuint; 6] = [
    0, 1, 3, // First triangle
    1, 2, 3, // Second triangle
];

const VERTEX_SHADER_SRC: &'static str = r#"
#version 330 core

layout (location = 0) in vec3 position;

void main() {
    gl_Position = vec4(position, 1.0);
}
"#;

const FRAGMENT_SHADER_SRC: &'static str = r#"
#version 330 core

out vec4 color;

void main() {
    color = vec4(1.0f, 0.5f, 0.2f, 1.0f);
}
"#;

fn main() {
    // This example is similar to the "Hello Triangle" one, but we use an Element Buffer Object to
    // render a Rectangle without needing to upload the shared vertices.

    let window  = create_window("Hello Rectangle");
    let program = create_program(VERTEX_SHADER_SRC, FRAGMENT_SHADER_SRC);

    let mut vao = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);
    }

    let vbo = create_vbo(&VERTICES);
    let ebo = create_ebo(&INDICES);

    unsafe {
        let stride = (3 * mem::size_of::<GLfloat>()) as GLsizei;
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null());
        gl::EnableVertexAttribArray(0);

        // Unbind the current VAO.
        //
        // While this ins not actually needed for this demo, it would prevent us from accidentally
        // messing around the VAO.
        gl::BindVertexArray(0)
    }

    // Uncomment this line to activate wireframe mode.
    // unsafe { gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE); }

    for event in window.wait_events() {
        unsafe {
            gl::UseProgram(program);

            gl::ClearColor(0.3, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // Rebind the VAO for this draw.
            gl::BindVertexArray(vao);

            // Instead of gl::DrawArrays, we use gl::DrawElements.
            gl::DrawElements(
                gl::TRIANGLES,            // The type of primitive to render.
                INDICES.len() as GLsizei, // The number of elements to be displayed.
                gl::UNSIGNED_INT,         // The type of each index value.
                0 as *const GLvoid        // An offset to the first index.
            );

            // Unbind VAO.
            gl::BindVertexArray(0);
        }

        window.swap_buffers().unwrap();

        if let Event::Closed = event {
            break;
        }
    }

    unsafe {
        gl::DeleteProgram(program);
        gl::DeleteBuffers(1, &vbo);
        gl::DeleteBuffers(1, &ebo);
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

fn create_ebo(indices: &[GLuint]) -> GLuint {
    unsafe {
        // 1. Create the EBO.
        let mut ebo = 0;
        gl::GenBuffers(1, &mut ebo);

        // 2. Bind the buffer as an Element Buffer.
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);

        // 3. Copy the indices into the buffer.
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            mem::size_of_val(indices) as GLsizeiptr,
            indices.as_ptr() as *const GLvoid,
            gl::STATIC_DRAW
        );

        // Return the EBO handle
        ebo
    }
}
