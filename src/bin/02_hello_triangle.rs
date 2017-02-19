extern crate glutin;
extern crate gl;

use std::mem;
use std::ptr;

use glutin::{Window, Event};

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
    // We already know how to create a window, so let's go ahead and just do it.
    let window = create_window("Hello Triangle");

    // OpenGL uses a graphics pipeline to transform 3D into colored pixels. We can hook into the
    // pipeline steps by writing our own shaders.
    //
    // OpenGL requires us to provide a vertex and fragment shader.
    let vertex_shader   = compile_shader(VERTEX_SHADER_SRC, gl::VERTEX_SHADER).unwrap();
    let fragment_shader = compile_shader(FRAGMENT_SHADER_SRC, gl::FRAGMENT_SHADER).unwrap();

    // Now we construct a program from the compiled shaders. The program knows how to feed data
    // from and to shaders.
    let program = link_program(vertex_shader, fragment_shader).unwrap();

    // After linking the program, we can clean up the shaders so that OpenGL can free up memory.
    cleanup_shader(vertex_shader, program);
    cleanup_shader(fragment_shader, program);

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

        // Tell OpenGL to use the program in the rendering pipeline.
        gl::UseProgram(program);
    }

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

fn compile_shader(src: &str, shader_type: GLenum) -> Result<GLuint, String> {
    use std::ffi::CString;

    // Transform the shader source into a C-compatible string.
    let src_as_cstring = CString::new(src.as_bytes()).unwrap();

    let mut compile_status = gl::FALSE as GLint;
    let shader;

    unsafe {
        // 1. Ask OpenGL to create a shader object.
        shader = gl::CreateShader(shader_type);

        // 2. Load the source code for the shader.
        gl::ShaderSource(
            shader, // The shader handle
            1,      // Source string count. Our source is a single string.
            &src_as_cstring.as_ptr(), // An array of pointers to strings containing to the shader
                                      // source.
            ptr::null() // An array of string lengths. When NULL, the source strings are assumed to
                        // be NUL-terminated.
        );

        // 3. Compile the shader.
        gl::CompileShader(shader);

        // 4. Ask whether compilation succeeded.
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut compile_status);
    }

    if compile_status == (gl::TRUE as GLint) {
        // Compilation succeeded, we can return the shader handle.
        Ok(shader)
    } else {
        unsafe {
            // Compilation failed. We'll ask OpenGL why.

            // We'll have to allocate some memory to hold the info log. Let's query the log size.
            let mut buffer_len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut buffer_len);

            // Allocate memory to store the log.
            let mut buffer: Vec<u8> = Vec::with_capacity(buffer_len as usize);

            // Now let's get the log.
            gl::GetShaderInfoLog(
                shader,             // The shader whose info log we want.
                buffer_len,         // Here we specify the log buffer size.
                &mut buffer_len,    // And here we allow OpenGL to modify buffer_len to be the size
                                    // of the log string, excluding the NUL character.
                buffer.as_mut_ptr() as *mut GLchar // The character array to hold the log.
            );

            // Set the length of the buffer. The vec needs to know the length of its data,
            // otherwise it'll think it's still empty. Notice that we haven't manipulated this vec
            // in the usual way (pushing, inserting, etc.), that's why this information is needed.
            buffer.set_len(buffer_len as usize);

            // Convert the character vec to an owned string, and return it as an error.
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
        // 1. Create a program object.
        program = gl::CreateProgram();

        // 2. Attach the shaders. Notice we don't need to specify their type, as OpenGL already has
        // that information.
        gl::AttachShader(program, vertex_shader_id);
        gl::AttachShader(program, fragment_shader_id);

        // 3. Link the program.
        gl::LinkProgram(program);

        // 4. Retrieve the link status.
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut link_status);
    }

    if link_status == (gl::TRUE as GLint) {
        Ok(program)
    } else {
        unsafe {
            // Getting the program info log is similar to getting the shader info log. We just need
            // to call different functions, eg. GetProgramiv instead of GetShaderiv.

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
