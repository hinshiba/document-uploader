/**
 * バックエンドへのリクエストを定義する
 */

import { ApiError, err, ok, type ApiErrorKind, type Result } from "../error";
import { log } from "../logging";
import type {
    Document,
    DocumentId,
    DocumentMetadata,
    Faculty,
    FacultyId,
    Grade,
    MajorId,
    Subject,
    SubjectBase,
    Term,
    SubjectId,
    Year,
    Teacher,
    ExamType,
} from "./constraints";

declare module "bun" {
    interface Env {
        /** APIのベースURL */
        BUN_PUBLIC_API_BASE: string;
    }
}

const API_BASE = process.env.BUN_PUBLIC_API_BASE;

// Cloudflare Accessが自動付与するヘッダのダミー
// モックは検証しないので何でもよい
const DEV_HEADERS: Record<string, string> = { "Cf-Access-Jwt-Assertion": "dev" };

/** リクエストのタイムアウト時間 */
const REQUEST_TIMEOUT_MS = 10_000;

/**
 * method，signal以外のオプション
 *
 * 規定動作がある関数において指定できないようにするため
 */
type RequestOptions = Omit<RequestInit, "method" | "signal" | "headers"> & {
    headers?: Record<string, string>;
};

/** このモジュールが返すResultの別名 */
export type ApiResult<T> = Result<T, ApiError>;

/**
 * エラーログを出力し，ApiErrorを持つerrを生成する
 */
function apiError(
    kind: ApiErrorKind,
    method: string,
    path: string,
    status?: number,
    cause?: unknown,
): Result<never, ApiError> {
    log.api.error("request failed", { method, path, status, kind });
    return err(new ApiError(kind, method, path, status, { cause }));
}

/**
 * タイムアウトと既定ヘッダを付けてfetchする
 *
 * 通信の失敗と 4xx/5xx を ApiError として返す
 *
 * @param method - HTTPメソッド
 * @param path - API_BASEからの相対パス
 * @param options - method，signal，headers以外のオプション
 * @returns 成功時はResponse，失敗時はApiError
 */
async function requestRaw(
    method: string,
    path: string,
    options?: RequestOptions,
): Promise<ApiResult<Response>> {
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
            return ok(res);
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
            return apiError("unexpected", method, path, undefined, e);
        });
}

/**
 * JSONをfetchする
 * @typeParam T - 期待するレスポンスの型
 * @returns 成功時はT，失敗時はApiError
 */
async function requestJson<T>(
    method: string,
    path: string,
    options?: RequestOptions,
): Promise<ApiResult<T>> {
    const res = await requestRaw(method, path, options);
    if (!res.ok) {
        return res;
    }

    return await res.value
        .json()
        .then((json) => ok(json as T))
        // ApiErrorなしでJsonにできないのはおそらくサーバーに問題あり
        .catch((e) => apiError("server", method, path, res.value.status, e));
}

/**
 * 学部専攻一覧を取得する
 * /faculties GET に対応
 * @returns 学部専攻一覧．失敗時はApiError
 */
export async function fetchFaculties(): Promise<ApiResult<Faculty[]>> {
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
): Promise<ApiResult<Subject[]>> {
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
 * 科目を登録する
 * /subjects POST に対応
 * @param subject - 登録する科目情報
 * @returns 登録された科目．失敗時はApiError
 */
export async function postSubject(subject: SubjectBase): Promise<ApiResult<Subject>> {
    return requestJson<Subject>("POST", "/subjects", {
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(subject),
    });
}

/**
 * ドキュメントをアップロードする
 * /docs POST に対応
 * @param files - アップロードする複数のファイル
 * @param metadata - APIの要求するメタデータ
 * @returns 成功時はundefined，失敗時はApiError
 */
export async function postDocuments(
    files: readonly File[],
    metadata: DocumentMetadata,
): Promise<ApiResult<undefined>> {
    const body = new FormData();
    for (const f of files) body.append("files", f);
    body.append("metadata", JSON.stringify(metadata));

    const res = await requestRaw("POST", "/docs", { body });
    if (!res.ok) return res;

    return ok(undefined);
}

/** ドキュメントを検索する
 * /docs GET に対応
 * @param subject - 検索する科目のID
 * @param year - 検索する年度
 * @param teacher - 検索する担当者
 * @param examtype - 検索する試験種別
 * @param isanswer - 解答付きかどうか
 * @returns 検索結果のドキュメント一覧．失敗時はApiError
 */
export async function searchDocuments(
    subject: SubjectId,
    year?: Year,
    teacher?: Teacher,
    examtype?: ExamType,
    isanswer?: boolean,
): Promise<ApiResult<Document[]>> {
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

    return requestJson<Document[]>("GET", `/docs?${params.toString()}`);
}

/** downloadDocumentの型を指定 */
export interface DownloadDocument {
    filename: string;
    blob: Blob;
}

/** ドキュメントをダウンロードする
 * /docs/{id} GET に対応
 * @param id - ダウンロードするドキュメントのID
 * @returns ファイル名とファイルデータ．失敗時はApiError
 */
export async function downloadDocument(id: DocumentId): Promise<ApiResult<DownloadDocument>> {
    const path = `/docs/${encodeURIComponent(id)}`;
    const res = await requestRaw("GET", path);
    if (!res.ok) return res;

    // レスポンスのファイルデータをBlobとして取得する
    const blob = await res.value
        .blob()
        .then((b) => ok(b))
        // ApiErrorなしでBlobにできないのはおそらくサーバーに問題あり
        .catch((e) => apiError("server", "GET", path, res.value.status, e));
    if (!blob.ok) return blob;

    // レスポンスヘッダーからダウンロード時のファイル名を取得する
    const disposition = res.value.headers.get("Content-Disposition");
    const filename = parseFilename(disposition) ?? DEFAULT_DOWNLOAD_FILENAME;

    return ok({
        filename,
        blob: blob.value,
    });
}
