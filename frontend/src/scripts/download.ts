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
form.addEventListener("submit", async (event) => {
    // 一時的に動作を止めて通信の安全性を高める
    event.preventDefault();

    const formData = new FormData(form);

    const faculty = formData.get("faculty") as string;
    const major = formData.get("major") as string;
    const gradeValue = formData.get("grade") as string;
    const termValue = formData.get("term") as string;
    const subjectValue = formData.get("subject") as string;
    const grade = gradeValue === "" ? undefined : Number(gradeValue);
    const term = termValue === "" ? undefined : Number(termValue);
    const subject = subjectValue === "" ? undefined : subjectValue;

    // 検索のたびに前回検索して表示されたものを削除
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

            // マウスカーソルをホバーする時用
            li.style.cursor = "pointer";

            li.addEventListener("click", () => {
                // idでAPIから情報を所得する予定らしいのでidにしている
                downloadDocument(doc.id);
            });

            resultList.append(li);
            console.log("検索成功");
        }
    } catch (error) {
        console.error(error);

        const li = document.createElement("li");
        li.textContent = "検索に失敗しました";
        resultList.append(li);
    }
});
