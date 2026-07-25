use crate::domain::{
    Id,
    Grade,
    Term,
    document::{
        Document,
        DocumentFile,
        DocumentFileType,
    },
    faculty::Faculty,
    major::Major,
    subject::Subject,
};

#[derive(Debug, Hash)]
pub struct UpdateSubjectContent {
    pub name: String,
    pub faculty_id: Id<Faculty>,
    pub major_id: Id<Major>,
    pub grade: Grade<Subject>,
    pub term: Term<Subject>,
}

#[derive(Debug, Hash, Default)]
pub struct SearchSubjectOption {
    pub subject_id: Option<Id<Subject>>,
    pub name: Option<String>,
    pub faculty_id: Option<Id<Faculty>>,
    pub major_id: Option<Id<Major>>,
    pub grade: Option<Grade<Subject>>,
    pub term: Option<Term<Subject>>,
}

pub trait DocumentRepository: Send + Sync {
    fn store_document(&self, document: Document) -> impl Future<Output=anyhow::Result<()>> + Send;
}

pub trait DocumentFileRepository: Send + Sync {
    fn store_document_file(&self, content: Vec<u8>, file_type: DocumentFileType) -> impl Future<Output=anyhow::Result<DocumentFile>> + Send;
    fn get_document_file_content(&self, document_file: &DocumentFile) -> impl Future<Output=anyhow::Result<Vec<u8>>> + Send;
}

pub trait FacultyRepository: Send + Sync {
    fn list_faculties(&self) -> impl Future<Output=anyhow::Result<Vec<Faculty>>> + Send;
}

pub trait SubjectRepository: Send + Sync {
    #[deprecated(note = "Use SubjectRepository::search_subjects instead")]
    fn list_subjects(&self) -> impl Future<Output=anyhow::Result<Vec<Subject>>> + Send;

    fn search_subjects(&self, option: SearchSubjectOption) -> impl Future<Output=anyhow::Result<Vec<Subject>>> + Send;

    /// # Returns
    /// 作成処理が完了したとき、[`Ok`]を返す
    /// 
    /// # Errors
    /// `subject.id`と等しいidを持つ[`Subject`]が既に存在する場合はエラーを返す
    fn create_subject(&self, subject: Subject) -> impl Future<Output=anyhow::Result<()>> + Send;

    /// # Returns
    /// 更新処理が完了したとき、更新後の[`Subject`]を[`Ok`]に包んで返す
    /// 
    /// # Errors
    /// `subject_id`に対応する[`Subject`]が存在しないときはエラーを返す
    fn update_subject(&self, subject_id: Id<Subject>, content: UpdateSubjectContent) -> impl Future<Output=anyhow::Result<Subject>> + Send;
    
    /// # Returns
    /// 削除処理が完了したとき、削除前の`subject_id`に対応する[`Subject`]を[`Ok`]に包んで返す
    /// 
    /// # Errors
    /// `subject_id`に対応する[`Subject`]が存在しないときはエラーを返す
    fn delete_subject(&self, subject_id: Id<Subject>) -> impl Future<Output=anyhow::Result<Subject>> + Send;
}
