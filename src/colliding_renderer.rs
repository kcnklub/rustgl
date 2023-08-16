use std::{cell::RefCell, rc::Rc};

use glm::{vec3, vec4, Vec3};

use crate::{
    camera::Camera,
    game::CubeGameState,
    program::Program,
    renderer::{self, VertexArray},
    shader::Shader,
};

#[derive(Debug, Clone)]
pub struct CubeObject
{
    pub id: i32,
    pub position: Vec3,
    pub is_colliding: bool,
    pub normal_vector: Vec3,
    pub colliding_objects: Vec<Rc<RefCell<CubeObject>>>,
    pub force: Vec3,
    pub velocity: Vec3,
    pub mass: f32,
}

impl CubeObject
{
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
        let a = model * vec4(0.5, 0.5, 0.5, 1.0);
        let a = vec3(a.x, a.y, a.z);
        let b = model * vec4(0.5, 0.5, -0.5, 1.0);
        let b = vec3(b.x, b.y, b.z);
        let c = model * vec4(0.5, -0.5, 0.5, 1.0);
        let c = vec3(c.x, c.y, c.z);
        let d = model * vec4(0.5, -0.5, -0.5, 1.0);
        let d = vec3(d.x, d.y, d.z);
        let e = model * vec4(-0.5, 0.5, 0.5, 1.0);
        let e = vec3(e.x, e.y, e.z);
        let f = model * vec4(-0.5, 0.5, -0.5, 1.0);
        let f = vec3(f.x, f.y, f.z);
        let g = model.mul_v(&vec4(-0.5, -0.5, 0.5, 1.0));
        let g = vec3(g.x, g.y, g.z);
        let h = model.mul_v(&vec4(-0.5, -0.5, -0.5, 1.0));
        let h = vec3(h.x, h.y, h.z);

        ret_val.push(a);
        ret_val.push(b);
        ret_val.push(c);

        ret_val.push(a);
        ret_val.push(e);
        ret_val.push(b);

        ret_val.push(a);
        ret_val.push(e);
        ret_val.push(c);

        ret_val.push(c);
        ret_val.push(g);
        ret_val.push(d);

        ret_val.push(f);
        ret_val.push(e);
        ret_val.push(h);

        ret_val.push(f);
        ret_val.push(b);
        ret_val.push(h);

        ret_val
    }

    pub fn integrate(
        &mut self,
        delta_time: f32,
    )
    {
        if !self.colliding_objects.is_empty()
        {
            elastic_collision_reaction(self, delta_time);
            //inelastic_collision_reaction(self, delta_time);
        }
        else
        {
            self.position = self.position + self.velocity * delta_time;
            self.velocity = self.velocity + (self.force / self.mass) * delta_time;
        }
    }
}

fn elastic_collision_reaction(
    cube: &mut CubeObject,
    delta_time: f32,
)
{
    let output = glm::dot(cube.normal_vector, cube.velocity);
    let reflected = cube.velocity - cube.normal_vector * 2.0 * output;
    cube.velocity = reflected;
    cube.position = cube.position + cube.velocity * delta_time;
}

fn inelastic_collision_reaction(
    cube: &mut CubeObject,
    delta_time: f32,
)
{
    let mut other_cube = cube.colliding_objects.first().unwrap().borrow_mut();
    let other_momentum = other_cube.velocity * other_cube.mass;
    let momentum = cube.velocity * cube.mass;
    let new_velocity = (other_momentum + momentum) / (cube.mass + other_cube.mass);
    cube.velocity = new_velocity;
    other_cube.velocity = new_velocity;
    cube.position = cube.position + cube.velocity * delta_time;
}

pub struct CubeRenderer
{
    program: Program,
    vertex_array: VertexArray,
}

impl CubeRenderer
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

    pub fn draw_state(
        &self,
        state: &CubeGameState,
        camera: &Camera,
    )
    {
        let cubes = state.cubes.as_slice();
        for cube in cubes
        {
            let color = match cube.borrow().is_colliding
            {
                true => vec3(1.0, 0.0, 0.0),
                false => vec3(0.0, 1.0, 0.0),
            };
            self.draw(&cube.borrow().position, &camera, &color);
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
