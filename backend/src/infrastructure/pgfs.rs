use crate::domain::{
    Id,
    document::{Document, DocumentFile, DocumentFileType},
    faculty::Faculty,
    subject::Subject,
};
use crate::infrastructure::{local_fs::LocalFileSystemRepository, postgresql::PostgresRepository};
use crate::usecase::repository::{
    DocumentFileRepository,
    SearchDocumentOption,
    DocumentRepository,
    FacultyRepository,
    SearchSubjectOption,
    SubjectRepository,
    UpdateSubjectContent,
};

/// Postgresqlとローカルファイルシステムによる完全なリポジトリ実装
#[derive(Debug, Clone)]
pub struct PgFsRepository {
    pool: PostgresRepository,
    fs: LocalFileSystemRepository,
}

impl PgFsRepository {
    pub fn new(pool: PostgresRepository, fs: LocalFileSystemRepository) -> Self {
        Self { pool, fs }
    }
}

// 以下どちらかのリポジトリ実装に振り分けるのみ

impl DocumentRepository for PgFsRepository {
    async fn find_document_by_id(&self, document_id: &Id<Document>) -> anyhow::Result<Option<Document>> {
        self.pool.find_document_by_id(document_id).await
    }

    async fn store_document(&self, document: Document) -> anyhow::Result<()> {
        self.pool.store_document(document).await
    }

    async fn search_documents(&self, option: SearchDocumentOption) -> anyhow::Result<Vec<Document>> {
        self.pool.search_documents(option).await
    }
}

impl DocumentFileRepository for PgFsRepository {
    async fn store_document_file(&self, content: Vec<u8>, file_type: DocumentFileType) -> anyhow::Result<DocumentFile> {
        self.fs.store_document_file(content, file_type).await
    }

    async fn get_document_file_content(&self, document_file: &DocumentFile) -> anyhow::Result<Vec<u8>> {
        self.fs.get_document_file_content(document_file).await
    }
}

impl FacultyRepository for PgFsRepository {
    async fn list_faculties(&self) -> anyhow::Result<Vec<Faculty>> {
        self.pool.list_faculties().await
    }
}

impl SubjectRepository for PgFsRepository {
    #[allow(deprecated)]
    async fn list_subjects(&self) -> anyhow::Result<Vec<Subject>> {
        self.pool.list_subjects().await
    }

    async fn search_subjects(&self, option: SearchSubjectOption) -> anyhow::Result<Vec<Subject>> {
        self.pool.search_subjects(option).await
    }

    async fn create_subject(&self, subject: Subject) -> anyhow::Result<()> {
        self.pool.create_subject(subject).await
    }

    async fn update_subject(&self, subject_id: Id<Subject>, content: UpdateSubjectContent) -> anyhow::Result<Subject> {
        self.pool.update_subject(subject_id, content).await
    }

    async fn delete_subject(&self, subject_id: Id<Subject>) -> anyhow::Result<Subject> {
        self.pool.delete_subject(subject_id).await
    }
}
