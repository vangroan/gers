#[macro_use]
extern crate slog;
extern crate slog_term;
extern crate slog_async;

use slog::Drain;
use rust_wren::{prelude::*, handle::{FnSymbol, WrenCallRef}};
use winit::{
    event::{Event, WindowEvent, KeyboardInput, ElementState, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
    dpi::LogicalSize,
};

mod window;
mod util;

use window::WrenWindowConfig;

fn main() -> Result<(), Box<dyn ::std::error::Error>> {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let root = slog::Logger::root(drain, o!());
    let logger = root.new(o!("lang" => "Rust"));

    let _scope_guard = slog_scope::set_global_logger(logger.clone());
    let _log_guard = slog_stdlog::init_with_level(log::Level::Debug).unwrap();

    // TODO: Provide VM with custom write and error print handlers so we can have a Wren logger.
    let _wren_logger = root.new(o!("lang" => "Wren"));
    let mut vm = WrenBuilder::new()
        .with_module("window", |module| {
            module.register::<window::WrenWindowConfig>();
        })
        .build();

    vm.interpret("window", include_str!("window.wren"))?;
    vm.interpret("main", include_str!("main.wren"))?;

    let mut window_conf: Option<WrenWindowConfig> = None;
    vm.context(|ctx| {
        info!(logger, "Get window configuration");

        let receiver = ctx.get_var("main", "Bootstrap").expect("Failed to lookup Bootstrap class");
        let func = FnSymbol::compile(ctx, "window()");
        let call_ref = WrenCallRef::new(receiver, func);

        window_conf = call_ref.call::<_, Option<WrenWindowConfig>>(ctx, ())
            .unwrap()// TODO: Will be WrenResult in future
            .map(|c| c.borrow().clone());
    });

    {
        let conf = window_conf.unwrap_or_else(|| WrenWindowConfig::new());
        debug!(logger, "{:?}", conf);

        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_inner_size(LogicalSize::new(conf.size[0], conf.size[1]))
            .with_title(conf.title)
            .build(&event_loop).unwrap();

        event_loop.run(move |event, _, control_flow| {
            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == window.id() => match event {
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
                Event::LoopDestroyed => {
                    debug!(logger, "Loop destroyed");
                    vm.context(|ctx| {
                        let receiver = ctx.get_var("main", "Bootstrap").expect("Failed to lookup Bootstrap class");
                        let func = FnSymbol::compile(ctx, "shutdown()");
                        let call_ref = WrenCallRef::new(receiver, func);
                        call_ref.call::<_, ()>(ctx, ());
                    });
                }
                _ => {}
            }

            vm.context(|ctx| {
                // TODO: Leak call handle so we can hang on to it and not lookup variables each frame.
                let receiver = ctx.get_var("main", "Bootstrap").expect("Failed to lookup Bootstrap class");
                let func = FnSymbol::compile(ctx, "update(_)");
                let call_ref = WrenCallRef::new(receiver, func);

                call_ref.call::<_, ()>(ctx, 16.0);
            });
        });
    }

    Ok(())
}
