#![allow(unused)]
mod builtins;
mod eval;
mod lex;
mod parse;
mod state;

use crate::state::State;

use anyhow::Result;
use notify::event::{CreateKind, DataChange, ModifyKind};
use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use vello::peniko::color::palette;
use vello::util::{RenderContext, RenderSurface};
use vello::{AaConfig, Renderer, RendererOptions, Scene};
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalPosition, PhysicalSize};
use winit::event::{MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::Window;

use vello::wgpu;

use crate::eval::{Interpreter, Output};
use crate::parse::parse;

#[derive(Debug)]
enum RenderState<'s> {
    /// `RenderSurface` and `Window` for active rendering.
    Active {
        // The `RenderSurface` and the `Window` must be in this order, so that the surface is dropped first.
        surface: Box<RenderSurface<'s>>,
        window: Arc<Window>,
    },
    /// Cache a window so that it can be reused when the app is resumed after being suspended.
    Suspended(Option<Arc<Window>>),
}

struct App<'s> {
    // The vello RenderContext which is a global context that lasts for the
    // lifetime of the application
    context: RenderContext,

    // An array of renderers, one per wgpu device
    renderers: Vec<Option<Renderer>>,

    // State for our example where we store the winit Window and the wgpu Surface
    render_state: RenderState<'s>,

    // A vello Scene which is a data structure which allows one to build up a
    // description a scene to be drawn (with paths, fills, images, text, etc)
    // which is then passed to a renderer for rendering
    scene: Scene,

    state: State,
}

impl ApplicationHandler<String> for App<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let RenderState::Suspended(cached_window) = &mut self.render_state else {
            return;
        };

        let window = cached_window
            .take()
            .unwrap_or_else(|| create_winit_window(event_loop));

        // Create a vello Surface
        let size = window.inner_size();
        let surface_future = self.context.create_surface(
            window.clone(),
            size.width,
            size.height,
            wgpu::PresentMode::AutoVsync,
        );
        let surface = pollster::block_on(surface_future).expect("Error creating surface");

        // Create a vello Renderer for the surface (using its device id)
        self.renderers
            .resize_with(self.context.devices.len(), || None);
        self.renderers[surface.dev_id]
            .get_or_insert_with(|| create_vello_renderer(&self.context, &surface));

        // Save the Window and Surface to a state variable
        self.render_state = RenderState::Active {
            surface: Box::new(surface),
            window,
        };
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        if let RenderState::Active { window, .. } = &self.render_state {
            self.render_state = RenderState::Suspended(Some(window.clone()));
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, input: String) {
        println!("--------------------------------");
        let items = match parse(input.as_str()) {
            Ok(items) => items,
            Err(err) => {
                println!("Parse error - {err}");
                return;
            }
        };
        let mut interpreter = Interpreter::new();
        let output = match interpreter.run(items) {
            Ok(output) => output,
            Err(err) => {
                println!("{err}");
                return;
            }
        };
        for (name, value) in output.constants.iter() {
            if let Some(name) = name {
                print!("{name} = ");
            }
            println!("{value}");
        }
        self.state.update(interpreter, output);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        // Only process events for our window, and only when we have a surface.
        let (surface, window) = match &mut self.render_state {
            RenderState::Active { surface, window } if window.id() == window_id => {
                (surface, window)
            }
            _ => return,
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::Resized(size) => {
                self.context
                    .resize_surface(surface, size.width, size.height);
                self.state.set_window_size(size.width, size.height);
            }

            WindowEvent::CursorMoved { position, .. } => {
                self.state.handle_cursor_move(position);
            }

            WindowEvent::MouseInput {
                state: mouse_state,
                button: MouseButton::Left,
                ..
            } => {
                self.state.handle_mouse_input(mouse_state);
            }

            WindowEvent::MouseWheel { delta, .. } => {
                self.state.handle_scroll(delta);
            }

            WindowEvent::RedrawRequested => {
                window.request_redraw();
                // Empty the scene of objects to draw. You could create a new Scene each time, but in this case
                // the same Scene is reused so that the underlying memory allocation can also be reused.
                self.scene.reset();

                let width = surface.config.width;
                let height = surface.config.height;

                // Re-add the objects to draw to the scene.
                self.state.render(&mut self.scene);

                // Get a handle to the device
                let device_handle = &self.context.devices[surface.dev_id];

                // Render to a texture, which we will later copy into the surface
                self.renderers[surface.dev_id]
                    .as_mut()
                    .unwrap()
                    .render_to_texture(
                        &device_handle.device,
                        &device_handle.queue,
                        &self.scene,
                        &surface.target_view,
                        &vello::RenderParams {
                            base_color: palette::css::WHITE, // Background color
                            width,
                            height,
                            antialiasing_method: AaConfig::Msaa16,
                        },
                    )
                    .expect("failed to render to surface");

                // Get the surface's texture
                let surface_texture = surface
                    .surface
                    .get_current_texture()
                    .expect("failed to get surface texture");

                // Perform the copy
                let mut encoder =
                    device_handle
                        .device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Surface Blit"),
                        });
                surface.blitter.copy(
                    &device_handle.device,
                    &mut encoder,
                    &surface.target_view,
                    &surface_texture
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default()),
                );
                device_handle.queue.submit([encoder.finish()]);
                // Queue the texture to be presented on the surface
                surface_texture.present();

                device_handle
                    .device
                    .poll(wgpu::Maintain::Poll)
                    .panic_on_timeout();
            }
            _ => {}
        }
    }
}

