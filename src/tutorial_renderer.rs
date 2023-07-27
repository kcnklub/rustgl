use std::{alloc::System, time::{SystemTime, Instant}};

use gl::types::GLuint;
use glm::Vec3;

use crate::{program::Program, shader::{ShaderError, Shader}, camera::Camera};

pub struct TutorialRenderer {
    program: Program,
    lighting_program: Program,
    vao: GLuint,
    vbo: GLuint,
    lighting_vao: GLuint,
    light_position: Vec3
}

impl TutorialRenderer {

    pub fn new() -> Result<Self, ShaderError> {
        unsafe {
            let vertex_shader = Shader::new("./resources/shaders/lighting/colors.vert", gl::VERTEX_SHADER)?;
            let fragment_shader = Shader::new("./resources/shaders/lighting/colors.frag", gl::FRAGMENT_SHADER)?;
            let program = Program::new(&[vertex_shader, fragment_shader])?;

            let lighting_vertex_shader = Shader::new("./resources/shaders/lighting/light_cube.vert", gl::VERTEX_SHADER)?;
            let lighting_frag_shader = Shader::new("./resources/shaders/lighting/light_cube.frag", gl::FRAGMENT_SHADER)?;
            let lighting_program = Program::new(&[lighting_vertex_shader, lighting_frag_shader])?;

            let mut vao = std::mem::zeroed();
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            let mut vbo = std::mem::zeroed();
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (VERTEX_DATA.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                VERTEX_DATA.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            // vertex attri
            gl::VertexAttribPointer(
                0 as gl::types::GLuint,
                3,
                gl::FLOAT,
                0,
                6 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                std::ptr::null(),
            );
            gl::EnableVertexAttribArray(0 as gl::types::GLuint);
            gl::Enable(gl::DEPTH_TEST);

            let mut light_vao: GLuint = 0;
            gl::GenVertexArrays(1, &mut light_vao);
            gl::BindVertexArray(light_vao);

            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

            gl::VertexAttribPointer(
                0 as gl::types::GLuint,
                3,
                gl::FLOAT,
                0,
                6 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                std::ptr::null(),
            );
            gl::EnableVertexAttribArray(0 as gl::types::GLuint);

            gl::VertexAttribPointer(
                1 as gl::types::GLuint,
                3,
                gl::FLOAT,
                0,
                6 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                (3 * std::mem::size_of::<f32>()) as *const () as *const _,
                
            );
            gl::EnableVertexAttribArray(1 as gl::types::GLuint);

            return Ok(Self {
                program,
                lighting_program,
                vao,
                vbo,
                lighting_vao: light_vao, 
                light_position: glm::vec3(1.2, 1.0, 2.0)
            });
        }
    }

    pub fn draw(&mut self, camera: &Camera) {
        unsafe {
            let time = SystemTime::now();
            let time_since = time
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis();
            println!("{}", time_since as f64);
            let offset = glm::sin(glm::radians(time_since as f64 * 0.05));
            println!("{}", offset);

            let moving_light = glm::vec3(
                self.light_position.x, 
                self.light_position.y + (offset as f32),
                self.light_position.z);

            gl::ClearColor(0.2, 0.3, 0.3, 0.7);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            gl::UseProgram(self.program.id);
            self.program.set_uniform_vec3("objectColor", glm::vec3(1.0, 0.5, 0.31));
            self.program.set_uniform_vec3("lightColor", glm::vec3(1.0, 1.0, 1.0));
            self.program.set_uniform_vec3("lightPos", moving_light);

            let view = camera.get_view_matrix();
            self.program.set_uniform_mat4("view", view);

            let projection = glm::ext::perspective(glm::radians(camera.fov), 1920.0 / 1080.0, 0.1, 100.0);
            self.program.set_uniform_mat4("projection", projection);

            #[rustfmt::skip]
            let model = glm::mat4(
                1.0, 0.0, 0.0, 0.0, 
                0.0, 1.0, 0.0, 0.0, 
                0.0, 0.0, 1.0, 0.0, 
                0.0, 0.0, 0.0, 1.0,
            );
            self.program.set_uniform_mat4("model", model);

            gl::BindVertexArray(self.vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 36);

            gl::UseProgram(self.lighting_program.id);
            self.lighting_program.set_uniform_mat4("view", view);
            self.lighting_program.set_uniform_mat4("projection", projection);

            #[rustfmt::skip]
            let mut model = glm::mat4(
                1.0, 0.0, 0.0, 0.0, 
                0.0, 1.0, 0.0, 0.0, 
                0.0, 0.0, 1.0, 0.0, 
                0.0, 0.0, 0.0, 1.0,
            );
            model = glm::ext::translate(&model, moving_light);
            model = glm::ext::scale(&model, glm::vec3(0.2, 0.2, 0.2));
            self.lighting_program.set_uniform_mat4("model", model);

            gl::BindVertexArray(self.lighting_vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 36);
        }
    }

    pub fn resize(&self, width: i32, height: i32) {
        unsafe {
            gl::Viewport(0, 0, width, height);
        }
    }
}

#[rustfmt::skip]
const VERTEX_DATA: [f32; 216] = [
    -0.5, -0.5, -0.5,  0.0,  0.0, -1.0,
     0.5, -0.5, -0.5,  0.0,  0.0, -1.0,
     0.5,  0.5, -0.5,  0.0,  0.0, -1.0,
     0.5,  0.5, -0.5,  0.0,  0.0, -1.0,
    -0.5,  0.5, -0.5,  0.0,  0.0, -1.0,
    -0.5, -0.5, -0.5,  0.0,  0.0, -1.0,

    -0.5, -0.5,  0.5,  0.0,  0.0,  1.0,
     0.5, -0.5,  0.5,  0.0,  0.0,  1.0,
     0.5,  0.5,  0.5,  0.0,  0.0,  1.0,
     0.5,  0.5,  0.5,  0.0,  0.0,  1.0,
    -0.5,  0.5,  0.5,  0.0,  0.0,  1.0,
    -0.5, -0.5,  0.5,  0.0,  0.0,  1.0,

    -0.5,  0.5,  0.5, -1.0,  0.0,  0.0,
    -0.5,  0.5, -0.5, -1.0,  0.0,  0.0,
    -0.5, -0.5, -0.5, -1.0,  0.0,  0.0,
    -0.5, -0.5, -0.5, -1.0,  0.0,  0.0,
    -0.5, -0.5,  0.5, -1.0,  0.0,  0.0,
    -0.5,  0.5,  0.5, -1.0,  0.0,  0.0,

     0.5,  0.5,  0.5,  1.0,  0.0,  0.0,
     0.5,  0.5, -0.5,  1.0,  0.0,  0.0,
     0.5, -0.5, -0.5,  1.0,  0.0,  0.0,
     0.5, -0.5, -0.5,  1.0,  0.0,  0.0,
     0.5, -0.5,  0.5,  1.0,  0.0,  0.0,
     0.5,  0.5,  0.5,  1.0,  0.0,  0.0,

    -0.5, -0.5, -0.5,  0.0, -1.0,  0.0,
     0.5, -0.5, -0.5,  0.0, -1.0,  0.0,
     0.5, -0.5,  0.5,  0.0, -1.0,  0.0,
     0.5, -0.5,  0.5,  0.0, -1.0,  0.0,
    -0.5, -0.5,  0.5,  0.0, -1.0,  0.0,
    -0.5, -0.5, -0.5,  0.0, -1.0,  0.0,

    -0.5,  0.5, -0.5,  0.0,  1.0,  0.0,
     0.5,  0.5, -0.5,  0.0,  1.0,  0.0,
     0.5,  0.5,  0.5,  0.0,  1.0,  0.0,
     0.5,  0.5,  0.5,  0.0,  1.0,  0.0,
    -0.5,  0.5,  0.5,  0.0,  1.0,  0.0,
    -0.5,  0.5, -0.5,  0.0,  1.0,  0.0,
];
