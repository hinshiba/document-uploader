import { postDocuments, type DocumentMetadata } from "./api/client";
import "./components/major-select.ts";
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
}

/**
 * フォームからメタデータを組み立てる
 * @throws faculty, major が未選択の場合
 * @remarks
 * TODO: 現状フォームは faculty, major しか持たないため部分的な値を返す．
 */
function buildMetadata(): DocumentMetadata {
    const formdata = new FormData(form);
    const faculty = formdata.get("faculty");
    const major = formdata.get("major");
    if (typeof faculty !== "string" || faculty === "") {
        throw new Error("学部が選択されていません");
    }
    if (typeof major !== "string" || major === "") {
        throw new Error("専攻が選択されていません");
    }
    return { faculty, major } as DocumentMetadata;
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
    console.log("送信開始", files, metadata);
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

// major-select の facultyId を subject-select の facultyId に反映する
majorSelect?.addEventListener("selection-change", (e) => {
    const event = e as CustomEvent<SelectionChangeDetail>;

    if (subjectSelect) {
        subjectSelect.facultyId = event.detail.facultyId;
    }
});
