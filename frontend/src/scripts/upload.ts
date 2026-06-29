



const dropArea = document.getElementById("drop-area") as HTMLDivElement;



dropArea.addEventListener("dragover", (event) => {
    event.preventDefault();
});

dropArea.addEventListener("drop", (event) => {
    event.preventDefault();

    const files = event.dataTransfer?.files;

    if (!files) return;
    // inputにセット
    file.files = files;

    for (const file of files) {
        console.log(file.name);

        message.style.display = "none";
        makelist.innerHTML += `<li>${file.name}</li>`;
    }
});

const file = document.getElementById("file") as HTMLInputElement;
const uploadbtn = document.getElementById("uploadbtn")
// 送信ボタンを押した後にユーザが入力したデータをformDataに内容を保存して得られたすべての情報を送信する
uploadbtn?.addEventListener("click", async () => {
    // ファイルに直接入力の場合
    const alldata = file.files;
    if (!alldata || alldata.length === 0) return;

    const formData = new FormData();

    for (const filedata of alldata) {
        formData.append("file", filedata)
    }
    // formData.append("metaData",JSON.stringify(metadata))すなくんに任せる場所

    await fetch("/docs", {
        method: "POST",
        body: formData
    });
    makelist.innerHTML = "";
    message.style.display = "block";
    thank.innerHTML = "協力ありがとうございます！"
});