use std::ffi::{CStr, CString};
use std::num::NonZeroU32;
use std::ops::Deref;

use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoopBuilder;
use winit::window::WindowBuilder;

use raw_window_handle::HasRawWindowHandle;

use glutin::config::ConfigTemplateBuilder;
use glutin::context::{ContextApi, ContextAttributesBuilder, Version};
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::surface::SwapInterval;

use glutin_winit::{self, DisplayBuilder, GlWindow};

pub mod gl {
    #![allow(clippy::all)]
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));

    pub use Gles2 as Gl;
}

fn main() {
    let event_loop = EventLoopBuilder::new().build();
    let window_builder = Some(
        WindowBuilder::new()
            .with_title("We using rust now baby!")
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

    println!("Sampling {}", gl_config.num_samples());

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
    event_loop.run(move |event, window_target, control_flow| {
        control_flow.set_wait();
        match event {
            Event::Resumed => {
                let window = window.take().unwrap_or_else(|| {
                    let window_builder = WindowBuilder::new().with_transparent(true);
                    glutin_winit::finalize_window(window_target, window_builder, &gl_config)
                        .unwrap()
                });

                let attrs = window.build_surface_attributes(<_>::default());
                let gl_surface = unsafe {
                    gl_config
                        .display()
                        .create_window_surface(&gl_config, &attrs)
                        .unwrap()
                };

                // Make it current.
                let gl_context = not_current_gl_context
                    .take()
                    .unwrap()
                    .make_current(&gl_surface)
                    .unwrap();

                // The context needs to be current for the Renderer to set up shaders and
                // buffers. It also performs function loading, which needs a current context on
                // WGL.
                renderer.get_or_insert_with(|| Renderer::new(&gl_display));

                // Try setting vsync.
                if let Err(res) = gl_surface
                    .set_swap_interval(&gl_context, SwapInterval::Wait(NonZeroU32::new(1).unwrap()))
                {
                    eprintln!("Error setting vsync: {res:?}");
                }

                assert!(state.replace((gl_context, gl_surface, window)).is_none());
            }
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
            Event::RedrawEventsCleared => {
                if let Some((gl_context, gl_surface, window)) = &state {
                    let renderer = renderer.as_ref().unwrap();
                    renderer.draw();
                    window.request_redraw();

                    gl_surface.swap_buffers(gl_context).unwrap();
                }
            }
            _ => {}
        }
    });
}

pub struct Renderer {
    program: gl::types::GLuint,
    vao: gl::types::GLuint,
    vbo: gl::types::GLuint,
    gl: gl::Gl,
}

impl Renderer {
    pub fn new<D: GlDisplay>(gl_display: &D) -> Self {
        unsafe {
            let gl = gl::Gl::load_with(|symbol| {
                let symbol = CString::new(symbol).unwrap();
                gl_display.get_proc_address(symbol.as_c_str()).cast()
            });

            if let Some(renderer) = get_gl_string(&gl, gl::RENDERER) {
                println!("running on {}", renderer.to_string_lossy());
            }
            if let Some(version) = get_gl_string(&gl, gl::VERSION) {
                println!("OpenGL version {}", version.to_string_lossy());
            }

            if let Some(shaders_version) = get_gl_string(&gl, gl::SHADING_LANGUAGE_VERSION) {
                println!("Shaders version on {}", shaders_version.to_string_lossy());
            }

            let vertex_shader = Shader::new(&gl, VERTEX_SHADER_SOURCE, gl::VERTEX_SHADER);
            let fragment_shader = create_shader(&gl, gl::FRAGMENT_SHADER, FRAGMENT_SHADER_SOURCE);

            let program = gl.CreateProgram();

            gl.AttachShader(program, vertex_shader.id);
            gl.AttachShader(program, fragment_shader);

            gl.LinkProgram(program);

            gl.UseProgram(program);

            gl.DeleteShader(vertex_shader.id);
            gl.DeleteShader(fragment_shader);

            let mut vao = std::mem::zeroed();
            gl.GenVertexArrays(1, &mut vao);
            gl.BindVertexArray(vao);

            let mut vbo = std::mem::zeroed();
            gl.GenBuffers(1, &mut vbo);
            gl.BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl.BufferData(
                gl::ARRAY_BUFFER,
                (VERTEX_DATA_OLD.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                VERTEX_DATA_OLD.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            let mut ebo = std::mem::zeroed();
            gl.GenBuffers(1, &mut ebo);
            gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl.BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (INDEX_DATA.len() * std::mem::size_of::<i32>()) as gl::types::GLsizeiptr,
                INDEX_DATA.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            let pos_attrib = gl.GetAttribLocation(program, b"position\0".as_ptr() as *const _);
            let color_attrib = gl.GetAttribLocation(program, b"color\0".as_ptr() as *const _);
            gl.VertexAttribPointer(
                pos_attrib as gl::types::GLuint,
                3,
                gl::FLOAT,
                0,
                6 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                std::ptr::null(),
            );
            gl.VertexAttribPointer(
                color_attrib as gl::types::GLuint,
                3,
                gl::FLOAT,
                0,
                6 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                (3 * std::mem::size_of::<f32>()) as *const () as *const _,
            );
            gl.EnableVertexAttribArray(pos_attrib as gl::types::GLuint);
            gl.EnableVertexAttribArray(color_attrib as gl::types::GLuint);

            return Self {
                program,
                vao,
                vbo,
                gl,
            };
        }
    }

    pub fn draw(&self) {
        unsafe {
            self.gl.ClearColor(0.2, 0.3, 0.3, 0.7);
            self.gl.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            self.gl.UseProgram(self.program);

            self.gl.BindVertexArray(self.vao);

            self.gl
               .DrawElements(gl::TRIANGLES, 3, gl::UNSIGNED_INT, std::ptr::null());
        }
    }

    pub fn resize(&self, width: i32, height: i32) {
        println!("width: {}, height: {}", width, height);
        unsafe {
            self.gl.Viewport(0, 0, width, height);
        }
    }
}

impl Deref for Renderer {
    type Target = gl::Gl;

    fn deref(&self) -> &Self::Target {
        &self.gl
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteProgram(self.program);
            self.gl.DeleteBuffers(1, &self.vbo);
            self.gl.DeleteVertexArrays(1, &self.vao);
        }
    }
}

unsafe fn create_shader(
    gl: &gl::Gl,
    shader: gl::types::GLenum,
    source: &[u8],
) -> gl::types::GLuint {
    let shader = gl.CreateShader(shader);
    gl.ShaderSource(
        shader,
        1,
        [source.as_ptr().cast()].as_ptr(),
        std::ptr::null(),
    );
    gl.CompileShader(shader);
    let mut success = std::mem::zeroed();
    gl.GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
    if success != 0 {
        println!("Unable to compile: {}", shader);
    }
    shader
}

fn get_gl_string(gl: &gl::Gl, variant: gl::types::GLenum) -> Option<&'static CStr> {
    unsafe {
        let s = gl.GetString(variant);
        (!s.is_null()).then(|| CStr::from_ptr(s.cast()))
    }
}

#[rustfmt::skip]
static VERTEX_DATA_OLD: [f32; 18] = [
    -0.5, -0.5, 0.0, 1.0,  0.0,  0.0,
     0.0,  0.5, 0.0, 0.0,  1.0,  0.0,
     0.5, -0.5, 0.0, 0.0,  0.0,  1.0,
];


#[rustfmt::skip]
static VERTEX_DATA: [f32; 12] = [
     0.5,  0.5, 0.0,
     0.5, -0.5, 0.0, 
    -0.5, -0.5, 0.0,
    -0.5,  0.5, 0.0,
];

#[rustfmt::skip]
static INDEX_DATA: [i32; 3] = [
    0, 1, 2, 
];

const VERTEX_SHADER_SOURCE: &[u8] = b"
#version 100 
precision mediump float;

attribute vec3 position;
attribute vec3 color;

varying vec3 v_color;

void main() {
    gl_Position = vec4(position, 1.0);
    v_color = color;
}
\0";

const FRAGMENT_SHADER_SOURCE: &[u8] = b"
#version 100 
precision mediump float;

varying vec3 v_color;

void main() {
    gl_FragColor = vec4(v_color, 1.0);
}
\0";

struct Shader {
    id: gl::types::GLuint,
    gl: gl::Gl,
}

impl Shader {
    pub fn new(gl: &gl::Gl, source: &[u8], shader_type: gl::types::GLenum) -> Self {

        unsafe {
            let shader = gl.CreateShader(shader_type); 

            gl.ShaderSource(
                shader,
                1,
                [source.as_ptr().cast()].as_ptr(),
                std::ptr::null(),
            );
            gl.CompileShader(shader);
            let mut success = std::mem::zeroed();
            gl.GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
            if success != 0 {
                println!("Unable to compile: {}", shader);
            }
            Self {id: shader, gl: gl.clone() }
        }
    }

    pub fn use_shader(&self) {



    }

    pub fn set_bool(&self, name: &str, value: bool) {

    }
    pub fn set_int(&self, name: &str, value: bool) {

    }
    pub fn set_float(&self, name: &str, value: bool) {

    }
    pub fn set_mat4(&self, name: &str, value: bool) {

    }
}
