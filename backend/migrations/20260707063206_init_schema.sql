-- Add migration script here
CREATE TABLE faculties (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL
);

CREATE TABLE majors (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    faculty_id UUID NOT NULL REFERENCES faculties(id)
);

CREATE TABLE subjects (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    major_id UUID NOT NULL REFERENCES majors(id),
    grade BIGINT NOT NULL CHECK (grade BETWEEN 1 AND 9),
    term BIGINT NOT NULL CHECK (term BETWEEN 1 AND 4)
);

CREATE TABLE documents (
    id UUID PRIMARY KEY,
    subject_id UUID NOT NULL REFERENCES subjects(id),
    year BIGINT NOT NULL CHECK (year >= 1949),
    teacher TEXT NOT NULL,
    exam_type BIGINT NOT NULL CHECK (exam_type BETWEEN 0 AND 3),
    is_answer BOOLEAN NOT NULL,
    num BIGINT NOT NULL CHECK (num >= 1)
);

CREATE TABLE document_files (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    file_type TEXT NOT NULL,
    path TEXT NOT NULL
);
