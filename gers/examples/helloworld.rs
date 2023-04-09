use gers::{prelude::*, App, GersControl, WindowConf};

const TITLE: &str = "Hello, World!";
const TITLE_DEVCONSOLE: &str = "Hello, World! (Dev Console Open)";

fn main() {
    let window_conf = WindowConf {
        width: 1024,
        height: 768,
        title: TITLE.into(),
    };
    let mut app = match App::new(&window_conf) {
        Ok(app) => app,
        Err(err) => {
            eprintln!("{err}");
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
                println!("{action}: pressed");
            } else if ctx.input.is_action_released(action) {
                println!("{action}: released");
            }
        }

        // Mutate Window
        if ctx.input.is_action_released("devconsole") {
            self.devconsole = !self.devconsole;

            if self.devconsole {
                println!("dev console open");
                ctx.window.set_title(TITLE_DEVCONSOLE);
            } else {
                println!("dev console closed");
                ctx.window.set_title(TITLE);
            }
        }
    }
}
