use std::error::Error;
use std::num::NonZeroU32;

use glutin::config::{Config, ConfigTemplateBuilder};
use glutin::context::{ContextApi, ContextAttributesBuilder, Version};
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::surface::SwapInterval;
use glutin_winit::{self, DisplayBuilder, GlWindow};
use renderer::Renderer;
use winit::event::{ElementState, Event, KeyEvent, WindowEvent};
use winit::event_loop::EventLoopBuilder;
use winit::keyboard::{Key, NamedKey};
use winit::window::WindowBuilder;

mod assets;
mod error;
mod foliage;
mod mesh;
mod renderer;
mod scene;
mod shader;
mod terrain;
mod texture;

/// This main function and the renderer architecture have been adapted and somewhat
/// slimmed down from
/// <https://github.com/rust-windowing/glutin/blob/e1bf1e22a3e2f0e3dc4213f85c10f33049ce8d77/glutin_examples/examples/window.rs>.
/// The better place to start reading is in [`renderer`] or [`scene`].
pub fn main() -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoopBuilder::new().build().unwrap();

    // Only Windows requires the window to be present before creating the display.
    // Other platforms don't really need one.
    let window_builder = WindowBuilder::new().with_title("Undergrowth");

    // The template will match only the configurations supporting rendering
    // to windows.
    //
    // XXX We force transparency only on macOS, given that EGL on X11 doesn't
    // have it, but we still want to show window. The macOS situation is like
    // that, because we can query only one config at a time on it, but all
    // normal platforms will return multiple configs, so we can find the config
    // with transparency ourselves inside the `reduce`.
    let template = ConfigTemplateBuilder::new()
        .with_alpha_size(8)
        .with_transparency(cfg!(cgl_backend));

    let display_builder = DisplayBuilder::new().with_window_builder(Some(window_builder));

    let (mut window, gl_config) = display_builder.build(&event_loop, template, gl_config_picker)?;

    // No idea what samples are meant here :shrug:
    // println!("Picked a config with {} samples", gl_config.num_samples());

    // XXX The display could be obtained from any object created by it, so we can
    // query it from the config.
    let gl_display = gl_config.display();

    // The context creation part.
    let context_attributes = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::OpenGl(Some(Version::new(4, 1))))
        .build(None);
    let mut not_current_gl_context =
        unsafe { gl_display.create_context(&gl_config, &context_attributes) }.ok();

    let mut state = None;
    let mut renderer = None;

    event_loop.run(move |event, window_target| {
        match event {
            Event::Resumed => {
                let window = window.take().unwrap_or_else(|| {
                    let window_builder = WindowBuilder::new()
                        .with_transparent(true)
                        .with_title("Glutin triangle gradient example (press Escape to exit)");
                    glutin_winit::finalize_window(window_target, window_builder, &gl_config)
                        .unwrap()
                });

                let attrs = window.build_surface_attributes(Default::default());
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
                        // Some platforms like EGL require resizing GL surface to update the size
                        // Notable platforms here are Wayland and macOS, other don't require it
                        // and the function is no-op, but it's wise to resize it for portability
                        // reasons.
                        if let Some((gl_context, gl_surface, _)) = &state {
                            gl_surface.resize(
                                gl_context,
                                NonZeroU32::new(size.width).unwrap(),
                                NonZeroU32::new(size.height).unwrap(),
                            );
                            let renderer = renderer.as_mut().unwrap();
                            renderer.resize(size.width as i32, size.height as i32);
                        }
                    }
                }
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            logical_key: Key::Named(NamedKey::Escape),
                            ..
                        },
                    ..
                } => window_target.exit(),
                WindowEvent::RedrawRequested => {
                    if let Some((gl_context, gl_surface, window)) = &state {
                        let renderer = renderer.as_mut().unwrap();
                        renderer.draw();
                        window.request_redraw();

                        gl_surface.swap_buffers(gl_context).unwrap();
                    }
                }
                WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            logical_key: Key::Named(NamedKey::ArrowRight),
                            state: ElementState::Released,
                            ..
                        },
                    ..
                } => 'block: {
                    let Some(renderer) = renderer.as_mut() else {
                        break 'block;
                    };
                    renderer.next_scene();
                }
                WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            logical_key: Key::Named(NamedKey::ArrowLeft),
                            state: ElementState::Released,
                            ..
                        },
                    ..
                } => 'block: {
                    let Some(renderer) = renderer.as_mut() else {
                        break 'block;
                    };
                    renderer.prev_scene();
                }
                _ => (),
            },
            _ => (),
        }
    })?;

    Ok(())
}

// Find the config with the maximum number of samples, so our triangle will be
// smooth.
pub fn gl_config_picker(configs: Box<dyn Iterator<Item = Config> + '_>) -> Config {
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
}
