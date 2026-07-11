import { html, LitElement, type PropertyValues } from "lit";
import { customElement, property, state } from "lit/decorators.js";
import { fetchSubjects, type Subject } from "../api/client";

enum Status {
    Loading,
    Ready,
    Error,
}

export interface SelectionChangeDetail {
    facultyId: string;
    subjectId: string;
    gradeId: string;
    termId: string;
}

@customElement("subject-select")
export class SubjectSelect extends LitElement {
    // formのネイティブ要素としてふるまうために必要
    static formAssociated = true;
    #internals: ElementInternals;
    // 学年と学期のHTML上のoptionのため，gradesとtermsを追加
    grades = ["1回生", "2回生", "3回生", "4回生"];
    terms = ["1学期", "2学期", "3学期", "4学期"];

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
    /** 選択した学年Id */
    @state()
    private selectedGradeId = "";
    /** 選択した学期Id */
    @state()
    private selectedTermId = "";

    /** 外部（upload.ts）から受け取る現在選択中の学部ID。
    //  @property にすることで外部から値を設定でき、変更時には updated() が実行される。*/
    @property()
    facultyId = "";

    /** 更新時にloadfacultiesを呼ぶ */
    override connectedCallback(): void {
        super.connectedCallback();
        void this.loadSubjects();
    }

    /** 更新時の処理 */
    protected override updated(changedProperties: PropertyValues) {
        // console.log(changedProperties);
        if (changedProperties.has("facultyId")) {
            this.selectedSubjectId = "";
            this.selectedGradeId = "";
            this.selectedTermId = "";
            void this.loadSubjects();
            void this.updateFormState();
        }
    }

    /** facultyIdが変更された際に教科一覧を取得する */
    private async loadSubjects() {
        // 学部が選択されていない場合はAPIを呼ばない
        if (!this.facultyId) {
            this.subjects = [];
            return;
        }

        this.status = Status.Loading;
        try {
            this.subjects = await fetchSubjects(this.facultyId);

            this.status = Status.Ready;
        } catch (e) {
            console.error("教科一覧の取得に失敗", e);
            this.status = Status.Error;
        }
    }

    /** formの状態を更新する */
    private updateFormState() {
        const data = new FormData();
        data.set("faculty", this.facultyId);
        data.set("subject", this.selectedSubjectId);
        data.set("grade", this.selectedGradeId);
        data.set("term", this.selectedTermId);

        this.#internals.setFormValue(data);

        if (
            !this.facultyId ||
            !this.selectedSubjectId ||
            !this.selectedGradeId ||
            !this.selectedTermId
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
            new CustomEvent<SelectionChangeDetail>("selection-change", {
                detail: {
                    facultyId: this.facultyId,
                    subjectId: this.selectedSubjectId,
                    gradeId: this.selectedGradeId,
                    termId: this.selectedTermId,
                },
                bubbles: true,
                composed: true,
            }),
        );
    }

    /** 画面表示設定HTMLそれぞれ教科，学年，学期 */
    override render() {
        const subject_options = this.subjects.map(
            (s) => html`
                <option value=${s.id} ?selected=${s.id === this.selectedSubjectId}>
                    ${s.name}
                </option>
            `,
        );
        const grade_options = this.grades.map(
            (g) => html` <option value=${g} ?selected=${g === this.selectedGradeId}>${g}</option> `,
        );
        const term_options = this.terms.map(
            (t) => html` <option value=${t} ?selected=${t === this.selectedTermId}>${t}</option> `,
        );
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
                <select .value=${this.selectedGradeId} @change=${this.onGradeChange}>
                    <option value="">--学年--</option>
                    ${grade_options}
                </select>
            </label>
            <label>
                学期
                <select .value=${this.selectedTermId} @change=${this.onTermChange}>
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
        this.selectedGradeId = (e.target as HTMLSelectElement).value;
        this.updateFormState();
    }

    /** 学期変更時に呼び出される updateFormState でformDataに保存する*/
    private onTermChange(e: Event) {
        this.selectedTermId = (e.target as HTMLSelectElement).value;
        this.updateFormState();
    }
}
