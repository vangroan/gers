pub use self::fps::{FpsCounter, FpsThrottle, FpsThrottlePolicy};
pub use self::game::{init_game, register_game, Game};

mod fps;
mod game;
