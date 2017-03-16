extern crate glutin;
extern crate gl;

pub mod debug;
pub mod program;

mod gl_object;

pub use glutin::Event;

pub fn create_window(title: &str) -> glutin::Window {
    use glutin::{Api, GlProfile, GlRequest, WindowBuilder};

    let window = WindowBuilder::new()
        .with_dimensions(1024, 768)
        .with_title(title)
        .with_gl(GlRequest::Specific(Api::OpenGl, (3, 3)))
        .with_gl_profile(GlProfile::Core)
        .with_vsync()
        .build()
        .unwrap();

    unsafe { window.make_current().unwrap() };

    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    window
}
