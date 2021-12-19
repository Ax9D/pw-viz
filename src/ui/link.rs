use super::Id;

#[derive(Debug)]
pub struct Link {
    pub id: u32,
    pub from_node: Id,
    pub to_node: Id,

    pub from_port: u32,
    pub to_port: u32,
    pub active: bool,
}

impl Link {
    pub fn is_self_link(&self) -> bool {
        self.from_node == self.to_node
    }
}
