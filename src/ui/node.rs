use std::collections::HashMap;

use super::port::Port;
use crate::pipewire_impl::MediaType;

#[derive(Debug)]
pub struct Node {
    pub id: u32,
    pub name: String,
    pub media_type: Option<MediaType>,
    pub ports: HashMap<u32, Port>, // Port id to Port
    pub position: Option<egui::Pos2>,
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
        self.ports.insert(port.id, port);
    }
    pub fn remove_port(&mut self, port_id: u32) {
        self.ports.remove(&port_id);
    }
}
