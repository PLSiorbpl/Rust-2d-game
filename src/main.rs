pub mod rendering;

use std::process::abort;
use std::rc::Rc;
use std::sync::Arc;
use log::error;
use wgpu::{Backends, Instance, InstanceDescriptor};
use winit::application::ApplicationHandler;
use winit::event::{KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::KeyCode::Escape;
use winit::window::WindowId;
use crate::rendering::renderer::Renderer;
use crate::rendering::window::{Window, WindowConfiguration};

pub struct State {
    window: Window,
    renderer: Renderer,
}

#[derive(Default)]
pub struct App {
    state: Option<State>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::PRIMARY,
            flags: Default::default(),
            memory_budget_thresholds: Default::default(),
            backend_options: Default::default(),
            display: None,
        });

        let mut window = Window::new(&instance, event_loop, WindowConfiguration {
            width: 800,
            height: 600,
            title: "ee",
        }).unwrap_or_else(|err| {
            error!("{:?}", err);
            abort();
        });

        let renderer = pollster::block_on(Renderer::new(&instance, Some(&window)));

        window.resize(&renderer, 0, 0);

        self.state = Some(State {
            window,
            renderer,
        })
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        let state = self.state.as_mut().unwrap();

        match event {
            WindowEvent::KeyboardInput {
                event, ..
            } => {
                match event {
                    KeyEvent { physical_key, .. } => {
                        if (physical_key == Escape) {
                            event_loop.exit();
                        }
                    }
                }
            },
            WindowEvent::Resized(size) => {
                state.window.resize(&state.renderer, size.width, size.height);
            },
            _ => {}
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();

    let event_loop = EventLoop::new()?;
    let mut app = App::default();
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut app)?;

    Ok(())
}
