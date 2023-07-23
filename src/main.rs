use std::ffi::CString;
use std::num::NonZeroU32;
use std::time::SystemTime;

use camera::Camera;
use gl::FALSE;
use gl::types::GLuint;
use image::EncodableLayout;
use winit::dpi::PhysicalSize;
use winit::event::{Event, VirtualKeyCode, WindowEvent};
use winit::event_loop::EventLoopBuilder;
use winit::window::WindowBuilder;

use raw_window_handle::HasRawWindowHandle;

use glutin::config::ConfigTemplateBuilder;
use glutin::context::{ContextApi, ContextAttributesBuilder, Version};
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::surface::SwapInterval;

use glutin_winit::{self, DisplayBuilder, GlWindow};

mod camera;
mod shader;
mod terrian;
mod texture;

fn main() {
    let event_loop = EventLoopBuilder::new().build();
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

                    if transparency_check || config.num_samples() > accum.num_samples() {
                        config
                    } else {
                        accum
                    }
                })
                .unwrap()
        })
        .unwrap();

    let raw_window_handle = window.as_ref().map(|window| window.raw_window_handle());

    let gl_display = gl_config.display();

    let context_attributes = ContextAttributesBuilder::new().build(raw_window_handle);

    let fallback_context_attributes = ContextAttributesBuilder::new()
        .with_context_api(glutin::context::ContextApi::Gles(None))
        .build(raw_window_handle);

    let legacy_context_attributes = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::OpenGl(Some(Version::new(2, 1))))
        .build(raw_window_handle);

    let mut not_current_gl_context = Some(unsafe {
        gl_display
            .create_context(&gl_config, &context_attributes)
            .unwrap_or_else(|_| {
                gl_display
                    .create_context(&gl_config, &fallback_context_attributes)
                    .unwrap_or_else(|_| {
                        gl_display
                            .create_context(&gl_config, &legacy_context_attributes)
                            .expect("failed to create context")
                    })
            })
    });

    let mut state = None;
    let mut renderer = None;

    let mut camera = Camera::new();
    let mut last_time = SystemTime::now();
    let mut delta_time = Default::default();
    let mut now_keys = [false; 255];
    event_loop.run(move |event, window_target, control_flow| {
        control_flow.set_poll();

        let current_time = SystemTime::now();
        let delta_time_duration = current_time
            .duration_since(last_time)
            .expect("can't get delta_time");
        delta_time = delta_time_duration.as_secs_f32();
        last_time = current_time;

        // println!("fps: {}", 1.0 / delta_time);

        match event {
            Event::Resumed => {
                let window = window.take().unwrap_or_else(|| {
                    let window_builder = WindowBuilder::new().with_transparent(true);
                    glutin_winit::finalize_window(window_target, window_builder, &gl_config)
                        .unwrap()
                });
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

                renderer.get_or_insert_with(|| Renderer::new());

                if let Err(res) = gl_surface
                    .set_swap_interval(&gl_context, SwapInterval::Wait(NonZeroU32::new(1).unwrap()))
                {
                    eprintln!("Error setting vsync: {res:?}");
                }

                assert!(state.replace((gl_context, gl_surface, window)).is_none());
            }
            Event::DeviceEvent { event, .. } => match event {
                winit::event::DeviceEvent::MouseMotion { delta } => {
                    camera.handle_mouse_input(delta.0 as f32, delta.1 as f32);
                }
                winit::event::DeviceEvent::MouseWheel { .. } => {}
                winit::event::DeviceEvent::Key(keyboard_input) => {
                    if let Some(key_code) = keyboard_input.virtual_keycode {
                        match keyboard_input.state {
                            winit::event::ElementState::Pressed => {
                                now_keys[key_code as usize] = true;
                            }
                            winit::event::ElementState::Released => {
                                now_keys[key_code as usize] = false;
                            }
                        }
                    }
                }
                _ => {}
            },
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(size) => {
                    if size.width != 0 && size.height != 0 {
                        if let Some((gl_context, gl_surface, _)) = &state {
                            gl_surface.resize(
                                gl_context,
                                NonZeroU32::new(size.width).unwrap(),
                                NonZeroU32::new(size.height).unwrap(),
                            );
                            let renderer = renderer.as_ref().unwrap();
                            renderer.resize(size.width as i32, size.height as i32);
                        }
                    }
                }
                WindowEvent::CloseRequested => {
                    control_flow.set_exit();
                }
                _ => {}
            },
            Event::MainEventsCleared => {
                if let Some((gl_context, gl_surface, window)) = &state {
                    let renderer = renderer.as_ref().unwrap();
                    renderer.draw(&camera);
                    window.request_redraw();

                    gl_surface.swap_buffers(gl_context).unwrap();
                }

                if now_keys[VirtualKeyCode::Escape as usize] {
                    control_flow.set_exit();
                }

                camera.handle_keyboard_input(&now_keys, delta_time)
            }
            _ => {}
        }
    });
}

pub struct Renderer {
    program: gl::types::GLuint,
    vao: gl::types::GLuint,
    vbo: gl::types::GLuint,
    texture_id: gl::types::GLuint,
}

