use gl::types::GLuint;

pub struct Texture {
    id: GLuint,
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe { gl::DeleteTextures(1, [self.id].as_ptr()) }
    }
}
