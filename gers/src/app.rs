//! Application wrapper
use std::{borrow::Cow, fmt, num::NonZeroU32};

use glutin::config::Config;
use glutin::config::ConfigTemplateBuilder;
use glutin::context::{ContextApi, ContextAttributesBuilder, GlProfile, PossiblyCurrentContext, Version as GlVersion};
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::surface::{Surface, SwapInterval, WindowSurface};
use glutin_winit::{DisplayBuilder, GlWindow};
use raw_window_handle::HasRawWindowHandle;
use winit::dpi::LogicalSize;
use winit::event_loop::{EventLoop, EventLoopBuilder};
use winit::platform::run_return::EventLoopExtRunReturn;
use winit::window::{Window, WindowBuilder};

use crate::app_layer::AppLayer;
use crate::gfx::{opengl::OpenGLBackend, Render};
use crate::{color::Color, input::InputMap, GersError, InternStr, UpdateCtx};

pub struct App {
    /// Main window
    window: Window,
    main_context: PossiblyCurrentContext,
    surface: Surface<WindowSurface>,
    // gl: glow::Context,
    render: Render,
    // context: Box<dyn NotCurrentGlContext>,
    /// Event loop target
    event_loop: EventLoop<()>,
    /// Mapping of input events to application specified actions.
    input_map: InputMap,
    layer: Option<Box<dyn AppLayer>>,
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

        let inner_size = LogicalSize::new(window_conf.width, window_conf.height);
        let window_builder = WindowBuilder::new()
            .with_inner_size(inner_size)
            .with_transparent(true)
            .with_title(window_conf.title.to_string());

        // The template will match only the configurations supporting rendering
        // to windows.
        //
        // We force transparency only on macOS, given that EGL on X11 doesn't
        // have it, but we still want to show a window. The macOS situation is like
        // that, because we can query only one config at a time on it, but all
        // normal platforms will return multiple configs, so we can find the config
        // with transparency ourselves inside the `reduce`.
        let template = ConfigTemplateBuilder::new()
            .with_alpha_size(8)
            // .with_transparency(cfg!(cgl_backend));
            .with_transparency(true);

        // https://github.com/rust-windowing/glutin/blob/master/glutin_examples/src/lib.rs
        let (window, gl_config) = DisplayBuilder::new()
            .with_window_builder(Some(window_builder))
            .build(&event_loop, template, |configs| {
                // Find a config that supports transparency, and has the maximum number of samples.
                let mut config: Option<Config> = None;

                for c in configs {
                    if log::max_level() >= log::Level::Debug {
                        log::debug!(
                            "consider config: num_samples={}, supports_transparency={}",
                            c.num_samples(),
                            c.supports_transparency().unwrap_or(false)
                        );
                    }

                    // Does the next config support transparency?
                    let next_transparency = c.supports_transparency().unwrap_or(false);
                    // Does the previous config support transparency?
                    let prev_transparency = config
                        .as_ref()
                        .and_then(|config| config.supports_transparency())
                        .unwrap_or(false);

                    // Does the next config support transparency, but the previous config does not?
                    let supports_transparency = next_transparency && !prev_transparency;

                    // Does the next config have more samples than the previous config?
                    let more_samples =
                        c.num_samples() > config.as_ref().map(|config| config.num_samples()).unwrap_or(0);

                    if supports_transparency || more_samples {
                        config = Some(c);
                    }
                }

                config.expect("the system must supply at least one GL config")
            })
            .unwrap();

        if log::max_level() >= log::Level::Info {
            log::info!(
                "picked GL config with {} samples and {} transparency",
                gl_config.num_samples(),
                if gl_config.supports_transparency().unwrap_or(false) {
                    "with"
                } else {
                    "without"
                }
            );
        }

        // On Android, the window is not available when the OpenGL display has to be created.
        // However on Windows the main window must first exist before OpenGL can be initialized.
        let window = window.unwrap();

        // Raw handle is required to build the OpenGL context.
        let raw_window_handle = window.raw_window_handle();

        // The display could be obtained from any object created by it, so we
        // can query it from the config.
        let gl_display = gl_config.display();

