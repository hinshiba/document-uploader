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
import { SubjectSelect } from "./components/subject-select";
import type { SelectionChangeDetail } from "./components/major-select";
/**
 * 要素を型付きで取得するヘルパ
 * @param selector セレクタ
 * @returns 見つかった要素
 * @throws 要素が存在しない場合
 */
function required<T extends Element>(selector: string): T {
    const el = document.querySelector<T>(selector);
    if (!el) throw new Error(`Element not found. selector: ${selector}`);
    return el;
}

/**
 * 検証済みの値を取り出す
 * コンストラクタの`undefined`を利用者向けのエラーに変換する
 * @throws 検証を通らなかった場合
 */
function orThrow<T>(value: T | undefined, message: string): T {
    if (value === undefined) throw new Error(message);
    return value;
}

// type="module" のスクリプトは defer 相当で DOM 構築後に実行されるため，
// ここで要素を取得してよい
const form = required<HTMLFormElement>("form");
const fileInput = required<HTMLInputElement>("#file");
const dropArea = required<HTMLDivElement>("#drop-area");
const fileList = required<HTMLUListElement>("#makelist");
const message = required<HTMLParagraphElement>("#message");
const submitButton = required<HTMLButtonElement>("#uploadbtn");
const statusText = required<HTMLParagraphElement>("#thank");
const majorSelect = document.querySelector("major-select");
const subjectSelect = document.querySelector("subject-select") as SubjectSelect | null;

/** 選択中のファイル一覧を画面に描画する */
function renderFileList(files: FileList): void {
    // 選択のたびに作り直し，ドロップと選択の二重表示を防ぐ
    fileList.replaceChildren();
    for (const file of files) {
        const li = document.createElement("li");
        li.textContent = file.name;
        fileList.appendChild(li);
    }
    // ファイルがあれば案内文を隠す
    message.hidden = files.length > 0;
    clearFileButton.hidden = files.length === 0;
}

/**
 * フォームからメタデータを組み立てる
 * @throws 要素が未選択の場合
 */
function buildMetadata(): DocumentMetadata {
    const formdata = new FormData(form);

    return {
        faculty: orThrow(toFacultyId(formdata.get("faculty")), "学部が選択されていません。"),
        major: orThrow(toMajorId(formdata.get("major")), "専攻が選択されていません。"),
        year: orThrow(
            toYear(formdata.get("year")),
            `年度の値が不正です。年度は${YEAR_MIN}年以降の整数で入力してください。`,
        ),
        term: orThrow(
            toTerm(formdata.get("term")),
            `学期の値が不正です。学期は${TERM_MIN}～${TERM_MAX}の整数で選択してください。`,
        ),
        grade: orThrow(
            toGrade(formdata.get("grade")),
            `学年の値が不正です。学年は${GRADE_MIN}～${GRADE_MAX}の整数で選択してください。`,
        ),
        subject: orThrow(toSubjectId(formdata.get("subject")), "科目が選択されていません。"),
        teacher: orThrow(toTeacher(formdata.get("teacher")), "担当教員名が入力されていません。"),
        examtype: orThrow(toExamType(formdata.get("examtype")), "試験種別が選択されていません。"),
        isanswer: formdata.has("isanswer"),
        num: orThrow(
            toNum(formdata.get("num")),
            `テストの回数は${NUM_MIN}以上の整数で入力してください。`,
        ),
    };
}

/** ドラッグ中はデフォルト動作を抑止し，ドロップを許可する */
dropArea.addEventListener("dragover", (event) => {
    event.preventDefault();
});

/** ドロップされたファイルを input に反映して一覧表示する */
dropArea.addEventListener("drop", (event) => {
    event.preventDefault();
    const files = event.dataTransfer?.files;
    if (!files || files.length === 0) return;

    // 送信時に読み出せるよう input へ代入する
    fileInput.files = files;
    renderFileList(files);
});

/** ファイル選択ダイアログでの変更を一覧に反映する */
fileInput.addEventListener("change", () => {
    if (fileInput.files) renderFileList(fileInput.files);
});

/** 送信ボタン(type="submit")によるフォーム送信を処理する */
form.addEventListener("submit", async (event) => {
    // 既定のページ再読み込みを防ぐ
    event.preventDefault();

    const files = fileInput.files;
    if (!files || files.length === 0) return;

    // 検証は送信前に済ませ，入力不備と通信失敗でメッセージを分ける
    let metadata: DocumentMetadata;
    try {
        metadata = buildMetadata();
    } catch (e) {
        console.error("入力内容が不正", e);
        statusText.textContent = e instanceof Error ? e.message : "入力内容を確認してください";
        return;
    }

    submitButton.disabled = true;
    submitButton.textContent = "送信中...";
    try {
        await postDocuments([...files], metadata);

        // 成功時はフォームを初期化して謝辞を表示する
        fileInput.value = "";
        fileList.replaceChildren();
        statusText.textContent = "送信完了！！協力ありがとうございました";
        submitButton.hidden = true;
        fileInput.hidden = true;
    } catch (e) {
        console.error("アップロードに失敗", e);
        statusText.textContent = "送信に失敗しました．時間をおいて再試行してください";
        submitButton.disabled = false;
        submitButton.textContent = "送信";
    }
});

// major-select の facultyIdとmajorId を subject-select の facultyIdとmajorId に反映する
majorSelect?.addEventListener("selection-change", (e) => {
    const event = e as CustomEvent<SelectionChangeDetail>;

    if (subjectSelect) {
        subjectSelect.facultyId = event.detail.facultyId;
        subjectSelect.majorId = event.detail.majorId;
    }
});

const clearFileButton = required<HTMLButtonElement>("#clear-file");

function clearFiles(): void {
    // inputの選択を解除
    fileInput.value = "";

    // 一覧を消す
    fileList.replaceChildren();

    // 「ファイルを選択してください」のような案内を再表示
    message.hidden = false;

    // ボタンを隠す
    clearFileButton.hidden = true;
}

clearFileButton.addEventListener("click", () => {
    clearFiles();
});
