use gl::types::GLuint;

pub type Handle = GLuint;

pub trait GlObject: Drop {
    fn id(&self) -> Handle;
}
