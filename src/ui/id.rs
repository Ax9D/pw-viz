use std::hash::Hash;

use egui::epaint::ahash::AHasher;
use std::hash::Hasher;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Id(u64);

impl Id {
    pub fn new(data: impl Hash) -> Self {
        let mut hasher = AHasher::new_with_keys(123, 456);
        data.hash(&mut hasher);
        Id(hasher.finish())
    }
    #[inline]
    pub fn value(&self) -> u64 {
        self.0
    }
}
