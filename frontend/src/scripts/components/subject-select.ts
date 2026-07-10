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
}

@customElement("subject-select")
export class SubjectSelect extends LitElement {
    //formのネイティブ要素としてふるまうために必要
    static formAssociated = true;
    #internals: ElementInternals;

    constructor() {
        super();
        this.#internals = this.attachInternals();
    }

    protected override createRenderRoot() {
        return this; // lightDom化
    }
    // コンポーネント状態
    @state()
    private status: Status = Status.Loading;
    // 取得した教科，未収得時は空配列
    @state()
    private subjects: Subject[] = [];
    // 洗濯した教科Id
    @state()
    private selectedSubjectId = "";
    // 初期値を設定
    @property()
    facultyId = "";
    // 更新時にloadgacultiesを呼ぶ
    override connectedCallback(): void {
        super.connectedCallback();
        void this.loadSubjects();
    }
    // 更新時の処理
    protected override updated(changedProperties: PropertyValues) {
        // console.log(changedProperties);
        if (changedProperties.has("facultyId")) {
            this.selectedSubjectId = "";
            void this.loadSubjects();
        }
    }
    // updatedで呼び出される，form関連の処理を行う関数
    private async loadSubjects() {
        // 学部が選択されていない場合はAPIを呼ばない
        if (!this.facultyId) {
            this.subjects = [];
            return;
        }

        try {
            this.status = Status.Loading;

            this.subjects = await fetchSubjects(this.facultyId);

            this.status = Status.Ready;

            const data = new FormData();
            data.set("faculty", this.facultyId);
            data.set("subject", this.selectedSubjectId);
            this.#internals.setFormValue(data);
            // 未選択時に無効とする
            if (this.facultyId === "" || this.selectedSubjectId === "") {
                this.#internals.setValidity({ valueMissing: true }, "学部と教科を選択してください");
            } else {
                this.#internals.setValidity({});
            }
        } catch {
            this.status = Status.Error;
        }
    }
    // 画面表示設定HTML
    override render() {
        const subject_options = this.subjects.map(
            (s) => html`
                <option value=${s.id} ?selected=${s.id === this.selectedSubjectId}>
                    ${s.name}
                </option>
            `,
        );

        return html`
            <label>
                <select @change=${this.onSubjectChange}>
                    <option value="">教科を選択してください</option>
                    ${subject_options}
                </select>
            </label>
        `;
    }
    // 教科変更時に呼び出される
    private onSubjectChange(e: Event) {
        this.selectedSubjectId = (e.target as HTMLSelectElement).value;

        const data = new FormData();
        data.set("faculty", this.facultyId);
        data.set("subject", this.selectedSubjectId);
        this.#internals.setFormValue(data);

        if (this.facultyId === "" || this.selectedSubjectId === "") {
            this.#internals.setValidity({ valueMissing: true }, "学部と教科を選択してください");
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
                },
                bubbles: true,
                composed: true,
            }),
        );
    }
}
