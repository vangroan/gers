use rust_wren::handle::WrenCallHandle;
use rust_wren::WrenContext;
use winit::event::{ElementState, VirtualKeyCode};

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
    ) -> Option<()> {
        match state {
            ElementState::Pressed => self
                .set_key_press
                .call::<_, ()>(ctx, format!("{:?}", keycode)),
            ElementState::Released => self
                .set_key_release
                .call::<_, ()>(ctx, format!("{:?}", keycode)),
        }
    }

    pub fn push_char(&mut self, ctx: &mut WrenContext, input: char) -> Option<()> {
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
