pub use super::Id;

#[derive(Debug, Hash)]
pub struct Major {
    id: Id<Major>,
    name: String,
}

impl Major {
    pub fn new(id: Id<Major>, name: String) -> Self {
        Self { id, name }
    }
    pub fn id(&self) -> &Id<Major> {
        &self.id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
}
