use std::time::SystemTime;

use gl::types::GLuint;
use glm::Vec3;

use crate::{
    camera::Camera,
    program::Program,
    renderer,
    shader::{Shader, ShaderError},
};

pub struct TutorialRenderer
{
    program: Program,
    lighting_program: Program,
    vertex_array: renderer::VertexArray,
    light_vertex_array: renderer::VertexArray,
    light_position: Vec3,
}

impl TutorialRenderer
{
    pub fn new() -> Result<Self, ShaderError>
    {
        unsafe {
            let vertex_shader = Shader::new(
                "./resources/shaders/lighting/colors.vert",
                gl::VERTEX_SHADER,
            )?;
            let fragment_shader = Shader::new(
                "./resources/shaders/lighting/colors.frag",
                gl::FRAGMENT_SHADER,
            )?;
            let program = Program::new(&[vertex_shader, fragment_shader])?;

            let lighting_vertex_shader = Shader::new(
                "./resources/shaders/lighting/light_cube.vert",
                gl::VERTEX_SHADER,
            )?;
            let lighting_frag_shader = Shader::new(
                "./resources/shaders/lighting/light_cube.frag",
                gl::FRAGMENT_SHADER,
            )?;
            let lighting_program = Program::new(&[lighting_vertex_shader, lighting_frag_shader])?;

            let mut vertex_array = renderer::VertexArray::new(&VERTEX_DATA, 6);
            vertex_array.add_vert_att_ptr(3);

            let mut light_vertex_array = renderer::VertexArray::new_with_vbo(vertex_array.vbo, 6);
            light_vertex_array.add_vert_att_ptr(3);
            light_vertex_array.add_vert_att_ptr(3);

            gl::Enable(gl::DEPTH_TEST);
            return Ok(Self {
                program,
                lighting_program,
                vertex_array,
                light_vertex_array,
                light_position: glm::vec3(1.2, 1.0, 2.0),
            });
        }
    }

    pub fn draw(
        &self,
        camera: &Camera,
    )
    {
        unsafe {
            let time = SystemTime::now();
            let time_since = time
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis();

            let offset = glm::sin(glm::radians(time_since as f64 * 0.05));
            let moving_light = glm::vec3(
                self.light_position.x + (offset as f32),
                self.light_position.y + (offset as f32),
                self.light_position.z + (offset as f32),
            );

            gl::ClearColor(0.2, 0.3, 0.3, 0.7);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            gl::UseProgram(self.program.id);
            self.program
                .set_uniform_vec3("objectColor", glm::vec3(1.0, 0.5, 0.31));
            self.program
                .set_uniform_vec3("lightColor", glm::vec3(1.0, 1.0, 1.0));
            self.program.set_uniform_vec3("lightPos", moving_light);
            self.program
                .set_uniform_vec3("viewPos", camera.camera_position);

            let view = camera.get_view_matrix();
            self.program.set_uniform_mat4("view", view);

            let projection =
                glm::ext::perspective(glm::radians(camera.fov), 1920.0 / 1080.0, 0.1, 100.0);
            self.program.set_uniform_mat4("projection", projection);

            #[rustfmt::skip]
            let model = glm::mat4(
                1.0, 0.0, 0.0, 0.0,
                0.0, 1.0, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                0.0, 0.0, 0.0, 1.0,
            );
            self.program.set_uniform_mat4("model", model);

            gl::BindVertexArray(self.vertex_array.vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 36);

            gl::UseProgram(self.lighting_program.id);
            self.lighting_program.set_uniform_mat4("view", view);
            self.lighting_program
                .set_uniform_mat4("projection", projection);

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

            gl::BindVertexArray(self.light_vertex_array.vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 36);
        }
    }

    pub fn resize(
        &self,
        width: i32,
        height: i32,
    )
    {
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
