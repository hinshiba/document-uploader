use super::{
    Id,
    major::Major,
};

#[derive(Debug, Hash)]
pub struct Faculty {
    id: Id<Faculty>,
    name: String,
    majors: Vec<Major>,
}

impl Faculty {
    pub fn new(id: Id<Faculty>, name: String, majors: Vec<Major>) -> Self {
        Self {
            id,
            name,
            majors,
        }
    }
    pub fn id(&self) -> &Id<Faculty> {
        &self.id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn majors(&self) -> &[Major] {
        &self.majors
    }
}
