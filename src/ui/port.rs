use crate::pipewire_impl::PortType;

#[derive(Debug)]
pub struct Port {
    pub id: u32,
    pub name: String,
    pub port_type: PortType,
}
