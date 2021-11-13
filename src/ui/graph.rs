use std::collections::HashMap;

use egui::Widget;
use egui_nodes::{LinkArgs, NodeArgs, NodeConstructor, PinArgs};

use crate::pipewire_impl::MediaType;

use super::{Theme, link::Link, node::Node};

pub enum LinkUpdate {
    Created {
        from_port: u32,
        to_port: u32,

        from_node: u32,
        to_node: u32,
    },

    Removed(u32),
}
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
            Some(link) => {
                log::debug!("{}-x-{}", link.from_port, link.to_port);
            }
            None => log::warn!("Link with id {} doesn't exist", id),
        }

        removed
    }
    // fn topo_sort(&self) -> Vec<u32> {
    //     let indegrees = self.nodes.iter().map(|(&id, node)| {
    //         let count = node.ports().values().filter(|port| match port.port_type() {
    //             crate::pipewire_impl::PortType::Input => true,
    //             _=> false
    //         }).count();

    //         (id, count)
    //     }).collect::<HashMap<u32, usize>>();

    //     let mut queue = Vec::new();

    //     for node_id in self.nodes.keys() {
    //         if indegrees[node_id] == 0 {
    //             queue.push(node_id);
    //         }
    //     }

    //     let mut top_order = Vec::new();
    //     while !queue.is_empty() {
    //         let u=*queue[0];
    //         queue.pop();

    //         top_order.push(u);

    //         for node_id in self.adj_list.get(&u) {
    //             if indegrees[node_id] == 0 {
    //                 queue.push(node_id);
    //             }
    //         }
    //     }

    //     top_order
    // }
    pub fn draw(
        &mut self,
        nodes_ctx: &mut egui_nodes::Context,
        ui: &mut egui::Ui,
        theme: &Theme
    ) -> Option<LinkUpdate> {
        //println!("{:?}", self.topo_sort());
        let nodes = self
            .nodes
            .values_mut()
            .map(|node| {
                let mut ui_node = NodeConstructor::new(node.id() as usize, NodeArgs {
                    titlebar: Some(theme.titlebar),
                    titlebar_hovered: Some(theme.titlebar_hovered),
                    ..Default::default()
                });

                if node.newly_added {

                    //FIX ME: Don't layout new nodes randomly, topological sort....
                    ui_node.with_origin(egui::pos2(
                        rand::random::<f32>() * ui.available_width(),
                        rand::random::<f32>() * ui.available_height(),
                    ));
                    node.newly_added = false;
                }

                ui_node.with_title(|ui| {
                    let kind = match node.media_type() {
                        Some(MediaType::Audio) => "🔉",
                        Some(MediaType::Video) => "💻",
                        Some(MediaType::Midi) => "🎹",
                        None => "",
                    };

                    egui::Label::new(format!("{} {}", node.name(),kind))
                    .text_color(theme.text_color).ui(ui) 
                });

                let mut ports = node.ports().values().collect::<Vec<_>>();

                ports.sort_by(|a, b| a.name().cmp(b.name()));

                for port in ports {
                    match port.port_type() {
                        crate::pipewire_impl::PortType::Input => {
                            ui_node.with_input_attribute(
                                port.id() as usize,
                                PinArgs {
                                    background: Some(theme.port_in),
                                    hovered: Some(theme.port_in_hovered),
                                    ..Default::default()
                                },
                                |ui| ui.label(port.name()),
                            );
                        }
                        crate::pipewire_impl::PortType::Output => {
                            ui_node.with_output_attribute(
                                port.id() as usize,
                                PinArgs {
                                        background: Some(theme.port_out),
                                        hovered: Some(theme.port_out_hovered),
                                        ..Default::default()
                                },
                                |ui| ui.label(port.name()),
                            );
                        }
                        crate::pipewire_impl::PortType::Unknown => {}
                    }
                }

                ui_node
                //.with_input_attribute(id, args, attribute)
            })
            .collect::<Vec<_>>();

        let links = self.links.values().map(|link| {
            (
                link.id as usize,
                link.from_port as usize,
                link.to_port as usize,
                LinkArgs::default(),
            )
        });
    
        nodes_ctx.show(nodes, links, ui);

        if let Some(link) = nodes_ctx.link_destroyed() {

            Some(LinkUpdate::Removed(link as u32))
        } else if let Some((from_port, from_node, to_port, to_node, _)) = nodes_ctx.link_created_node() {
            log::debug!("Created new link:\nfrom_port {}, to_port {}, from_node {}, to_node {}", from_port,to_port, from_node, to_node);

            let from_port = from_port as u32;
            let to_port = to_port as u32;

            let from_node = from_node as u32;
            let to_node = to_node as u32;

            Some(LinkUpdate::Created {
                from_port,
                to_port,

                from_node,
                to_node,
            })
        } else {
            None
        }
    }
}
