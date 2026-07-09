use tokio::io::AsyncWriteExt;

use crate::domain::document::{
    Document,
    DocumentFile,
    DocumentFileType,
    DocumentMetadata,
};
use crate::usecase::repository::DocumentRepository;

#[derive(Debug, Clone, Hash)]
pub struct StoreDocumentInputFile {
    file_name: String,
    file_type: DocumentFileType,
    content: Vec<u8>,
}

#[derive(Debug, Clone, Hash)]
pub struct StoreDocumentInput {
    /// FIXME: `save_dir`が漏れるのはおかしい
    pub save_dir: std::path::PathBuf,
    pub metadata: DocumentMetadata,
    pub files: Vec<StoreDocumentInputFile>,
}

pub struct StoreDocumentUseCase<I> {
    repository: I
}

impl<I> StoreDocumentUseCase<I> {
    pub fn new(repository: I) -> Self {
        Self { repository }
    }
}

impl<I: DocumentRepository> StoreDocumentUseCase<I> {
    #[tracing::instrument(skip(self), ret(level="debug"), err)]
    pub async fn execute(&self, input: StoreDocumentInput) -> anyhow::Result<()> {
        let mut files = Vec::with_capacity(input.files.len());
        for file in input.files {
            files.push(
                save_file(
                    file.file_name,
                    file.file_type,
                    file.content,
                    &input.save_dir
                ).await?
            );
        }
        let document = Document::new(input.metadata, files)?;

        self.repository.store_document(document).await
    }
}

// 以下 helper functions

/// FIXME: ファイルの保存はUseCaseで行うべきではない
#[tracing::instrument(ret(level="debug"), err)]
async fn save_file(filename: String, filetype: DocumentFileType, content: Vec<u8>, save_dir: &std::path::Path) -> anyhow::Result<DocumentFile> {
    let file_path = save_dir.with_file_name(filename);

    let mut buffer = tokio::io::BufWriter::new(
        tokio::fs::File::create_new(&file_path).await?
    );
    buffer.write_all(&content).await?;
    buffer.flush().await?;

    Ok(DocumentFile::new(filetype, file_path))
}
