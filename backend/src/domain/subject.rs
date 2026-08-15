use super::{
    Id,
    Grade,
    Term,
    faculty::Faculty,
    major::Major,
    FormatValidationError,
};

#[derive(Debug, Hash)]
pub struct Subject {
    id: Id<Subject>,
    name: String,
    course_code: CourseCode,
    faculty_id: Id<Faculty>,
    major_id: Id<Major>,
    grade: Grade<Subject>,
    term: Term<Subject>,
}

impl Subject {
    pub fn new(
        id: Id<Subject>,
        name: String,
        course_code: CourseCode,
        faculty_id: Id<Faculty>,
        major_id: Id<Major>,
        grade: Grade<Subject>,
        term: Term<Subject>,
    ) -> Self {
        Self {
            id,
            name,
            course_code,
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
    pub fn course_code(&self) -> &CourseCode {
        &self.course_code
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

/// 講義番号と一致する文字列
/// 
/// 重複判定に用いる
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CourseCode {
    inner: String,
}

impl CourseCode {
    /// 講義番号を文字列から作成する
    /// 
    /// TODO: 検証を追加する(実存確認は大変なので......)
    pub fn new(code: impl Into<String>) -> Result<Self, FormatValidationError> {
        Ok(Self {
            inner: code.into()
        })
    }

    pub fn code(&self) -> &str {
        &self.inner
    }
}
