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





// const homebtn = document.querySelector(".homebtn");
// const changescreens = document.querySelector(".addbtn") as HTMLElement;
// const behindbtn = document.querySelector(".homebtn");


// homebtn?.addEventListener("click", () => {
//     homebtn?.classList.add("behind");
//     changescreens?.classList.add("ahead");
// })

// for (const screen of changescreens) {
//     screen.addEventListener("click", () => {
//         screen.classList.add("afterchange");
//     }
// }



const file = document.getElementById("file") as HTMLInputElement;
const uploadbtn = document.getElementById("uploadbtn")

// 送信ボタンを押した後にユーザが入力したデータをformDataに内容を保存して得られたすべての情報を送信する
uploadbtn?.addEventListener("click", async () => {
    const alldata = file.files;
    if (!alldata || alldata.length === 0) return;

    const formData = new FormData();

    for (const filedata of alldata) {
        formData.append("file", filedata)
    }
    // formData.append("metaData",JSON.stringify(metadata))

    await fetch("../../../docs", {
        method: "POST",
        body: formData
    });
});