impl Renderer {
    pub fn new() -> Self {
        unsafe {

            let vertex_shader = create_shader(gl::VERTEX_SHADER, VERTEX_SHADER_SOURCE);
            let fragment_shader = create_shader(gl::FRAGMENT_SHADER, FRAGMENT_SHADER_SOURCE);

            let program = gl::CreateProgram();

            gl::AttachShader(program, vertex_shader);
            gl::AttachShader(program, fragment_shader);

            gl::LinkProgram(program);

            gl::UseProgram(program);

            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);

            let vertex_data = terrian::generate_terrian_vertices(50.0, 127);
            let ebo_data = terrian::generate_terrian_ebo(127);

            println!("length: {}", ebo_data.len());

            let mut vao = std::mem::zeroed();
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            let mut vbo = std::mem::zeroed();
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertex_data.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                vertex_data.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            let mut ebo = std::mem::zeroed();
            gl::GenBuffers(1, &mut ebo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (ebo_data.len() * std::mem::size_of::<i32>()) as gl::types::GLsizeiptr,
                ebo_data.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            // vertex attri
            gl::VertexAttribPointer(
                0 as gl::types::GLuint,
                3,
                gl::FLOAT,
                0,
                5 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                std::ptr::null(),
            );

            // texture attri
            gl::VertexAttribPointer(
                1 as gl::types::GLuint,
                2,
                gl::FLOAT,
                0,
                5 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                (3 * std::mem::size_of::<f32>()) as *const () as *const _,
            );

            // enable both attribute pointers.
            gl::EnableVertexAttribArray(0 as gl::types::GLuint);
            gl::EnableVertexAttribArray(1 as gl::types::GLuint);


            // load texture
            let mut texture_id: GLuint = 0;
            gl::GenTextures(1, &mut texture_id);
            gl::BindTexture(gl::TEXTURE_2D, texture_id);

            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_S,
                gl::MIRRORED_REPEAT as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_T,
                gl::MIRRORED_REPEAT as i32,
            );

            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MIN_FILTER,
                gl::LINEAR_MIPMAP_LINEAR as i32,
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

            let bit_map = image::open("desert_mountains.png").unwrap().into_rgb8();
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                bit_map.width() as i32,
                bit_map.height() as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                bit_map.as_bytes().as_ptr() as *const _,
            );

            let c_str = CString::new("texture0").unwrap();
            gl::Uniform1i(gl::GetUniformLocation(program, c_str.as_ptr()), 0);

            return Self {
                program,
                vao,
                vbo,
                texture_id
            };
        }
    }

    pub fn draw(&self, camera: &Camera) {
        unsafe {
            gl::ClearColor(0.2, 0.3, 0.3, 0.7);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.texture_id);

            gl::UseProgram(self.program);

            let view = camera.get_view_matrix();
            let c_view = CString::new("view").unwrap();
            let view_uniform =
                gl::GetUniformLocation(self.program, c_view.as_ptr() as *const i8);
            gl::UniformMatrix4fv(view_uniform, 1, FALSE, view.as_array().as_ptr() as *const _);

            let projection =
                glm::ext::perspective(glm::radians(camera.fov), 800.0 / 600.0, 0.1, 100.0);
            let c_projection = CString::new("projection").unwrap();
            let projection_uniform = gl::GetUniformLocation(self.program, c_projection.as_ptr() as *const i8);
            gl::UniformMatrix4fv(
                projection_uniform,
                1,
                FALSE,
                projection.as_array().as_ptr() as *const _,
            );

            #[rustfmt::skip]
            let model = glm::mat4(
                1.0, 0.0, 0.0, 0.0, 
                0.0, 1.0, 0.0, 0.0, 
                0.0, 0.0, 1.0, 0.0, 
                0.0, 0.0, 0.0, 1.0,
            );

            let c_model = CString::new("model").unwrap();
            let model_uniform = gl::GetUniformLocation(self.program, c_model.as_ptr() as *const i8);
            gl::UniformMatrix4fv(
                model_uniform,
                1,
                FALSE,
                model.as_array().as_ptr() as *const _,
            );

            gl::BindVertexArray(self.vao);

            gl::DrawElements(gl::TRIANGLES, 95256, gl::UNSIGNED_INT, std::ptr::null());
        }
    }

    pub fn resize(&self, width: i32, height: i32) {
        unsafe {
            gl::Viewport(0, 0, width, height);
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.program);
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}

unsafe fn create_shader(
    shader: gl::types::GLenum,
    source: &[u8],
) -> gl::types::GLuint {
    let shader = gl::CreateShader(shader);
    gl::ShaderSource(
        shader,
        1,
        [source.as_ptr().cast()].as_ptr(),
        std::ptr::null(),
    );
    gl::CompileShader(shader);
    let mut success = std::mem::zeroed();
    gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
    if success != 1 {
        println!("failed to compile shader");
    }
    shader
}

const VERTEX_SHADER_SOURCE: &[u8] = b"
#version 330 core 
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec2 aTexCoord;

out vec2 TexCoord;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

void main() 
{
    gl_Position = projection * view * model * vec4(aPos, 1.0);
    TexCoord = aTexCoord;
}
\0";

const FRAGMENT_SHADER_SOURCE: &[u8] = b"
#version 330 core
out vec4 FragColor;

in vec2 TexCoord;

uniform sampler2D texture0;

void main() 
{
    FragColor = texture(texture0, TexCoord);
} 
\0";
