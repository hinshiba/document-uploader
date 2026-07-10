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
    static formAssociated = true;

    #internals: ElementInternals;

    constructor() {
        super();
        this.#internals = this.attachInternals();
    }

    protected override createRenderRoot() {
        return this;
    }

    @state()
    private status: Status = Status.Loading;

    @state()
    private subjects: Subject[] = [];

    @state()
    private selectedSubjectId = "";

    @property()
    facultyId = "";

    override connectedCallback(): void {
        super.connectedCallback();
        void this.loadSubjects();
    }

    protected override updated(changedProperties: PropertyValues) {
        // console.log(changedProperties);
        if (changedProperties.has("facultyId")) {
            this.selectedSubjectId = "";
            void this.loadSubjects();
        }
    }

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

            if (this.facultyId === "" || this.selectedSubjectId === "") {
                this.#internals.setValidity({ valueMissing: true }, "学部と教科を選択してください");
            } else {
                this.#internals.setValidity({});
            }
        } catch {
            this.status = Status.Error;
        }
    }

    override render() {
        console.log(this.subjects);
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
