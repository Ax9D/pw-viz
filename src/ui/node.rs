use std::collections::HashMap;

use crate::pipewire_impl::MediaType;

use super::port::Port;

#[derive(Debug)]
pub struct Node {
    id: u32,
    name: String,
    media_type: Option<MediaType>,
    ports: HashMap<u32, Port>, //Port id to Port
    pub(super) position: Option<egui::Pos2>,
}

impl Node {
    pub fn new(id: u32, name: String, media_type: Option<MediaType>) -> Self {
        Self {
            id,
            name,
            media_type,
            ports: HashMap::new(),
            position: None,
        }
    }
    pub fn add_port(&mut self, port: Port) {
        self.ports.insert(port.id(), port);
    }
    pub fn remove_port(&mut self, port_id: u32) {
        self.ports.remove(&port_id);
    }

    pub fn id(&self) -> u32 {
        self.id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn media_type(&self) -> Option<MediaType> {
        self.media_type
    }
    pub fn ports(&self) -> &HashMap<u32, Port> {
        &self.ports
    }
}
