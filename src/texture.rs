use gl::types::GLuint;
use image;
use image::EncodableLayout;

pub struct Texture
{
    pub id: GLuint,
}

impl Drop for Texture
{
    fn drop(&mut self)
    {
        unsafe { gl::DeleteTextures(1, [self.id].as_ptr()) }
    }
}

impl Texture
{
    pub unsafe fn new() -> Self
    {
        let mut id: GLuint = 0;
        gl::GenTextures(1, &mut id);
        Self { id }
    }

    pub unsafe fn load(&self)
    {
        self.bind();

        let img = image::open("desert_mountains.png").unwrap();
        let bit_map = img.into_rgba8();

        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA as i32,
            bit_map.width() as i32,
            bit_map.height() as i32,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            bit_map.as_bytes().as_ptr() as *const _,
        );
        gl::GenerateMipmap(gl::TEXTURE_2D);
    }

    pub unsafe fn set_wrap_settings(&self)
    {
        self.bind();
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
    }

    pub unsafe fn set_filter_settings(&self)
    {
        self.bind();
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
    }

    pub unsafe fn bind(&self)
    {
        gl::BindTexture(gl::TEXTURE_2D, self.id)
    }

    pub unsafe fn activate(&self)
    {
        gl::ActiveTexture(gl::TEXTURE0);
        self.bind();
    }
}
