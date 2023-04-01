use gers::{App, GersControl, WindowConf};

fn main() {
    let window_conf = WindowConf {
        width: 1024,
        height: 768,
        title: "Hello, World!".into(),
    };
    let mut app = App::new(&window_conf).expect("failed to build GERS application");

    while let GersControl::Restart = app.run().expect("error during event loop") {
        // Recreate window and OpenGL context
        app.recreate_window(&window_conf)
            .expect("failed to recreate main window");
    }
}
