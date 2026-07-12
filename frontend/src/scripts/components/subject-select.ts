import { html, LitElement, type PropertyValues } from "lit";
import { customElement, property, state } from "lit/decorators.js";
import { fetchSubjects, type Subject } from "../api/client";

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

    // 通信の競合状態を防ぐための設定
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
    private status: Status = Status.Loading;

    /** 取得した教科，未収得時は空配列 */
    @state()
    private subjects: Subject[] = [];

    /** 選択した教科Id */
    @state()
    private selectedSubjectId = "";

    /** 選択した学年 */
    @state()
    private selectedGrade: number | undefined = undefined;

    /** 選択した学期 */
    @state()
    private selectedTerm: number | undefined = undefined;

    /** 外部（upload.ts）から受け取る現在選択中の学部ID
     * @property にすることで外部から値を設定でき、変更時には updated() が実行される */
    @property()
    facultyId: string = "";

    /** facultyIdと同様に選択中の専攻Id */
    @property()
    majorId: string = "";

    /** 更新時にfetchSubjectsByFacultyIdを呼ぶ */
    override connectedCallback(): void {
        super.connectedCallback();
        void this.loadSubject();
    }

    /** 更新時の処理 */
    protected override updated(changedProperties: PropertyValues) {
        if (changedProperties.has("facultyId") || changedProperties.has("majorId")) {
            this.selectedSubjectId = "";
            void this.loadSubject();
            void this.updateFormState();
        }
    }

    /** APIを所得し，選択された学部IDに対応する教科一覧を取得する */
    private async loadSubject() {
        // 学部が選択されていない場合はAPIを呼ばない
        if (!this.facultyId) {
            this.subjects = [];
            this.status = Status.Ready;
            return;
        }

        this.status = Status.Loading;

        // 通信するごと１ずつ増やす増やす
        const id = ++this.#loadId;

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
        data.set("faculty", this.facultyId);
        data.set("subject", this.selectedSubjectId);
        data.set("grade", this.selectedGrade != null ? String(this.selectedGrade) : "");
        data.set("term", this.selectedTerm != null ? String(this.selectedTerm) : "");

        this.#internals.setFormValue(data);

        if (
            !this.facultyId ||
            !this.selectedSubjectId ||
            !this.selectedGrade ||
            !this.selectedTerm
        ) {
            this.#internals.setValidity(
                { valueMissing: true },
                "学部、教科、学年、学期を選択してください",
            );
        } else {
            this.#internals.setValidity({});
        }

        this.emitChange();
    }

    private emitChange() {
        this.dispatchEvent(
            new CustomEvent("selection-change", {
                detail: {
                    facultyId: this.facultyId,
                    subjectId: this.selectedSubjectId,
                    gradeId: this.selectedGrade,
                    termId: this.selectedTerm,
                },
                bubbles: true,
                composed: true,
            }),
        );
    }

    /** 画面表示設定HTMLそれぞれ教科，学年，学期 */
    override render() {
        if (this.status === Status.Loading) return html`<p>読み込み中...</p>`;
        if (this.status === Status.Error) return html`<p>学部一覧の取得に失敗しました</p>`;

        const grades = ["1回生", "2回生", "3回生", "4回生", "M1", "M2", "D1", "D2", "D3"];
        const terms = ["1学期", "2学期", "3学期", "4学期"];

        const subject_options = this.subjects.map(
            (s) => html`<option value=${s.id}>${s.name}</option>`,
        );

        const grade_options = grades.map((g, n) => html`<option value=${n + 1}>${g}</option>`);

        const term_options = terms.map((t, n) => html`<option value=${n + 1}>${t}</option>`);

        // HTMLに教科選択，学年選択，学期選択のoptionを生成する。学部が選択されていない場合は空の配列となる。
        // @changeごとに変更される
        return html`
            <label>
                教科
                <select @change=${this.onSubjectChange}>
                    <option value="">教科を選択してください</option>
                    ${subject_options}
                </select>
            </label>
            <label>
                学年
                <select .value=${this.selectedGrade} @change=${this.onGradeChange}>
                    <option value="">--学年--</option>
                    ${grade_options}
                </select>
            </label>
            <label>
                学期
                <select .value=${this.selectedTerm} @change=${this.onTermChange}>
                    <option value="">--学期--</option>
                    ${term_options}
                </select>
            </label>
        `;
    }

    /** 教科変更時に呼び出される updateFormState でformDataに保存する*/
    private onSubjectChange(e: Event) {
        this.selectedSubjectId = (e.target as HTMLSelectElement).value;
        this.updateFormState();
    }

    /** 学年変更時に呼び出される updateFormState でformDataに保存する*/
    private onGradeChange(e: Event) {
        const value = (e.target as HTMLSelectElement).value;
        this.selectedGrade = value ? Number(value) : undefined;
        void this.loadSubject();
        this.updateFormState();
    }

    /** 学期変更時に呼び出される updateFormState でformDataに保存する*/
    private onTermChange(e: Event) {
        const value = (e.target as HTMLSelectElement).value;
        this.selectedTerm = value ? Number(value) : undefined;
        void this.loadSubject();
        this.updateFormState();
    }
}
