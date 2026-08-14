import "./proc-message";

import { html, LitElement, type PropertyValues } from "lit";
import { customElement, property, query, state } from "lit/decorators.js";
import { fetchSubjects, type ApiResult } from "../api/client";
import {
    GRADE_MAX,
    TERM_MAX,
    toGrade,
    toSubjectId,
    toTerm,
    type FacultyId,
    type Grade,
    type MajorId,
    type Subject,
    type SubjectId,
    type Term,
} from "../api/constraints";
import { ok } from "../error";

export interface SubjectSelectChangeDetail {
    subjectId: SubjectId | undefined;
    grade: Grade | undefined;
    term: Term | undefined;
}

/* Eventに型を設ける */
declare global {
    interface HTMLElementEventMap {
        "subject-select-change": CustomEvent<SubjectSelectChangeDetail>;
    }
}

/**
 * 学年と学期，教科の連動した選択のコンポーネント
 *
 * 絞り込みに使う学部と専攻は外部から与えられる
 */
@customElement("subject-select")
export class SubjectSelect extends LitElement {
    // formのネイティブ要素としてふるまうために必要
    static formAssociated = true;
    #internals: ElementInternals = this.attachInternals();

    /** 通信の競合状態を防ぐための最新のクエリ番号 */
    #loadId = 0;

    /** 取得した教科またはエラー．読み込み中は`undefined`
     * 学部が未選択のときはAPIを呼ばないため空配列とする */
    @state()
    private response: ApiResult<Subject[]> | undefined = ok([]);

    /** 選択した教科Id．未選択は`undefined` */
    @state()
    private selectedSubjectId: SubjectId | undefined = undefined;

    /** 選択した学年．未選択は`undefined` */
    @state()
    private selectedGrade: Grade | undefined = undefined;

    /** 選択した学期．未選択は`undefined` */
    @state()
    private selectedTerm: Term | undefined = undefined;

    /** 外部（upload.ts）から受け取る現在選択中の学部ID
     * @property にすることで外部から値を設定でき、変更時には updated() が実行される
     * 属性は検証を経ない`string`しか渡せないため`attribute: false`とする */
    @property({ attribute: false })
    facultyId: FacultyId | undefined = undefined;

    /** facultyIdと同様に選択中の専攻Id */
    @property({ attribute: false })
    majorId: MajorId | undefined = undefined;

    /** 検証メッセージの表示先とする`select`．描画前は`null` */
    @query('[data-field="grade"]')
    private gradeSelect!: HTMLSelectElement | null;

    /** gradeSelectと同様に検証メッセージの表示先とする`select` */
    @query('[data-field="term"]')
    private termSelect!: HTMLSelectElement | null;

    /** gradeSelectと同様に検証メッセージの表示先とする`select` */
    @query('[data-field="subject"]')
    private subjectSelect!: HTMLSelectElement | null;

    // lightDom化
    protected override createRenderRoot() {
        return this;
    }

    // 絞り込み条件が変わったら教科を取り直す
    // 取り消しを同じ更新サイクルに反映させるため描画前に行う
    protected override willUpdate(changedProperties: PropertyValues) {
        super.willUpdate(changedProperties);

        const filterKeys = ["facultyId", "majorId", "selectedGrade", "selectedTerm"];
        if (filterKeys.some((key) => changedProperties.has(key))) {
            // 絞り込みの結果消える可能性があるため教科の選択を取り消す
            this.selectedSubjectId = undefined;
            void this.loadSubjects();
        }
    }

    // UIが更新されたらformが取得できる値を更新
    protected override updated(changedProperties: PropertyValues) {
        super.updated(changedProperties);

        this.syncFormValue();

        // willUpdateでの取り消しを反映してから通知するためここで呼び出し
        const selectionKeys = ["selectedSubjectId", "selectedGrade", "selectedTerm"];
        if (selectionKeys.some((key) => changedProperties.has(key))) {
            this.emitChange();
        }
    }

