import { html, LitElement, type PropertyValues } from "lit";

import { customElement, state } from "lit/decorators.js";
import { type Subject } from "../api/client";

enum Status {
    Loading,
    Ready,
    Error,
}

export interface SelectionChangeDetail {
    facultyId: string;
    subjectId: string;
}

// /* Eventに型を設ける */
// declare global {
//     interface HTMLElementEventMap {
//         "selection-change": CustomEvent<SelectionChangeDetail>;
//     }
// }

@customElement("subject-select")
export class SubjectSelect extends LitElement {
    // formのネイティブ要素としてふるまうために必要
    static formAssociated = true;
    #internals: ElementInternals;

    constructor() {
        super();
        this.#internals = this.attachInternals();
    }

    protected override createRenderRoot() {
        return this; // lightDom化
    }

    @state()
    private status: Status = Status.Loading;
    // 状態を記入
    @state()
    private subjects: Subject[] = [];

    // 選択し学部ID
    @state()
    private selectedFacultyId: string = "";

    // 選択した教科ID
    @state()
    private selectedSubjectId: string = "";

    // // facultyが変わったら所得する
    // @property
    // facultyId: string = "";
    // @property
    // majorId: string = ""

    override connectedCallback(): void {
        super.connectedCallback();
        void this.loadSubjects();
    }

    protected override updated(changedProperties: PropertyValues) {
        if (changedProperties.has("facultyId")) {
            void this.loadSubjects();
        }
    }

    private async loadSubjects() {
        const data = new FormData();
        data.set("faculty", this.selectedFacultyId);
        data.set("subject", this.selectedSubjectId);
        this.#internals.setFormValue(data);

        if (this.selectedFacultyId === "" || this.selectedSubjectId === "") {
            this.#internals.setValidity({ valueMissing: true }, "学部と教科を選択してください");
        } else {
            this.#internals.setValidity({});
        }
    }

    override render() {
        const subject_options = this.subjects.find((f) => f.id === this.selectedFacultyId);
        this.subjects.map((s) => html`<option value=${s.id}>${s.name}</option>`);

        return html` <label>
            <select @change=${this.selectedSubjectId}>
            <select @change=${this.onSubjectChange}>
                <option value="">教科を選択してください</option>
                ${subject_options}
            </select></label
        >`;
    }

    private onSubjectChange(e: Event) {
        this.selectedSubjectId = (e.target as HTMLSelectElement).value;
        this.emitChange();
    }

    private emitChange() {
        this.dispatchEvent(
            new CustomEvent("selection-change", {
                detail: { facultyId: this.selectedFacultyId, subjectId: this.selectedSubjectId },
                bubbles: true,
            }),
        );
    }
}
