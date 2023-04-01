//! Input mapping
use std::{borrow::Cow, collections::HashMap};

use serde::Deserialize;
use winit::event::{ElementState, MouseButton, VirtualKeyCode};

use crate::{errors::GersResultExt, GersError};

pub struct InputMap {
    actions: HashMap<String, ActionDef>,
    state: InputState,
}

type InputMapDef = Vec<ActionDef>;

#[derive(Deserialize)]
pub struct ActionDef {
    pub name: String,
    pub keyboard_keys: Option<Vec<VirtualKeyCode>>,
    pub mouse_buttons: Option<Vec<MouseButton>>,
    pub debounce: Option<f32>,
}

/// Action event instance.
pub struct Action {}

#[derive(Default)]
struct InputState {
    /// Keyboard keys
    keys: Vec<KeyState>,
}

struct KeyState {
    action: String,
    virtual_keycode: VirtualKeyCode,
    element_state: ElementState,
}

impl InputMap {
    pub fn new() -> Self {
        Self {
            actions: HashMap::new(),
            state: InputState::default(),
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

        for action_def in definitions {
            if action_def.debounce.is_some() {
                println!("input key debounce not implemented yet");
            }
            self.actions.insert(action_def.name.clone(), action_def);
        }

        Ok(())
    }
}

/// State management.
impl InputMap {
    pub fn action_def(&self, name: &str) -> Option<&ActionDef> {
        self.actions.get(name)
    }

    pub fn key_action(&self, keycode: VirtualKeyCode) -> Option<&ActionDef> {
        self.actions.values().find(|action| {
            if let Some(keys) = &action.keyboard_keys {
                keys.iter().any(|key| *key == keycode)
            } else {
                false
            }
        })
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
                if let Some(action_def) = self.key_action(keycode) {
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
    pub fn is_action_pressed<'a, S>(&self, name: S) -> bool
    where
        S: Into<Cow<'a, str>>,
    {
        let lookup_key = name.into();
        self.state
            .keys
            .iter()
            .find(|state| state.action == lookup_key)
            .map(|state| state.element_state == ElementState::Pressed)
            .unwrap_or(false)
    }

    pub fn is_action_released<'a, S>(&self, name: S) -> bool
    where
        S: Into<Cow<'a, str>>,
    {
        let lookup_key = name.into();
        self.state
            .keys
            .iter()
            .find(|state| state.action == lookup_key)
            .map(|state| state.element_state == ElementState::Released)
            .unwrap_or(false)
    }
}
