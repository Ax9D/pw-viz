#[derive(Debug)]
pub struct Link {
    pub id: u32,
    pub from_node: u32,
    pub to_node: u32,

    pub from_port: u32,
    pub to_port: u32,
    pub active: bool,
}
