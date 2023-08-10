use std::time::SystemTime;

use glm::{vec3, vec4, Vec3};

use crate::{
    camera::Camera,
    program::Program,
    renderer::{self, VertexArray},
    shader::Shader,
};

pub struct SquareObject
{
    pub position: Vec3,
    pub renderer: SquareRenderer,
}

impl SquareObject
{
    pub fn process_square(
        &mut self,
        axis: &str,
        camera: &Camera,
        is_colliding: bool,
    )
    {
        let time = SystemTime::now();
        let time_since = time
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis();

        let color = match is_colliding
        {
            true => vec3(1.0, 0.0, 0.0),
            false => vec3(0.0, 1.0, 0.0),
        };

        if axis == "x"
        {
            let offset = glm::sin(glm::radians(time_since as f64 * 0.05));
            let moving_light = glm::vec3(
                self.position.x + (offset as f32 * 2.0),
                self.position.y,
                self.position.z,
            );
            self.renderer.draw(&moving_light, camera, &color);
        }
        else if axis == "y"
        {
            let offset = glm::sin(glm::radians(time_since as f64 * 0.05));
            let moving_light = glm::vec3(
                self.position.x,
                self.position.y + (offset as f32 * 2.0),
                self.position.z,
            );
            self.renderer.draw(&moving_light, camera, &color);
        }
    }

    pub fn get_verts(&self) -> Vec<Vec3>
    {
        #[rustfmt::skip]
        let mut model = glm::mat4(
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        );

        model = glm::ext::translate(&model, self.position);

        let mut ret_val = vec![];
        let vert = model * vec4(0.5, 0.5, 0.5, 1.0);
        ret_val.push(vec3(vert.x, vert.y, vert.z));
        let vert = model * vec4(0.5, 0.5, -0.5, 1.0);
        ret_val.push(vec3(vert.x, vert.y, vert.z));
        let vert = model * vec4(0.5, -0.5, 0.5, 1.0);
        ret_val.push(vec3(vert.x, vert.y, vert.z));
        let vert = model * vec4(0.5, -0.5, -0.5, 1.0);
        ret_val.push(vec3(vert.x, vert.y, vert.z));
        let vert = model * vec4(-0.5, 0.5, 0.5, 1.0);
        ret_val.push(vec3(vert.x, vert.y, vert.z));
        let vert = model * vec4(-0.5, 0.5, -0.5, 1.0);
        ret_val.push(vec3(vert.x, vert.y, vert.z));
        let vert = model * vec4(-0.5, -0.5, 0.5, 1.0);
        ret_val.push(vec3(vert.x, vert.y, vert.z));
        let vert = model * vec4(-0.5, -0.5, -0.5, 1.0);
        ret_val.push(vec3(vert.x, vert.y, vert.z));

        ret_val
    }
}

pub struct SquareRenderer
{
    program: Program,
    vertex_array: VertexArray,
}

impl SquareRenderer
{
    pub fn new() -> Self
    {
        unsafe {
            let v_shader =
                Shader::new("./resources/shaders/basic_vert.glsl", gl::VERTEX_SHADER).unwrap();
            let f_shader =
                Shader::new("./resources/shaders/basic_frag.glsl", gl::FRAGMENT_SHADER).unwrap();
            let program = Program::new(&[v_shader, f_shader]).unwrap();

            let mut vertex_array = VertexArray::new(&VERTEX_DATA, 6);
            vertex_array.add_vert_att_ptr(3);
            vertex_array.add_vert_att_ptr(3);

            gl::Enable(gl::DEPTH_TEST);
            Self {
                program,
                vertex_array,
            }
        }
    }

    pub fn draw(
        &self,
        position: &Vec3,
        camera: &Camera,
        color: &Vec3,
    )
    {
        unsafe {
            self.program.bind();

            let view = camera.get_view_matrix();
            self.program.set_uniform_mat4("view", view);

            let projection =
                glm::ext::perspective(glm::radians(camera.fov), 1920.0 / 1080.0, 0.1, 100.0);
            self.program.set_uniform_mat4("projection", projection);

            #[rustfmt::skip]
            let mut model = glm::mat4(
                1.0, 0.0, 0.0, 0.0,
                0.0, 1.0, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                0.0, 0.0, 0.0, 1.0,
            );
            model = glm::ext::translate(&model, *position);

            self.program.set_uniform_mat4("model", model);

            self.program.set_uniform_vec3("color", *color);

            renderer::draw_without_ibo(&self.vertex_array, &self.program);
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
