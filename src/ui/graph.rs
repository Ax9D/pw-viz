use egui::Widget;
use egui_nodes::{LinkArgs, NodeArgs, NodeConstructor, PinArgs};
use std::collections::{HashMap, HashSet};

use super::{link::Link, node::Node, Theme};
use crate::pipewire_impl::MediaType;

/// Represents changes to any links that might happend in the ui
/// These changes are used to send updates to the pipewire thread
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
    nodes_ctx: egui_nodes::Context,
    nodes: HashMap<u32, Node>, // Node id to Node
    links: HashMap<u32, Link>, // Link id to Link
}

impl Graph {
    pub fn new() -> Self {
        Self {
            nodes_ctx: egui_nodes::Context::default(),
            nodes: HashMap::new(),
            links: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node: Node) {
        log::debug!("New node: {}", node.name);
        self.nodes.insert(node.id, node);
    }

    pub fn remove_node(&mut self, id: u32) -> Option<Node> {
        let removed = self.nodes.remove(&id);
        match removed {
            Some(ref node) => log::debug!("Removed node: {}", node.name),
            None => log::warn!("Node with id {} doesn't exist", id),
        }
        removed
    }

    #[allow(dead_code)]
    pub fn get_node(&self, id: u32) -> Option<&Node> {
        self.nodes.get(&id)
    }
    pub fn get_node_mut(&mut self, id: u32) -> Option<&mut Node> {
        self.nodes.get_mut(&id)
    }

    #[allow(dead_code)]
    pub fn get_link(&self, id: u32) -> Option<&Link> {
        self.links.get(&id)
    }
    #[allow(dead_code)]
    pub fn get_link_mut(&mut self, id: u32) -> Option<&mut Link> {
        self.links.get_mut(&id)
    }

    pub fn add_link(&mut self, link: Link) {
        log::debug!("{}->{}", link.from_port, link.to_port);
        self.links.insert(link.id, link);
    }
    pub fn remove_link(&mut self, id: u32) -> Option<Link> {
        let removed = self.links.remove(&id);
        match removed {
            Some(ref link) => log::debug!("{}-x-{}", link.from_port, link.to_port),
            None => log::warn!("Link with id {} doesn't exist", id),
        }
        removed
    }

    /// Naive, inefficient and weird implementation of Kahn's algorithm
    fn topo_sort(&self) -> Vec<u32> {
        // FIX ME

        // Node id to in-degree(no. of nodes that output to this node)
        let mut indegrees = self
            .nodes
            .keys()
            .map(|&id| {
                let count = self
                    .links
                    .values()
                    .filter(|link| link.to_node == id)
                    .map(|link| link.from_node)
                    .collect::<HashSet<u32>>()
                    .len();
                (id, count)
            })
            .collect::<HashMap<u32, usize>>();

        // Adjacency hashmap, maps node id to neighbouring node ids
        let adj_list = self
            .nodes
            .keys()
            .map(|&id| {
                let adj = self
                    .links
                    .values()
                    .filter(|link| link.from_node == id)
                    .map(|link| link.to_node)
                    .collect::<HashSet<u32>>();
                (id, adj)
            })
            .collect::<HashMap<u32, _>>();

        // println!("Indegrees {:?}", indegrees);
        // println!("Adj list {:?}", self.adj_list);

        let mut queue: Vec<u32> = Vec::new();

        for node_id in self.nodes.keys() {
            // Put nodes which are "detached"(i.e of in-degree=0) from the graph into the queue for processing
            if indegrees[node_id] == 0 {
                queue.push(*node_id);
            }
        }

        let mut top_order = Vec::new();
        let mut count = 0;
        while !queue.is_empty() {
            // println!("Queue: {:?}", queue);
            let u = queue.remove(0); // Remove from the front of the queue
            top_order.push(u);
            if let Some(adj_nodes) = adj_list.get(&u) {
                // Check nodes that lead out from this node
                for node_id in adj_nodes {
                    // Remove link from parent node to this node
                    let indegree_of_node = indegrees.get_mut(node_id).unwrap();
                    *indegree_of_node -= 1;

                    // Check if that detached the node from the graph
                    if *indegree_of_node == 0 {
                        // If it did, we have a new detached node to process
                        queue.push(*node_id);
                    }
                }
            }
            count += 1;
        }

        if count != self.nodes.len() {
            log::error!("Cycle detected");
        }
        top_order
    }

    pub fn draw(
        &mut self,
        ctx: &egui::CtxRef,
        ui: &mut egui::Ui,
        theme: &Theme,
    ) -> Option<LinkUpdate> {
        // Find the topologically sorted order of nodes in the graph
        // Nodes are currently laid out based on this order
        let order = self.topo_sort();

        // println!("{:?}", order);

        // Ctrl is used to trigger the debug view
        let debug_view = ctx.input().modifiers.ctrl;
        let mut ui_nodes = Vec::with_capacity(self.nodes.len());
        let mut prev_pos = egui::pos2(ui.available_width() / 4.0, ui.available_height() / 2.0);
        let mut padding = egui::pos2(75.0, 150.0);
        for node_id in order {
            let node = self.nodes.get_mut(&node_id).unwrap();

            let mut ui_node = NodeConstructor::new(
                node.id as usize,
                NodeArgs {
                    titlebar: Some(theme.titlebar),
                    titlebar_hovered: Some(theme.titlebar_hovered),
                    titlebar_selected: Some(theme.titlebar_hovered),
                    ..Default::default()
                },
            );

            // if node.position.is_none() {
            //     // Horizontally shift each node to the right of the previous one
            //     // Also put it at a random point vertically
            //     ui_node.with_origin(egui::pos2(
            //         prev_pos.x + padding.x,
            //         rand::random::<f32>() * ui.available_height(),
            //     ));
            // } else {
            //     ui_node.with_origin(egui::pos2(
            //         ui.available_width() / 4.0,
            //         rand::random::<f32>() * ui.available_height(),
            //     ));
            // }

            let node_position = node.position.unwrap_or_else(|| {
                padding.y *= -1.0;
                egui::pos2(prev_pos.x + padding.x, prev_pos.y + padding.y)
            });
            ui_node.with_origin(node_position);

            prev_pos = node_position;

            let kind = match node.media_type {
                Some(MediaType::Audio) => "🔉",
                Some(MediaType::Video) => "💻",
                Some(MediaType::Midi) => "🎹",
                None => "",
            };

            let title = {
                if debug_view {
                    // Display node id if in debug view
                    format!("{}[{}]{}", node.name, node.id, kind)
                } else {
                    format!("{} {}", node.name, kind)
                }
            };

            ui_node.with_title(|ui| egui::Label::new(title).text_color(theme.text_color).ui(ui));
            Self::draw_ports(&mut ui_node, node, theme, debug_view);
            ui_nodes.push(ui_node);
        }

        let links = self.links.values().map(|link| {
            (
                link.id as usize,
                link.from_port as usize,
                link.to_port as usize,
                LinkArgs::default(),
            )
        });

        self.nodes_ctx.show(ui_nodes, links, ui);

        for (&id, node) in self.nodes.iter_mut() {
            node.position = self.nodes_ctx.get_node_pos_screen_space(id as usize);
        }

        if let Some(link) = self.nodes_ctx.link_destroyed() {
            Some(LinkUpdate::Removed(link as u32))
        } else if let Some((from_port, from_node, to_port, to_node, _)) =
            self.nodes_ctx.link_created_node()
        {
            log::debug!(
                "Created new link:\nfrom_port {}, to_port {}, from_node {}, to_node {}",
                from_port,
                to_port,
                from_node,
                to_node
            );

            Some(LinkUpdate::Created {
                from_port: from_port as u32,
                to_port: to_port as u32,
                from_node: from_node as u32,
                to_node: to_node as u32,
            })
        } else {
            None
        }
    }

    fn draw_ports(ui_node: &mut NodeConstructor, node: &Node, theme: &Theme, debug: bool) {
        let mut ports = node.ports.values().collect::<Vec<_>>();

        // Sorts ports based on alphabetical ordering
        ports.sort_by(|a, b| a.name.cmp(&b.name));

        for port in ports {
            let (background, hovered) = match node.media_type {
                Some(MediaType::Audio) => (theme.audio_port, theme.audio_port_hovered),
                Some(MediaType::Video) => (theme.video_port, theme.video_port_hovered),
                Some(MediaType::Midi) => (egui::Color32::RED, egui::Color32::LIGHT_RED),
                None => (egui::Color32::GRAY, egui::Color32::LIGHT_GRAY),
            };
            let port_name = {
                if debug {
                    format!("{} [{}]", port.name, port.id)
                } else {
                    format!("{} ", port.name)
                }
            };

            match port.port_type {
                crate::pipewire_impl::PortType::Input => {
                    ui_node.with_input_attribute(
                        port.id as usize,
                        PinArgs {
                            background: Some(background),
                            hovered: Some(hovered),
                            ..Default::default()
                        },
                        |ui| {
                            egui::Label::new(port_name)
                                //.text_color(theme.text_color)
                                .ui(ui)
                        },
                    );
                }
                crate::pipewire_impl::PortType::Output => {
                    ui_node.with_output_attribute(
                        port.id as usize,
                        PinArgs {
                            background: Some(background),
                            hovered: Some(hovered),
                            ..Default::default()
                        },
                        |ui| {
                            egui::Label::new(port_name)
                                //.text_color(theme.text_color)
                                .ui(ui)
                        },
                    );
                }
                crate::pipewire_impl::PortType::Unknown => {}
            }
        }
    }
}
