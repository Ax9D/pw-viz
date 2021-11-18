use std::collections::HashMap;

pub enum GlobalObject {
    Node,
    Link,
    Port { node_id: u32, id: u32 },
}

/// For internal state tracking, this has to be done because pipewire only provides ids of the objects it removes,
/// which is insufficient to safely remove an object of a particular type, hence this struct serves as a lookup from id to object specific info
pub struct State {
    objects: HashMap<u32, GlobalObject>,
}

impl State {
    pub fn new() -> Self {
        Self {
            objects: HashMap::new(),
        }
    }
    pub fn get(&self, id: u32) -> Option<&GlobalObject> {
        self.objects.get(&id)
    }
    pub fn add(&mut self, id: u32, object: GlobalObject) {
        self.objects.insert(id, object);
    }
    pub fn remove(&mut self, id: u32) -> Option<GlobalObject> {
        self.objects.remove(&id)
    }
}
