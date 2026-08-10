-- Joinを毎回行うのは非効率的なのでViewを作成しておく

CREATE VIEW subject_details AS
SELECT
    s.id,
    s.name,
    m.faculty_id,
    s.major_id,
    s.grade,
    s.term
FROM subjects AS s
INNER JOIN majors AS m ON m.id = s.major_id;
