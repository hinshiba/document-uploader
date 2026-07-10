use crate::domain::document::{
    Document,
    DocumentFileType,
    DocumentMetadata,
};
use crate::usecase::repository::{
    DocumentRepository,
    DocumentFileRepository,
};

#[derive(Debug, Clone, Hash)]
pub struct StoreDocumentInputFile {
    pub file_type: DocumentFileType,
    pub content: Vec<u8>,
}

#[derive(Debug, Clone, Hash)]
pub struct StoreDocumentInput {
    pub metadata: DocumentMetadata,
    pub files: Vec<StoreDocumentInputFile>,
}

#[derive(Debug)]
pub struct StoreDocumentUseCase<I> {
    repository: I
}

impl<I> StoreDocumentUseCase<I> {
    pub fn new(repository: I) -> Self {
        Self { repository }
    }
}

impl<I: DocumentRepository + DocumentFileRepository> StoreDocumentUseCase<I> {
    #[tracing::instrument(skip(self), ret(level="debug"), err)]
    pub async fn execute(&self, input: StoreDocumentInput) -> anyhow::Result<()> {
        let mut files = Vec::with_capacity(input.files.len());
        for file in input.files {
            files.push(
                self.repository.store_document_file(
                    file.content,
                    file.file_type,
                ).await?
            );
        }
        let document = Document::new(input.metadata, files)?;

        self.repository.store_document(document).await
    }
}
