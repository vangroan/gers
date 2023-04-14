use winit::window::Window;

use crate::InputMap;

pub trait AppLayer {
    fn update(&mut self, _ctx: UpdateCtx<'_>) {}
}

pub struct UpdateCtx<'a> {
    pub window: &'a mut Window,
    pub input: &'a InputMap,
}
