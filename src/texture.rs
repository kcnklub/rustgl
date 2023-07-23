use gl::types::GLuint;
use image;

pub struct Texture {
    id: GLuint,
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe { gl::DeleteTextures(1, [self.id].as_ptr()) }
    }
}

impl Texture {
    pub unsafe fn new() -> Self {
        let mut id: GLuint = 0;
        gl::GenTextures(1, &mut id);
        Self { id }
    }

    pub unsafe fn load(&self) {
        self.bind();

        let img = image::open("desert_mountains.png").unwrap().into_rgb8();
    }

    pub unsafe fn bind(&self) {
        gl::BindTexture(gl::TEXTURE_2D, self.id)
    }
}
