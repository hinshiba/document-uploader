-- docs/db/学部専攻_2026-08-11.md に基づき学部・学科の初期データを投入する

WITH inserted_faculties AS (
    INSERT INTO faculties (id, name) VALUES
        (gen_random_uuid(), '教養教育'),
        (gen_random_uuid(), '文学部'),
        (gen_random_uuid(), '教育学部'),
        (gen_random_uuid(), '法学部'),
        (gen_random_uuid(), '経済学部'),
        (gen_random_uuid(), '理学部'),
        (gen_random_uuid(), '医学部'),
        (gen_random_uuid(), '歯学部'),
        (gen_random_uuid(), '薬学部'),
        (gen_random_uuid(), '工学部'),
        (gen_random_uuid(), '農学部'),
        (gen_random_uuid(), 'GDP')
    RETURNING id, name
)
INSERT INTO majors (id, name, faculty_id)
SELECT gen_random_uuid(), v.major_name, f.id
FROM inserted_faculties AS f
INNER JOIN (VALUES
    ('教養教育', '共通'),
    ('文学部', '人文学科'),
    ('教育学部', '教員養成'),
    ('教育学部', '養護教諭養成'),
    ('法学部', '法学科'),
    ('経済学部', '経済学科'),
    ('理学部', '共通'),
    ('理学部', '数学科'),
    ('理学部', '物理学科'),
    ('理学部', '化学科'),
    ('理学部', '生物学科'),
    ('理学部', '地球科学科'),
    ('医学部', '医学科'),
    ('医学部', '保健-看護'),
    ('医学部', '保健-放射線'),
    ('医学部', '保健-検査技術'),
    ('歯学部', '歯学科'),
    ('薬学部', '薬学科'),
    ('薬学部', '創薬科学科'),
    ('工学部', '共通'),
    ('工学部', '機械システム共通'),
    ('工学部', '機械システム-機械'),
    ('工学部', '機械システム-知能'),
    ('工学部', '環境社会共通'),
    ('工学部', '環境社会-都市'),
    ('工学部', '環境社会-環境'),
    ('工学部', '情電数共通'),
    ('工学部', '情電数-IT'),
    ('工学部', '情電数-NE'),
    ('工学部', '情電数-EE'),
    ('工学部', '情電数-DS'),
    ('工学部', '化学生命共通'),
    ('工学部', '化学生命-応用化学'),
    ('工学部', '化学生命-生命工学'),
    ('農学部', '農学科'),
    ('GDP', 'GDP')
) AS v(faculty_name, major_name) ON f.name = v.faculty_name;
