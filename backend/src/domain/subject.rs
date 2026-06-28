use super::{
    Id,
    Grade,
    Term,
    faculty::Faculty,
    major::Major,
};

#[derive(Debug, Hash)]
pub struct Subject {
    id: Id<Subject>,
    name: String,
    faculty_id: Id<Faculty>,
    major_id: Id<Major>,
    grade: Grade<Subject>,
    term: Term<Subject>,
}

impl Subject {
    pub fn new(
        id: Id<Subject>,
        name: String,
        faculty_id: Id<Faculty>,
        major_id: Id<Major>,
        grade: Grade<Subject>,
        term: Term<Subject>,
    ) -> Self {
        Self {
            id,
            name,
            faculty_id,
            major_id,
            grade,
            term,
        }
    }
    pub fn id(&self) -> &Id<Subject> {
        &self.id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn faculty_id(&self) -> &Id<Faculty> {
        &self.faculty_id
    }
    pub fn major_id(&self) -> &Id<Major> {
        &self.major_id
    }
    pub fn grade(&self) -> &Grade<Subject> {
        &self.grade
    }
    pub fn term(&self) -> &Term<Subject> {
        &self.term
    }
}
