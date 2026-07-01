import type { components } from "./api/types";

// 実バックエンドテスト
// "http://localhost:3000/api/v1"
const API_BASE = "http://127.0.0.1:4010";

// Cloudflare Accessが自動付与するヘッダのダミー
// モックは検証しないので何でもよい
const DEV_HEADERS: HeadersInit = { "Cf-Access-Jwt-Assertion": "dev" };

type Faculty = components["schemas"]["Faculty"];

async function checkAlive(): Promise<string> {
    const res = await fetch(`${API_BASE}/alive`);
    if (!res.ok) throw new Error(`/alive return ${res.status}`);
    return res.text();
}

async function fetchFaculties(): Promise<Faculty[]> {
    const res = await fetch(`${API_BASE}/faculties`, { headers: DEV_HEADERS });
    if (!res.ok) throw new Error(`/faculties return ${res.status}`);
    return (await res.json()) as Faculty[];
}

checkAlive()
    .then((text) => console.info("/alive", text))
    .catch((err) => console.error("/alive 失敗", err));

fetchFaculties()
    .then((faculties) => console.info("/faculties", faculties))
    .catch((err) => console.error("/faculties 失敗", err));