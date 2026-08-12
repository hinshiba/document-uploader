import { postDocuments } from "./api/client";
import {
    GRADE_MAX,
    GRADE_MIN,
    NUM_MIN,
    TERM_MAX,
    TERM_MIN,
    YEAR_MIN,
    toExamType,
    toFacultyId,
    toGrade,
    toMajorId,
    toNum,
    toSubjectId,
    toTeacher,
    toTerm,
    toYear,
    type DocumentMetadata,
} from "./api/constraints";
import "./components/major-select.ts";
import "./components/subject-select.ts";
import "./components/proc-message.ts";
import { SubjectSelect } from "./components/subject-select";
import { MajorSelect } from "./components/major-select";
import { ProcMessage } from "./components/proc-message";
import { required } from "./dom.ts";
import { err, ok, ValidationError, type Result } from "./error.ts";
import { log } from "./logging.ts";

const form = required<HTMLFormElement>("form");
const fileInput = required<HTMLInputElement>("#file");
const dropArea = required<HTMLDivElement>("#drop-area");
const fileList = required<HTMLUListElement>("#makelist");
const dropMessage = required<HTMLParagraphElement>("#message");
const submitButton = required<HTMLButtonElement>("#uploadbtn");
const procMessage = required<ProcMessage>("#proc-message");
const majorSelect = required<MajorSelect>("major-select");
const subjectSelect = required<SubjectSelect>("subject-select");

// ドラッグ中はデフォルト動作を抑止し，ドロップを許可する
dropArea.addEventListener("dragover", (event) => {
    event.preventDefault();
});

// ドロップされたファイルを input に反映して一覧表示する
dropArea.addEventListener("drop", (event) => {
    event.preventDefault();
    const files = event.dataTransfer?.files;
    if (!files || files.length === 0) return;

    // 送信時に読み出せるよう input へ代入する
    fileInput.files = files;
    renderFileList(files);
});

// ファイル選択ダイアログでの変更を一覧に反映する
fileInput.addEventListener("change", () => {
    if (fileInput.files) renderFileList(fileInput.files);
});

// major-select の facultyIdとmajorId を subject-select の facultyIdとmajorId に反映する
majorSelect.addEventListener("major-select-change", (event) => {
    subjectSelect.facultyId = event.detail.facultyId;
    subjectSelect.majorId = event.detail.majorId;
});

/** 選択中のファイル一覧を画面に描画する */
function renderFileList(files: FileList): void {
    log.upload.debug("files selected", { count: files.length });

    // 選択のたびに作り直し，ドロップと選択の二重表示を防ぐ
    fileList.replaceChildren();
    for (const file of files) {
        const li = document.createElement("li");
        li.textContent = file.name;
        fileList.appendChild(li);
    }
    // ファイルがあれば案内文を隠す
    dropMessage.hidden = 0 < files.length;
}

// 送信ボタンによるフォーム送信を処理する
form.addEventListener("submit", async (event) => {
    // 既定のページ再読み込みを防ぐ
    event.preventDefault();

    const files = fileInput.files;
    if (!files || files.length === 0) return;

    // 前回の結果表示を消す
    procMessage.status = "";
    procMessage.error = undefined;

    // 検証は送信前に済ませ，入力不備と通信失敗でメッセージを分ける
    const metadata = buildMetadata();
    if (!metadata.ok) {
        log.upload.info("invalid metadata", { message: metadata.error.message });
        procMessage.error = metadata.error;
        return;
    }

    submitButton.disabled = true;
    procMessage.status = "送信中...";

    const res = await postDocuments([...files], metadata.value);
    if (!res.ok) {
        procMessage.status = "";
        procMessage.error = res.error;
        submitButton.disabled = false;
        return;
    }

    log.upload.info("upload succeeded", { count: files.length });

    // 成功時はフォームを初期化して謝辞を表示する
    fileInput.value = "";
    fileList.replaceChildren();
    procMessage.status = "送信完了！！協力ありがとうございました";
    submitButton.hidden = true;
    fileInput.hidden = true;
});

/**
 * フォームからメタデータを組み立てる
 * @returns 検証済みのメタデータ．不備があれば最初に見つかったValidationError
 */
function buildMetadata(): Result<DocumentMetadata, ValidationError> {
    const formdata = new FormData(form);

    const faculty = toFacultyId(formdata.get("faculty"));
    if (faculty === undefined) return err(new ValidationError("学部が選択されていません．"));

    const major = toMajorId(formdata.get("major"));
    if (major === undefined) return err(new ValidationError("専攻が選択されていません．"));

    const year = toYear(formdata.get("year"));
    if (year === undefined)
        return err(
            new ValidationError(
                `年度の値が不正です．年度は${YEAR_MIN}年以降の整数で入力してください．`,
            ),
        );

    const term = toTerm(formdata.get("term"));
    if (term === undefined)
        return err(
            new ValidationError(
                `学期の値が不正です．学期は${TERM_MIN}～${TERM_MAX}の整数で選択してください．`,
            ),
        );

    const grade = toGrade(formdata.get("grade"));
    if (grade === undefined)
        return err(
            new ValidationError(
                `学年の値が不正です．学年は${GRADE_MIN}～${GRADE_MAX}の整数で選択してください．`,
            ),
        );

    const subject = toSubjectId(formdata.get("subject"));
    if (subject === undefined) return err(new ValidationError("科目が選択されていません．"));

    const teacher = toTeacher(formdata.get("teacher"));
    if (teacher === undefined) return err(new ValidationError("担当教員名が入力されていません．"));

    const examtype = toExamType(formdata.get("examtype"));
    if (examtype === undefined) return err(new ValidationError("試験種別が選択されていません．"));

    const num = toNum(formdata.get("num"));
    if (num === undefined)
        return err(new ValidationError(`テストの回数は${NUM_MIN}以上の整数で入力してください．`));

    return ok({
        faculty,
        major,
        year,
        term,
        grade,
        subject,
        teacher,
        examtype,
        isanswer: formdata.has("isanswer"),
        num,
    });
}
