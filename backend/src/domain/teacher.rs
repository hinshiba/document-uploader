use super::{
    Id,
    faculty::Faculty,
};

#[derive(Debug, Hash)]
pub struct Teacher {
    id: Id<Teacher>,
    name: String,
    belong_faculty_id: Option<Id<Faculty>>,
}

impl Teacher {
    pub fn new(id: Id<Teacher>, name: String, belong_faculty_id: Option<Id<Faculty>>) -> Self {
        Self {
            id,
            name,
            belong_faculty_id,
        }
    }
    pub fn id(&self) -> &Id<Teacher> {
        &self.id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn belong_faculty_id(&self) -> Option<&Id<Faculty>> {
        self.belong_faculty_id.as_ref()
    }
}
