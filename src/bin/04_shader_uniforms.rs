extern crate glutin;
extern crate gl;

use std::mem;
use std::ptr;
use std::time::Instant;

use glutin::{Window, Event};
use gl::types::*;

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

uniform vec4 our_color;

void main() {
    color = our_color;
}
"#;

const VERTICES: [GLfloat; 9] = [
    -0.5, -0.5, 0.0,
     0.5, -0.5, 0.0,
     0.0,  0.5, 0.0,
];

fn main() {
    // In this demo, we use a uniform variable to make the triangle color a pulsing green.
    // Because we are only changing the color of the triangle, the uniform is only used in the
    // fragment shader (see the source above).

    let window  = create_window("Shader Uniforms");
    let program = create_program(VERTEX_SHADER_SRC, FRAGMENT_SHADER_SRC);

    let mut vao = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);
    }

    let vbo = create_vbo(&VERTICES);

    unsafe {
        let stride = (3 * mem::size_of::<GLfloat>()) as GLsizei;
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null());
        gl::EnableVertexAttribArray(0);
        gl::BindVertexArray(0)
    }

    // Retrieve the uniform location. We'll need this to be able to update its value later on.
    let vertex_color_location = unsafe {
        gl::GetUniformLocation(program, "our_color".as_ptr() as *const GLchar)
    };

    let start = Instant::now();

    // As we're going to update the triangle color every frame, we're no longer waiting for window
    // events. Instead, we use an infinite loop and ask glutin if there are new events every time_since
    // before we render.
    //
    // This 'gameloop thingy is a label that we can use to we can break out of the loop.
    'gameloop: loop {
        for event in window.poll_events() {
            if let Event::Closed = event {
                break 'gameloop;
            }
        }

        // Calculate the green color component.
        let green_value = seconds_since(start).sin() / 2.0 + 0.5;

        unsafe {
            gl::UseProgram(program);

            // Modify the uniform value.
            gl::Uniform4f(vertex_color_location, 0.0, green_value, 0.0, 1.0);

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

fn seconds_since(instant: Instant) -> f32 {
    let elapsed = instant.elapsed();
    let elapsed_in_ms = (elapsed.as_secs() * 1_000 + (elapsed.subsec_nanos() / 1_000_000) as u64) as f32;

    elapsed_in_ms / 1_000.0
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
