import { html, LitElement, type PropertyValues } from "lit";
import { customElement, property, state } from "lit/decorators.js";
import { fetchSubjects } from "../api/client";
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

enum Status {
    Loading,
    Ready,
    Error,
}

@customElement("subject-select")
export class SubjectSelect extends LitElement {
    // formのネイティブ要素としてふるまうために必要
    static formAssociated = true;
    #internals: ElementInternals;

    /** 通信の競合状態を防ぐための最新のクエリ番号 */
    #loadId = 0;

    constructor() {
        super();
        this.#internals = this.attachInternals();
    }

    protected override createRenderRoot() {
        return this; // lightDom化
    }

    /** コンポーネント状態 */
    @state()
    private status: Status = Status.Ready;

    /** 取得した教科，未収得時は空配列 */
    @state()
    private subjects: Subject[] = [];

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

    /** 更新時の処理 */
    protected override updated(changedProperties: PropertyValues) {
        super.updated(changedProperties);
        const filterKeys = ["facultyId", "majorId", "selectedGrade", "selectedTerm"];
        if (filterKeys.some((key) => changedProperties.has(key))) {
            this.selectedSubjectId = undefined;
            void this.loadSubject();
            this.updateFormState();
        }
    }

    /** APIから選択された学部IDに対応する教科一覧を取得する */
    private async loadSubject() {
        // 通信するごと１ずつ増やす
        const id = ++this.#loadId;

        // 学部が選択されていない場合はAPIを呼ばない
        if (this.facultyId === undefined) {
            this.subjects = [];
            this.status = Status.Ready;
            return;
        }

        this.status = Status.Loading;

        try {
            const subjects = await fetchSubjects(
                this.facultyId,
                this.majorId,
                this.selectedGrade,
                this.selectedTerm,
            );

            // 通信時のloadIdが一致した場合のみsubjectsに代入される
            if (id !== this.#loadId) return; // stale response
            this.subjects = subjects;

            this.status = Status.Ready;
        } catch (e) {
            if (id !== this.#loadId) return;
            console.error("教科一覧の取得に失敗", e);
            this.status = Status.Error;
        }
    }

    /** formの状態を更新する */
    private updateFormState() {
        const data = new FormData();
        data.set("subject", this.selectedSubjectId ?? "");
        data.set("grade", this.selectedGrade !== undefined ? String(this.selectedGrade) : "");
        data.set("term", this.selectedTerm !== undefined ? String(this.selectedTerm) : "");

        this.#internals.setFormValue(data);
        if (
            this.facultyId === undefined ||
            this.selectedSubjectId === undefined ||
            this.selectedGrade === undefined ||
            this.selectedTerm === undefined
        ) {
            this.#internals.setValidity(
                { valueMissing: true },
                "学部、教科、学年、学期を選択してください",
            );
        } else {
            this.#internals.setValidity({});
        }
    }

    /** 画面表示設定HTMLそれぞれ教科，学年，学期 */
    override render() {
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

        const subject_options = this.subjects.map(
            (s) => html`<option value=${s.id}>${s.name}</option>`,
        );

        const grade_options = grades.map((g, n) => html`<option value=${n + 1}>${g}</option>`);

        const term_options = terms.map((t, n) => html`<option value=${n + 1}>${t}</option>`);

        // HTMLに教科選択，学年選択，学期選択のoptionを生成する。学部が選択されていない場合は空の配列となる。
        // @changeごとに変更される
        return html`<div class="section-content">
            <label>
                学年
                <select .value=${String(this.selectedGrade ?? "")} @change=${this.onGradeChange}>
                    <option value="">--学年--</option>
                    ${grade_options}
                </select>
            </label>
            <label>
                学期
                <select .value=${String(this.selectedTerm ?? "")} @change=${this.onTermChange}>
                    <option value="">--学期--</option>
                    ${term_options}
                </select>
            </label>
            <label>
                教科
                <select .value=${this.selectedSubjectId ?? ""} @change=${this.onSubjectChange}>
                    <option value="">教科を選択してください</option>
                    ${subject_options}
                </select>
                ${this.status === Status.Loading ? html`読み込み中...` : ""}
                ${this.status === Status.Error ? html`教科一覧の取得に失敗しました` : ""}
            </label>
        </div>`;
    }

    /** 教科変更時に呼び出される updateFormState でformDataに保存する*/
    private onSubjectChange(e: Event) {
        // 未選択の`option`は空文字なので検証で弾かれる
        this.selectedSubjectId = toSubjectId((e.target as HTMLSelectElement).value);
        this.updateFormState();
    }

    /** 学年変更時に呼び出される updateFormState でformDataに保存する*/
    private onGradeChange(e: Event) {
        this.selectedGrade = toGrade((e.target as HTMLSelectElement).value);
    }

    /** 学期変更時に呼び出される updateFormState でformDataに保存する*/
    private onTermChange(e: Event) {
        this.selectedTerm = toTerm((e.target as HTMLSelectElement).value);
    }
}
