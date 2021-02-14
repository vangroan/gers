//! Game script entrypoint and hooks.
use super::{FpsCounter, FpsThrottle, FpsThrottlePolicy};
use crate::{
    errors::GersResult,
    input::{Keyboard, Mouse},
    window::WrenWindowConfig,
};
use rust_wren::{
    handle::{FnSymbolRef, WrenCallHandle, WrenCallRef},
    prelude::*,
    WrenContext, WrenResult,
};
use slog::Logger;
use std::time::{Duration, Instant};
use winit::platform::run_return::EventLoopExtRunReturn;
use winit::{
    event::Event,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

pub fn init_game(ctx: &mut WrenContext, logger: Logger) -> Game {
    // TODO:
    //  Change signature to return WrenResult
    //  Change all unwraps to ?

    // The user's game instance, which is the entry point from Rust into
    // the Wren program, is stored in a property.
    let get_handler = ctx.make_call_ref("game", "Game", "handler_").unwrap();

    // Delta Time
    let set_delta_time = ctx
        .make_call_ref("game", "Game", "deltaTime_=(_)")
        .unwrap()
        .leak()
        .unwrap();

    // Init
    let init = {
        let handler = get_handler.call::<_, WrenRef>(ctx, ()).unwrap();
        let init_ref = FnSymbolRef::compile(ctx, "init()").unwrap();
        WrenCallRef::new(handler, init_ref).leak().unwrap()
    };

    // Update
    let update = {
        let handler = get_handler.call::<_, WrenRef>(ctx, ()).unwrap();
        let update_ref = FnSymbolRef::compile(ctx, "process_()").unwrap();
        WrenCallRef::new(handler, update_ref).leak().unwrap()
    };

    // Mouse Input
    let mouse = Mouse {
        set_pos: ctx
            .make_call_ref("input", "Mouse", "setPos_(_,_,_,_)")
            .unwrap()
            .leak()
            .unwrap(),
        push_button: ctx
            .make_call_ref("input", "Mouse", "pushButton_(_,_)")
            .unwrap()
            .leak()
            .unwrap(),
    };

    // Keyboard Input
    let keyboard = Keyboard {
        set_key_press: ctx
            .make_call_ref("input", "Keyboard", "setKeyPress_(_)")
            .unwrap()
            .leak()
            .unwrap(),
        set_key_release: ctx
            .make_call_ref("input", "Keyboard", "setKeyRelease_(_)")
            .unwrap()
            .leak()
            .unwrap(),
        push_char: ctx
            .make_call_ref("input", "Keyboard", "pushChar_(_)")
            .unwrap()
            .leak()
            .unwrap(),
    };

    Game {
        logger,
        window_conf: WrenWindowConfig::new(),
        scale_factor: 1.0,
        set_delta_time,
        init,
        update,
        mouse,
        keyboard,
    }
}

/// Register builtin game module.
pub fn register_game(vm: &mut WrenVm) -> WrenResult<()> {
    vm.interpret("game", include_str!("game.wren"))
}

pub struct Game {
    pub logger: Logger,
    pub window_conf: WrenWindowConfig,
    pub scale_factor: f64,
    pub set_delta_time: WrenCallHandle,
    pub init: WrenCallHandle,
    pub update: WrenCallHandle,
    pub mouse: Mouse,
    pub keyboard: Keyboard,
}

impl Game {
    /// Run the game loop.
    ///
    /// Consumes the game and never returns.
    ///
    /// VM is borrowed so that it's dropped after the `Game`, which
    /// contains handles that need to be released first.
    pub fn run(mut self, vm: &'_ mut WrenVm, mut event_loop: EventLoop<()>, mut window: Window) -> GersResult<()> {
        // Initialisation hook.
        //
        // After Window has been initialised, before event loop starts.
        vm.context(|ctx| {
            self.init.call::<_, ()>(ctx, ()).unwrap();
        });

        // Frame rate throttle to prevent excessive CPU usage, battery drainage
        // and laptop fans freaking out.
        let mut fps_throttle = FpsThrottle::new(60, FpsThrottlePolicy::Yield);

        // Frame rate metric.
        let mut fps_counter = FpsCounter::new();

        // Time at which delta time is calculated.
        // Also serves as the mark where one frame ends and the next frame starts.
        let mut last_time = Instant::now();
        let mut delta_time = Duration::from_millis(16);

        // Event loop is a diverging function, it never returns.
        //
        // Some considerations must be taken for the drop order. The Main
        // Wren VM must be dropped last, otherwise all the call handles
        // will be dangling.
        event_loop.run_return(move |event, _, control_flow| {
            // Game loop, so we want each loop iteration to start
            // immediately when the previous one is done.
            *control_flow = ControlFlow::Poll;

            let args = FrameArgs {
                vm,
                event,
                control_flow,
                window: &mut window,

                last_time: &mut last_time,
                delta_time: &mut delta_time,

                fps_counter: &mut fps_counter,
                fps_throttle: &mut fps_throttle,
            };

            // TODO: handle error return.
            //       Either panic or channel error to script.
            let _result = self.handle_event(args);
        });

        Ok(())
    }

    /// Dispatch single event.
    #[inline]
    fn handle_event(&mut self, args: FrameArgs) -> GersResult<()> {
        use winit::event::{ElementState, Event as E, KeyboardInput, VirtualKeyCode, WindowEvent as WE};

        let FrameArgs {
            vm,
            event,
            control_flow,
            window,

            last_time,
            delta_time,

            fps_counter,
            fps_throttle,
        } = args;

        match event {
            E::NewEvents(_) => {
                // Boundary where frame starts.
                let now = Instant::now();
                *delta_time = now - *last_time;
                *last_time = now;

                fps_counter.add(*delta_time);
                Ok(())
            }
            E::WindowEvent { ref event, window_id } if window_id == window.id() => match event {
                WE::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                    Ok(())
                }
                WE::Resized(_inner_size) => {
                    // TODO: Resize windowed context when glutin openGL is in place.
                    Ok(())
                }
                WE::ScaleFactorChanged { scale_factor, .. } => {
                    self.scale_factor = *scale_factor;
                    Ok(())
                }
                WE::MouseInput { button, state, .. } => {
                    vm.context(|ctx| {
                        self.mouse.push_button(ctx, *button, *state).unwrap();
                    });
                    Ok(())
                }
                WE::CursorMoved { position, .. } => {
                    vm.context(|ctx| {
                        let logical = position.to_logical(self.scale_factor);
                        self.mouse.set_pos(ctx, logical, *position).unwrap();
                    });
                    Ok(())
                }
                WE::ReceivedCharacter(c) => {
                    vm.context(|ctx| {
                        self.keyboard.push_char(ctx, *c).unwrap();
                    });
                    Ok(())
                }
                WE::KeyboardInput { input, .. } => {
                    if let Some(virtual_keycode) = input.virtual_keycode {
                        vm.context(|ctx| {
                            self.keyboard.set_key_state(ctx, virtual_keycode, input.state).unwrap();
                        });
                    };

                    // TODO: Script should decide what to do with escape
                    if let KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::Escape),
                        ..
                    } = input
                    {
                        *control_flow = ControlFlow::Exit;
                    }

                    Ok(())
                }
                _ => Ok(()),
            },
            E::MainEventsCleared => {
                // Frame update after events have been flushed.
                vm.context(|ctx| {
                    self.set_delta_time
                        .call::<_, ()>(ctx, delta_time.as_secs_f64())
                        .unwrap();
                    self.update.call::<_, ()>(ctx, ()).unwrap();
                });

                window.set_title(&format!("{} - {:.2} FPS", self.window_conf.title, fps_counter.fps()));

                // Emit redraw event for rendering. Integrates
                // our render step with redraw requests from OS.
                window.request_redraw();

                Ok(())
            }
            E::RedrawRequested(_window_id) => {
                // TODO: When glutin and opengl are in.
                Ok(())
            }
            E::RedrawEventsCleared => {
                // Fill up the rest of the frame so we can hit the target FPS.
                fps_throttle.throttle(*last_time);
                Ok(())
            }
            E::LoopDestroyed => {
                debug!(self.logger, "Loop destroyed");
                vm.context(|ctx| {
                    let receiver = ctx
                        .get_var("main", "Bootstrap")
                        .expect("Failed to lookup Bootstrap class");
                    let func = FnSymbolRef::compile(ctx, "shutdown()").unwrap();
                    let call_ref = WrenCallRef::new(receiver, func);
                    call_ref.call::<_, ()>(ctx, ());
                });

                // Release reference before VM is dropped.
                // drop(self);
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

struct FrameArgs<'a, 'b> {
    vm: &'a mut WrenVm,
    event: Event<'b, ()>,
    control_flow: &'a mut ControlFlow,
    window: &'a mut Window,

    last_time: &'a mut Instant,
    delta_time: &'a mut Duration,

    fps_throttle: &'a mut FpsThrottle,
    fps_counter: &'a mut FpsCounter,
}
