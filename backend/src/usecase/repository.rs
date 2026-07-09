use crate::domain::{
    document::{
        Document,
        DocumentFile,
        DocumentFileType,
    },
    faculty::Faculty,
    subject::Subject,
};

pub trait DocumentRepository: Send + Sync {
    fn store_document(&self, document: Document) -> impl Future<Output=anyhow::Result<()>> + Send;
}

pub trait DocumentFileRepository: Send + Sync {
    fn store_document_file(&self, content: Vec<u8>, file_type: DocumentFileType) -> impl Future<Output=anyhow::Result<DocumentFile>> + Send;
}

pub trait FacultyRepository: Send + Sync {
    fn list_faculties(&self) -> impl Future<Output=anyhow::Result<Vec<Faculty>>> + Send;
}

pub trait SubjectRepository: Send + Sync {
    fn list_subjects(&self) -> impl Future<Output=anyhow::Result<Vec<Subject>>> + Send;
}
