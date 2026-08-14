import "./proc-message";

import { html, LitElement, type PropertyValues } from "lit";
import { customElement, query, state } from "lit/decorators.js";
import { fetchFaculties, type ApiResult } from "../api/client";
import {
    toFacultyId,
    toMajorId,
    type Faculty,
    type FacultyId,
    type MajorId,
} from "../api/constraints";

export interface MajorSelectChangeDetail {
    facultyId: FacultyId | undefined;
    majorId: MajorId | undefined;
}

/* Eventに型を設ける */
declare global {
    interface HTMLElementEventMap {
        "major-select-change": CustomEvent<MajorSelectChangeDetail>;
    }
}

/**
 * 学部と専攻の連動した選択のコンポーネント
 */
@customElement("major-select")
export class MajorSelect extends LitElement {
    // formのネイティブ要素としてふるまうために必要
    static formAssociated = true;
    #internals: ElementInternals = this.attachInternals();

    /** 取得した学部と専攻の対応またはエラー．未取得時は`undefined` */
    @state()
    private response: ApiResult<Faculty[]> | undefined;

    /** 選択した学部ID．未選択は`undefined` */
    @state()
    private selectedFacultyId: FacultyId | undefined = undefined;

    /** 選択した専攻ID．未選択は`undefined` */
    @state()
    private selectedMajorId: MajorId | undefined = undefined;

    /** 検証メッセージの表示先とする`select`．描画前は`null` */
    @query('[data-field="faculty"]')
    private facultySelect!: HTMLSelectElement | null;

    /** facultySelectと同様に検証メッセージの表示先とする`select` */
    @query('[data-field="major"]')
    private majorSelect!: HTMLSelectElement | null;

    // lightDom化
    protected override createRenderRoot() {
        return this;
    }

    // domにアタッチされた時に学部一覧を取得する
    override connectedCallback(): void {
        super.connectedCallback();
        void this.loadFaculties();
    }

    private async loadFaculties() {
        this.response = await fetchFaculties();
    }

    // UIが更新されたらformが取得できる値を更新
    protected override updated(changedProperties: PropertyValues): void {
        super.updated(changedProperties);
        this.syncFormValue();
    }

    override render() {
        const faculties = this.response?.ok ? this.response.value : [];
        const error = this.response?.ok === false ? this.response.error : undefined;

        const faculty_options = faculties.map(
            (f) => html`<option value=${f.id}>${f.name}</option>`,
        );
        const major_options = faculties
            .find((f) => f.id === this.selectedFacultyId)
            ?.majors.map((m) => html`<option value=${m.id}>${m.name}</option>`);

        return html` <proc-message
                .status=${this.response === undefined ? "読み込み中" : ""}
                .error=${error}
            ></proc-message>
            <div class="section-content">
                <label>
                    学部
                    <select
                        data-field="faculty"
                        .value=${this.selectedFacultyId ?? ""}
                        @change=${this.onFacultyChange}
                    >
                        <option value="">--学部--</option>
                        ${faculty_options}
                    </select>
                </label>
                <label>
                    系/コース/専攻
                    <select
                        data-field="major"
                        .value=${this.selectedMajorId ?? ""}
                        @change=${this.onMajorChange}
                    >
                        <option value="">--系/コース/専攻--</option>
                        ${major_options}
                    </select>
                </label>
            </div>`;
    }

    /** formが使える情報を設定する  */
    private syncFormValue() {
        const data = new FormData();
        data.set("faculty", this.selectedFacultyId ?? "");
        data.set("major", this.selectedMajorId ?? "");
        this.#internals.setFormValue(data);

        // 未選択があれば無効とする
        // カスタム要素自身はフォーカスできないため，第3引数で未選択の`select`をメッセージの表示先とする
        if (this.selectedFacultyId === undefined) {
            this.#internals.setValidity(
                { valueMissing: true },
                "学部を選択してください",
                this.facultySelect ?? undefined,
            );
        } else if (this.selectedMajorId === undefined) {
            this.#internals.setValidity(
                { valueMissing: true },
                "系/コース/専攻を選択してください",
                this.majorSelect ?? undefined,
            );
        } else {
            this.#internals.setValidity({});
        }
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
            new CustomEvent("major-select-change", {
                detail: { facultyId: this.selectedFacultyId, majorId: this.selectedMajorId },
                bubbles: true,
            }),
        );
    }
}
