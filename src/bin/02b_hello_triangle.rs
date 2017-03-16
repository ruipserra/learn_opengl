extern crate learn_opengl as lgl;
extern crate gl;

use std::mem;
use std::ptr;

use lgl::{create_window, Event};
use lgl::program::{SourceCompiler, ShaderType};

use gl::types::*;

const VERTICES: [GLfloat; 9] = [
    -0.5, -0.5, 0.0,
     0.5, -0.5, 0.0,
     0.0,  0.5, 0.0,
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
    let window = create_window("Hello Triangle");

    let program = SourceCompiler::compile(&[
        (ShaderType::Vertex, VERTEX_SHADER_SRC),
        (ShaderType::Fragment, FRAGMENT_SHADER_SRC),
    ]).unwrap();

    // Next, we'll upload the triangle vertices to the GPU, where they'll be processed by the program
    // we just linked.
    //
    // But before we do that, we'll create a Vertex Array Object. This will let us store vertex
    // attribute pointer configuration so that we only have to do it once. Whenever we use the VBO
    // again, we only have to make sure the corresponding VAO is also bound for the same
    // configuration to be used.

    let mut vao = 0;
    unsafe {
        // Create and bind a VAO
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);
    }

    // Create a Vertex Buffer Object and upload the vertices to the GPU.
    let vbo = create_vbo(&VERTICES);

    unsafe {
        // And now, we configure the vertex attributes. This instructs the vertex shader how to
        // interpret the vertex data.
        let stride = (3 * mem::size_of::<GLfloat>()) as GLsizei;

        gl::VertexAttribPointer(
            0, // The index of the generic vertex attribute to be modified.
            3, // The number of components per generic vertex attribute. vec3, in this case.
            gl::FLOAT,  // The data type of each component in the array
            gl::FALSE,  // Whether the data values should be normalized.
            stride,     // The byte offset between consecutive generic vertex attributes.
                        // We could have passed 0, which would mean the data is tightly packed.
            ptr::null() // Offset of the first component of the first generic vertex attribute.
                        // NULL means 0.
        );

        gl::EnableVertexAttribArray(0);
    }

    program.activate();

    for event in window.wait_events() {
        unsafe {
            // Lastly, we draw the object.
            gl::ClearColor(0.3, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
        }

        window.swap_buffers().unwrap();

        if let Event::Closed = event {
            break;
        }
    }

    unsafe {
        // And let's not forget to cleanup after ourselves.
        gl::DeleteBuffers(1, &vbo);
        gl::DeleteVertexArrays(1, &vao);
    }
}

fn create_vbo(vertices: &[GLfloat]) -> GLuint {
    unsafe {
        // 1. Create a buffer.
        let mut vbo = 0;
        gl::GenBuffers(1, &mut vbo);

        // 2. Bind the buffer as a vertex buffer.
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

        // 3. Copy the vertices into the buffer.
        gl::BufferData(
            gl::ARRAY_BUFFER,
            mem::size_of_val(vertices) as GLsizeiptr, // Byte size of the vertex data.
            vertices.as_ptr() as *const GLvoid,        // Pointer to the vertex data to copy.
            gl::STATIC_DRAW // Expected usage pattern for this data. In this example, the data
                            // doesn't change and is used for drawing.
        );

        // Return the VBO handle
        vbo
    }
}
