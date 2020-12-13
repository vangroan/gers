use rust_wren::prelude::*;
use winit::{
    event::{Event, WindowEvent, KeyboardInput, ElementState, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
    dpi::LogicalSize,
};
use crate::util::crate_version;

#[wren_class(name = Window)]
pub struct WrenWindow {
    event_loop: Option<EventLoop<()>>,
    window: Window,
}

#[wren_methods]
impl WrenWindow {
    #[construct]
    pub fn new(width: f64, height: f64) -> Self {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_inner_size(LogicalSize::new(width, height))
            .build(&event_loop).unwrap();
        Self { event_loop: Some(event_loop), window }
    }

    pub fn run(&mut self) {
        // NOTE: Since foreign functions are suppose to be leaves in call graphs, putting the
        //       event loop in a foreign func is a non-starter.
        //
        //       Rather have Wren pass Rust a window configuration.
        let self_id = self.window.id();
        let event_loop = self.event_loop.take().unwrap();
        event_loop.run(move |event, _, control_flow| {
            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == self_id => match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput {
                        input,
                        ..
                    } => {
                        match input {
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            } => *control_flow = ControlFlow::Exit,
                            _ => {}
                        }
                    }
                    _ => {}
                }
                _ => {}
            }
        });
    }
}

#[wren_class(name = WindowConf)]
#[derive(Debug, Clone)]
pub struct WrenWindowConfig {
    pub size: [f64; 2],
    pub title: String,
}

#[wren_methods]
impl WrenWindowConfig {
    #[construct]
    pub fn new() -> Self {
        Self {
            size: [512., 512.],
            title: format!("Game Engine v{}", crate_version().full),
        }
    }

    pub fn set_size(&mut self, width: f64, height: f64) {
        self.size = [width, height];
    }

    pub fn set_title(&mut self, title: String) {
        self.title = title;
    }
}
