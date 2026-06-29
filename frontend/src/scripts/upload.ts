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
    // formData.append("metaData",JSON.stringify(metadata))すなくんに任せる場所

    await fetch("../../../docs", {
        method: "POST",
        body: formData
    });
});

// const dropArea = document.getElementById("drop-area") as HTMLDivElement;

// dropArea.addEventListener("dragover", (event) => {
//     event.preventDefault();
// });

// dropArea.addEventListener("drop", (event) => {
//     event.preventDefault();

//     const files = event.dataTransfer?.files;

//     if (!files) return;

//     for (const file of files) {
//         console.log(file.name);
//     }
// });