pub use super::{
    Id,
    faculty::Faculty,
};

#[derive(Debug, Hash)]
pub struct Major {
    id: Id<Major>,
    name: String,
    faculty_id: Id<Faculty>,
}

impl Major {
    pub fn new(id: Id<Major>, name: String, faculty_id: Id<Faculty>) -> Self {
        Self {
            id,
            name,
            faculty_id
        }
    }
    pub fn id(&self) -> &Id<Major> {
        &self.id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn faculty_id(&self) -> &Id<Faculty> {
        &self.faculty_id
    }
}
