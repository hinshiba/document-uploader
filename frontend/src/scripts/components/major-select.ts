import { html, LitElement } from "lit";

import { customElement, state } from "lit/decorators.js";
import { fetchFaculties, type Faculty } from "../api/client";

enum Status {
    Loading,
    Ready,
    Error,
}

@customElement("major-select")
export class MajorSelect extends LitElement {
    /** コンポーネント状態 */
    @state()
    private status: Status = Status.Loading;

    /** 取得した学部と専攻の対応．未取得時は空配列 */
    @state()
    private faculties: Faculty[] = [];

    /** 選択した学部ID */
    @state()
    private selectedFacultyId: string = "";

    /** 選択した専攻ID */
    @state()
    private selectedMajorId: string = "";

    override connectedCallback(): void {
        super.connectedCallback();
        void this.loadFaclties();
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

        return html`<select .value=${this.selectedFacultyId} @change=${this.onFacultyChange}>
                <option value="">--学部--</option>
                ${facluty_options}
            </select>
            <select .value=${this.selectedMajorId} @change=${this.onMajorChange}>
                <option value="">--系/コース/専攻--</option>
                ${major_options}
            </select>`;
    }

    private onFacultyChange(e: Event) {
        this.selectedFacultyId = (e.target as HTMLSelectElement).value;
        this.selectedMajorId = ""; // 学部が変更時に専攻をリセット
    }

    private onMajorChange(e: Event) {
        this.selectedMajorId = (e.target as HTMLSelectElement).value;
    }

    protected override createRenderRoot() {
        return this; // lightDom化
    }
}
