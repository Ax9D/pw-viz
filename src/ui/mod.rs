mod graph;
mod id;
mod link;
mod node;
mod port;

use crate::pipewire_impl::PipewireMessage;

use pipewire::channel::Sender;
use serde::{Deserialize, Serialize};
use std::sync::mpsc::Receiver;

use graph::Graph;
use id::Id;
use port::Port;

pub const INITIAL_WIDTH: u32 = 1280;
pub const INITIAL_HEIGHT: u32 = 720;

#[derive(Debug)]
pub enum UiMessage {
    RemoveLink(u32),
    AddLink { from_port: u32, to_port: u32 },
    Exit,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Theme {
    titlebar: egui::Color32,
    titlebar_hovered: egui::Color32,

    audio_port: egui::Color32,
    audio_port_hovered: egui::Color32,

    video_port: egui::Color32,
    video_port_hovered: egui::Color32,

    midi_port: egui::Color32,
    midi_port_hovered: egui::Color32,

    text_color: egui::Color32,

    node_background: egui::Color32,
    node_background_hovered: egui::Color32,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            titlebar: egui::Color32::from_rgba_unmultiplied(78, 107, 181, 255),
            titlebar_hovered: egui::Color32::from_rgba_unmultiplied(112, 127, 192, 255),

            audio_port: egui::Color32::from_rgba_unmultiplied(72, 184, 121, 255),
            audio_port_hovered: egui::Color32::from_rgba_unmultiplied(95, 210, 170, 255),

            video_port: egui::Color32::from_rgba_unmultiplied(149, 56, 173, 255),
            video_port_hovered: egui::Color32::from_rgba_unmultiplied(148, 96, 182, 255),

            midi_port: egui::Color32::RED,
            midi_port_hovered: egui::Color32::LIGHT_RED,

            text_color: egui::Color32::WHITE,
            node_background: egui::Color32::from_rgba_unmultiplied(50, 50, 50, 255),
            node_background_hovered: egui::Color32::from_rgba_unmultiplied(75, 75, 75, 255),
        }
    }
}

pub struct GraphUI {
    graph: Graph,
    pipewire_receiver: Receiver<PipewireMessage>,
    pipewire_sender: Sender<UiMessage>,
    theme: Theme,
    show_theme: bool,
    show_about: bool,
    show_controls: bool,
}

impl GraphUI {
    pub fn new(
        pipewire_receiver: Receiver<PipewireMessage>,
        pipewire_sender: Sender<UiMessage>,
    ) -> Self {
        GraphUI {
            graph: Graph::new(),
            pipewire_receiver,
            pipewire_sender,
            theme: Theme::default(),
            show_theme: false,
            show_about: false,
            show_controls: false,
        }
    }

    fn theme_window(&mut self, ctx: &egui::Context, _ui: &mut egui::Ui) {
        let theme = &mut self.theme;
        egui::Window::new("Theme")
            .open(&mut self.show_theme)
            .resizable(true)
            .show(ctx, |ui| {
                egui::Grid::new("theme_grid").num_columns(2).show(ui, |ui| {
                    ui.label("Node titlebar");
                    ui.color_edit_button_srgba(&mut theme.titlebar);
                    ui.end_row();

                    ui.label("Node titlebar hovered");
                    ui.color_edit_button_srgba(&mut theme.titlebar_hovered);
                    ui.end_row();

                    ui.label("Audio port");
                    ui.color_edit_button_srgba(&mut theme.audio_port);
                    ui.end_row();

                    ui.label("Audio port hovered");
                    ui.color_edit_button_srgba(&mut theme.audio_port_hovered);
                    ui.end_row();

                    ui.label("Video port");
                    ui.color_edit_button_srgba(&mut theme.video_port);
                    ui.end_row();

                    ui.label("Video port hovered");
                    ui.color_edit_button_srgba(&mut theme.video_port_hovered);
                    ui.end_row();

                    ui.label("MIDI port");
                    ui.color_edit_button_srgba(&mut theme.midi_port);
                    ui.end_row();

                    ui.label("MIDI port hovered");
                    ui.color_edit_button_srgba(&mut theme.midi_port_hovered);
                    ui.end_row();

                    ui.label("Node background");
                    ui.color_edit_button_srgba(&mut theme.node_background);
                    ui.end_row();

                    ui.label("Node background hovered");
                    ui.color_edit_button_srgba(&mut theme.node_background_hovered);
                    ui.end_row();

                    ui.label("Text color");
                    ui.color_edit_button_srgba(&mut theme.text_color);
                    ui.end_row();
                });

                if ui.button("Default").clicked() {
                    #[cfg(debug_assertions)]
                    log::debug!("Old theme:\n{:?}", theme);

                    *theme = Theme::default();
                }
            });
    }

    fn about_window(&mut self, ctx: &egui::Context, _ui: &mut egui::Ui) {
        egui::Window::new("About")
            .open(&mut self.show_about)
            .resizable(false)
            .show(ctx, |ui| {
                egui::Grid::new("about_grid").show(ui, |ui| {
                    ui.label(env!("CARGO_PKG_NAME"));
                    ui.end_row();

                    ui.label("Version");
                    ui.label(env!("CARGO_PKG_VERSION"));
                    ui.end_row();

                    ui.label("Project page");
                    ui.hyperlink("https://github.com/Ax9D/pw-viz");
                    ui.end_row();
                })
            });
    }

