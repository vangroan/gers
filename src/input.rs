use rust_wren::handle::WrenCallHandle;
use rust_wren::{prelude::*, WrenContext};
use serde_json;
use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    rc::Rc,
};
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent};

#[wren_class(name = Input)]
#[derive(Default)]
pub struct WrenInput {
    keyboard: HashMap<String, KeyboardInput>,
}

#[wren_methods]
impl WrenInput {
    #[construct]
    pub fn new() -> Self {
        Default::default()
    }

    #[method(name = isPressed)]
    pub fn is_pressed(&self, key: &str) -> bool {
        // self.keyboard
        //     .get(key as usize)
        //     .and_then(|input| input.map(|input| input.state == ElementState::Pressed))
        //     .unwrap_or_else(|| false)
        todo!()
    }
}

impl WrenInput {
    pub fn set_keyboard_input(&mut self, keyboard_input: KeyboardInput) {
        println!("Storing keyboard input {:?}", keyboard_input);
        println!(
            "Serialized: {}",
            serde_json::to_string(&keyboard_input.virtual_keycode).unwrap()
        );
        let key = serde_json::to_string(&keyboard_input.virtual_keycode).unwrap();
        self.keyboard.insert(key, keyboard_input);
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
