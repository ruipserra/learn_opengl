extern crate glutin;
extern crate gl;

use glutin::{
    Api,
    Event,
    GlProfile,
    GlRequest,
    WindowBuilder,
};

fn main() {
    // Glutin gives us a `WindowBuilder` that we can use to configure and create a window. Below,
    // w're going to ask for OpenGL 3.3 Core profile.
    let window = WindowBuilder::new()
        .with_gl(GlRequest::Specific(Api::OpenGl, (3, 3)))
        .with_gl_profile(GlProfile::Core)
        .with_title("Hello Window")
        .with_dimensions(800, 600)
        .build()
        .unwrap();

    // We have to set the window as the current window before loading the OpenGL function pointers.
    unsafe { window.make_current() };

    // Now load the OpenGL function pointers.
    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    // Let's clear the window. `gl` functions are basically FFI calls, so these functions are
    // unsafe.
    unsafe {
        gl::ClearColor(0.3, 0.3, 0.3, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);
    }

    // The commands above are applied to the back buffer, and are not visible until we swap
    // buffers.
    //
    // With double buffering, we first prepare a frame by applying commands to it, and then render
    // that frame on the screen.
    //
    // This prevents glitches that could happen when trying to render many commands directly to the
    // screen.
    window.swap_buffers().unwrap();

    // And now we wait for events, so we can terminate once the window is closed.
    for event in window.wait_events() {
        if let Event::Closed = event {
            break;
        }
    }
}
