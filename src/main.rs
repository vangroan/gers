#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_term;

use self::collections::{bind_collections, register_collections, COLLECTIONS_MODULE};
use self::errors::{log_wren_error, GersError};
use self::game::{init_game, register_game, Game};
use self::graphics::{
    bind_graphic_device, bind_graphics, init_graphic_device, register_graphic_device, register_graphics, GraphicDevice,
    GRAPHICS_MODULE,
};
use self::window::WrenWindowConfig;
use glutin::{
    dpi::LogicalSize, event_loop::EventLoop, window::WindowBuilder, Api, ContextBuilder, GlProfile, GlRequest,
};
use rust_wren::{
    handle::{FnSymbolRef, WrenCallRef},
    prelude::*,
    WrenResult,
};
use slog::Drain;
use std::{env, fs, path::Path, process};

mod errors;
mod game;
#[macro_use]
mod graphics;
mod collections;
mod input;
mod marker;
mod util;
mod window;

/// Register the builtin modules in the given Wren VM by interpreting
/// the script contents.
fn load_builtins(vm: &mut WrenVm) -> WrenResult<()> {
    register_collections(vm)?;
    vm.interpret("window", include_str!("window.wren"))?;
    vm.interpret("input", include_str!("input.wren"))?;
    register_graphics(vm)?;
    register_graphic_device(vm)?;
    vm.interpret("main", include_str!("main.wren"))?;
    register_game(vm)?;

    Ok(())
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
        .with_module(GRAPHICS_MODULE, |module| {
            bind_graphic_device(module);
            bind_graphics(module);
        })
        .with_module(COLLECTIONS_MODULE, |module| {
            bind_collections(module);
        })
        .with_write_fn(move |msg| {
            if msg != "\n" {
                info!(wren_logger, "{}", msg)
            }
        })
        .build();

    // Wren logger for runtime and compiler errors.
    let wren_logger = root.new(o!("lang" => "Wren"));

    // Builtin modules
    if let Err(err) = load_builtins(&mut vm) {
        log_wren_error(&wren_logger, &err);
        return Err(err.into());
    };

    // Validate the entry point exists
    let args: Vec<String> = env::args().collect();
    if args.len() <= 1 {
        error!(logger, "Specify an entry point script.");
        return Err(GersError::InvalidCmdArgs.into());
    }

    let script_entry = args[1].as_str();
    info!(logger, "Entry point: {}", script_entry);
    let entry_path = Path::new(script_entry);
    if !entry_path.exists() {
        error!(logger, "Entry point does not exist: {}", entry_path.display());
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

    let conf = window_conf.unwrap_or_else(WrenWindowConfig::new);
    debug!(logger, "{:?}", conf);

    // Create OpenGL context from window.
    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = WindowBuilder::new()
        .with_title(conf.title.clone())
        .with_inner_size(LogicalSize::new(conf.size[0], conf.size[1]));
    let windowed_context = ContextBuilder::new()
        .with_vsync(false)
        .with_gl(GlRequest::Specific(Api::OpenGl, (4, 5)))
        .with_gl_profile(GlProfile::Core)
        .build_windowed(wb, &event_loop)?;
    let windowed_context = unsafe { windowed_context.make_current().unwrap() };
    let device = unsafe { GraphicDevice::from_windowed_context(&windowed_context) };

    let mut game = {
        // In block so source is dropped when loading is done.
        // It's copied into Wren so no need to keep it in memory.
        let source = fs::read_to_string(entry_path)?;
        let interp_result = vm.interpret("core", &source);
        if let Err(err) = interp_result {
            log_wren_error(&wren_logger, &err);
            return Err(err.into());
        };

        let init_result = vm.context_result(|ctx| {
            let graphic_device_hooks = init_graphic_device(ctx, device);
            init_game(ctx, logger.clone(), windowed_context, graphic_device_hooks)
        });
        if let Err(err) = &init_result {
            log_wren_error(&wren_logger, &err);
        };
        init_result?
    };

    game.window_conf = conf;
    game.run(&mut vm, event_loop)?;

    Ok(())
}
