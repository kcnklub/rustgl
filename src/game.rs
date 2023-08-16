use std::{cell::RefCell, rc::Rc};

use glm::vec3;
use winit::event::VirtualKeyCode;

use crate::{colliding_renderer::CubeObject, collision};

#[derive(Debug)]
pub struct CubeGameState
{
    pub cubes: Vec<Rc<RefCell<CubeObject>>>,
}

impl CubeGameState
{
    pub fn new() -> Self
    {
        let mut cubes = vec![];
        let cube = CubeObject {
            id: 0,
            position: vec3(0.0, 0.0, 0.0),
            is_colliding: false,
            colliding_objects: vec![],
            normal_vector: vec3(0.0, 0.0, 0.0),
            force: vec3(0.0, 0.0, 0.0),
            velocity: vec3(0.0, 0.0, 0.0),
            mass: 0.5,
        };
        cubes.push(Rc::new(RefCell::new(cube)));
        let cube = CubeObject {
            id: 0,
            position: vec3(2.0, 2.3, 0.0),
            is_colliding: false,
            colliding_objects: vec![],
            normal_vector: vec3(0.0, 0.0, 0.0),
            force: vec3(0.0, 0.0, 0.0),
            velocity: vec3(0.0, 0.0, 0.0),
            mass: 0.5,
        };
        cubes.push(Rc::new(RefCell::new(cube)));
        Self { cubes }
    }

    pub fn handle_keyboard_input(
        &mut self,
        now_keys: &[bool; 255],
    )
    {
        let cube = self.cubes.get_mut(0).unwrap();
        if let Some(cube) = Rc::get_mut(cube)
        {
            if now_keys[VirtualKeyCode::G as usize]
            {
                cube.borrow_mut().force = glm::vec3(10000.0, 5000.0, 0.0);
            }
            else if now_keys[VirtualKeyCode::B as usize]
            {
                cube.borrow_mut().force = glm::vec3(10000.0, -10000.0, 0.0);
            }
            else
            {
                cube.borrow_mut().force = glm::vec3(0.0, 0.0, 0.0);
            }
        }
    }

    pub fn integrate(
        &mut self,
        delta_time: f32,
    )
    {
        calc_colliding(&mut self.cubes);
        for cube in self.cubes.as_slice()
        {
            cube.borrow_mut().integrate(delta_time);
        }
    }
}

fn calc_colliding(cubes: &mut Vec<Rc<RefCell<CubeObject>>>)
{
    let cube_len = cubes.len();
    for i in 0..cube_len
    {
        let main_cube = &mut cubes[i];
        main_cube.borrow_mut().colliding_objects = vec![];
        let main_verts = main_cube.borrow().get_verts();

        let mut is_colliding = false;
        let mut normal_vector = None;
        for j in 0..cube_len
        {
            if i == j
            {
                continue;
            }
            let other_cube = &cubes[j].clone();
            let other_verts = other_cube.borrow().get_verts();
            (is_colliding, normal_vector) =
                collision::test_collision_3d(&main_verts, 3, &other_verts, 3);

            if is_colliding
            {
                if let Some(cube) = Rc::get_mut(&mut cubes[i])
                {
                    let mut borrowed = cube.borrow_mut();
                    borrowed.colliding_objects.push(other_cube.to_owned());
                    if let Some(normal) = normal_vector
                    {
                        borrowed.normal_vector = normal;
                    }
                }
                break;
            }
        }

        let main_cube = &mut cubes[i];
        main_cube.borrow_mut().is_colliding = is_colliding;
    }
}