    /** APIから選択された学部IDなどに対応する教科一覧を取得する */
    private async loadSubjects() {
        // 通信するごと１ずつ増やす
        const id = ++this.#loadId;

        // 学部が選択されていない場合はAPIを呼ばない
        if (this.facultyId === undefined) {
            this.response = ok([]);
            return;
        }

        // 読み込み中を表す
        this.response = undefined;

        const response = await fetchSubjects(
            this.facultyId,
            this.majorId,
            this.selectedGrade,
            this.selectedTerm,
        );

        // 通信時のloadIdが一致した場合のみ反映される
        if (id !== this.#loadId) return; // stale response
        this.response = response;
    }

    override render() {
        const subjects = this.response?.ok ? this.response.value : [];
        const error = this.response?.ok === false ? this.response.error : undefined;

        // 添字+1を`option`の値とするため，要素数を上限値と一致させる
        const grades = [
            "1回生",
            "2回生",
            "3回生",
            "4回生",
            "M1",
            "M2",
            "D1",
            "D2",
            "D3",
        ] as const satisfies { length: typeof GRADE_MAX };
        const terms = ["1学期", "2学期", "3学期", "4学期"] as const satisfies {
            length: typeof TERM_MAX;
        };

        const subject_options = subjects.map((s) => html`<option value=${s.id}>${s.name}</option>`);
        const grade_options = grades.map((g, n) => html`<option value=${n + 1}>${g}</option>`);
        const term_options = terms.map((t, n) => html`<option value=${n + 1}>${t}</option>`);

        return html` <proc-message
                .status=${this.response === undefined ? "読み込み中" : ""}
                .error=${error}
            ></proc-message>
            <label>
                学年
                <select
                    data-field="grade"
                    .value=${String(this.selectedGrade ?? "")}
                    @change=${this.onGradeChange}
                >
                    <option value="">--学年--</option>
                    ${grade_options}
                </select>
            </label>
            <label>
                学期
                <select
                    data-field="term"
                    .value=${String(this.selectedTerm ?? "")}
                    @change=${this.onTermChange}
                >
                    <option value="">--学期--</option>
                    ${term_options}
                </select>
            </label>
            <label>
                教科
                <select
                    data-field="subject"
                    .value=${this.selectedSubjectId ?? ""}
                    @change=${this.onSubjectChange}
                >
                    <option value="">教科を選択してください</option>
                    ${subject_options}
                </select>
            </label>`;
    }

    /** formが使える情報を設定する */
    private syncFormValue() {
        const data = new FormData();
        data.set("subject", this.selectedSubjectId ?? "");
        data.set("grade", this.selectedGrade !== undefined ? String(this.selectedGrade) : "");
        data.set("term", this.selectedTerm !== undefined ? String(this.selectedTerm) : "");
        this.#internals.setFormValue(data);

        // 未選択があれば無効とする
        // カスタム要素自身はフォーカスできないため，第3引数で未選択の`select`をメッセージの表示先とする
        if (this.selectedGrade === undefined) {
            this.#internals.setValidity(
                { valueMissing: true },
                "学年を選択してください",
                this.gradeSelect ?? undefined,
            );
        } else if (this.selectedTerm === undefined) {
            this.#internals.setValidity(
                { valueMissing: true },
                "学期を選択してください",
                this.termSelect ?? undefined,
            );
            // 学部が未選択なら教科の選択肢が空なので，教科の未選択として扱う
        } else if (this.facultyId === undefined || this.selectedSubjectId === undefined) {
            this.#internals.setValidity(
                { valueMissing: true },
                "教科を選択してください",
                this.subjectSelect ?? undefined,
            );
        } else {
            this.#internals.setValidity({});
        }
    }

    private onSubjectChange(e: Event) {
        // 未選択の`option`は空文字なので検証で弾かれる
        this.selectedSubjectId = toSubjectId((e.target as HTMLSelectElement).value);
    }

    private onGradeChange(e: Event) {
        this.selectedGrade = toGrade((e.target as HTMLSelectElement).value);
    }

    private onTermChange(e: Event) {
        this.selectedTerm = toTerm((e.target as HTMLSelectElement).value);
    }

    private emitChange() {
        this.dispatchEvent(
            new CustomEvent("subject-select-change", {
                detail: {
                    subjectId: this.selectedSubjectId,
                    grade: this.selectedGrade,
                    term: this.selectedTerm,
                },
                bubbles: true,
            }),
        );
    }
}
