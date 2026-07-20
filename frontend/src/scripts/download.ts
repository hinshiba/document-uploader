import {
    fetchFaculties,
    fetchSubjects,
    searchDocuments,
    downloadDocument,
    type DownloadDocument,
} from "./api/client";

// HTML要素を取得
const facultySelect = document.querySelector<HTMLSelectElement>("#faculty")!;
const majorSelect = document.querySelector<HTMLSelectElement>("#major")!;
const gradeSelect = document.querySelector<HTMLSelectElement>("#grade")!;
const termSelect = document.querySelector<HTMLSelectElement>("#term")!;
const subjectSelect = document.querySelector<HTMLSelectElement>("#subject")!;

const searchButton = document.querySelector<HTMLButtonElement>("#search")!;
const resultList = document.querySelector<HTMLUListElement>("#result")!;

/**
 * 学部一覧を読み込む
 */
async function loadFaculties(): Promise<void> {
    const faculties = await fetchFaculties();

    facultySelect.replaceChildren();

    for (const faculty of faculties) {
        const option = document.createElement("option");
        option.value = faculty.id;
        option.textContent = faculty.name;
        facultySelect.append(option);
    }
}

/**
 * 専攻一覧を更新する
 */
async function loadMajors(): Promise<void> {
    const faculties = await fetchFaculties();

    const majors = faculties.find((f) => f.id === facultySelect.value)?.majors ?? [];

    majorSelect.replaceChildren();

    for (const major of majors) {
        const option = document.createElement("option");
        option.value = major.id;
        option.textContent = major.name;
        majorSelect.append(option);
    }
}

/**
 * 科目一覧を更新する
 */
async function loadSubjects(): Promise<void> {
    const subjects = await fetchSubjects(
        facultySelect.value,
        majorSelect.value,
        Number(gradeSelect.value),
        Number(termSelect.value),
    );

    subjectSelect.replaceChildren();

    for (const subject of subjects) {
        const option = document.createElement("option");
        option.value = subject.id;
        option.textContent = subject.name;
        subjectSelect.append(option);
    }
}

/**
 * 検索
 */
async function search(): Promise<void> {
    const documents = await searchDocuments(
        facultySelect.value,
        majorSelect.value,
        Number(gradeSelect.value),
        Number(termSelect.value),
        subjectSelect.value,
    );

    renderDocuments(documents);
}

/**
 * 検索結果を表示
 */
function renderDocuments(documents: DownloadDocument[]): void {
    resultList.replaceChildren();

    for (const doc of documents) {
        const li = document.createElement("li");

        li.textContent = doc.filename;

        li.addEventListener("click", () => {
            void download(doc.id);
        });

        resultList.append(li);
    }
}

/**
 * ファイルをダウンロード
 */
async function download(id: string): Promise<void> {
    const blob = await downloadDocument(id);

    const url = URL.createObjectURL(blob);

    const a = document.createElement("a");
    a.href = url;
    a.download = "";

    a.click();

    URL.revokeObjectURL(url);
}

/**
 * イベント登録
 */
function registerEvents(): void {
    facultySelect.addEventListener("change", async () => {
        await loadMajors();
        await loadSubjects();
    });

    majorSelect.addEventListener("change", () => {
        void loadSubjects();
    });

    gradeSelect.addEventListener("change", () => {
        void loadSubjects();
    });

    termSelect.addEventListener("change", () => {
        void loadSubjects();
    });

    searchButton.addEventListener("click", () => {
        void search();
    });
}

/**
 * 初期化
 */
async function init(): Promise<void> {
    await loadFaculties();
    await loadMajors();
    await loadSubjects();

    registerEvents();
}

void init();
