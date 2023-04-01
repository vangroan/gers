use gers::{prelude::*, App, GersControl, WindowConf};

fn main() {
    let window_conf = WindowConf {
        width: 1024,
        height: 768,
        title: "Hello, World!".into(),
    };
    let mut app = match App::new(&window_conf) {
        Ok(app) => app,
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1);
        }
    };

    app.load_input_conf("gers/examples/inputmap.yaml")
        .gers_expect("failed to load input map");

    while let GersControl::Restart = app.run().gers_expect("error during event loop") {
        // Recreate window and OpenGL context
        app.recreate_window(&window_conf)
            .expect("failed to recreate main window");
    }
}
