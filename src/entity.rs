use legion::*;
use rust_wren::{
    handle::{WrenCallHandle, WrenHandle},
    prelude::*,
};
use smol_str::SmolStr;
use std::{borrow::Cow, fmt, sync::Mutex};

#[wren_class(name = Entity)]
#[derive(Debug, Clone)]
pub struct WrenEntity {
    pub entity: Entity,
}

#[wren_methods]
impl WrenEntity {
    #[construct]
    pub fn unused() -> Self {
        unimplemented!("Must be instantiated by Rust")
    }
}

impl WrenEntity {
    pub fn create(world: &mut World) -> Self {
        lazy_static! {
            static ref COUNT: Mutex<usize> = Mutex::new(0);
        }
        let mut counter = COUNT.lock().unwrap();
        *counter += 1;

        let entity = world.push((
            Tag {
                name: SmolStr::new(format!("Entity {}", counter)),
            },
            Scripts { scripts: vec![] },
        ));

        WrenEntity { entity }
    }
}

pub struct Tag {
    name: SmolStr,
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.name, f)
    }
}

/// Container of scripts attached to this entity.
pub struct Scripts {
    scripts: Vec<ScriptHandle>,
}

struct EntityCounter(usize);

impl EntityCounter {
    fn increment(&mut self) -> usize {
        self.0 += 1;
        self.0
    }
}

pub struct ScriptHandle {
    /// Handle to class declaration.
    cls: WrenHandle,
    /// Handle to instance of script object.
    obj: WrenHandle,
    /// Call handle to update function.
    update: Option<WrenCallHandle>,
}

trait FindTagExt {
    fn find_by_tag(&self, name: &str) -> Option<Entity>;
}

impl FindTagExt for World {
    fn find_by_tag(&self, name: &str) -> Option<Entity> {
        let mut query = Read::<(Entity, Tag)>::query();

        for (ent, tag) in query.iter(self) {
            if tag.name == name {
                return Some(*ent);
            }
        }

        None
    }
}
