import type {
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

/**
 * 科目一覧を取得する
 * /subjects GET に対応
 * @returns 条件に一致する科目一覧
 * @throws
 */
export async function fetchSubjects(
    facultyId: FacultyId,
    majorId?: MajorId,
    grade?: Grade,
    term?: Term,
): Promise<Subject[]> {
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
 * @throws APIへのアップロードに失敗した場合
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

/**
 * 科目を登録する
 * /subjects POST に対応
 * @param subject 登録する科目情報
 * @returns 登録された科目情報
 * @throws 登録に失敗した場合
 */
export async function postSubject(subject: SubjectBase): Promise<Subject> {
    const res = await fetchWithTimeout(`${API_BASE}/subjects`, {
        method: "POST",
        headers: {
            ...DEV_HEADERS,
            "Content-Type": "application/json",
        },
        body: JSON.stringify(subject),
    });

    if (!res.ok) {
        throw new Error(`POST /subjects -> ${res.status}`);
    }

    return (await res.json()) as Subject;
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
 * @returns 検索結果のドキュメント一覧
 * @throws
 */
export async function searchDocuments(
    subject: SubjectId,
    year?: Year,
    teacher?: Teacher,
    examtype?: ExamType,
    isanswer?: boolean,
): Promise<DocumentSearchResult[]> {
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
    const url = `${API_BASE}/docs?${params.toString()}`;

    const res = await fetchWithTimeout(url, {
        headers: DEV_HEADERS,
    });

    if (!res.ok) {
        throw new Error(`GET /docs -> ${res.status}`);
    }

    return (await res.json()) as DocumentSearchResult[];
}

/** downloadDocumentの型を指定 */
export interface DownloadDocument {
    filename: string;
    blob: Blob;
}

/** ドキュメントをダウンロードする
 * /docs/{id} GET に対応
 * @param id ダウンロードするドキュメントのID
 * @returns ファイル名とファイルデータ
 * @throws
 */
export async function downloadDocument(id: string): Promise<DownloadDocument> {
    const res = await fetchWithTimeout(`${API_BASE}/docs/${id}`, {
        headers: DEV_HEADERS,
    });

    if (!res.ok) {
        throw new Error(`GET /docs/${id} -> ${res.status}`);
    }

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
