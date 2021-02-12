#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_term;

use rust_wren::{
    handle::{FnSymbolRef, WrenCallHandle, WrenCallRef},
    prelude::*,
};
use slog::Drain;
use std::{cell::RefCell, collections::VecDeque, env, fs, io::prelude::*, path::Path, rc::Rc};
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod input;
mod util;
mod window;

use rust_wren::handle::WrenHandle;
use window::WrenWindowConfig;

struct Game {
    init: WrenCallHandle,
    update: WrenCallHandle,
    push_event: WrenCallHandle,
    clear_events: WrenCallHandle,
    keyboard: input::Keyboard,
}

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
        .with_module("input", |module| {
            module.register::<input::WrenInput>();
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
    vm.interpret("game", include_str!("game.wren"))?;

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
        let source = fs::read_to_string(entry_path)?;
        vm.interpret("core", &source)?;

        // TODO: We need this outer variable because the context call has no return yet.
        //       https://gitlab.com/vangroan/rust-wren/-/issues/12
        let mut maybe_game: Option<Game> = None;
        vm.context(|ctx| {
            // FIXME: If function does not exist, the error is incorrect slot count.
            let get_handler = ctx.make_call_ref("game", "Game", "handler_").unwrap();
            println!("Slot count {}", ctx.slot_count());
            ctx.ensure_slots(10);
            println!("Slot count {}", ctx.slot_count());
            let undef = ctx.make_call_ref("game", "Game", "undefined").unwrap();
            println!("Slot type {:?}", ctx.slot_type(0));
            let handler = get_handler.call::<_, WrenRef>(ctx, ()).unwrap();
            println!("Update ref");

            // Init
            let init = {
                let handler = get_handler.call::<_, WrenRef>(ctx, ()).unwrap();
                let init_ref = FnSymbolRef::compile(ctx, "init()").unwrap();
                WrenCallRef::new(handler, init_ref).leak().unwrap()
            };

            // Update
            let update_ref = FnSymbolRef::compile(ctx, "process_()").unwrap();
            let update = WrenCallRef::new(handler, update_ref).leak().unwrap();

            // Keyboard Input (static scoped)
            let push_event = ctx
                .make_call_ref("game", "Game", "push_event_(_)")
                .unwrap()
                .leak()
                .unwrap();
            let clear_events = ctx
                .make_call_ref("game", "Game", "clear_events_()")
                .unwrap()
                .leak()
                .unwrap();
            let keyboard = input::Keyboard {
                set_key_press: ctx
                    .make_call_ref("input", "KeyboardInput", "setKeyPress_(_)")
                    .unwrap()
                    .leak()
                    .unwrap(),
                set_key_release: ctx
                    .make_call_ref("input", "KeyboardInput", "setKeyRelease_(_)")
                    .unwrap()
                    .leak()
                    .unwrap(),
                push_char: ctx
                    .make_call_ref("input", "KeyboardInput", "pushChar_(_)")
                    .unwrap()
                    .leak()
                    .unwrap(),
            };

            maybe_game = Some(Game {
                init,
                update,
                push_event,
                clear_events,
                keyboard,
            });
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

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == window.id() => {
                    match event {
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::ReceivedCharacter(c) => {
                            info!(logger, "ReceivedCharacter({})", c);
                            if let Some(game) = &mut game {
                                vm.context(|ctx| {
                                    game.keyboard.push_char(ctx, *c).unwrap();
                                });
                            }
                        }
                        WindowEvent::KeyboardInput { input, .. } => {
                            // println!(
                            //     "Serialized: {} {} {}",
                            //     input.scancode,
                            //     serde_json::to_string(&input.virtual_keycode).unwrap(),
                            //     serde_json::to_string(&input.state).unwrap(),
                            // );
                            // println!(
                            //     "Deserialized: {:?} {:?}",
                            //     serde_json::from_str::<winit::event::VirtualKeyCode>("\"W\""),
                            //     serde_json::from_str::<winit::event::ElementState>("\"Pressed\"")
                            // );

                            if let Some(game) = &mut game {
                                if let Some(virtual_keycode) = input.virtual_keycode {
                                    vm.context(|ctx| {
                                        game.push_event
                                            .call::<_, ()>(ctx, format!("{:?}", virtual_keycode));

                                        game.keyboard
                                            .set_key_state(
                                                ctx,
                                                virtual_keycode,
                                                input.state.clone(),
                                            )
                                            .unwrap();
                                    });
                                };
                            };

                            match input {
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    ..
                                } => {
                                    *control_flow = ControlFlow::Exit;
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
                Event::MainEventsCleared => {
                    if let Some(game) = &game {
                        vm.context(|ctx| {
                            game.update.call::<_, ()>(ctx, ()).unwrap();
                            game.clear_events.call::<_, ()>(ctx, ()).unwrap();
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

            // vm.context(|ctx| {
            //     // TODO: Leak call handle so we can hang on to it and not lookup variables each frame.
            //     let receiver = ctx
            //         .get_var("main", "Bootstrap")
            //         .expect("Failed to lookup Bootstrap class");
            //     let func = FnSymbolRef::compile(ctx, "update(_)").unwrap();
            //     let call_ref = WrenCallRef::new(receiver, func);
            //
            //     call_ref.call::<_, ()>(ctx, 16.0);
            // });
        });
    }
}
