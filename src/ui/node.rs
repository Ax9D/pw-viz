use std::collections::HashMap;

use egui_nodes::{NodeConstructor, PinArgs};

use crate::pipewire_impl::MediaType;

use super::{port::Port, Id, Theme};

#[derive(Debug)]
pub struct Node {
    id: Id,
    name: String,
    pw_nodes: Vec<PwNode>,
    pub(super) position: Option<egui::Pos2>,
}

impl Node {
    pub fn new(id: Id, name: String) -> Self {
        Self {
            id,
            name,
            pw_nodes: Vec::new(),
            position: None,
        }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn id(&self) -> Id {
        self.id
    }

    pub(super) fn add_pw_node(
        &mut self,
        id: u32,
        description: Option<String>,
        media_type: Option<MediaType>,
    ) {
        self.pw_nodes.push(PwNode {
            id,
            description,
            media_type,
            ports: HashMap::new(),
        });
    }
    //TODO: Use pooling
    pub(super) fn remove_pw_node(&mut self, id: u32) -> bool {
        self.pw_nodes.retain(|node| node.id != id);

        self.pw_nodes.is_empty()
    }

    #[inline]
    fn get_pw_node(&mut self, id: u32) -> Option<&mut PwNode> {
        self.pw_nodes.iter_mut().find(|node| node.id == id)
    }
    pub fn add_port(&mut self, node_id: u32, port: Port) {
        let pw_node = self.get_pw_node(node_id);

        pw_node
            .expect(&format!(
                "Couldn't find pipewire node with id {}",
                port.id()
            ))
            .ports
            .insert(port.id(), port);
    }
    pub fn remove_port(&mut self, node_id: u32, port_id: u32) {
        if let Some(pw_node) = self.get_pw_node(node_id) {
            pw_node.ports.remove(&port_id);
        } else {
            log::error!("Pipewire node with id: {} was never added", node_id);
        }
    }
    fn draw_ports<'graph, 'node>(
        ui_node: &'graph mut NodeConstructor<'node>,
        node: &'node PwNode,
        theme: &'node Theme,
        debug: bool,
    ) {
        let mut ports = node.ports.values().collect::<Vec<_>>();

        //Sorts ports based on natural ordering
        ports.sort_by(|a, b| natord::compare(a.name(), b.name()));

        for (ix, port) in ports.iter().enumerate() {
            let (background, hovered) = match &node.media_type {
                Some(MediaType::Audio) => (theme.audio_port, theme.audio_port_hovered),
                Some(MediaType::Video) => (theme.video_port, theme.video_port_hovered),
                Some(MediaType::Midi) => (theme.midi_port, theme.midi_port_hovered),
                None => (egui::Color32::GRAY, egui::Color32::LIGHT_GRAY),
            };
            let port_name = {
                if debug {
                    format!("{} [{}]", port.name(), port.id())
                } else {
                    format!("{} ", port.name())
                }
            };

            let first = debug && ix == 0;

            let node_desc_str = if let Some(desc) = &node.description {
                desc
            } else {
                ""
            };

            let node_desc = format!("{} [{}]", node_desc_str, node.id);

            match port.port_type() {
                crate::pipewire_impl::PortType::Input => {
                    if first {
                        ui_node.with_input_attribute(
                            port.id() as usize,
                            PinArgs {
                                background: Some(background),
                                hovered: Some(hovered),
                                ..Default::default()
                            },
                            move |ui| {
                                ui.colored_label(egui::Color32::WHITE, node_desc);
                                ui.label(port_name)
                            },
                        );
                    } else {
                        ui_node.with_input_attribute(
                            port.id() as usize,
                            PinArgs {
                                background: Some(background),
                                hovered: Some(hovered),
                                ..Default::default()
                            },
                            |ui| ui.label(port_name),
                        );
                    }
                }
                crate::pipewire_impl::PortType::Output => {
                    if first {
                        ui_node.with_output_attribute(
                            port.id() as usize,
                            PinArgs {
                                background: Some(background),
                                hovered: Some(hovered),
                                ..Default::default()
                            },
                            move |ui| {
                                ui.colored_label(egui::Color32::WHITE, node_desc);
                                ui.label(port_name)
                            },
                        );
                    } else {
                        ui_node.with_output_attribute(
                            port.id() as usize,
                            PinArgs {
                                background: Some(background),
                                hovered: Some(hovered),
                                ..Default::default()
                            },
                            |ui| ui.label(port_name),
                        );
                    }
                }
                crate::pipewire_impl::PortType::Unknown => {}
            }
        }
    }

    pub fn draw<'graph, 'node>(
        &'node self,
        ui_node: &'graph mut NodeConstructor<'node>,
        theme: &'node Theme,
        debug_view: bool,
    ) {
        // let media_type = node.media_type;
        // let media_emoji = match media_type {
        //     Some(MediaType::Audio) => "🔉",
        //     Some(MediaType::Video) => "💻",
        //     Some(MediaType::Midi) => "🎹",
        //     None => "",
        // };
        let mut media_type = String::new();
        for node in self.pw_nodes.iter() {
            let media_emoji = match node.media_type {
                Some(MediaType::Audio) => "🔉",
                Some(MediaType::Video) => "💻",
                Some(MediaType::Midi) => "🎹",
                None => "",
            };

            if !media_type.contains(media_emoji) {
                media_type.push_str(&format!(" {}", media_emoji));
            }
        }

        ui_node.with_title(move |ui| {
            ui.colored_label(theme.text_color, &format!("{} {}", self.name(), media_type))
        });

        for node in self.pw_nodes.iter() {
            Self::draw_ports(ui_node, node, theme, debug_view);
        }
    }
}

#[derive(Debug)]
struct PwNode {
    id: u32, //Pipewire id of the node
    description: Option<String>,
    media_type: Option<MediaType>,
    ports: HashMap<u32, Port>,
}
