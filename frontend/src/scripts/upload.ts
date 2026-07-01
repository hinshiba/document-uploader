const DEV_HEADERS: HeadersInit = { "Cf-Access-Jwt-Assertion": "dev" };
const API_BASE = "http://127.0.0.1:4010";
const dropArea = document.getElementById("drop-area") as HTMLDivElement;


// ドラッグした時の変化
dropArea.addEventListener("dragover", (event) => {
    event.preventDefault();
});
// ドラッグした時の処理
dropArea.addEventListener("drop", (event) => {
    event.preventDefault();
    // ドラッグしたファイルをfilesに格納する
    const files = event.dataTransfer?.files ?? null;
    if (!files) return;

    file.files = files; //送信ボタン押したときに処理するための代入//
    const makelist = document.getElementById("makelist") as HTMLElement;
    const message = document.getElementById("message") as HTMLElement;
    for (const file of files) {
        // 無事できたかコンソールに表示
        console.log(file.name);
        // ファイルをアップロードした時の文字関連
        message.style.display = "none";
        const li = document.createElement("li");
        li.textContent = `${file.name}`;
        makelist.appendChild(li);
    }
});

// ファイルを選択して選んだ場合に処理する
function showFiles(files: FileList) {
    const makelist = document.getElementById("makelist") as HTMLUListElement;

    for (const file of files) {
        // 無事できたかコンソールに表示
        console.log(file.name);
        // 箇条書きにしている
        const li = document.createElement("li");
        li.textContent = file.name;
        makelist.appendChild(li);
    }
    // ファイルアップロード時の文字関連
    const message = document.getElementById("message") as HTMLElement;
    message.style.display = "none";
}
// ファイルに入力された内容をshowFiles関数で表示処理する
const fileInput = document.getElementById("file") as HTMLInputElement;
fileInput.addEventListener("change", () => {
    if (!fileInput.files) return;
    showFiles(fileInput.files);
});




// 送信ボタンを押した後にユーザが入力したデータをformDataに内容を保存して得られたすべての情報を送信する
const file = document.getElementById("file") as HTMLInputElement;
const uploadbtn = document.getElementById("uploadbtn") as HTMLButtonElement;

uploadbtn?.addEventListener("click", async () => {
    // ファイルに直接入力の場合
    const alldata = file.files;
    if (!alldata || alldata.length === 0) return;

    const formData = new FormData();
    // 送信中に送信中を表示
    uploadbtn.disabled = true;
    uploadbtn.textContent = "送信中...";

    for (const filedata of alldata) {
        formData.append("file", filedata)
    }
    // DocumentMetadata.append("metaData", JSON.stringify(metadata)); //すなくんに任せる場所
    try {
        const res = await fetch(`${API_BASE}/docs`, {
            headers: DEV_HEADERS,
            method: "POST",
            body: formData
        });

        if (!res.ok) {
            throw new Error(`送信失敗: ${res.status}`);
        }

        console.log("アップロード成功");
    } catch (error) {
        console.error(error);
    }

    // ユーザ通知として画面に表示する＋送信ボタンなどを画面から削除，ほぼ初期化されている
    file.value = ""
    const makelist = document.getElementById("makelist") as HTMLElement;
    makelist.textContent = "";
    const thank = document.getElementById("thank") as HTMLParagraphElement;
    thank.textContent = "送信完了！！協力ありがとうございました";
    uploadbtn.style.display = "none"
    file.style.display = "none"
});