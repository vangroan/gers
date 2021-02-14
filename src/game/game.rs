//! Game script entrypoint and hooks.
use crate::input::{Keyboard, Mouse};
use rust_wren::{
    handle::{FnSymbolRef, WrenCallHandle, WrenCallRef},
    prelude::*,
    WrenContext, WrenResult,
};

pub fn init_game(ctx: &mut WrenContext) -> Game {
    // TODO:
    //  Change signature to return WrenResult
    //  Change all unwraps to ?

    // The user's game instance, which is the entry point from Rust into
    // the Wren program, is stored in a property.
    let get_handler = ctx.make_call_ref("game", "Game", "handler_").unwrap();

    // Delta Time
    let set_delta_time = ctx
        .make_call_ref("game", "Game", "deltaTime_=(_)")
        .unwrap()
        .leak()
        .unwrap();

    // Init
    let init = {
        let handler = get_handler.call::<_, WrenRef>(ctx, ()).unwrap();
        let init_ref = FnSymbolRef::compile(ctx, "init()").unwrap();
        WrenCallRef::new(handler, init_ref).leak().unwrap()
    };

    // Update
    let update = {
        let handler = get_handler.call::<_, WrenRef>(ctx, ()).unwrap();
        let update_ref = FnSymbolRef::compile(ctx, "process_()").unwrap();
        WrenCallRef::new(handler, update_ref).leak().unwrap()
    };

    // Mouse Input
    let mouse = Mouse {
        set_pos: ctx
            .make_call_ref("input", "Mouse", "setPos_(_,_,_,_)")
            .unwrap()
            .leak()
            .unwrap(),
        push_button: ctx
            .make_call_ref("input", "Mouse", "pushButton_(_,_)")
            .unwrap()
            .leak()
            .unwrap(),
    };

    // Keyboard Input
    let keyboard = Keyboard {
        set_key_press: ctx
            .make_call_ref("input", "Keyboard", "setKeyPress_(_)")
            .unwrap()
            .leak()
            .unwrap(),
        set_key_release: ctx
            .make_call_ref("input", "Keyboard", "setKeyRelease_(_)")
            .unwrap()
            .leak()
            .unwrap(),
        push_char: ctx
            .make_call_ref("input", "Keyboard", "pushChar_(_)")
            .unwrap()
            .leak()
            .unwrap(),
    };

    Game {
        scale_factor: 1.0,
        set_delta_time,
        init,
        update,
        mouse,
        keyboard,
    }
}

/// Register builtin game module.
pub fn register_game(vm: &mut WrenVm) -> WrenResult<()> {
    vm.interpret("game", include_str!("game.wren"))
}

pub struct Game {
    pub scale_factor: f64,
    pub set_delta_time: WrenCallHandle,
    pub init: WrenCallHandle,
    pub update: WrenCallHandle,
    pub mouse: Mouse,
    pub keyboard: Keyboard,
}