fn main() -> Result<()> {
    let mut app = App {
        context: RenderContext::new(),
        renderers: vec![],
        render_state: RenderState::Suspended(None),
        scene: Scene::new(),
        state: State::new(),
    };

    let event_loop = EventLoop::<String>::with_user_event().build()?;
    let proxy = event_loop.create_proxy();

    // println!("{interpreter:#?}");

    let mut last_event_time = Instant::now();
    let mut watcher = notify::recommended_watcher(move |event| match event {
        Ok(Event {
            kind,
            //     EventKind::Any
            //     | EventKind::Create(CreateKind::Any | CreateKind::File)
            //     | EventKind::Modify(ModifyKind::Any | ModifyKind::Data(_)),
            paths,
            ..
        }) => {
            if last_event_time.elapsed() < Duration::from_millis(100) {
                return;
            }
            last_event_time = Instant::now();

            let Some((path, input)) = paths.into_iter().rev().find_map(|path| {
                match std::fs::read_to_string(path.as_path()) {
                    Ok(string) => Some((path, string)),
                    Err(_) => None,
                }
            }) else {
                return;
            };

            // println!("Running {}", path.display());
            // println!("Kind: {kind:?}");
            // println!("Input: {}", input.as_str());
            proxy.send_event(input);
        }
        _ => {}
    })?;
    watcher.watch(Path::new("."), RecursiveMode::Recursive)?;

    event_loop.set_control_flow(ControlFlow::Wait);
    event_loop
        .run_app(&mut app)
        .expect("Couldn't run event loop");

    Ok(())
}

fn create_winit_window(event_loop: &ActiveEventLoop) -> Arc<Window> {
    let attr = Window::default_attributes()
        .with_min_inner_size(LogicalSize::new(300, 100))
        .with_resizable(true)
        .with_title("Vello Shapes");
    let window = event_loop.create_window(attr).unwrap();
    let size = window.primary_monitor().unwrap().size();
    window.set_outer_position(PhysicalPosition { x: 0, y: 0 });
    let _ = window.request_inner_size(PhysicalSize {
        width: size.width / 2,
        height: size.height / 2,
    });
    Arc::new(window)
}

fn create_vello_renderer(render_cx: &RenderContext, surface: &RenderSurface<'_>) -> Renderer {
    Renderer::new(
        &render_cx.devices[surface.dev_id].device,
        RendererOptions::default(),
    )
    .expect("Couldn't create renderer")
}
