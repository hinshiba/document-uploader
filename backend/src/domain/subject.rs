use super::{
    Id,
    faculty::Faculty,
};

#[derive(Debug, Hash)]
pub struct Subject {
    id: Id<Subject>,
    name: String,
    faculty_id: Option<Id<Faculty>>,
}

impl Subject {
    pub fn new(id: Id<Subject>, name: String, faculty_id: Option<Id<Faculty>>) -> Self {
        Self {
            id,
            name,
            faculty_id,
        }
    }
    pub fn id(&self) -> &Id<Subject> {
        &self.id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn faculty_id(&self) -> Option<&Id<Faculty>> {
        self.faculty_id.as_ref()
    }
}
