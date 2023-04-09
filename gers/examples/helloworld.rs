#[macro_use]
extern crate slog;
use gers::{prelude::*, App, GersControl, WindowConf};
use slog::Drain;

const TITLE: &str = "Hello, World!";
const TITLE_DEVCONSOLE: &str = "Hello, World! (Dev Console Open)";

fn main() {
    let decorator = slog_term::PlainDecorator::new(std::io::stdout());
    let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let logger = slog::Logger::root(drain, o!("version" => gers::version::VERSION));

    let _scope_guard = slog_scope::set_global_logger(logger);
    let _log_guard = slog_stdlog::init_with_level(log::Level::Trace).unwrap();

    let window_conf = WindowConf {
        width: 1024,
        height: 768,
        title: TITLE.into(),
    };
    let mut app = match App::new(&window_conf) {
        Ok(app) => app,
        Err(err) => {
            log::error!("{err}");
            std::process::exit(1);
        }
    };

    app.set_layer(HelloWorld::new());

    app.load_input_conf("gers/examples/inputmap.yaml")
        .gers_expect("failed to load input map");

    while let GersControl::Restart = app.run().gers_expect("error during event loop") {
        // Recreate window and OpenGL context
        app.recreate_window(&window_conf)
            .expect("failed to recreate main window");
    }
}

struct HelloWorld {
    devconsole: bool,
}

impl HelloWorld {
    fn new() -> Self {
        Self { devconsole: false }
    }
}

impl AppLayer for HelloWorld {
    fn update(&mut self, ctx: gers::UpdateCtx) {
        // TODO: This is temporary. We need app layers to inject custom logic.
        const ACTIONS: &[&str] = &["move_up", "move_down", "move_left", "move_right"];
        for action in ACTIONS {
            if ctx.input.is_action_pressed(action) {
                log::info!("{action}: pressed");
            } else if ctx.input.is_action_released(action) {
                log::info!("{action}: released");
            }
        }

        // Mutate Window
        if ctx.input.is_action_released("devconsole") {
            self.devconsole = !self.devconsole;

            if self.devconsole {
                log::info!("dev console open");
                ctx.window.set_title(TITLE_DEVCONSOLE);
            } else {
                log::info!("dev console closed");
                ctx.window.set_title(TITLE);
            }
        }
    }
}
