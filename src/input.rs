use rust_wren::{handle::WrenCallHandle, WrenContext, WrenResult};
use winit::{
    dpi::{LogicalPosition, PhysicalPosition},
    event::{ElementState, MouseButton, VirtualKeyCode},
};

pub struct Mouse {
    pub set_pos: WrenCallHandle,
    pub push_button: WrenCallHandle,
}

impl Mouse {
    pub fn set_pos(
        &mut self,
        ctx: &mut WrenContext,
        logical: LogicalPosition<f64>,
        physical: PhysicalPosition<f64>,
    ) -> WrenResult<()> {
        self.set_pos
            .call::<_, ()>(ctx, (logical.x, logical.y, physical.x, physical.y))
    }

    pub fn push_button(&mut self, ctx: &mut WrenContext, button: MouseButton, state: ElementState) -> WrenResult<()> {
        // TODO: Do one of these 3 overlap with possible `Other(_)` values?
        let button_id = match button {
            MouseButton::Left => 1,
            MouseButton::Middle => 2,
            MouseButton::Right => 3,
            MouseButton::Other(_) => unimplemented!("Other buttons not implemented yet"),
        };
        let state_bool = match state {
            ElementState::Pressed => true,
            ElementState::Released => false,
        };
        self.push_button.call::<_, ()>(ctx, (button_id, state_bool))
    }
}

pub struct Keyboard {
    pub set_key_press: WrenCallHandle,
    pub set_key_release: WrenCallHandle,
    pub push_char: WrenCallHandle,
}

impl Keyboard {
    pub fn set_key_state(
        &mut self,
        ctx: &mut WrenContext,
        keycode: VirtualKeyCode,
        state: ElementState,
    ) -> WrenResult<()> {
        match state {
            ElementState::Pressed => self.set_key_press.call::<_, ()>(ctx, format!("{:?}", keycode)),
            ElementState::Released => self.set_key_release.call::<_, ()>(ctx, format!("{:?}", keycode)),
        }
    }

    pub fn push_char(&mut self, ctx: &mut WrenContext, input: char) -> WrenResult<()> {
        // TODO:
        //  Horribly inefficient. We're copying a string (ToWren) on each press,
        //  and incurring an allocation in Wren.
        //  It will be much better to send the char to Wren as a u32, but we
        //  will have to encode it into a UTF-8 string in Wren.
        let mut buf = [0; 4];
        let s: &str = input.encode_utf8(&mut buf);
        self.push_char.call::<_, ()>(ctx, s)
    }
}
