use std::ops::Add;

use glm::{vec3, Mat4, Vec3};
use winit::event::VirtualKeyCode;

pub struct Camera {
    camera_position: Vec3,
    camera_front: Vec3,
    camera_up: Vec3,

    right: Vec3,
    world_up: Vec3,

    pitch: f32,
    yaw: f32,

    pub fov: f32,
}

impl Camera {
    pub fn new() -> Self {
        #[rustfmt::skip]
        let mut new_camera = Self {
            camera_position: vec3(0.0, 0.0, 3.0),
            camera_front:    vec3(0.0, 0.0, -1.0),
            camera_up:       vec3(0.0, 1.0,  0.0),
            right:           vec3(0.0, 0.0,  0.0),
            world_up:        vec3(0.0, 1.0,  0.0),
            pitch:           0.0,
            yaw:             -90.0,
            fov:             45.0,
        };
        new_camera.update_camera_vectors();
        new_camera
    }

    pub fn handle_keyboard_input(&mut self, now_keys: &[bool; 255], delta_time: f32) {
        let movespeed = 500.0 * delta_time;
        if now_keys[VirtualKeyCode::W as usize] {
            self.camera_position = self.camera_position + self.camera_front * movespeed;
        }
        if now_keys[VirtualKeyCode::S as usize] {
            self.camera_position = self.camera_position - self.camera_front * movespeed;
        }
        if now_keys[VirtualKeyCode::A as usize] {
            self.camera_position = self.camera_position - self.right * (movespeed);
        }
        if now_keys[VirtualKeyCode::D as usize] {
            self.camera_position = self.camera_position + self.right * movespeed;
        }
    }

    pub fn handle_mouse_input(&mut self, x_offset: f32, y_offset: f32) {
        let sensitivity = 0.1;

        self.yaw = self.yaw + (x_offset * sensitivity);
        self.pitch = self.pitch - (y_offset * sensitivity);

        if self.pitch > 89.0 {
            self.pitch = 89.0;
        }
        if self.pitch < -89.0 {
            self.pitch = -89.0;
        }
        self.update_camera_vectors();
    }

    pub fn get_view_matrix(&self) -> Mat4 {
        return glm::ext::look_at_rh(
            self.camera_position,
            self.camera_position.add(self.camera_front),
            self.camera_up,
        );
    }

    fn update_camera_vectors(&mut self) {
        let mut direction = vec3(0.0, 0.0, 0.0);
        direction.x = glm::radians(self.yaw).cos() * glm::radians(self.pitch).cos();
        direction.y = glm::radians(self.pitch).sin();
        direction.z = glm::radians(self.yaw).sin() * glm::radians(self.pitch).cos();

        self.camera_front = glm::normalize(direction);
        self.right = glm::normalize(glm::cross(self.camera_front, self.world_up));
        self.camera_up = glm::normalize(glm::cross(self.right, self.camera_front));
    }
}
