extern crate glutin;
extern crate gl;
extern crate image;

use std::mem;
use std::ptr;
use std::path::Path;

use glutin::{Window, Event};
use gl::types::*;

use image::GenericImage;

const POSITION_LOCATION:  u8 = 0;
const COLOR_LOCATION:     u8 = 1;
const TEX_COORD_LOCATION: u8 = 2;

const VERTEX_SHADER_SRC: &'static str = r#"
#version 330 core

layout (location = 0) in vec3 vbo_position;
layout (location = 1) in vec3 vbo_color;
layout (location = 2) in vec2 vbo_tex_coord;

out vec3 color;
out vec2 tex_coord;

void main() {
    gl_Position = vec4(vbo_position, 1.0);

    color = vbo_color;
    tex_coord = vec2(vbo_tex_coord.x, 1.0 - vbo_tex_coord.y);
}
"#;

const FRAGMENT_SHADER_SRC: &'static str = r#"
#version 330 core

in vec3 color;
in vec2 tex_coord;

uniform sampler2D our_texture_1;
uniform sampler2D our_texture_2;

out vec4 frag_color;

void main() {
    frag_color = mix(
        mix(
            texture(our_texture_1, tex_coord),
            texture(our_texture_2, tex_coord),
            0.2
        ),
        vec4(color, 1.0),
        0.2
    );
}
"#;

const VALUES_PER_POSITION:  u8 = 3;
const VALUES_PER_COLOR:     u8 = 3;
const VALUES_PER_TEX_COORD: u8 = 2;

const VALUES_PER_VERTEX: usize = (VALUES_PER_POSITION + VALUES_PER_COLOR + VALUES_PER_TEX_COORD) as usize;
const VERTEX_COUNT: usize = 4;
const VERTEX_DATA: [GLfloat; VERTEX_COUNT * VALUES_PER_VERTEX] = [
    // Positions        // Colors           // Texture coords
    -0.5, -0.5, 0.0,    1.0, 0.0, 0.0,      0.0, 0.0, // Bottom left
     0.5, -0.5, 0.0,    0.0, 1.0, 0.0,      1.0, 0.0, // Bottom right
    -0.5,  0.5, 0.0,    0.0, 0.0, 1.0,      0.0, 1.0, // Top Left
     0.5,  0.5, 0.0,    1.0, 0.0, 1.0,      1.0, 1.0, // Top Right
];

const INDICES: [GLuint; 6] = [
    0, 1, 2, // First triangle
    1, 2, 3, // Second triangle
];

fn main() {
    let window  = create_window("Textures");
    let program = create_program(VERTEX_SHADER_SRC, FRAGMENT_SHADER_SRC);

    let mut vao = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);
    }

    let vbo = create_vbo(&VERTEX_DATA);
    let ebo = create_ebo(&INDICES);

    unsafe {
        let gl_float_size = mem::size_of::<GLfloat>();
        let stride = (VALUES_PER_VERTEX * gl_float_size) as GLsizei;

        // The `vbo_position` attribute
        gl::VertexAttribPointer(POSITION_LOCATION as GLuint, VALUES_PER_POSITION as GLint, gl::FLOAT,
            gl::FALSE, stride, (0 * gl_float_size) as *const GLvoid);
        gl::EnableVertexAttribArray(POSITION_LOCATION as GLuint);

        // The `vbo_color` attribute
        gl::VertexAttribPointer(COLOR_LOCATION as GLuint, VALUES_PER_COLOR as GLint, gl::FLOAT,
            gl::FALSE, stride, (3 * gl_float_size) as *const GLvoid);
        gl::EnableVertexAttribArray(COLOR_LOCATION as GLuint);

        // The `vbo_tex_coords` attribute
        gl::VertexAttribPointer(TEX_COORD_LOCATION as GLuint, VALUES_PER_TEX_COORD as GLint,
            gl::FLOAT, gl::FALSE, stride, (6 * gl_float_size) as *const GLvoid);
        gl::EnableVertexAttribArray(TEX_COORD_LOCATION as GLuint);

        gl::BindVertexArray(0)
    }

    // Load the box and face textures from their files.
    let texture_box  = create_texture_from_image_file("assets/textures/container.jpg");
    let texture_face = create_texture_from_image_file("assets/textures/awesomeface_no-alpha.png");

    'gameloop: loop {
        for event in window.poll_events() {
            if let Event::Closed = event {
                break 'gameloop;
            }
        }

        unsafe {
            gl::UseProgram(program);

            gl::ClearColor(0.3, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // Bind the box texture to the `our_texture_1` uniform sampler.
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, texture_box);
            gl::Uniform1i(gl::GetUniformLocation(program, "our_texture_1".as_ptr() as *const GLchar), 0);

            // Bind the face texture to the `our_texture_2` uniform sampler.
            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, texture_face);
            gl::Uniform1i(gl::GetUniformLocation(program, "our_texture_2".as_ptr() as *const GLchar), 1);

            // Draw the rectangle.
            gl::BindVertexArray(vao);
            gl::DrawElements(gl::TRIANGLES, INDICES.len() as GLsizei, gl::UNSIGNED_INT, 0 as *const GLvoid);
            gl::BindVertexArray(0);
        }

        window.swap_buffers().unwrap();
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
        let mut ebo = 0;

        gl::GenBuffers(1, &mut ebo);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            mem::size_of_val(indices) as GLsizeiptr,
            indices.as_ptr() as *const GLvoid,
            gl::STATIC_DRAW
        );

        ebo
    }
}

fn create_texture_from_image_file(path: &str) -> GLuint {
    // I'm using the `image` crate to read image data.
    let img = image::open(&Path::new(path)).unwrap();
    let (width, height) = img.dimensions();

    let mut texture = 0;
    unsafe {
        // 1. Create a texture
        gl::GenTextures(1, &mut texture);

        // 2. Bind the texture target to TEXTURE_2D.
        gl::BindTexture(gl::TEXTURE_2D, texture);

        // 3. Load the image data.
        gl::TexImage2D(
            gl::TEXTURE_2D,     // The texture target.
            0,                  // Level of detail. Level 0 is the base image.
            gl::RGB as GLint,   // Internal format. Specifies the number of colors in the texture.
            width as GLsizei,
            height as GLsizei,
            0,                  // border. Must be 0, always.
            gl::RGB,            // The format of the pixel data.
            gl::UNSIGNED_BYTE,  // The data type of the pixel data.
            img.raw_pixels().as_ptr() as *const GLvoid // A pointer to the image in memory.
        );

        // 4. Generate the Mipmap.
        gl::GenerateMipmap(gl::TEXTURE_2D);

        // 5. Unbind the texture (so we don't accidentally modify it).
        gl::BindTexture(gl::TEXTURE_2D, 0);
    }

    texture
}
