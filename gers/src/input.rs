//! Input mapping
use serde::Deserialize;
use smol_str::SmolStr;

// This module glues `winit` to the engine's input framework.
pub use winit::event::{ElementState, MouseButton, VirtualKeyCode};

use crate::{errors::GersResultExt, GersError};

/// # Implementation
///
/// Actions potentially need to be looked up using multiple input.
/// By keyboard key, mouse button, gamepad button, or name.
///
/// Lookup maps are stored in vectors to speed up iteration.
pub struct InputMap {
    actions2: Vec<ActionInfo>,
    state: InputState,
    /// Mapping of virtual key codes to actions, by index.
    keymap: Vec<(VirtualKeyCode, usize)>,
    /// Mapping of mouse buttons to actions, by index.
    mousemap: Vec<(MouseButton, usize)>,
    namemap: Vec<(SmolStr, usize)>,
}

type InputMapDef = Vec<ActionDef>;

pub struct ActionInfo {
    pub name: SmolStr,
    pub keyboard_keys: Vec<VirtualKeyCode>,
    pub mouse_buttons: Vec<MouseButton>,
}

impl From<ActionDef> for ActionInfo {
    fn from(def: ActionDef) -> Self {
        Self {
            name: def.name.into(),
            keyboard_keys: def.keyboard_keys.unwrap_or_default(),
            mouse_buttons: def.mouse_buttons.unwrap_or_default(),
        }
    }
}

/// Schema of action as it appears in a configuration file.
///
/// Mappings are optional for the sake of config ergonomics,
/// but will be mapped to infallible fields when loaded into
/// the input mapper.
#[derive(Deserialize)]
pub struct ActionDef {
    pub name: String,
    pub keyboard_keys: Option<Vec<VirtualKeyCode>>,
    pub mouse_buttons: Option<Vec<MouseButton>>,
    // TODO: Input settings:
    // - debounce
    // - hold down or OS repeat
    // - deadzone
    // - axis
}

/// Action event instance.
#[allow(dead_code)]
pub struct InputEvent {}

#[derive(Default)]
struct InputState {
    /// Keyboard keys
    keys: Vec<KeyState>,
}

struct KeyState {
    action: SmolStr,
    virtual_keycode: VirtualKeyCode,
    element_state: ElementState,
}

impl InputMap {
    pub fn new() -> Self {
        Self {
            actions2: Vec::new(),
            state: InputState::default(),
            keymap: Vec::new(),
            mousemap: Vec::new(),
            namemap: Vec::new(),
        }
    }

    /// Rebuild the input mappings to actions, for when the actions have been changed.
    fn rebuild_mappings(&mut self) {
        // Start fresh
        self.keymap.clear();
        self.mousemap.clear();
        self.namemap.clear();

        for (index, action) in self.actions2.iter().enumerate() {
            for key in action.keyboard_keys.iter().cloned() {
                self.keymap.push((key, index));
            }

            for button in action.mouse_buttons.iter().cloned() {
                self.mousemap.push((button, index));
            }

            self.namemap.push((action.name.clone(), index));
        }
    }

    /// Discard input state.
    ///
    /// Keep pressed state so action stays down.
    pub fn clear_releases(&mut self) {
        self.state
            .keys
            .retain(|state| state.element_state == ElementState::Pressed);
    }

    /// Load input map from file.
    pub fn load_file(&mut self, filepath: &str) -> Result<(), GersError> {
        let file = std::fs::File::open(filepath)
            .map_err(GersError::from)
            .with_message("failed to load input map from yaml file")?;

        let definitions: InputMapDef = serde_yaml::from_reader(file)?;

        for def in definitions {
            self.actions2.push(ActionInfo::from(def));
        }

        // for action_def in definitions {
        //     self.actions.insert(action_def.name.clone(), action_def);
        // }

        self.rebuild_mappings();

        Ok(())
    }
}

/// State management.
impl InputMap {
    pub fn action_by_name(&self, name: &str) -> Option<&ActionInfo> {
        self.namemap
            .iter()
            .find(|(n, _)| n == name)
            .and_then(|(_, index)| self.actions2.get(*index))
    }

    /// Lookup an action by keyboard key.
    pub fn action_by_key(&self, keycode: VirtualKeyCode) -> Option<&ActionInfo> {
        self.keymap
            .iter()
            .find(|(k, _)| *k == keycode)
            .and_then(|(_, index)| self.actions2.get(*index))
    }

    pub fn set_key_pressed(&mut self, keycode: VirtualKeyCode) {
        self.set_key_state(keycode, ElementState::Pressed)
    }

    pub fn set_key_released(&mut self, keycode: VirtualKeyCode) {
        self.set_key_state(keycode, ElementState::Released)
    }

    pub fn set_key_state(&mut self, keycode: VirtualKeyCode, state: ElementState) {
        let maybe = self.state.keys.iter_mut().find(|el| el.virtual_keycode == keycode);
        match maybe {
            Some(keystate) => {
                keystate.element_state = state;
            }
            None => {
                if let Some(action_def) = self.action_by_key(keycode) {
                    self.state.keys.push(KeyState {
                        action: action_def.name.clone(),
                        virtual_keycode: keycode,
                        element_state: state,
                    });
                }
            }
        }
    }
}

impl InputMap {
    pub fn is_action_pressed(&self, name: impl AsRef<str>) -> bool {
        let lookup_key = name.as_ref();
        self.state
            .keys
            .iter()
            .find(|state| state.action == lookup_key)
            .map(|state| state.element_state == ElementState::Pressed)
            .unwrap_or(false)
    }

    pub fn is_action_released(&self, name: impl AsRef<str>) -> bool {
        let lookup_key = name.as_ref();
        self.state
            .keys
            .iter()
            .find(|state| state.action == lookup_key)
            .map(|state| state.element_state == ElementState::Released)
            .unwrap_or(false)
    }
}
