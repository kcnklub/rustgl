use std::ffi::CString;
use std::num::NonZeroU32;
use std::time::SystemTime;

use camera::Camera;
use glm::vec3;
use glutin::surface::WindowSurface;
use winit::dpi::PhysicalSize;
use winit::event::{Event, VirtualKeyCode, WindowEvent};
use winit::event_loop::{EventLoop, EventLoopBuilder};
use winit::window::WindowBuilder;

use raw_window_handle::HasRawWindowHandle;

use glutin::config::ConfigTemplateBuilder;
use glutin::context::ContextAttributesBuilder;
use glutin::display::GetGlDisplay;
use glutin::prelude::*;

use glutin_winit::{self, DisplayBuilder, GlWindow};

use crate::colliding_renderer::{CubeObject, CubeRenderer};
use crate::terrian::TerrianRenderer;
use crate::tutorial_renderer::TutorialRenderer;

mod camera;
mod colliding_renderer;
mod collision;
mod debug_gui;
mod program;
mod renderer;
mod shader;
mod terrian;
mod texture;
mod tutorial_renderer;

fn main()
{
    let event_loop = EventLoopBuilder::new().build();

    let state = build_gl_state(&event_loop);

    let mut cube_1 = CubeObject {
        position: vec3(0.0, 0.0, 0.0),
        renderer: CubeRenderer::new(),
    };
    let mut cube_2 = CubeObject {
        position: vec3(0.0, 0.0, 0.0),
        renderer: CubeRenderer::new(),
    };

    let mut camera = Camera::new();
    let mut last_time = SystemTime::now();
    let mut now_keys = [false; 255];

    event_loop.run(move |event, _, control_flow| {
        control_flow.set_poll();

        let current_time = SystemTime::now();
        let delta_time_duration = current_time
            .duration_since(last_time)
            .expect("can't get delta_time");
        let delta_time = delta_time_duration.as_secs_f32();
        last_time = current_time;

        match event
        {
            Event::DeviceEvent { event, .. } => match event
            {
                winit::event::DeviceEvent::MouseMotion { delta } =>
                {
                    camera.handle_mouse_input(delta.0 as f32, delta.1 as f32);
                }
                winit::event::DeviceEvent::MouseWheel { .. } =>
                {}
                winit::event::DeviceEvent::Key(keyboard_input) =>
                {
                    if let Some(key_code) = keyboard_input.virtual_keycode
                    {
                        match keyboard_input.state
                        {
                            winit::event::ElementState::Pressed =>
                            {
                                now_keys[key_code as usize] = true;
                            }
                            winit::event::ElementState::Released =>
                            {
                                now_keys[key_code as usize] = false;
                            }
                        }
                    }
                }
                _ =>
                {}
            },
            Event::WindowEvent { event, .. } => match event
            {
                WindowEvent::Resized(size) =>
                {
                    if size.width != 0 && size.height != 0
                    {
                        if let Some((gl_context, gl_surface, _)) = &state
                        {
                            gl_surface.resize(
                                gl_context,
                                NonZeroU32::new(size.width).unwrap(),
                                NonZeroU32::new(size.height).unwrap(),
                            );
                            cube_1.resize(size.width as i32, size.height as i32);
                            cube_2.resize(size.width as i32, size.height as i32);
                        }
                    }
                }
                WindowEvent::CloseRequested =>
                {
                    control_flow.set_exit();
                }
                _ =>
                {}
            },
            Event::MainEventsCleared =>
            {
                if let Some((gl_context, gl_surface, window)) = &state
                {
                    // let try some physic??
                    camera.apply_gravity(delta_time);

                    unsafe {
                        gl::ClearColor(0.2, 0.3, 0.3, 0.7);
                        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
                    }

                    let first = cube_1.get_verts();
                    let second = cube_2.get_verts();
                    let are_colliding =
                        collision::test_collision_3d(first.as_slice(), 3, second.as_slice(), 3);

                    cube_2.process_square("y", &camera, are_colliding);
                    cube_1.process_square("x", &camera, are_colliding);

                    window.request_redraw();

                    gl_surface.swap_buffers(gl_context).unwrap();
                }

                if now_keys[VirtualKeyCode::Escape as usize]
                {
                    control_flow.set_exit();
                }

                camera.handle_keyboard_input(&now_keys, delta_time)
            }
            _ =>
            {}
        }
    });
}

fn build_gl_state(
    event_loop: &EventLoop<()>
) -> Option<(
    glutin::context::PossiblyCurrentContext,
    glutin::surface::Surface<WindowSurface>,
    winit::window::Window,
)>
{
    let window_builder = Some(
        WindowBuilder::new()
            .with_min_inner_size(PhysicalSize::new(1920, 1080))
            .with_title("We using rust now baby!"),
    );

    let template = ConfigTemplateBuilder::new()
        .with_alpha_size(8)
        .with_transparency(cfg!(cgl_backend));

    let display_builder = DisplayBuilder::new().with_window_builder(window_builder);

    let (mut window, gl_config) = display_builder
        .build(&event_loop, template, |configs| {
            configs
                .reduce(|accum, config| {
                    let transparency_check = config.supports_transparency().unwrap_or(false)
                        & !accum.supports_transparency().unwrap_or(false);

                    if transparency_check || config.num_samples() > accum.num_samples()
                    {
                        config
                    }
                    else
                    {
                        accum
                    }
                })
                .unwrap()
        })
        .unwrap();

    let raw_window_handle = window.as_ref().map(|window| window.raw_window_handle());

    let gl_display = gl_config.display();

    let context_attributes = ContextAttributesBuilder::new().build(raw_window_handle);

    let mut not_current_gl_context = Some(unsafe {
        gl_display
            .create_context(&gl_config, &context_attributes)
            .expect("unable to load context")
    });

    let window = window.take().unwrap();
    let _ = window
        .set_cursor_grab(winit::window::CursorGrabMode::Confined)
        .unwrap();
    let _ = window.set_cursor_visible(false);

    let attrs = window.build_surface_attributes(<_>::default());
    let gl_surface = unsafe {
        gl_config
            .display()
            .create_window_surface(&gl_config, &attrs)
            .unwrap()
    };

    let gl_context = not_current_gl_context
        .take()
        .unwrap()
        .make_current(&gl_surface)
        .unwrap();
    gl::load_with(|ptr| {
        let c_str = CString::new(ptr).unwrap();
        gl_display.get_proc_address(c_str.as_c_str())
    });
    let mut state = None;
    state.replace((gl_context, gl_surface, window));
    state
}
