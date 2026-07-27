import { html, LitElement, type PropertyValues } from "lit";

import { customElement, state } from "lit/decorators.js";
import { fetchFaculties } from "../api/client";
import {
    toFacultyId,
    toMajorId,
    type Faculty,
    type FacultyId,
    type MajorId,
} from "../api/constraints";

enum Status {
    Loading,
    Ready,
    Error,
}

export interface SelectionChangeDetail {
    facultyId: FacultyId | undefined;
    majorId: MajorId | undefined;
}

/* Eventに型を設ける */
declare global {
    interface HTMLElementEventMap {
        "selection-change": CustomEvent<SelectionChangeDetail>;
    }
}

@customElement("major-select")
export class MajorSelect extends LitElement {
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

    /** コンポーネント状態 */
    @state()
    private status: Status = Status.Loading;

    /** 取得した学部と専攻の対応．未取得時は空配列 */
    @state()
    private faculties: Faculty[] = [];

    /** 選択した学部ID．未選択は`undefined` */
    @state()
    private selectedFacultyId: FacultyId | undefined = undefined;

    /** 選択した専攻ID．未選択は`undefined` */
    @state()
    private selectedMajorId: MajorId | undefined = undefined;

    override connectedCallback(): void {
        super.connectedCallback();
        void this.loadFaclties();
    }

    protected override updated(changedProperties: PropertyValues): void {
        super.updated(changedProperties);
        this.syncFormValue();
    }

    /** formが使える情報と妥当性を設定する．updatedで呼び出される  */
    private syncFormValue() {
        const data = new FormData();
        data.set("faculty", this.selectedFacultyId ?? "");
        data.set("major", this.selectedMajorId ?? "");
        this.#internals.setFormValue(data);

        // 未選択があれば無効とする
        if (this.selectedFacultyId === undefined || this.selectedMajorId === undefined) {
            this.#internals.setValidity(
                { valueMissing: true },
                "学部と系/コース/専攻を選択してください",
            );
        } else {
            this.#internals.setValidity({});
        }
    }

    private async loadFaclties() {
        this.status = Status.Loading;
        try {
            this.faculties = await fetchFaculties();
            this.status = Status.Ready;
        } catch (e) {
            console.error("学部一覧の取得に失敗", e);
            this.status = Status.Error;
        }
    }

    override render() {
        if (this.status === Status.Loading) return html`<p>読み込み中...</p>`;
        if (this.status === Status.Error) return html`<p>学部一覧の取得に失敗しました</p>`;

        // 読み込めた場合
        const facluty_options = this.faculties.map(
            (f) => html`<option value=${f.id}>${f.name}</option>`,
        );

        const major_options = this.faculties
            .find((f) => f.id === this.selectedFacultyId)
            ?.majors.map((m) => html`<option value=${m.id}>${m.name}</option>`);

        return html` <div class="section-content">
            <label>
                学部
                <select .value=${this.selectedFacultyId ?? ""} @change=${this.onFacultyChange}>
                    <option value="">--学部--</option>
                    ${facluty_options}
                </select>
            </label>
            <label>
                系/コース/専攻
                <select .value=${this.selectedMajorId ?? ""} @change=${this.onMajorChange}>
                    <option value="">--系/コース/専攻--</option>
                    ${major_options}
                </select></label
            >
        </div>`;
    }

    private onFacultyChange(e: Event) {
        // 未選択の`option`は空文字なので検証で弾かれる
        this.selectedFacultyId = toFacultyId((e.target as HTMLSelectElement).value);
        this.selectedMajorId = undefined; // 学部が変更時に専攻をリセット
        this.emitChange();
    }

    private onMajorChange(e: Event) {
        this.selectedMajorId = toMajorId((e.target as HTMLSelectElement).value);
        this.emitChange();
    }

    private emitChange() {
        this.dispatchEvent(
            new CustomEvent("selection-change", {
                detail: { facultyId: this.selectedFacultyId, majorId: this.selectedMajorId },
                bubbles: true,
            }),
        );
    }
}
