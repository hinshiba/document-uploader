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
    UpdateSubjectContent,
    SearchSubjectOption,
    DocumentRepository,
    DocumentFileRepository,
    FacultyRepository,
    SubjectRepository,
};

macro_rules! construct_faculty {
    ( $fname:ident : [ $($sname:expr),* ] ) => {
        {
            let f_id: Id<Faculty> = Id::new(::uuid::Uuid::new_v4());
            Faculty::new(
                f_id.clone(),
                String::from(stringify!($fname)),
                vec![
                    $( Major::new(
                        Id::new(::uuid::Uuid::new_v4()),
                        String::from(stringify!($sname)),
                        f_id.clone(),
                    )),*
                ]
            )
        }
    };

    ( $( $fname:ident : [ $($sname:expr),* ] ),+ ) => {
        vec![
            $(
                {
                    let f_id: Id<Faculty> = Id::new(::uuid::Uuid::new_v4());
                    Faculty::new(
                        f_id.clone(),
                        String::from(stringify!($fname)),
                        vec![
                            $( Major::new(
                                Id::new(::uuid::Uuid::new_v4()),
                                String::from(stringify!($sname)),
                                f_id.clone(),
                            )),*
                        ]
                    )
                }
            ),+
        ]
    };
}

/// Repositoryの参考実装
#[derive(Debug)]
pub struct ExampleRepository {
    documents: std::sync::Mutex<Vec<Document>>,
    faculties: Vec<Faculty>,
    subjects: std::sync::Mutex<Vec<Subject>>,
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
            subjects: std::sync::Mutex::new(Self::example_subjects()),
            save_dir: save_dir.canonicalize()?,
        } )
    }

    // 以下helper functions

    fn example_faculties() -> Vec<Faculty> {
        construct_faculty![
            教養教育 : [ 共通 ],
            文学部 : [ 人文学科 ],
            教育学部 : [ 教員養成, 養護教諭養成 ],
            法学部 : [ 法学科 ],
            経済学部 : [ 経済学科 ],
            理学部 : [ 共通, 数学科, 物理学科, 化学科, 生物学科, 地球科学科 ],
            医学部 : [ 医学科, 保健-看護, 保健-放射線, 保健-検査技術 ],
            歯学部 : [ 薬学科, 創薬科学科 ],
            工学部 : [
                共通,
                機械システム共通,
                機械システム-機械,
                機械システム-知能,
                環境社会共通,
                環境社会-都市,
                環境社会-環境,
                情電数学共通,
                情電数学-IT,
                情電数学-NE,
                情電数学-EE,
                情電数学-DS,
                化学生命共通,
                化学生命-応用化学,
                化学生命-生命工学
            ],
            農学部 : [ 農学科 ],
            GDP : [ GDP ]
        ]
    }

    fn example_subjects() -> Vec<Subject> {
        vec![]
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

    fn clone_document(document: &Document) -> Document {
        Document::new(
            document.id().clone(),
            document.metadata().clone(),
            document.files().iter().map(Self::clone_document_file).collect(),
        ).unwrap()
    }
    fn clone_document_file(document_file: &DocumentFile) -> DocumentFile {
        DocumentFile::new(
            document_file.ty().clone(),
            document_file.path().to_owned(),
        )
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
    #[tracing::instrument(skip(self), err)]
    async fn find_document_by_id(&self, document_id: &Id<Document>) -> anyhow::Result<Option<Document>> {
        let inner = self.documents.lock().unwrap();

        let document = inner.iter()
            .find(|&d| d.id() == document_id)
            .map(Self::clone_document);

        Ok(document)
    }

    #[tracing::instrument(skip(self))]
    async fn store_document(&self, document: Document) -> anyhow::Result<()> {
        let mut inner = self.documents.lock().unwrap();

        inner.push(document);

        tracing::info!("document is successfully stored.");

        Ok(())
    }
}

impl DocumentFileRepository for ExampleRepository {
    #[tracing::instrument(skip_all, ret(level="info"), err)]
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

    #[tracing::instrument(skip(self), err)]
    async fn get_document_file_content(&self, document_file: &DocumentFile) -> anyhow::Result<Vec<u8>> {
        let file_path = document_file.path();
        let content = tokio::fs::read(file_path).await?;
        Ok(content)
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
        let subjects = self.subjects.lock().unwrap();
        let subjects = Self::clone_subjects(&subjects);
        Ok(subjects)
    }

    #[tracing::instrument(skip(self), err)]
    async fn search_subjects(&self, option: SearchSubjectOption) -> anyhow::Result<Vec<Subject>> {
        let subjects = self.list_subjects().await?;

        let subjects = subjects
            .into_iter()
            .filter(|subject| {
                   option.subject_id.as_ref().map_or(true, |inner| inner.id() == subject.id().id())
                && option.name.as_ref().map_or(true, |inner| inner == subject.name())
                && option.faculty_id.as_ref().map_or(true, |inner| inner.id() == subject.faculty_id().id())
                && option.major_id.as_ref().map_or(true, |inner| inner.id() == subject.major_id().id())
                && option.grade.map_or(true, |inner| inner.grade() == subject.grade().grade())
                && option.term.map_or(true, |inner| inner.term() == subject.term().term())
            })
            .collect();

        Ok(subjects)
    }

    #[tracing::instrument(skip(self), err)]
    async fn create_subject(&self, subject: Subject) -> anyhow::Result<()> {
        let mut subjects = self.subjects.lock().unwrap();

        if subjects.iter().any(|existing| existing.id() == subject.id()) {
            return Err(anyhow::anyhow!("subject already exists"));
        }

        subjects.push(subject);

        Ok(())
    }

    #[tracing::instrument(skip(self), err, ret(level="info"))]
    async fn update_subject(&self, subject_id: Id<Subject>, content: UpdateSubjectContent) -> anyhow::Result<Subject> {
        let mut subjects = self.subjects.lock().unwrap();

        let Some(subject) = subjects.iter_mut().find(|s| s.id() == &subject_id)
        else {
            return Err(anyhow::anyhow!("subject not found"));
        };

        *subject = Subject::new(
            subject.id().clone(),
            content.name,
            content.faculty_id,
            content.major_id,
            content.grade,
            content.term
        );

        Ok(Self::clone_subject(subject))
    }

    #[tracing::instrument(skip(self), err, ret(level="info"))]
    async fn delete_subject(&self, subject_id: Id<Subject>) -> anyhow::Result<Subject> {
        let mut subjects = self.subjects.lock().unwrap();

        let Some(pos) = subjects.iter().position(|s| s.id() == &subject_id)
        else {
            return Err(anyhow::anyhow!("subject does not exist"));
        };

        Ok(subjects.remove(pos))
    }
}
