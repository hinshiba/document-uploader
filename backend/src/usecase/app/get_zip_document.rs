use tokio::io::AsyncWriteExt;

use crate::domain::{
    Id,
    document::{
        Document,
        DocumentFile,
    },
};
use crate::usecase::repository::{
    DocumentRepository,
    DocumentFileRepository,
};

#[derive(Debug, Clone, Hash)]
pub struct GetZipDocumentInput {
    pub document_id: Id<Document>,
}

#[derive(Debug, Clone, Hash)]
pub struct GetZipDocumentOutput {
    pub name: String,
    pub content: Vec<u8>,
}

#[derive(Debug)]
pub struct GetZipDocumentUseCase<I> {
    repository: I
}

impl<I> GetZipDocumentUseCase<I> {
    pub fn new(repository: I) -> Self {
        Self { repository }
    }
}

impl<I: DocumentRepository + DocumentFileRepository> GetZipDocumentUseCase<I> {
    #[tracing::instrument(skip_all, err)]
    pub async fn execute(&self, input: GetZipDocumentInput) -> anyhow::Result<Option<GetZipDocumentOutput>> {
        let document_id = input.document_id;

        let Some(document) = self.repository.find_document_by_id(&document_id).await?
        else {
            return Ok(None)
        };

        let document_name = format!("{}.zip", uuid::Uuid::new_v4().as_simple());
        let document_zip = self.make_zip_document(document).await?;

        Ok(Some(GetZipDocumentOutput {
            name: document_name,
            content: document_zip,
        }))
    }

    // 以下helper functions

    #[tracing::instrument(skip_all)]
    async fn make_zip_document(&self, document: Document) -> anyhow::Result<Vec<u8>> {
        const COMPRESSION_LEVEL: u32 = 6;

        let mut ret = Vec::new();

        let mut buf = tokio::io::BufWriter::new(&mut ret);
        let mut zipper = async_deflate_zip::ZipWriter::new(&mut buf)
            .with_level(async_deflate_zip::Compression::new(COMPRESSION_LEVEL));

        for document_file in document.files() {
            let file_name = get_file_name(document_file)?;

            let entry_option = async_deflate_zip::EntryOptions {
                mtime: std::time::SystemTime::now(),
                permissions: None,
                uid_gid: None,
                comment: None,
            };
            let mut entry = zipper.append_file(&file_name, entry_option).await?;

            let file_content = self.repository.get_document_file_content(document_file).await?;

            entry.write_all(&file_content).await?;
            entry.close().await?;
        }

        zipper.finalize().await?;
        buf.flush().await?;

        Ok(ret)
    }
}

// 以下helper functions

#[tracing::instrument(skip_all)]
fn get_file_name(document_file: &DocumentFile) ->anyhow::Result<String> {
    let with_extension = document_file
        .path()
        .with_added_extension(document_file.ty().to_string());

    let Some(file_name) = with_extension.file_name()
    else {
        tracing::error!("document file name terminates in '..'");
        return Err(anyhow::anyhow!("document file name terminates in '..'"));
    };

    let Some(file_name) = file_name.to_str()
    else {
        tracing::error!("document file name contains invalid UTF-8 chars");
        return Err(anyhow::anyhow!("document file name contains invalid UTF-8 chars"));
    };

    Ok(file_name.to_string())
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uuid_simple() {
        let uuid = uuid::Uuid::from_u128(0x1234567890abcdef1234567890abcdef_u128);
        assert_eq!(&uuid.as_simple().to_string(), &"1234567890abcdef1234567890abcdef");
    }
}
