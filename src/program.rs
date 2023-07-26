use std::ffi::CString;

use gl::{
    types::{GLint, GLuint},
    FALSE,
};
use glm::Mat4;

use crate::shader::{Shader, ShaderError};

pub struct Program {
    pub id: GLuint,
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe { gl::DeleteProgram(self.id) }
    }
}

impl Program {
    pub unsafe fn new(shaders: &[Shader]) -> Result<Self, ShaderError> {
        let program = Self {
            id: gl::CreateProgram(),
        };

        for shader in shaders {
            gl::AttachShader(program.id, shader.id);
        }
        gl::LinkProgram(program.id);

        let mut success: GLint = 0;
        gl::GetProgramiv(program.id, gl::LINK_STATUS, &mut success);

        if success == 1 {
            Ok(program)
        } else {
            let mut error_size: GLint = 0;
            gl::GetProgramiv(program.id, gl::INFO_LOG_LENGTH, &mut error_size);
            let mut message = Vec::with_capacity(error_size as usize);
            gl::GetProgramInfoLog(
                program.id,
                error_size,
                &mut error_size,
                message.as_mut_ptr() as *mut _,
            );

            message.set_len(error_size as usize);
            let log = String::from_utf8(message).unwrap();
            Err(ShaderError::LinkingError(log))
        }
    }

    pub unsafe fn set_uniform_int(&self, name: &str, value: i32) {
        let c_str = CString::new(name).unwrap();
        gl::Uniform1i(gl::GetUniformLocation(self.id, c_str.as_ptr()), value);
    }

    pub unsafe fn set_uniform_mat4(&self, name: &str, value: Mat4) {
        let c_str = CString::new(name).unwrap();
        let uniform = gl::GetUniformLocation(self.id, c_str.as_ptr() as *const i8);
        gl::UniformMatrix4fv(uniform, 1, FALSE, value.as_array().as_ptr() as *const _);
    }

    pub unsafe fn set_uniform_vec3(&self, name: &str, value: glm::Vector3<f32>) {
        let c_str = CString::new(name).unwrap();
        let uniform = gl::GetUniformLocation(self.id, c_str.as_ptr() as *const i8);
        gl::Uniform3f(uniform, value.x, value.y, value.z);
    }
}
