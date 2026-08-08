import "./components/major-select";
import "./components/subject-select";
import { toExamType, toSubjectId, toTeacher, toYear } from "./api/constraints";
import type { DocumentSearchResult } from "./api/client";
import { searchDocuments, downloadDocument } from "./api/client";
import type { SelectionChangeDetail } from "./components/major-select";
import type { SubjectSelect } from "./components/subject-select";
import type { MajorSelect } from "./components/major-select";

const form = document.querySelector<HTMLFormElement>("#search-form");
const majorSelect = document.querySelector<MajorSelect>("major-select");
const subjectSelect = document.querySelector<SubjectSelect>("subject-select");
const resultList = document.querySelector<HTMLUListElement>("#drop-area");
const status = document.querySelector<HTMLFormElement>("#status");
let activeSearchId = 0;
const errorMessage = document.createElement("span");
/**
 * 最新の検索リクエストを識別するID
 */

// FIXME: requiredによるエラー処理の統一
if (!form || !majorSelect || !subjectSelect || !resultList) {
    throw new Error("必要なHTML要素が見つかりません");
}

/**
 * major-select の facultyIdとmajorId を subject-select の facultyIdとmajorId に反映する
 */
majorSelect.addEventListener("selection-change", (event) => {
    // major-select の facultyIdとmajorId を subject-select の facultyIdとmajorId に反映する
    const detail = (event as CustomEvent<SelectionChangeDetail>).detail;

    subjectSelect.facultyId = detail.facultyId;
    subjectSelect.majorId = detail.majorId;
});

/**
 * 検索結果一覧を画面に描画する
 *
 * @param documents APIから取得したドキュメント検索結果
 * @param listElement 検索結果を表示するHTML要素
 * @param statusElement 検索状態（検索中など）を表示するHTML要素
 *
 * 検索結果が0件の場合は「検索結果はありません」を表示する。
 * 結果がある場合は各ドキュメントをリスト要素として生成する。
 */
function renderResults(
    documents: DocumentSearchResult[],
    ListElement: HTMLElement,
    statusElement: HTMLElement,
) {
    ListElement.replaceChildren();

    if (documents.length === 0) {
        const li = document.createElement("li");
        li.textContent = "検索結果はありません";
        ListElement.append(li);
        statusElement.textContent = "";
        return;
    }

    for (const document of documents) {
        ListElement.append(createResultItem(document));
    }

    statusElement.textContent = "";
}

/**
 * 1件分の検索結果表示用HTML要素を生成する
 *
 * @param doc - 表示対象のドキュメント情報
 * @returns 検索結果1件分のli要素
 *
 * 年度・担当教員・試験種別・解答有無を表示し、
 * ダウンロードボタンを押すと対象ファイルを取得する。
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
 * APIからZIPファイルを取得し、
 * Blob URLを生成してブラウザのダウンロード処理を実行する。
 */
async function download(id: string) {
    try {
        const file = await downloadDocument(id);

        const url = URL.createObjectURL(file.blob);

        const a = document.createElement("a");
        a.href = url;
        a.download = file.filename;
        a.click();

        URL.revokeObjectURL(url);
    } catch (error) {
        console.error(error);
        errorMessage.textContent = " ダウンロードに失敗しました";
    }
}

/**
 * 検索フォーム送信時の処理
 *
 * FormDataから検索条件を取得し、
 * 型変換後にAPIへ検索リクエストを送信する。
 *
 * activeSearchIdを利用して、
 * 古い検索結果が新しい検索結果を上書きしないよう制御する。
 */
form.addEventListener("submit", async (event) => {
    event.preventDefault();

    if (!status) {
        return;
    }

    const currentSearchId = ++activeSearchId;

    errorMessage.textContent = "";
    status.textContent = "検索中...";
    resultList.replaceChildren();

    const formData = new FormData(form);

    const subject = toSubjectId(formData.get("subject"));
    if (!subject) {
        return;
    }
    const year = toYear(formData.get("year"));
    const teacher = toTeacher(formData.get("teacher"));
    const examtype = toExamType(formData.get("examtype"));
    const isanswer = formData.has("isanswer");

    try {
        const documents = await searchDocuments(subject, year, teacher, examtype, isanswer);

        if (currentSearchId !== activeSearchId) {
            return;
        }

        renderResults(documents, resultList, status);
    } catch (error) {
        if (currentSearchId !== activeSearchId) {
            return;
        }

        console.error(error);
        status.textContent = "";
        errorMessage.textContent = "検索に失敗しました";
    }
});
