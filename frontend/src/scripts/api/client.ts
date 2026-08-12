import { ApiError, type ApiErrorKind } from "../error";
import { log } from "../logging";
import type {
    DocumentMetadata,
    Faculty,
    FacultyId,
    Grade,
    MajorId,
    Subject,
    Term,
    SubjectId,
    Year,
    Teacher,
    ExamType,
} from "./constraints";

declare module "bun" {
    interface Env {
        /** APIのベースURL．スクリプトから注入され文字列リテラルへ置換される */
        BUN_PUBLIC_API_BASE: string;
    }
}

const API_BASE = process.env.BUN_PUBLIC_API_BASE;

// Cloudflare Accessが自動付与するヘッダのダミー
// モックは検証しないので何でもよい
const DEV_HEADERS: Record<string, string> = { "Cf-Access-Jwt-Assertion": "dev" };

/** リクエストのタイムアウト時間．遅延や停止でUIが固まるのを防ぐ */
const REQUEST_TIMEOUT_MS = 10_000;

/**
 * method，signal，headers以外のオプション
 *
 * 規定動作がある関数において指定できないようにするため
 */
type RequestOptions = Omit<RequestInit, "method" | "signal" | "headers"> & {
    headers?: Record<string, string>;
};

/**
 * ApiErrorを生成し，エラーログを出力する
 */
function apiError(
    kind: ApiErrorKind,
    method: string,
    path: string,
    status?: number,
    cause?: unknown,
): ApiError {
    log.api.error("request failed", { method, path, status, kind });
    return new ApiError(kind, method, path, status, { cause });
}

/**
 * タイムアウトと既定ヘッダを付けてfetchする
 *
 * 通信の失敗と 4xx/5xx を ApiError として返す
 *
 * @param method HTTPメソッド
 * @param path API_BASEからの相対パス
 * @param options method，signal，headers以外のオプション
 * @returns 成功時はResponse，失敗時はApiError
 * @throws fetchでTimeoutErrorかTypeError以外が生じた場合
 */
async function requestRaw(
    method: string,
    path: string,
    options?: RequestOptions,
): Promise<Response | ApiError> {
    return await fetch(`${API_BASE}${path}`, {
        ...options,
        method,
        headers: { ...DEV_HEADERS, ...options?.headers },
        signal: AbortSignal.timeout(REQUEST_TIMEOUT_MS),
    })
        .then((res) => {
            if (!res.ok) {
                const kind: ApiErrorKind = res.status < 500 ? "client" : "server";
                return apiError(kind, method, path, res.status);
            }
            log.api.debug("request succeeded", { method, path, status: res.status });
            return res;
        })
        .catch((e) => {
            if (e instanceof Error && e.name === "TimeoutError") {
                return apiError("timeout", method, path, undefined, e);
            }

            // fetchが通信に失敗した場合はなぜかTypeError
            if (e instanceof TypeError) {
                return apiError("network", method, path, undefined, e);
            }

            // 想定外エラー
            throw e;
        });
}

/**
 * JSONをfetchする
 * @typeParam T 期待するレスポンスの型
 * @returns 成功時はT，失敗時はApiError
 * @throws fetchでTimeoutErrorかTypeError以外が生じた場合
 */
async function requestJson<T>(
    method: string,
    path: string,
    options?: RequestOptions,
): Promise<T | ApiError> {
    const res = await requestRaw(method, path, options);
    if (res instanceof ApiError) {
        return res;
    }

    return await res
        .json()
        .then((json) => json as T)
        // ApiErrorなしでJsonにできないのはおそらくサーバーに問題あり
        .catch((e) => apiError("server", method, path, res.status, e));
}

/**
 * 学部専攻一覧を取得する
 * /faculties GET に対応
 * @returns 学部専攻一覧．失敗時はApiError
 */
export async function fetchFaculties(): Promise<Faculty[] | ApiError> {
    return requestJson<Faculty[]>("GET", "/faculties");
}

/**
 * 科目一覧を取得する
 * /subjects GET に対応
 * @returns 条件に一致する科目一覧．失敗時はApiError
 */
export async function fetchSubjects(
    facultyId: FacultyId,
    majorId?: MajorId,
    grade?: Grade,
    term?: Term,
): Promise<Subject[] | ApiError> {
    const params = new URLSearchParams();
    // faculty必須
    params.set("faculty", facultyId);

    if (majorId !== undefined) {
        params.set("major", majorId);
    }

    if (grade !== undefined) {
        params.set("grade", String(grade));
    }

    if (term !== undefined) {
        params.set("term", String(term));
    }

    return requestJson<Subject[]>("GET", `/subjects?${params.toString()}`);
}

/**
 * ドキュメントをアップロードする
 * /docs POST に対応
 * @param files アップロードする複数のファイル
 * @param metadata APIの要求するメタデータ
 * @returns 成功時はundefined，失敗時はApiError
 */
export async function postDocuments(
    files: readonly File[],
    metadata: DocumentMetadata,
): Promise<undefined | ApiError> {
    const body = new FormData();
    for (const f of files) body.append("files", f);
    body.append("metadata", JSON.stringify(metadata));

    const res = await requestRaw("POST", "/docs", { body });
    if (res instanceof ApiError) return res;

    return undefined;
}

/** searchDocumentsの型を指定 */
export interface DocumentSearchResult {
    id: string;
    metadata: DocumentMetadata;
}

/** ドキュメントを検索する
 * /docs GET に対応
 * @param subject 検索する科目のID
 * @param year 検索する年度
 * @param teacher 検索する担当者
 * @param examtype 検索する試験種別
 * @param isanswer 解答付きかどうか
 * @returns 検索結果のドキュメント一覧．失敗時はApiError
 */
export async function searchDocuments(
    subject: SubjectId,
    year?: Year,
    teacher?: Teacher,
    examtype?: ExamType,
    isanswer?: boolean,
): Promise<DocumentSearchResult[] | ApiError> {
    const params = new URLSearchParams();

    params.set("subject", subject);

    if (year !== undefined) {
        params.set("year", String(year));
    }

    if (teacher !== undefined) {
        params.set("teacher", teacher);
    }

    if (examtype !== undefined) {
        params.set("examtype", String(examtype));
    }

    if (isanswer !== undefined) {
        params.set("isanswer", String(isanswer));
    }

    return requestJson<DocumentSearchResult[]>("GET", `/docs?${params.toString()}`);
}

/** downloadDocumentの型を指定 */
export interface DownloadDocument {
    filename: string;
    blob: Blob;
}

/** ドキュメントをダウンロードする
 * /docs/{id} GET に対応
 * @param id ダウンロードするドキュメントのID
 * @returns ファイル名とファイルデータ．失敗時はApiError
 */
export async function downloadDocument(id: string): Promise<DownloadDocument | ApiError> {
    const res = await requestRaw("GET", `/docs/${encodeURIComponent(id)}`);
    if (res instanceof ApiError) return res;

    // レスポンスのファイルデータをBlobとして取得する
    const blob = await res.blob();

    // レスポンスヘッダーからダウンロード時のファイル名を取得する
    const disposition = res.headers.get("Content-Disposition");

    // デフォルトのファイル名を設定;
    let filename = "download";

    if (disposition) {
        const match = disposition.match(/filename="?(.+?)"?$/);
        if (match) {
            filename = String(match[1]);
        }
    }

    return {
        filename,
        blob,
    };
}
