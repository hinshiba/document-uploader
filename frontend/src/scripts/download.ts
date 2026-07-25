import "./components/major-select";
import "./components/subject-select";

import { searchDocuments, downloadDocument } from "./api/client";
import type { SelectionChangeDetail } from "./components/major-select";
import type { SubjectSelect } from "./components/subject-select";

const form = document.querySelector<HTMLFormElement>("#search-form");
const majorSelect = document.querySelector("major-select");
const subjectSelect = document.querySelector<SubjectSelect>("subject-select");
const resultList = document.querySelector<HTMLUListElement>("#result-list");

/**これ以降のコードの型を絞り込むため*/
if (!form || !majorSelect || !subjectSelect || !resultList) {
    throw new Error("必要なHTML要素が見つかりません");
}

/**
 * major-select の facultyIdとmajorId を subject-select の facultyIdとmajorId に反映する
 */
majorSelect.addEventListener("selection-change", (event) => {
    const detail = (event as CustomEvent<SelectionChangeDetail>).detail;

    subjectSelect.facultyId = detail.facultyId;
    subjectSelect.majorId = detail.majorId;
});
/**
 * 検索ボタン
 */
let activeSearchId = 0;
form.addEventListener("submit", async (event) => {
    // 一時的に動作を止めて通信の安全性を高める
    event.preventDefault();

    const searchId = ++activeSearchId;

    const formData = new FormData(form);
    const status = document.querySelector("#status");
    if (!status) {
        throw new Error("statusが見つかりません");
    }

    const subject = formData.get("subject");
    if (typeof subject !== "string" || subject === "") {
        status.textContent = "科目を選択してください。";
        return;
    }
    const yearValue = formData.get("year");
    const teacher = formData.get("teacher") as string;
    const examtype = formData.get("examtype") as string;
    const year = typeof yearValue === "string" && yearValue !== "" ? Number(yearValue) : undefined;

    const isanswer =
        formData.get("isanswer") === null ? undefined : formData.get("isanswer") === "true";

    // 検索のたびに前回検索して表示されたものを削除
    resultList.replaceChildren();
    status.textContent = "検索中...";

    try {
        const documents = await searchDocuments(
            subject,
            year,
            typeof teacher === "string" && teacher !== "" ? teacher : undefined,
            typeof examtype === "string" && examtype !== "" ? examtype : undefined,
            isanswer,
        );

        if (searchId !== activeSearchId) {
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
            button.textContent = `「downloadする」${metadata.year}年度 ${metadata.teacher}`;
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
                    li.textContent = "ダウンロードに失敗しました";
                }
            });

            li.append(button);
            resultList.appendChild(li);
            status.textContent = "";
        }
    } catch (error) {
        if (searchId !== activeSearchId) {
            return;
        }
        console.error(error);

        const li = document.createElement("li");
        status.textContent = "検索に失敗しました";
        resultList.append(li);
    }
});
