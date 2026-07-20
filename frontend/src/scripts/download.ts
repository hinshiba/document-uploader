import "./components/major-select";
import "./components/subject-select";

import { searchDocuments, downloadDocument } from "./api/client";
import type { SelectionChangeDetail } from "./components/major-select";
import type { SubjectSelect } from "./components/subject-select";

const form = document.querySelector<HTMLFormElement>("#search-form");
const majorSelect = document.querySelector("major-select");
const subjectSelect = document.querySelector<SubjectSelect>("subject-select");
const resultList = document.querySelector<HTMLUListElement>("#result-list");

if (!form || !majorSelect || !subjectSelect || !resultList) {
    throw new Error("必要なHTML要素が見つかりません");
}

/**
 * 学部・専攻が変更されたらsubject-selectへ通知
 */
majorSelect.addEventListener("selection-change", (event) => {
    const detail = (event as CustomEvent<SelectionChangeDetail>).detail;

    subjectSelect.facultyId = detail.facultyId;
    subjectSelect.majorId = detail.majorId;
});
/**
 * 検索ボタン
 */
form.addEventListener("submit", async (event) => {
    event.preventDefault();

    const formData = new FormData(form);

    const faculty = formData.get("faculty") as string;
    const major = formData.get("major") as string;
    const grade = Number(formData.get("grade"));
    const term = Number(formData.get("term"));
    const subject = formData.get("subject") as string;

    resultList.replaceChildren();

    try {
        const documents = await searchDocuments(faculty, major, grade, term, subject);

        if (documents.length === 0) {
            const li = document.createElement("li");
            li.textContent = "検索結果はありません";
            resultList.append(li);
            return;
        }

        for (const doc of documents) {
            const li = document.createElement("li");

            li.textContent = doc.filename;
            li.style.cursor = "pointer";

            li.addEventListener("click", () => {
                downloadDocument(doc.id);
            });

            resultList.append(li);
        }
    } catch (error) {
        console.error(error);

        const li = document.createElement("li");
        li.textContent = "検索に失敗しました";
        resultList.append(li);
    }
});
