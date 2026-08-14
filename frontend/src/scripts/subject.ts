import { postSubject } from "./api/client";
import {
    GRADE_MAX,
    GRADE_MIN,
    TERM_MAX,
    TERM_MIN,
    toFacultyId,
    toGrade,
    toMajorId,
    toRequiredString,
    toTerm,
    type SubjectBase,
} from "./api/constraints";
import "./components/major-select";
import "./components/proc-message";
import type { ProcMessage } from "./components/proc-message";
import { required } from "./dom";
import { err, ok, ValidationError, type Result } from "./error";
import { log } from "./logging";

const form = required<HTMLFormElement>("form");
const submitButton = required<HTMLButtonElement>("#registerbtn");
const procMessage = required<ProcMessage>("#proc-message");

// 送信ボタンによるフォーム送信を処理する
form.addEventListener("submit", async (event) => {
    // 既定のページ再読み込みを防ぐ
    event.preventDefault();

    // 前回の結果表示を消す
    procMessage.status = "";
    procMessage.error = undefined;

    // 検証は送信前に済ませ，入力不備と通信失敗でメッセージを分ける
    const subject = buildSubject();
    if (!subject.ok) {
        log.subject.info("invalid subject", { message: subject.error.message });
        procMessage.error = subject.error;
        return;
    }

    submitButton.disabled = true;
    procMessage.status = "登録中...";

    const res = await postSubject(subject.value);
    if (!res.ok) {
        procMessage.status = "";
        procMessage.error = res.error;
        submitButton.disabled = false;
        return;
    }

    log.subject.info("subject registered", { id: res.value.id });

    // 成功時は再送信できないよう送信ボタンを隠す
    procMessage.status = "科目を登録しました．ご協力ありがとうございました．";
    submitButton.hidden = true;
});

/**
 * フォームから科目情報を組み立てる
 * @returns 検証済みの科目情報．不備があれば最初に見つかったValidationError
 */
function buildSubject(): Result<SubjectBase, ValidationError> {
    const formdata = new FormData(form);

    const faculty = toFacultyId(formdata.get("faculty"));
    if (faculty === undefined) return err(new ValidationError("学部が選択されていません．"));

    const major = toMajorId(formdata.get("major"));
    if (major === undefined) return err(new ValidationError("専攻が選択されていません．"));

    const grade = toGrade(formdata.get("grade"));
    if (grade === undefined)
        return err(
            new ValidationError(
                `学年の値が不正です．学年は${GRADE_MIN}～${GRADE_MAX}の整数で選択してください．`,
            ),
        );

    const term = toTerm(formdata.get("term"));
    if (term === undefined)
        return err(
            new ValidationError(
                `学期の値が不正です．学期は${TERM_MIN}～${TERM_MAX}の整数で選択してください．`,
            ),
        );

    const name = toRequiredString(formdata.get("name"));
    if (name === undefined) return err(new ValidationError("科目名が入力されていません．"));

    const course_code = toRequiredString(formdata.get("course_code"));
    if (course_code === undefined)
        return err(new ValidationError("講義番号が入力されていません．"));

    return ok({
        faculty,
        major,
        grade,
        term,
        name,
        course_code,
    });
}
