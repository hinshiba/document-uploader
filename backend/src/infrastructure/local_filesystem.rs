use std::path::PathBuf;

use anyhow::anyhow;
use tokio::io::{AsyncWriteExt, BufWriter};

use crate::{
    domain::document::{DocumentFile, DocumentFileType},
    usecase::repository::DocumentFileRepository,
};

/// ローカルファイルシステムにドキュメントの実体を保存するためのリポジトリ実装
#[derive(Debug)]
pub struct LocalFileSystemRepository {
    /// 保存のルートとなるディレクトリ
    save_dir: PathBuf,
}

impl LocalFileSystemRepository {
    /// `save_dir`をルートとするドキュメントの実体を保存するリポジトリを作成する
    ///
    /// # Errors
    ///
    /// 次のような場合はエラーとなる
    /// - `save_dir`が`fs::create_dir_all`で作成できない
    /// - 同名のファイルが存在する
    pub fn new(save_dir: PathBuf) -> anyhow::Result<Self> {
        std::fs::create_dir_all(&save_dir)?;

        if !save_dir.is_dir() {
            return Err(anyhow!(format!(
                "{save_dir:?} is not dir. And same name file exists."
            )));
        }

        Ok(Self {
            save_dir: std::fs::canonicalize(save_dir)?,
        })
    }
}

impl DocumentFileRepository for LocalFileSystemRepository {
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
        let repository = LocalFileSystemRepository::new(save_dir.clone()).unwrap();
        assert!(save_dir.is_dir());

        // storeのテスト
        let document_file = repository
            .store_document_file(b"hello".to_vec(), DocumentFileType::Pdf)
            .await
            .unwrap();

        assert_eq!(document_file.ty(), &DocumentFileType::Pdf);
        assert_eq!(
            document_file.path().parent(),
            Some(std::fs::canonicalize(&save_dir).unwrap().as_path())
        );
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