        // The context creation part. It can be created before surface and that's how
        // it's expected in multithreaded + multiwindow operation mode, since you
        // can *send* `NotCurrentContext`, but not `Surface`.
        let context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::OpenGl(Some(GlVersion::new(3, 3))))
            .with_profile(GlProfile::Core)
            .build(Some(raw_window_handle));

        // Since glutin by default tries to create OpenGL core context, which may not be
        // present we should try gles.
        let fallback_context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::Gles(None))
            .build(Some(raw_window_handle));

        // Finally we can create the OpenGL context
        let not_current_gl_context = unsafe {
            gl_display
                .create_context(&gl_config, &context_attributes)
                .unwrap_or_else(|_| {
                    log::warn!("falling back to OpenGL ES");
                    gl_display
                        .create_context(&gl_config, &fallback_context_attributes)
                        .expect("failed to create context")
                })
        };

        let attrs = window.build_surface_attributes(<_>::default());
        let gl_surface = unsafe { gl_display.create_window_surface(&gl_config, &attrs).unwrap() };

        let gl_context = not_current_gl_context.make_current(&gl_surface).unwrap();

        // Attempt setting VSync
        if let Err(err) = gl_surface.set_swap_interval(&gl_context, SwapInterval::Wait(NonZeroU32::new(1).unwrap())) {
            log::error!("error setting vsync: {err:?}");
        }

        // Create glow context.
        //
        // NOTE: For WGL (Windows) the OpenGL context must be current,
        // otherwise a subset of functions are loaded.
        debug_assert!(
            gl_context.is_current(),
            "context must be current to load OpenGL functions"
        );

        // Create the renderer for this window.
        let backend = OpenGLBackend::new(&gl_display);
        log::info!("{}", backend.info());

        let render = Render::new(backend);

        // let window = Self::create_main_window(window_conf, &event_loop)
        //     .with_context(|| "creating the main window failed during application constructor")?;

        let input_map = InputMap::new();

        let app = App {
            window,
            main_context: gl_context,
            surface: gl_surface,
            render,
            event_loop,
            input_map,
            layer: None,
        };

        Ok(app)
    }

    /// Recreates the main window.
    pub fn recreate_window(&mut self, _window_conf: &WindowConf) -> Result<(), GersError> {
        log::info!("recreating window");
        // FIXME: Implement recreating app without recreating event loop
        // - Recreating EventLoop is not supported by winit.
        // - Creating the OpenGL context via glutin has gotten more complicated in version 0.30
        todo!("fix recreating window")
        // self.window = Self::create_main_window(window_conf, &self.event_loop)
        //     .with_context(|| "attempt to recreate the main window failed")?;
    }

    /// Initialise the hardware renderer context.
    pub fn init_graphics(&mut self) -> Result<(), GersError> {
        todo!()
    }

    /// Load input mappings from the given file.
    pub fn load_input_conf(&mut self, filepath: &str) -> Result<(), GersError> {
        self.input_map.load_file(filepath)
    }

    pub fn set_layer(&mut self, layer: impl AppLayer + 'static) {
        self.layer = Some(Box::new(layer));
    }
}

/// Event loop runner
impl App {
    #[allow(clippy::single_match)] // `if let` is less readable in this heavily nested event loop
    pub fn run(&mut self) -> Result<GersControl, GersError> {
        use winit::event::{Event as E, WindowEvent as WE};

        if self.layer.is_none() {
            log::warn!("warn: no app layer added");
        }

        // Main window ID
        let main_id = self.window.id();

        let mut app_control = GersControl::Shutdown;
        // let mut devconsole_open = false;

        let bkg_color = Color::from_rgba(29, 33, 40, 229);
        log::debug!("background color: {bkg_color}");

        self.event_loop.run_return(|event, _, control_flow| {
            control_flow.set_poll();

            match event {
                E::NewEvents(_) => {
                    // Frame start.
                    self.input_map.clear_releases();
                }
                E::MainEventsCleared => {
                    // Application update code.

                    if self.input_map.is_action_released("restart") {
                        log::info!("restart released");
                        control_flow.set_exit();
                        app_control.set_restart();
                    }

                    if let Some(layer) = &mut self.layer {
                        let ctx = UpdateCtx {
                            window: &mut self.window,
                            input: &self.input_map,
                        };

                        layer.update(ctx);
                    }

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
                    self.render.clear_color(bkg_color);

                    self.surface.swap_buffers(&self.main_context).unwrap();
                }
                E::WindowEvent { window_id, event } if window_id == main_id => {
                    match event {
                        WE::Resized(size) => {
                            // Zero sized surface is invalid.
                            if size.width != 0 && size.height != 0 {
                                // Some platforms like EGL require resizing GL surface to update the size.
                                // Notable platforms here are Wayland and macOS, others don't require it
                                // and the function is no-op, but it's wise to resize it for portability
                                // reasons.
                                self.surface.resize(
                                    &self.main_context,
                                    NonZeroU32::new(size.width).unwrap(),
                                    NonZeroU32::new(size.height).unwrap(),
                                );
                                // TODO: Resize OpenGL viewport.
                            }
                        }
                        WE::KeyboardInput { input, .. } => {
                            if let Some(keycode) = input.virtual_keycode {
                                self.input_map.set_key_state(keycode, input.state);
                            }
                        }
                        WE::CloseRequested => {
                            log::info!("Close Requested");
                            control_flow.set_exit();
                        }
                        _ => { /* blank */ }
                    }
                }
                _ => { /* blank */ }
            }
        });

        InternStr::gc();

        Ok(app_control)
    }
}
