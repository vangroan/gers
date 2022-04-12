extern crate sfml;

pub use sfml::window::{Event, Style, Window};

pub fn create_window() {
    // Create a new window
    let mut window = Window::new((800, 600), "SFML window", Style::CLOSE, &Default::default());
    // Limit the framerate to 60 frames per second (this step is optional)
    window.set_framerate_limit(60);
    run_loop(window);
}

pub fn run_loop(mut window: Window) {
    // The main loop - ends as soon as the window is closed
    while window.is_open() {
        // Event processing
        while let Some(event) = window.poll_event() {
            // Request closing for the window
            if event == Event::Closed {
                window.close();
            }
        }

        // Activate the window for OpenGL rendering
        window.set_active(true);

        // OpenGL drawing commands go here...

        // End the current frame and display its contents on screen
        window.display();
    }
}

pub fn say_hello() {
    println!("Hello world");
}
