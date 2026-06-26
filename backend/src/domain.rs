pub struct Id<T> {
    id: uuid::Uuid,
    _phantom: std::marker::PhantomData<fn() -> T>,
}

impl<T> Id<T> {
    pub fn new(id: uuid::Uuid) -> Self {
        Self {
            id,
            _phantom: std::marker::PhantomData,
        }
    }
    pub fn id(&self) -> &uuid::Uuid {
        &self.id
    }
}

impl<T> std::fmt::Debug for Id<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.id)
    }
}

impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T> Copy for Id<T> {}

impl<T> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(other.id())
    }
}

impl<T> Eq for Id<T> {}

impl<T> std::hash::Hash for Id<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self._phantom.hash(state);
    }
}

#[derive(Clone, Debug)]
pub struct ParseExamTypeError;

impl std::fmt::Display for ParseExamTypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid examtype")
    }
}
impl std::error::Error for ParseExamTypeError {}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
#[repr(i64)]
pub enum ExamType {
    Quiz = 0,
    MidTerm,
    FinalTerm,
    Other,
}

impl ExamType {
    pub fn from_int(t: i64) -> Option<ExamType> {
        match t {
            0 => Some(ExamType::Quiz),
            1 => Some(ExamType::MidTerm),
            2 => Some(ExamType::FinalTerm),
            3 => Some(ExamType::Other),
            _ => None,
        }
    }
    pub fn to_int(self) -> i64 {
        self as i64
    }
}

impl std::str::FromStr for ExamType {
    type Err = ParseExamTypeError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use ExamType::*;
        match s {
            "quiz" => Ok(Quiz),
            "midterm" => Ok(MidTerm),
            "final" | "finalterm" => Ok(FinalTerm),
            "other" => Ok(Other),
            _ => Err(ParseExamTypeError),
        }
    }
}

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
            majors: majors,
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

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct DocumentMetadata {
    faculty_id: Id<Faculty>,
    major_id: Id<Major>,
    year: i64,
    term: i64,
    grade: i64,
    subject_id: Id<Subject>,
    teacher_id: Id<Teacher>,
    exam_type: ExamType,
    is_answer: bool,
    num: i64,
}

impl DocumentMetadata {
    pub fn new(
        faculty_id: Id<Faculty>,
        major_id: Id<Major>,
        year: i64,
        term: i64,
        grade: i64,
        subject_id: Id<Subject>,
        teacher_id: Id<Teacher>,
        exam_type: ExamType,
        is_answer: bool,
        num: i64,
    ) -> Self {
        Self {
            faculty_id,
            major_id,
            year,
            term,
            grade,
            subject_id,
            teacher_id,
            exam_type,
            is_answer,
            num,
        }
    }
    pub fn faculty_id(&self) -> &Id<Faculty> {
        &self.faculty_id
    }
    pub fn major_id(&self) -> &Id<Major> {
        &self.major_id
    }
    pub fn year(&self) -> &i64 {
        &self.year
    }
    pub fn term(&self) -> &i64 {
        &self.term
    }
    pub fn grade(&self) -> &i64 {
        &self.grade
    }
    pub fn subject_id(&self) -> &Id<Subject> {
        &self.subject_id
    }
    pub fn teacher_id(&self) -> &Id<Teacher> {
        &self.teacher_id
    }
    pub fn exam_type(&self) -> &ExamType {
        &self.exam_type
    }
    pub fn is_answer(&self) -> &bool {
        &self.is_answer
    }
    pub fn num(&self) -> &i64 {
        &self.num
    }
}

#[derive(Clone, Debug)]
pub struct UnsupportedFileType(String);

impl std::fmt::Display for UnsupportedFileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "file type `{}` is unsupported", self.0)
    }
}
impl std::error::Error for UnsupportedFileType {}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum DocumentFileType {
    Jpeg,
    Webp,
    Png,
    Doc,
    Docx,
    Pdf,
    Txt,
    Markdown,
    Typst,
    Tex,
}

impl std::str::FromStr for DocumentFileType {
    type Err = UnsupportedFileType;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use DocumentFileType::*;
        match s {
            "jpg" | "jpe" | "jpeg" => Ok(Jpeg),
            "webp" => Ok(Webp),
            "png" => Ok(Png),
            "doc" => Ok(Doc),
            "docx" => Ok(Docx),
            "pdf" => Ok(Pdf),
            "txt" => Ok(Txt),
            "md" => Ok(Markdown),
            "typ" => Ok(Typst),
            "tex" => Ok(Tex),
            _ => Err(UnsupportedFileType(s.to_owned())),
        }
    }
}

#[derive(Debug, Hash)]
pub struct DocumentFile {
    ty: DocumentFileType,
    path: std::path::PathBuf,
}

impl DocumentFile {
    pub fn new(ty: DocumentFileType, path: std::path::PathBuf) -> Self {
        Self { ty, path }
    }
    pub fn ty(&self) -> &DocumentFileType {
        &self.ty
    }
    pub fn path(&self) -> &std::path::Path {
        &self.path
    }
}

#[derive(Debug, Hash)]
pub struct Document {
    metadata: DocumentMetadata,
    files: Vec<DocumentFile>,
}

impl Document {
    pub fn new(metadata: DocumentMetadata, files: Vec<DocumentFile>) -> Self {
        Self { metadata, files }
    }
    pub fn metadata(&self) -> &DocumentMetadata {
        &self.metadata
    }
    pub fn files(&self) -> &[DocumentFile] {
        &self.files
    }
}
