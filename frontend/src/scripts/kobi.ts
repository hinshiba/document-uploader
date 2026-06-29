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