use std::fs;

use gl::types::{GLenum, GLint, GLuint};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ShaderError {
    #[error("Error while reading shader source file: {0}")]
    ReadingFileError(String),
    #[error("Error while compiling shader: {0}")]
    CompileError(String),
    #[error("Error while linking program: {0}")]
    LinkingError(String),
}

pub struct Shader {
    pub id: GLuint,
}

impl Shader {
    pub unsafe fn new(path_to_source_code: &str, shader_type: GLenum) -> Result<Self, ShaderError> {
        let source_code = match fs::read_to_string(path_to_source_code) {
            Ok(source) => source,
            Err(error) => return Err(ShaderError::ReadingFileError(error.to_string())),
        };
        let shader = Self {
            id: gl::CreateShader(shader_type),
        };
        gl::ShaderSource(
            shader.id,
            1,
            [source_code.as_ptr().cast()].as_ptr(),
            std::ptr::null(),
        );
        gl::CompileShader(shader.id);
        let mut success: GLint = 0;
        gl::GetShaderiv(shader.id, gl::COMPILE_STATUS, &mut success);
        if success == 1 {
            Ok(shader)
        } else {
            let mut error_size: GLint = 0;
            gl::GetShaderiv(shader.id, gl::INFO_LOG_LENGTH, &mut error_size);
            let mut message = Vec::with_capacity(error_size as usize);
            gl::GetShaderInfoLog(
                shader.id,
                error_size,
                &mut error_size,
                message.as_mut_ptr() as *mut _,
            );

            message.set_len(error_size as usize);
            let log = String::from_utf8(message).unwrap();
            Err(ShaderError::CompileError(log))
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe { gl::DeleteShader(self.id) }
    }
}
