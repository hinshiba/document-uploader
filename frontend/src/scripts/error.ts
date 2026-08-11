/** 入力の不備によって生じるエラー */
export class ValidationError extends Error {
    override readonly name = "ValidationError";
}

/** Api呼出し時に発生しうるエラーの種類 */
export type ApiErrorKind = "timeout" | "network" | "client" | "server";

/** Apiエラーごとの標準メッセージ */
const API_ERROR_MESSAGE: Record<ApiErrorKind, string> = {
    timeout: "タイムアウトエラー: 時間を置いて再試行してください．",
    network: "ネットワークエラー: ネットワーク接続を確認してください．",
    client: "クライアントエラー: 開発者にお問い合わせください．",
    server: "サーバーエラー: サーバー管理者にお問い合わせください．",
};

/** Api呼出しで発生しうるエラー */
export class ApiError extends Error {
    override readonly name = "ApiError";

    constructor(
        readonly kind: ApiErrorKind,
        readonly method: string,
        readonly path: string,
        /** network/timeout ではundefined */
        readonly status?: number,
        options?: ErrorOptions,
    ) {
        // 標準のmessageを自動生成して埋める
        super(`${method} ${path} -> ${status ?? kind}`, options);
    }
}

/** エラーをユーザーが見える文字列に変換する */
export function toUserMessage(e: unknown): string {
    if (e instanceof ValidationError) {
        return e.message;
    }

    if (e instanceof ApiError) {
        return API_ERROR_MESSAGE[e.kind];
    }
    return "想定外のエラーが発生しました．開発者にお問い合わせください．";
}
