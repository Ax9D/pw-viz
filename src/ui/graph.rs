use std::collections::HashMap;

use egui_nodes::{LinkArgs, NodeArgs, NodeConstructor, PinArgs};

use super::{link::Link, node::Node};

pub struct Graph {
    nodes: HashMap<u32, Node>,
    links: HashMap<u32, Link>,
}

impl Graph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            links: HashMap::new(),
        }
    }
    pub fn add_node(&mut self, node: Node) {
        log::debug!("New node: {}", node.name());

        self.nodes.insert(node.id(), node);
    }
    pub fn remove_node(&mut self, id: u32) -> Option<Node> {
        let removed = self.nodes.remove(&id);

        match &removed {
            Some(node) => log::debug!("Removed node: {}", node.name()),
            None => log::warn!("Node with id {} doesn't exist", id),
        }

        removed
    }
    pub fn get_node(&self, id: u32) -> Option<&Node> {
        self.nodes.get(&id)
    }
    pub fn get_node_mut(&mut self, id: u32) -> Option<&mut Node> {
        self.nodes.get_mut(&id)
    }
    pub fn add_link(&mut self, link: Link) {
        log::debug!("{}->{}", link.from_port, link.to_port);

        self.links.insert(link.id, link);
    }
    pub fn remove_link(&mut self, id: u32) -> Option<Link> {
        let removed = self.links.remove(&id);

        match &removed {
            Some(link) => log::debug!("{}-x-{}", link.from_port, link.to_port),
            None => log::warn!("Link with id {} doesn't exist", id),
        }

        removed
    }

    pub fn draw(&mut self, nodes_ctx: &mut egui_nodes::Context, ui: &mut egui::Ui) {
        let nodes = self.nodes.values().map(|node| {
            let mut ui_node = NodeConstructor::new(node.id() as usize, NodeArgs::default());
            ui_node.with_title(|ui| ui.label(node.name()));

            let mut ports = node.ports().values().collect::<Vec<_>>();
            ports.sort_by(|a,b| a.name().cmp(b.name()));

            for port in ports {
                match port.port_type() {
                    crate::pipewire_impl::PortType::Input => {
                        ui_node.with_input_attribute(
                            port.id() as usize,
                            PinArgs::default(),
                            |ui| ui.label(port.name()),
                        );
                    }
                    crate::pipewire_impl::PortType::Output => {
                        ui_node.with_output_attribute(
                            port.id() as usize,
                            PinArgs::default(),
                            |ui| ui.label(port.name()),
                        );
                    }
                    crate::pipewire_impl::PortType::Unknown => {}
                }
            }

            ui_node
            //.with_input_attribute(id, args, attribute)
        });

        let links = self.links.values().map(|link| {
            (link.id as usize, link.from_port as usize, link.to_port as usize, LinkArgs::default())
        });

        nodes_ctx.show(nodes, links, ui);
    }
}
