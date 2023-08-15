use std::{ffi::CString, num::NonZeroU32};

use glutin::{
    config::ConfigTemplateBuilder,
    context::{ContextAttributesBuilder, PossiblyCurrentContext},
    display::GetGlDisplay,
    prelude::{GlConfig, GlDisplay, NotCurrentGlContextSurfaceAccessor},
    surface::{GlSurface, Surface, WindowSurface},
};
use glutin_winit::{DisplayBuilder, GlWindow};
use raw_window_handle::HasRawWindowHandle;
use winit::{
    dpi::PhysicalSize,
    event::{DeviceEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use crate::{camera::Camera, colliding_renderer::CubeRenderer};

pub fn build_gl_state(
    event_loop: &EventLoop<()>
) -> Option<(PossiblyCurrentContext, Surface<WindowSurface>, Window)>
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

pub fn track_user_input(
    event: DeviceEvent,
    camera: &mut Camera,
    now_keys: &mut [bool; 255],
)
{
    match event
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
    }
}

pub fn handle_window_event(
    event: WindowEvent,
    control_flow: &mut ControlFlow,
    state: &Option<(PossiblyCurrentContext, Surface<WindowSurface>, Window)>,
    renderer: &CubeRenderer,
)
{
    match event
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
                    renderer.resize(size.width as i32, size.height as i32);
                }
            }
        }
        WindowEvent::CloseRequested =>
        {
            control_flow.set_exit();
        }
        _ =>
        {}
    }
}
