#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_term;

use rust_wren::{
    handle::{FnSymbolRef, WrenCallHandle, WrenCallRef},
    prelude::*,
};
use slog::Drain;
use std::{env, fs, path::Path, time::Instant};
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod game;
mod graphics;
mod input;
mod util;
mod window;

use self::game::{init_game, register_game, Game};
use self::window::WrenWindowConfig;

fn main() -> Result<(), Box<dyn ::std::error::Error>> {
    // Logging
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let root = slog::Logger::root(drain, o!());
    let logger = root.new(o!("lang" => "Rust"));

    let _scope_guard = slog_scope::set_global_logger(logger.clone());
    let _log_guard = slog_stdlog::init_with_level(log::Level::Debug).unwrap();

    // Wren VM
    let wren_logger = root.new(o!("lang" => "Wren"));
    let mut vm = WrenBuilder::new()
        .with_module("window", |module| {
            module.register::<window::WrenWindowConfig>();
        })
        .with_write_fn(move |msg| {
            if msg != "\n" {
                info!(wren_logger, "{}", msg)
            }
        })
        .build();

    // Builtin modules
    vm.interpret("window", include_str!("window.wren"))?;
    vm.interpret("input", include_str!("input.wren"))?;
    vm.interpret("main", include_str!("main.wren"))?;
    register_game(&mut vm)?;

    // Validate the entry point exists
    let args: Vec<String> = env::args().collect();
    let script_entry = args[1].as_str();
    info!(logger, "Entry point: {}", script_entry);
    let entry_path = Path::new(script_entry);
    if !entry_path.exists() {
        error!(logger, "Entry point does not exist");
        return Err("Entry point does not exist".into());
    }

    // Window configuration
    // TODO: Move the Bootstrap stuff to the game.wren class.
    let mut window_conf: Option<WrenWindowConfig> = None;
    vm.context(|ctx| {
        info!(logger, "Get window configuration");

        let receiver = ctx
            .get_var("main", "Bootstrap")
            .expect("Failed to lookup Bootstrap class");
        let func = FnSymbolRef::compile(ctx, "window()").unwrap();
        let call_ref = WrenCallRef::new(receiver, func);

        window_conf = call_ref
            .call::<_, Option<WrenWindowConfig>>(ctx, ())
            .unwrap() // TODO: Will be WrenResult in future
            .map(|c| c.borrow().clone());
    });

    let mut game = {
        // In block so source is dropped when loading is done.
        // It's copied into Wren so no need to keep it in memory.
        let source = fs::read_to_string(entry_path)?;
        vm.interpret("core", &source)?;

        // TODO: We need this outer variable because the context call has no return yet.
        //       https://gitlab.com/vangroan/rust-wren/-/issues/12
        let mut maybe_game: Option<Game> = None;
        vm.context(|ctx| {
            maybe_game = Some(init_game(ctx));
        });
        maybe_game
    };

    {
        let conf = window_conf.unwrap_or_else(WrenWindowConfig::new);
        debug!(logger, "{:?}", conf);

        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_inner_size(LogicalSize::new(conf.size[0], conf.size[1]))
            .with_title(conf.title)
            .build(&event_loop)
            .unwrap();

        if let Some(game) = &game {
            vm.context(|ctx| {
                game.init.call::<_, ()>(ctx, ()).unwrap();
            });
        };

        let mut last_time = Instant::now();

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == window.id() => match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                        if let Some(game) = &mut game {
                            game.scale_factor = *scale_factor;
                        }
                    }
                    WindowEvent::ReceivedCharacter(c) => {
                        info!(logger, "ReceivedCharacter({})", c);
                        if let Some(game) = &mut game {
                            vm.context(|ctx| {
                                game.keyboard.push_char(ctx, *c).unwrap();
                            });
                        }
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        if let Some(game) = &mut game {
                            vm.context(|ctx| {
                                let logical = position.to_logical(game.scale_factor);
                                game.mouse.set_pos(ctx, logical, *position).unwrap();
                            });
                        }
                    }
                    WindowEvent::KeyboardInput { input, .. } => {
                        if let Some(game) = &mut game {
                            if let Some(virtual_keycode) = input.virtual_keycode {
                                vm.context(|ctx| {
                                    game.keyboard
                                        .set_key_state(ctx, virtual_keycode, input.state)
                                        .unwrap();
                                });
                            };
                        };

                        if let KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        } = input
                        {
                            *control_flow = ControlFlow::Exit;
                        }
                    }
                    _ => {}
                },
                Event::MainEventsCleared => {
                    // Frame update after events have been flushed.
                    let now = Instant::now();
                    let delta_time = now - last_time;
                    last_time = now;

                    if let Some(game) = &game {
                        vm.context(|ctx| {
                            game.set_delta_time
                                .call::<_, ()>(ctx, delta_time.as_secs_f64())
                                .unwrap();
                            game.update.call::<_, ()>(ctx, ()).unwrap();
                        });
                    };
                }
                Event::LoopDestroyed => {
                    debug!(logger, "Loop destroyed");
                    vm.context(|ctx| {
                        let receiver = ctx
                            .get_var("main", "Bootstrap")
                            .expect("Failed to lookup Bootstrap class");
                        let func = FnSymbolRef::compile(ctx, "shutdown()").unwrap();
                        let call_ref = WrenCallRef::new(receiver, func);
                        call_ref.call::<_, ()>(ctx, ());
                    });

                    // Release reference before VM is dropped.
                    if let Some(game) = game.take() {
                        drop(game);
                    };
                }
                _ => {}
            }
        });
    }
}
