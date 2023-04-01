//! Application wrapper
use std::{borrow::Cow, fmt};

use winit::{
    dpi::LogicalSize,
    event_loop::{EventLoop, EventLoopBuilder},
    platform::run_return::EventLoopExtRunReturn,
    window::{Window, WindowBuilder},
};

use crate::GersError;

pub struct App {
    /// Main window
    window: Window,
    /// Event loop target
    event_loop: EventLoop<()>,
}

pub struct WindowConf {
    pub width: u32,
    pub height: u32,
    pub title: Cow<'static, str>,
}

impl Default for WindowConf {
    fn default() -> Self {
        Self {
            width: 640,
            height: 480,
            title: "gers".into(),
        }
    }
}

/// GERS application control flow.
///
/// This is to notify the program on why the application stopped.
#[derive(Debug)]
pub enum GersControl {
    /// Application is done and process can be shutdown.
    Shutdown,
    /// Application requests to be restarted.
    Restart,
}

impl GersControl {
    pub fn set_restart(&mut self) {
        *self = GersControl::Restart
    }

    pub fn set_shutdown(&mut self) {
        *self = GersControl::Shutdown
    }
}

impl fmt::Display for GersControl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Shutdown => write!(f, "shutdown"),
            Self::Restart => write!(f, "restart"),
        }
    }
}

impl App {
    pub fn new(window_conf: &WindowConf) -> Result<Self, GersError> {
        // Event loop can only be created once per process.
        let event_loop = EventLoopBuilder::new().build();

        let window = Self::create_main_window(window_conf, &event_loop)?;
        let app = App { window, event_loop };

        Ok(app)
    }

    /// Recreates the main window.
    pub fn recreate_window(&mut self, window_conf: &WindowConf) -> Result<(), GersError> {
        println!("recreating window");
        self.window = Self::create_main_window(window_conf, &self.event_loop)?;
        Ok(())
    }

    fn create_main_window(window_conf: &WindowConf, event_loop: &EventLoop<()>) -> Result<Window, GersError> {
        let inner_size = LogicalSize::new(window_conf.width, window_conf.height);
        let window = WindowBuilder::new()
            .with_inner_size(inner_size)
            .with_title(window_conf.title.to_string())
            .build(event_loop)?;
        Ok(window)
    }

    /// Initialise the hardware renderer context.
    pub fn init_graphics(&mut self) -> Result<(), GersError> {
        todo!()
    }
}

/// Event loop runner
impl App {
    #[allow(clippy::single_match)] // `if let` is less readable in this heavily nested event loop
    pub fn run(&mut self) -> Result<GersControl, GersError> {
        use winit::event::{ElementState, Event as E, VirtualKeyCode as K, WindowEvent as WE};

        // Main window ID
        let main_id = self.window.id();

        let mut app_control = GersControl::Shutdown;

        self.event_loop.run_return(|event, _, control_flow| {
            control_flow.set_poll();

            match event {
                E::MainEventsCleared => {
                    // Application update code.

                    // Queue a RedrawRequested event.
                    //
                    // You only need to call this if you've determined that you need to redraw, in
                    // applications which do not always need to. Applications that redraw continuously
                    // can just render here instead.
                    self.window.request_redraw();
                }
                E::RedrawRequested(_) => {
                    // Redraw the application.
                    //
                    // It's preferable for applications that do not render continuously to render in
                    // this event rather than in MainEventsCleared, since rendering in here allows
                    // the program to gracefully handle redraws requested by the OS.
                }
                E::WindowEvent { window_id, event } if window_id == main_id => {
                    match event {
                        WE::KeyboardInput { input, .. } => {
                            if input.state == ElementState::Released {
                                match input.virtual_keycode {
                                    Some(K::Escape) => {
                                        control_flow.set_exit();
                                        app_control.set_restart();
                                    }
                                    _ => { /* blank */ }
                                }
                            }
                        }
                        WE::CloseRequested => {
                            println!("Close Requested");
                            control_flow.set_exit();
                        }
                        _ => { /* blank */ }
                    }
                }
                _ => { /* blank */ }
            }
        });

        Ok(app_control)
    }
}
