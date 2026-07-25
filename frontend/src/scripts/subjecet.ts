import { postSubject, type SubjectCreate } from "./api/client";
import "./components/major-select.ts";

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
const submitButton = required<HTMLButtonElement>("#registerbtn");
const statusText = required<HTMLParagraphElement>("#thank");

/**
 * フォームから科目情報を組み立てる
 * @throws 入力内容が不正な場合
 */
function buildSubject(): SubjectCreate {
    const formdata = new FormData(form);

    const faculty = formdata.get("faculty");
    const major = formdata.get("major");
    const grade = Number(formdata.get("grade"));
    const term = Number(formdata.get("term"));
    const name = formdata.get("name");
    const course_code = formdata.get("course_code");

    if (typeof faculty !== "string" || faculty === "") {
        throw new Error("学部が選択されていません。");
    }

    if (typeof major !== "string" || major === "") {
        throw new Error("専攻が選択されていません。");
    }

    if (Number.isInteger(grade) === false || grade < 1 || grade > 9) {
        throw new Error("学年の値が不正です。学年は1～9の整数で選択してください。");
    }

    if (Number.isInteger(term) === false || term < 1 || term > 4) {
        throw new Error("学期の値が不正です。学期は1～4の整数で選択してください。");
    }

    if (typeof name !== "string" || name === "") {
        throw new Error("科目名を入力してください。");
    }

    if (typeof course_code !== "string" || course_code === "") {
        throw new Error("講義番号を入力してください。");
    }

    return {
        faculty,
        major,
        grade,
        term,
        name,
        course_code,
    };
}

/** 送信ボタン(type="submit")によるフォーム送信を処理する */
form.addEventListener("submit", async (event) => {
    // 既定のページ再読み込みを防ぐ
    event.preventDefault();

    let subject: SubjectCreate;

    try {
        subject = buildSubject();
    } catch (e) {
        console.error("入力内容が不正", e);
        statusText.textContent = e instanceof Error ? e.message : "入力内容を確認してください";
        return;
    }

    submitButton.disabled = true;
    submitButton.textContent = "登録中...";

    try {
        await postSubject(subject);

        statusText.textContent = "科目を登録しました。ご協力ありがとうございました。";
        submitButton.hidden = true;
    } catch (e) {
        console.error("科目登録に失敗", e);
        statusText.textContent = "登録に失敗しました。時間をおいて再試行してください。";

        submitButton.disabled = false;
        submitButton.textContent = "登録";
    }
});
