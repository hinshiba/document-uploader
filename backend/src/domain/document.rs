use super::{
    Id,
    Grade,
    Term,
    Year,
    Num,
    faculty::Faculty,
    major::Major,
    subject::Subject,
};

#[derive(Clone, Debug)]
pub struct UnsupportedFileType(String);

impl std::fmt::Display for UnsupportedFileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "file type `{}` is unsupported", self.0)
    }
}

impl std::error::Error for UnsupportedFileType {}

#[derive(Clone, Debug)]
pub struct ParseExamTypeError;

impl std::fmt::Display for ParseExamTypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid examtype")
    }
}

impl std::error::Error for ParseExamTypeError {}

#[derive(Clone, Debug)]
pub struct EmptyDocumentFiles;

impl std::fmt::Display for EmptyDocumentFiles {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "must have at least one document file")
    }
}

impl std::error::Error for EmptyDocumentFiles {}

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

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct DocumentMetadata {
    faculty_id: Id<Faculty>,
    major_id: Id<Major>,
    year: Year<DocumentMetadata>,
    term: Term<DocumentMetadata>,
    grade: Grade<DocumentMetadata>,
    subject_id: Id<Subject>,
    teacher: String,
    exam_type: ExamType,
    is_answer: bool,
    num: Num<DocumentMetadata>,
}

impl DocumentMetadata {
    pub fn new(
        faculty_id: Id<Faculty>,
        major_id: Id<Major>,
        year: Year<DocumentMetadata>,
        term: Term<DocumentMetadata>,
        grade: Grade<DocumentMetadata>,
        subject_id: Id<Subject>,
        teacher: String,
        exam_type: ExamType,
        is_answer: bool,
        num: Num<DocumentMetadata>,
    ) -> Self {
        Self {
            faculty_id,
            major_id,
            year,
            term,
            grade,
            subject_id,
            teacher,
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
    pub fn year(&self) -> &Year<DocumentMetadata> {
        &self.year
    }
    pub fn term(&self) -> &Term<DocumentMetadata> {
        &self.term
    }
    pub fn grade(&self) -> &Grade<DocumentMetadata> {
        &self.grade
    }
    pub fn subject_id(&self) -> &Id<Subject> {
        &self.subject_id
    }
    pub fn teacher(&self) -> &str {
        &self.teacher
    }
    pub fn exam_type(&self) -> &ExamType {
        &self.exam_type
    }
    pub fn is_answer(&self) -> &bool {
        &self.is_answer
    }
    pub fn num(&self) -> &Num<DocumentMetadata> {
        &self.num
    }
}

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
    pub fn new(metadata: DocumentMetadata, files: Vec<DocumentFile>) -> Result<Self, EmptyDocumentFiles> {
        if files.len() >= 1 {
            Ok(Self { metadata, files })
        } else {
            Err(EmptyDocumentFiles)
        }
    }
    pub fn metadata(&self) -> &DocumentMetadata {
        &self.metadata
    }
    pub fn files(&self) -> &[DocumentFile] {
        &self.files
    }
}
