use std::time::SystemTime;

use camera::Camera;
use game::CubeGameState;
use window_utils::{build_gl_state, handle_window_event, track_user_input};
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::EventLoopBuilder;

use glutin::prelude::*;

use crate::colliding_renderer::CubeRenderer;

mod camera;
mod colliding_renderer;
mod collision;
mod debug_gui;
mod game;
mod program;
mod renderer;
mod shader;
mod terrian;
mod texture;
mod tutorial_renderer;
mod window_utils;

fn main()
{
    let event_loop = EventLoopBuilder::new().build();

    let state = build_gl_state(&event_loop);

    let renderer = CubeRenderer::new();

    let mut game_state = CubeGameState::new();

    let mut camera = Camera::new();
    let mut current_time = SystemTime::now();
    let mut now_keys = [false; 255];

    let dt = 1.0 / 60.0; // 60 fps

    event_loop.run(move |event, _, control_flow| {
        control_flow.set_poll();

        // time delta is calc here doing the main in the MainEventsCleared processing
        // really throws off the time values.
        let new_time = SystemTime::now();
        let delta_time_duration = new_time
            .duration_since(current_time)
            .expect("can't get delta_time");
        let mut frame_time = delta_time_duration.as_secs_f32();
        current_time = new_time;

        match event
        {
            Event::DeviceEvent { event, .. } => track_user_input(event, &mut camera, &mut now_keys),
            Event::WindowEvent { event, .. } =>
            {
                handle_window_event(event, control_flow, &state, &renderer)
            }
            // this is the main loop of the game engine!
            Event::MainEventsCleared =>
            {
                // let try some physic??
                camera.apply_gravity(frame_time);

                unsafe {
                    gl::ClearColor(0.2, 0.3, 0.3, 0.7);
                    gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
                }

                camera.handle_keyboard_input(&now_keys, frame_time);

                game_state.handle_keyboard_input(&now_keys);

                while frame_time > 0.0
                {
                    let delta_time = frame_time.min(dt);
                    frame_time = frame_time - delta_time;
                    game_state.integrate(delta_time);
                }

                renderer.draw_state(&game_state, &camera); //

                if let Some((gl_context, gl_surface, window)) = &state
                {
                    window.request_redraw();
                    gl_surface.swap_buffers(gl_context).unwrap();
                }

                if now_keys[VirtualKeyCode::Escape as usize]
                {
                    control_flow.set_exit();
                }
            }
            _ =>
            {}
        }
    });
}
