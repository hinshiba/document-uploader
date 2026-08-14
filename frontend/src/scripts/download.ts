import "./components/major-select";
import "./components/subject-select";
import "./components/proc-message";
import { toExamType, toSubjectId, toTeacher, toYear } from "./api/constraints";
import type { DocumentSearchResult } from "./api/client";
import { searchDocuments, downloadDocument } from "./api/client";
import type { SubjectSelect } from "./components/subject-select";
import type { MajorSelect } from "./components/major-select";
import type { ProcMessage } from "./components/proc-message";
import { required } from "./dom";
import { ValidationError } from "./error";
import { log } from "./logging";

const form = required<HTMLFormElement>("#search-form");
const majorSelect = required<MajorSelect>("major-select");
const subjectSelect = required<SubjectSelect>("subject-select");
const resultList = required<HTMLUListElement>("#download-result");
const procMessage = required<ProcMessage>("#proc-message");

/**
 * 最新の検索リクエストを識別するID
 */
let activeSearchId = 0;

// major-select の facultyIdとmajorId を subject-select の facultyIdとmajorId に反映する
majorSelect.addEventListener("major-select-change", (event) => {
    const detail = event.detail;

    subjectSelect.facultyId = detail.facultyId;
    subjectSelect.majorId = detail.majorId;
});

/**
 * 検索結果一覧を画面に描画する
 *
 * @param docs - APIから取得したドキュメント検索結果
 *
 * 検索結果が0件の場合は「検索結果はありません」を表示する．
 * 結果がある場合は各ドキュメントをリスト要素として生成する．
 */
function renderResults(docs: DocumentSearchResult[]) {
    resultList.replaceChildren();

    if (docs.length === 0) {
        const li = document.createElement("li");
        li.textContent = "検索結果はありません";
        resultList.append(li);
        return;
    }

    for (const doc of docs) {
        resultList.append(createResultItem(doc));
    }
}

/**
 * 1件分の検索結果表示用HTML要素を生成する
 *
 * @param doc - 表示対象のドキュメント情報
 * @returns 検索結果1件分のli要素
 *
 * 年度・担当教員・試験種別・解答有無を表示し，
 * ダウンロードボタンを押すと対象ファイルを取得する．
 */
function createResultItem(doc: DocumentSearchResult) {
    const metadata = doc.metadata;

    const li = document.createElement("li");
    const button = document.createElement("button");

    button.type = "button";
    button.classList.add("download-button");
    button.textContent = "downloadする";

    li.append(
        `${metadata.year}年度 ` +
            `${metadata.teacher} ` +
            `${metadata.examtype}` +
            (metadata.isanswer ? "（解答）" : ""),
    );

    button.addEventListener("click", () => {
        void download(doc.id);
    });

    li.append(button);

    return li;
}

/**
 * 指定されたドキュメントをダウンロードする
 *
 * @param id - ダウンロード対象ドキュメントのID
 *
 * APIからZIPファイルを取得し，
 * Blob URLを生成してブラウザのダウンロード処理を実行する．
 */
async function download(id: string) {
    procMessage.error = undefined;

    const res = await downloadDocument(id);
    if (!res.ok) {
        procMessage.error = res.error;
        return;
    }

    const file = res.value;
    const url = URL.createObjectURL(file.blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = file.filename;
    a.click();
    // ブラウザがダウンロードを開始する前に失効しないようにする
    setTimeout(() => URL.revokeObjectURL(url), 0);

    log.download.info("download succeeded", { id, filename: file.filename });
}

/**
 * 検索フォーム送信時の処理
 *
 * FormDataから検索条件を取得し，
 * 型変換後にAPIへ検索リクエストを送信する．
 *
 * activeSearchIdを利用して，
 * 古い検索結果が新しい検索結果を上書きしないよう制御する．
 */
form.addEventListener("submit", async (event) => {
    event.preventDefault();

    const currentSearchId = ++activeSearchId;

    // 前回の結果表示を消す
    procMessage.status = "";
    procMessage.error = undefined;
    resultList.replaceChildren();

    const formData = new FormData(form);

    // 教科以外は絞り込みの任意項目なので検証しない
    const subject = toSubjectId(formData.get("subject"));
    if (subject === undefined) {
        const error = new ValidationError("教科が選択されていません．");
        log.download.info("invalid search condition", { message: error.message });
        procMessage.error = error;
        return;
    }

    const year = toYear(formData.get("year"));
    const teacher = toTeacher(formData.get("teacher"));
    const examtype = toExamType(formData.get("examtype"));
    const isanswer = formData.has("isanswer");

    procMessage.status = "検索中...";

    const res = await searchDocuments(subject, year, teacher, examtype, isanswer);

    // 古い検索の結果は捨てる
    if (currentSearchId !== activeSearchId) return;

    procMessage.status = "";

    if (!res.ok) {
        procMessage.error = res.error;
        return;
    }

    log.download.info("search succeeded", { count: res.value.length });
    renderResults(res.value);
});