    fn controls_window(&mut self, ctx: &egui::Context, _ui: &mut egui::Ui) {
        egui::Window::new("Controls")
            .open(&mut self.show_controls)
            .resizable(false)
            .show(ctx, |ui| {
                egui::Grid::new("controls_grid").show(ui, |ui| {
                    ui.label("Left Click + Drag");
                    ui.label("Move nodes, create links between nodes");
                    ui.end_row();

                    ui.label("Alt + Left Click + Drag");
                    ui.label("Remove Links");
                    ui.end_row();

                    ui.label("Middle Mouse + Drag");
                    ui.label("Pan the graph");
                    ui.end_row();

                    ui.label("Ctrl");
                    ui.label("Show pipewire ids of nodes and ports");
                    ui.end_row();
                })
            });
    }

    /// Update the graph ui based on the message sent by the pipewire thread
    fn process_message(&mut self, message: PipewireMessage) {
        match message {
            PipewireMessage::NodeAdded {
                id,
                name,
                description,
                media_type,
            } => {
                self.graph.add_node(name, id, description, media_type);
            }
            PipewireMessage::NodeRemoved { name, id } => {
                self.graph.remove_node(&name, id);
            }

            PipewireMessage::PortAdded {
                node_name,
                node_id,
                id,
                name,
                port_type,
            } => {
                let port = Port::new(id, name, port_type);

                self.graph.add_port(node_name, node_id, port);
            }

            PipewireMessage::LinkAdded {
                id,
                from_node_name,
                to_node_name,
                from_port,
                to_port,
            } => {
                self.graph
                    .add_link(id, from_node_name, to_node_name, from_port, to_port);
            }
            PipewireMessage::PortRemoved {
                node_name,
                node_id,
                id,
            } => {
                self.graph.remove_port(&node_name, node_id, id);
            }
            PipewireMessage::LinkRemoved { id } => {
                self.graph.remove_link(id);
            }
            PipewireMessage::LinkStateChanged { id: _, active: _ } => {}
        };
    }

    /// Keep processing messages in a non blocking way until there aren't any new messages
    fn pump_messages(&mut self) {
        loop {
            match self.pipewire_receiver.try_recv() {
                Ok(message) => self.process_message(message),
                Err(err) => match err {
                    std::sync::mpsc::TryRecvError::Empty => break,
                    std::sync::mpsc::TryRecvError::Disconnected => {
                        panic!("Pipewire channel disconnected!")
                    }
                },
            }
        }
    }
}

impl eframe::App for GraphUI {
    // /// Called once before the first frame.
    // fn setup(
    //     &mut self,
    //     _ctx: &egui::Context,
    //     _frame: &mut epi::Frame<'_>,
    //     storage: Option<&dyn eframe::Storage>,
    // ) {
    //     if let Some(storage) = storage {
    //         self.theme = epi::get_value(storage, "theme").unwrap_or_default();
    //     }
    // }

    /// Called by the frame work to save state before shutdown.
    /// Note that you must enable the `persistence` feature for this to work.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, "theme", &self.theme);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.pump_messages();

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                egui::menu::menu_button(ui, "File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.close();
                    }
                });
                egui::menu::menu_button(ui, "Settings", |ui| {
                    if ui.button("Theme").clicked() {
                        self.show_theme = true;
                    }
                });
                egui::menu::menu_button(ui, "Help", |ui| {
                    if ui.button("Controls").clicked() {
                        self.show_controls = true;
                    }
                    if ui.button("About").clicked() {
                        self.show_about = true;
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // If any new links were created/removed, notify the pipewire thread
            if let Some(link_update) = self.graph.draw(ctx, ui, &self.theme) {
                match link_update {
                    graph::LinkUpdate::Created {
                        from_port,
                        to_port,
                        from_node: _,
                        to_node: _,
                    } => {
                        self.pipewire_sender
                            .send(UiMessage::AddLink { from_port, to_port })
                            .expect("Failed to send ui message");
                    }
                    graph::LinkUpdate::Removed(link_id) => {
                        self.pipewire_sender
                            .send(UiMessage::RemoveLink(link_id))
                            .expect("Failed to send ui message");
                    }
                }
            }

            if self.show_theme {
                self.theme_window(ctx, ui);
            }
            if self.show_about {
                self.about_window(ctx, ui);
            }
            if self.show_controls {
                self.controls_window(ctx, ui);
            }

            egui::TopBottomPanel::bottom("control_hints").show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("[MMB] Move canvas");
                    ui.label("[LMB] Move node");
                    ui.label("[LMB] Connect port");
                    ui.label("[ALT]+[LMB] Disconnect port");
                })
            });
        });
    }

    fn on_close_event(&mut self) -> bool {
        self.pipewire_sender
            .send(UiMessage::Exit)
            .expect("Failed to send ui message");

        true
    }
}

fn create_app(
    cc: &eframe::CreationContext,
    reciever: Receiver<PipewireMessage>,
    sender: Sender<UiMessage>,
) -> Box<dyn eframe::App> {
    let mut graph = GraphUI::new(reciever, sender);

    //Load theme config
    if let Some(storage) = cc.storage {
        graph.theme = eframe::get_value(storage, "theme").unwrap_or_default();
    }

    Box::new(graph)
}
pub fn run_graph_ui(receiver: Receiver<PipewireMessage>, sender: Sender<UiMessage>) {
    let initial_window_size = egui::vec2(INITIAL_WIDTH as f32, INITIAL_HEIGHT as f32);

    eframe::run_native(
        env!("CARGO_PKG_NAME"),
        eframe::NativeOptions {
            initial_window_size: Some(initial_window_size),
            ..Default::default()
        },
        Box::new(|cc| create_app(cc, receiver, sender)),
    );
}
