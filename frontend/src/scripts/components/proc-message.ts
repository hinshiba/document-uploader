import { html, LitElement } from "lit";
import { customElement, property } from "lit/decorators.js";
import { toUserMessage, type AppError } from "../error";

/** 処理の状態とアラートを表示するコンポーネント */
@customElement("proc-message")
export class ProcMessage extends LitElement {
    /** ステータスメッセージ */
    @property()
    status: string = "";

    /** アラート表示のためのエラー型 */
    @property({ attribute: false })
    error: undefined | AppError = undefined;

    protected override createRenderRoot() {
        return this; // lightDom化
    }

    override render() {
        return html`
            <p role="status">${this.status}</p>
            <p role="alert">${this.error ? toUserMessage(this.error) : ""}</p>
        `;
    }
}
