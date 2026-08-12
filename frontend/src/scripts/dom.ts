/**
 * 共通して行うdom操作を集約する
 */

import { toUserMessage } from "./error";

/**
 * 要素を型付きで取得するヘルパ
 * @param selector セレクタ
 * @returns 見つかった要素
 * @throws 要素が存在しない場合
 */
export function required<T extends Element>(selector: string): T {
    const el = document.querySelector<T>(selector);
    if (!el) {
        throw new Error(`Element not found. selector: ${selector}`);
    }
    return el;
}

/**
 * 指定要素の`textContent`にエラーメッセージを出力する
 */
export function putError(element: HTMLElement, err: unknown): void {
    element.textContent = toUserMessage(err);
}
