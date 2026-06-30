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

const select = document.querySelector<HTMLSelectElement>("select");

const form = document.querySelector<HTMLFormElement>("form");

// selectをAPIからとってくる
fetchFaculties()
    .then((faculties) => {
        if (!select) {
            console.error("Html側にtag \"select\"がありません");
            return;
        }

        faculties.forEach((faculty) => {
            const option = document.createElement("option");

            // APIの数だけoptionを作る
            option.value = faculty.id;

            // optionの表示名をAPIのnameにする
            option.textContent = faculty.name;
            select.appendChild(option);
        });
    })
    .catch((err) => console.error("/faculties 失敗", err));


// form送信処理
form?.addEventListener("submit", (event)=>{
        // 通常起きるページの再読み込みを防ぐ
        event.preventDefault();

        // formの値を取得してオブジェクトにする
        const formData = new FormData(form);

        const data = {
            faculty: formData.get("faculty"),
        };
        // JSON化
        const json = JSON.stringify(data, null, 2);

        console.log(json);
        alert(json);
    }
);
