import "./components/major-select";
import "./components/subject-select";

import { searchDocuments, downloadDocument } from "./api/client";
import type { SelectionChangeDetail } from "./components/major-select";
import type { SubjectSelect } from "./components/subject-select";

const form = document.querySelector<HTMLFormElement>("#search-form");
const majorSelect = document.querySelector("major-select");
const subjectSelect = document.querySelector<SubjectSelect>("subject-select");
const resultList = document.querySelector<HTMLUListElement>("#result-list");
const status = document.querySelector("#status");
/**
 * 最新の検索リクエストを識別するID
 */
let activeSearchId = 0;
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

form.addEventListener("submit", async (event) => {
    // スクリプト側で送信するので既定の送信動作を無効化
    event.preventDefault();

    const currentSearchId = ++activeSearchId;

    const formData = new FormData(form);

    if (!status) {
        throw new Error("statusが見つかりません");
    }

    const subject = formData.get("subject");
    if (typeof subject !== "string" || subject === "") {
        status.textContent = "科目を選択してください。";
        return;
    }
    const yearValue = formData.get("year");
    let year: number | undefined;
    if (typeof yearValue === "string" && yearValue !== "") {
        const parsedYear = Number(yearValue);

        if (
            Number.isFinite(parsedYear) &&
            Number.isInteger(parsedYear) &&
            parsedYear >= 1900 &&
            parsedYear <= 2100
        ) {
            year = parsedYear;
        } else {
            status.textContent = "年度が正しくありません。";
            return;
        }
    }

    const teacherValue = formData.get("teacher") as string;
    const teacher = typeof teacherValue === "string" ? teacherValue : undefined;
    const examtypeValue = formData.get("examtype") as string;
    const examtype = typeof examtypeValue === "string" ? teacherValue : undefined;
    const isanswer =
        formData.get("isanswer") === null ? undefined : formData.get("isanswer") === "true";
    const errorMessage = document.createElement("span");

    // 検索のたびに前回検索して表示されたものを削除
    resultList.replaceChildren();
    status.textContent = "検索中...";

    try {
        const documents = await searchDocuments(subject, year, teacher, examtype, isanswer);

        if (currentSearchId !== activeSearchId) {
            return;
        }

        if (documents.length === 0) {
            status.textContent = "";
            const li = document.createElement("li");
            li.textContent = "検索結果はありません";
            resultList.append(li);
            return;
        }

        for (const result of documents) {
            const metadata = result.metadata;
            const li = document.createElement("li");
            const button = document.createElement("button");
            button.type = "button";
            button.textContent = `「download」${metadata.year}年度 ${metadata.teacher}`;
            button.classList.add("download-button");

            li.append(
                `${metadata.year}年度 ` +
                    `${metadata.teacher} ` +
                    `${metadata.examtype}` +
                    (metadata.isanswer ? "（解答）" : ""),
            );

            button.addEventListener("click", async () => {
                try {
                    const file = await downloadDocument(result.id);

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
            });

            li.append(button);
            resultList.appendChild(li);
            status.textContent = "";
        }
    } catch (error) {
        if (currentSearchId !== activeSearchId) {
            return;
        }
        console.error(error);

        const li = document.createElement("li");
        status.textContent = "検索に失敗しました";
        resultList.append(li);
    }
});
