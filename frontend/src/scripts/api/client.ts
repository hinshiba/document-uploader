import type { components } from "./types";

export type Faculty = components["schemas"]["Faculty"];
export type Major = components["schemas"]["Major"];
export type Subject = components["schemas"]["Subject"];
export type DocumentMetadata = components["schemas"]["DocumentMetadata"];

// 実バックエンドテスト
// "http://localhost:3000/api/v1"
const API_BASE = "http://127.0.0.1:4010";

// Cloudflare Accessが自動付与するヘッダのダミー
// モックは検証しないので何でもよい
const DEV_HEADERS: HeadersInit = { "Cf-Access-Jwt-Assertion": "dev" };

/** リクエストのタイムアウト時間．遅延や停止でUIが固まるのを防ぐ */
const REQUEST_TIMEOUT_MS = 10_000;

/**
 * タイムアウト付きでfetchする
 * AbortControllerで中断し，全リクエストで挙動を揃える
 * @throws タイムアウト時はAbortError，その他fetchのエラー
 */
async function fetchWithTimeout(input: string, init: RequestInit = {}): Promise<Response> {
    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), REQUEST_TIMEOUT_MS);
    try {
        return await fetch(input, { ...init, signal: controller.signal });
    } finally {
        clearTimeout(timer);
    }
}

/**
 * 学部専攻一覧を取得する
 * /faculties GET に対応
 * @returns 学部専攻一覧
 * @throws
 */
export async function fetchFaculties(): Promise<Faculty[]> {
    const res = await fetchWithTimeout(`${API_BASE}/faculties`, { headers: DEV_HEADERS });
    if (!res.ok) throw new Error(`GET /faculties -> ${res.status}`);
    return (await res.json()) as Faculty[];
}

export async function fetchSubjects(
    faculty: string,
    major?: string,
    grade?: string,
    term?: string,
): Promise<Subject[]> {
    const params = new URLSearchParams();
    // faculty必須
    params.set("faculty", faculty);

    if (major) {
        params.set("major", major);
    }

    if (grade) {
        params.set("grade", grade);
    }

    if (term) {
        params.set("term", term);
    }

    const res = await fetchWithTimeout(`${API_BASE}/subjects?${params.toString()}`, {
        headers: DEV_HEADERS,
    });

    if (!res.ok) {
        throw new Error(`GET /subjects -> ${res.status}`);
    }

    return (await res.json()) as Subject[];
}

/**
 * ドキュメントをアップロードする
 * /docs POST に対応
 * @param files アップロードする複数のファイル
 * @param metadata APIの要求するメタデータ
 * @throws
 */
export async function postDocuments(
    files: readonly File[],
    metadata: DocumentMetadata,
): Promise<void> {
    const body = new FormData();
    for (const f of files) body.append("files", f);
    body.append("metadata", JSON.stringify(metadata));

    const res = await fetchWithTimeout(`${API_BASE}/docs`, {
        method: "POST",
        headers: DEV_HEADERS,
        body,
    });
    if (!res.ok) throw new Error(`POST /docs -> ${res.status}`);
}
