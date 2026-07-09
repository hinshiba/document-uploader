use tokio::io::AsyncWriteExt;

use crate::domain::{
    Grade,
    Id,
    Term,
    document::{
        Document,
        DocumentFile,
        DocumentFileType,
    },
    faculty::Faculty,
    subject::Subject,
    major::Major,
};

use crate::usecase::repository::{
    DocumentRepository,
    DocumentFileRepository,
    FacultyRepository,
    SubjectRepository,
};

/// 現行のOpenAPIドキュメントの`example`に従うRepository
#[derive(Debug)]
pub struct ExampleRepository {
    documents: std::sync::Mutex<Vec<Document>>,
    faculties: Vec<Faculty>,
    subjects: Vec<Subject>,
    save_dir: std::path::PathBuf,
}

impl ExampleRepository {
    pub fn new(save_dir: std::path::PathBuf) -> std::io::Result<Self> {
        if !save_dir.exists() {
            std::fs::create_dir_all(&save_dir)?;
        }

        Ok( Self {
            documents: std::sync::Mutex::new(Vec::new()),
            faculties: Self::example_faculties(),
            subjects: Self::example_subjects(),
            save_dir,
        } )
    }

    // 以下helper functions

    fn example_faculties() -> Vec<Faculty> {
        vec![
            Faculty::new(
                Id::new(uuid::uuid!("f47ac10b-58cc-4372-a567-0e02b2c3d479")),
                "工学部".to_owned(),
                vec![
                    Major::new(
                        Id::new(uuid::uuid!("550e8400-e29b-41d4-a716-446655440000")),
                        "情電数理系/情報工学コース".to_owned(),
                        Id::new(uuid::uuid!("f47ac10b-58cc-4372-a567-0e02b2c3d479")),
                    ),
                    Major::new(
                        Id::new(uuid::uuid!("6ba7b810-9dad-11d1-80b4-00c04fd430c8")),
                        "情電数理系/ネットワーク工学コース".to_owned(),
                        Id::new(uuid::uuid!("f47ac10b-58cc-4372-a567-0e02b2c3d479")),
                    ),
                ]
            ),
            Faculty::new(
                Id::new(uuid::uuid!("6ba7b812-9dad-11d1-80b4-00c04fd430c8")),
                "理学部".to_owned(),
                vec![
                    Major::new(
                        Id::new(uuid::uuid!("6ba7b813-9dad-11d1-80b4-00c04fd430c8")),
                        "数学科".to_owned(),
                        Id::new(uuid::uuid!("6ba7b812-9dad-11d1-80b4-00c04fd430c8")),
                    ),
                    Major::new(
                        Id::new(uuid::uuid!("6ba7b814-9dad-11d1-80b4-00c04fd430c8")),
                        "物理学科".to_owned(),
                        Id::new(uuid::uuid!("6ba7b812-9dad-11d1-80b4-00c04fd430c8")),
                    ),
                ]
            ),
        ]
    }

    fn example_subjects() -> Vec<Subject> {
        vec![
            Subject::new(
                Id::new(uuid::uuid!("9b2e4c6a-1f3d-4e5b-8a7c-0d1e2f3a4b5c")),
                "線形代数".to_owned(),
                Id::new(uuid::uuid!("f47ac10b-58cc-4372-a567-0e02b2c3d479")),
                Id::new(uuid::uuid!("550e8400-e29b-41d4-a716-446655440000")),
                Grade::new(1).unwrap(),
                Term::new(1).unwrap(),
            ),
            Subject::new(
                Id::new(uuid::uuid!("a1b2c3d4-e5f6-4a5b-9c8d-7e6f5a4b3c2d")),
                "アルゴリズムと計算量".to_owned(),
                Id::new(uuid::uuid!("f47ac10b-58cc-4372-a567-0e02b2c3d479")),
                Id::new(uuid::uuid!("550e8400-e29b-41d4-a716-446655440000")),
                Grade::new(2).unwrap(),
                Term::new(2).unwrap(),
            )
        ]
    }

    fn clone_faculties(faculties: &[Faculty]) -> Vec<Faculty> {
        faculties.iter().map(Self::clone_faculty).collect()
    }
    fn clone_subjects(subjects: &[Subject]) -> Vec<Subject> {
        subjects.iter().map(Self::clone_subject).collect()
    }
    fn clone_majors(majors: &[Major]) -> Vec<Major> {
        majors.iter().map(Self::clone_major).collect()
    }

    fn clone_faculty(faculty: &Faculty) -> Faculty {
        Faculty::new(
            faculty.id().clone(),
            faculty.name().to_owned(),
            Self::clone_majors(faculty.majors()),
        )
    }
    fn clone_subject(subject: &Subject) -> Subject {
        Subject::new(
            subject.id().clone(),
            subject.name().to_owned(),
            subject.faculty_id().clone(),
            subject.major_id().clone(),
            subject.grade().clone(),
            subject.term().clone(),
        )
    }
    fn clone_major(major: &Major) -> Major {
        Major::new(
            major.id().clone(),
            major.name().to_owned(),
            major.faculty_id().clone(),
        )
    }
}

impl DocumentRepository for ExampleRepository {
    #[tracing::instrument(skip(self))]
    async fn store_document(&self, document: Document) -> anyhow::Result<()> {
        let mut inner = self.documents.lock().unwrap();

        inner.push(document);

        tracing::info!("document is successfully stored.");

        Ok(())
    }
}

impl DocumentFileRepository for ExampleRepository {
    #[tracing::instrument(skip(self), ret(level="info"), err)]
    async fn store_document_file(&self, content: Vec<u8>, file_type: DocumentFileType) -> anyhow::Result<DocumentFile> {
        let file_name = uuid::Uuid::new_v4().to_string();
        let file_path = self.save_dir.join(file_name);

        let mut buffer = tokio::io::BufWriter::new(
            tokio::fs::File::create_new(&file_path).await?
        );
        buffer.write_all(&content).await?;
        buffer.flush().await?;

        Ok(DocumentFile::new(
            file_type,
            file_path
        ))
    }
}

impl FacultyRepository for ExampleRepository {
    #[tracing::instrument(skip(self), ret(level="info"))]
    async fn list_faculties(&self) -> anyhow::Result<Vec<Faculty>> {
        let faculties = Self::clone_faculties(&self.faculties);
        Ok(faculties)
    }    
}

impl SubjectRepository for ExampleRepository {
    #[tracing::instrument(skip(self), ret(level="info"))]
    async fn list_subjects(&self) -> anyhow::Result<Vec<Subject>> {
        let subjects = Self::clone_subjects(&self.subjects);
        Ok(subjects)
    }
}
