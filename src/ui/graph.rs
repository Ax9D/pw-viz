use std::collections::{HashMap, HashSet};

use egui_nodes::{LinkArgs, NodeArgs, NodeConstructor};

use crate::pipewire_impl::MediaType;

use super::id::Id;

use super::{link::Link, node::Node, port::Port, Theme};

/// Represents changes to any links that might have happend in the ui
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
    nodes: HashMap<Id, Node>,  //Node id to Node
    links: HashMap<u32, Link>, //Link id to Link
}

impl Graph {
    pub fn new() -> Self {
        //context.attribute_flag_push(egui_nodes::AttributeFlags::EnableLinkCreationOnSnap);
        //context.attribute_flag_push(egui_nodes::AttributeFlags::EnableLinkDetachWithDragClick);
        let mut nodes_ctx = egui_nodes::Context::default();

        nodes_ctx.style.link_bezier_offset_coefficient = egui::vec2(0.50, 0.0);
        nodes_ctx.style.link_line_segments_per_length = 0.15;

        Self {
            nodes_ctx,
            nodes: HashMap::new(),
            links: HashMap::new(),
        }
    }
    fn get_or_create_node(&mut self, name: String) -> &mut Node {
        let id = Id::new(&name);
        self.nodes.entry(id).or_insert_with(|| {
            log::debug!("Created new ui node: {}", name);

            Node::new(id, name)
        })
    }
    pub fn add_node(
        &mut self,
        name: String,
        id: u32,
        description: Option<String>,
        media_type: Option<MediaType>,
    ) {
        self.get_or_create_node(name)
            .add_pw_node(id, description, media_type)
    }
    pub fn remove_node(&mut self, name: &str, id: u32) {
        let mut remove_ui_node = false;

        if let Some(node) = self.nodes.get_mut(&Id::new(name)) {
            remove_ui_node = node.remove_pw_node(id);
        } else {
            log::error!("Node with name: {} was not registered", name);
        }

        //If there are no more pw nodes remove the ui node
        if remove_ui_node {
            let removed_node = self
                .nodes
                .remove(&Id::new(name))
                .expect("Node was never added");

            log::debug!("Removing node {}", removed_node.name());
        }
    }
    pub fn add_port(&mut self, node_name: String, node_id: u32, port: Port) {
        self.get_or_create_node(node_name).add_port(node_id, port)
    }
    pub fn remove_port(&mut self, node_name: &str, node_id: u32, port_id: u32) {
        if let Some(node) = self.nodes.get_mut(&Id::new(node_name)) {
            node.remove_port(node_id, port_id);
        } else {
            log::error!("Node with name: {} was not registered", node_name);
        }
    }
    pub fn add_link(
        &mut self,
        id: u32,
        from_node_name: String,
        to_node_name: String,
        from_port: u32,
        to_port: u32,
    ) {
        log::debug!(
            "{}.{}->{}.{}",
            from_node_name,
            from_port,
            to_node_name,
            to_port
        );

        let from_node = self
            .nodes
            .get(&Id::new(from_node_name))
            .expect("Node with provided name doesn't exist")
            .id();

        let to_node = self
            .nodes
            .get(&Id::new(to_node_name))
            .expect("Node with provided name doesn't exist")
            .id();
        log::debug!("{:?} {:?}", from_node, to_node);

        self.links.insert(
            id,
            Link {
                id,
                from_node,
                to_node,
                from_port,
                to_port,
                active: true,
            },
        );
    }
    pub fn remove_link(&mut self, id: u32) -> Option<Link> {
        let removed = self.links.remove(&id);
        match removed {
            Some(ref link) => log::debug!("{}-x-{}", link.from_port, link.to_port),
            None => log::warn!("Link with id {} doesn't exist", id),
        }
        removed
    }
    #[allow(dead_code)]
    fn get_link(&self, id: u32) -> Option<&Link> {
        self.links.get(&id)
    }
    #[allow(dead_code)]
    fn get_link_mut(&mut self, id: u32) -> Option<&mut Link> {
        self.links.get_mut(&id)
    }
    fn topo_sort_(
        node_id: Id,
        visited: &mut HashSet<Id>,
        adj_list: &HashMap<Id, HashSet<Id>>,
        stack: &mut Vec<Id>,
    ) {
        visited.insert(node_id);

        for node_id in &adj_list[&node_id] {
            if !visited.contains(node_id) {
                Self::topo_sort_(*node_id, visited, adj_list, stack);
            }
        }

        stack.push(node_id);
    }
    //TODO: Handle stack overflows
    fn top_sort(&self) -> Vec<Id> {
        let mut stack = Vec::new();

        let mut visited = HashSet::new();

        let adj_list = self
            .nodes
            .values()
            .map(|node| {
                let adj = self
                    .links
                    .values()
                    .filter(|link| !link.is_self_link())
                    .filter(|link| link.from_node == node.id())
                    .map(|link| link.to_node)
                    .collect::<HashSet<Id>>();
                (node.id(), adj)
            })
            .collect::<HashMap<Id, _>>();

        for node in self.nodes.values() {
            if !visited.contains(&node.id()) {
                Self::topo_sort_(node.id(), &mut visited, &adj_list, &mut stack)
            }
        }

        stack.reverse();

        stack
    }
    pub fn draw<'graph, 'ui>(
        &'graph mut self,
        ctx: &'ui egui::CtxRef,
        ui: &'ui mut egui::Ui,
        theme: &'ui Theme,
    ) -> Option<LinkUpdate> {
        // Ctrl is used to trigger the debug view
        let debug_view = ctx.input().modifiers.ctrl;
        let mut ui_nodes = Vec::with_capacity(self.nodes.len());

        self.nodes_ctx.style.colors[egui_nodes::ColorStyle::NodeBackground as usize] =
            theme.node_background;
        self.nodes_ctx.style.colors[egui_nodes::ColorStyle::NodeBackgroundHovered as usize] =
            theme.node_background_hovered;
        self.nodes_ctx.style.colors[egui_nodes::ColorStyle::NodeBackgroundSelected as usize] =
            theme.node_background_hovered;

        ui.vertical_centered(|ui| {
            if ui.button("Arrange").clicked() {
                log::debug!("Relayouting");
                for node in self.nodes.values_mut() {
                    node.position = None;
                }

                //self.nodes_ctx.reset_panniing(egui::Vec2::ZERO);
            }
        });

        for node in self.nodes.values() {
            let mut ui_node = NodeConstructor::new(
                node.id().value() as usize,
                NodeArgs {
                    titlebar: Some(theme.titlebar),
                    titlebar_hovered: Some(theme.titlebar_hovered),
                    titlebar_selected: Some(theme.titlebar_hovered),
                    ..Default::default()
                },
            );

            node.draw(&mut ui_node, theme, debug_view);

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
        egui::TopBottomPanel::bottom("control_hints").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("[MMB] Move canvas");
                ui.label("[LMB] Move node");
                ui.label("[LMB] Connect port");
                ui.label("[ALT]+[LMB] Disconnect port");
            })
        });

        let mut prev_pos = egui::pos2(ui.available_width() / 4.0, ui.available_height() / 2.0);
        let mut padding = egui::pos2(75.0, 150.0);

        //Find the topologically sorted order of nodes in the graph
        //Nodes are currently laid out based on this order
        let order = self.top_sort();
        for node_id in order {
            let node = self.nodes.get_mut(&node_id).unwrap();

            if !node.position.is_some() {
                padding.y *= -1.0;
                let node_position = egui::pos2(prev_pos.x + padding.x, prev_pos.y + padding.y);

                node.position = Some(node_position);
                self.nodes_ctx
                    .set_node_pos_grid_space(node_id.value() as usize, node_position);

                prev_pos = node_position;
            } else {
                prev_pos = self
                    .nodes_ctx
                    .get_node_pos_grid_space(node_id.value() as usize)
                    .unwrap();
            }
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
}
