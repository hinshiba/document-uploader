use std::path::PathBuf;

use tokio::io::{AsyncWriteExt, BufWriter};

use crate::{
    domain::document::{DocumentFile, DocumentFileType},
    usecase::repository::DocumentFileRepository,
};

/// ローカルファイルシステムにドキュメントの実体を保存するためのリポジトリ実装
#[derive(Debug)]
pub struct LocalFileSystem {
    /// 保存のルートとなるディレクトリ
    save_dir: PathBuf,
}

impl LocalFileSystem {
    /// `save_dir`をルートとするドキュメントの実体を保存するリポジトリを作成する
    ///
    /// # Errors
    ///
    /// `save_dir`が`fs::create_dir_all`で作成できない場合はエラーとなる
    pub fn new(save_dir: PathBuf) -> std::io::Result<Self> {
        if !save_dir.exists() {
            // すでに存在する場合はエラー扱いなので事前検証
            std::fs::create_dir_all(&save_dir)?;
        }

        Ok(Self {
            save_dir: std::fs::canonicalize(save_dir)?,
        })
    }
}

impl DocumentFileRepository for LocalFileSystem {
    #[tracing::instrument(skip_all, ret(level = "info"), err)]
    async fn store_document_file(
        &self,
        content: Vec<u8>,
        file_type: DocumentFileType,
    ) -> anyhow::Result<DocumentFile> {
        let file_name = uuid::Uuid::new_v4().to_string();
        let file_path = self.save_dir.join(file_name);

        let mut buffer = BufWriter::new(tokio::fs::File::create_new(&file_path).await?);
        buffer.write_all(&content).await?;
        buffer.flush().await?;

        Ok(DocumentFile::new(file_type, file_path))
    }

    #[tracing::instrument(skip(self), err)]
    async fn get_document_file_content(
        &self,
        document_file: &DocumentFile,
    ) -> anyhow::Result<Vec<u8>> {
        let content = tokio::fs::read(document_file.path()).await?;
        Ok(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// ファイルが保存可能で，適切な場所にあるか検証する．
    /// そのファイルを取得し，同じ内容を復元できるか検証する．
    #[tokio::test]
    async fn store_and_get_document_file() {
        // 一時ファイルの準備
        let save_dir = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
        let repository = LocalFileSystem::new(save_dir.clone()).unwrap();
        assert!(save_dir.is_dir());

        // storeのテスト
        let document_file = repository
            .store_document_file(b"hello".to_vec(), DocumentFileType::Pdf)
            .await
            .unwrap();

        assert_eq!(document_file.ty(), &DocumentFileType::Pdf);
        assert_eq!(document_file.path().parent(), Some(save_dir.as_path()));
        assert!(document_file.path().is_file());

        // getのテスト
        let content = repository
            .get_document_file_content(&document_file)
            .await
            .unwrap();
        assert_eq!(content, b"hello");

        // 後始末
        fs::remove_dir_all(&save_dir).unwrap();
    }
}
