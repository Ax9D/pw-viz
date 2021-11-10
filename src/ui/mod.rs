mod graph;
mod link;
mod node;
mod port;

use crate::pipewire_impl::PipewireMessage;
use eframe::epi;
use std::sync::mpsc::Receiver;

use graph::Graph;
use link::Link;
use node::Node;
use port::Port;

pub const WIDTH: u32 = 1280;
pub const HEIGHT: u32 = 720;
pub struct GraphUI {
    graph: Graph,
    pipewire_receiver: Receiver<PipewireMessage>,
    nodes_ctx: egui_nodes::Context,
}

impl GraphUI {
    pub fn new(pipewire_receiver: Receiver<PipewireMessage>) -> Self {
        GraphUI {
            graph: Graph::new(),
            pipewire_receiver,
            nodes_ctx: egui_nodes::Context::default(),
        }
    }
    fn process_message(&mut self, message: PipewireMessage) {
        let graph = &mut self.graph;

        match message {
            PipewireMessage::NodeAdded {
                id,
                name,
                media_type,
            } => {
                let node = Node::new(id, name, media_type);

                self.graph.add_node(node);
            }
            PipewireMessage::PortAdded {
                node_id,
                id,
                name,
                port_type,
            } => {
                let port = Port::new(id, name, port_type);

                self.graph
                    .get_node_mut(node_id)
                    .expect("Node with provided id doesn't exist")
                    .add_port(port);
            }

            PipewireMessage::LinkAdded {
                id,
                from_node,
                to_node,
                from_port,
                to_port,
            } => {
                let link = Link {
                    id,
                    from_node,
                    to_node,
                    from_port,
                    to_port,
                    active: true,
                };

                self.graph.add_link(link);
            }
            PipewireMessage::LinkStateChanged { id, active } => {}

            PipewireMessage::NodeRemoved { id } => {
                self.graph.remove_node(id);
            }
            PipewireMessage::PortRemoved { node_id, id } => {
                self.graph
                    .get_node_mut(node_id)
                    .expect("Node with provided id doesn't exist")
                    .remove_port(id);
            }
            PipewireMessage::LinkRemoved { id } => {
                self.graph.remove_link(id);
            }
        };
    }
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
impl epi::App for GraphUI {
    fn name(&self) -> &str {
        env!("CARGO_PKG_NAME")
    }

    /// Called once before the first frame.
    fn setup(
        &mut self,
        _ctx: &egui::CtxRef,
        _frame: &mut epi::Frame<'_>,
        _storage: Option<&dyn epi::Storage>,
    ) {
    }
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }
    fn update(&mut self, ctx: &egui::CtxRef, _frame: &mut epi::Frame<'_>) {
        self.pump_messages();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.hyperlink("https://github.com/Ax9D");

            self.graph.draw(&mut self.nodes_ctx, ui);
        });
    }
}

pub fn run_graph_ui(receiver: Receiver<PipewireMessage>) {
    let initial_window_size = egui::vec2(WIDTH as f32, HEIGHT as f32);
    eframe::run_native(
        Box::new(GraphUI::new(receiver)),
eframe::NativeOptions {
                initial_window_size: Some(initial_window_size),
                ..Default::default()
            },
    );
}
