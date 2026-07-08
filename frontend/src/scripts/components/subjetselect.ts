import { html, LitElement, type PropertyValues } from "lit";

import { customElement, state } from "lit/decorators.js";
import { fetchFaculties, type Faculty } from "../api/client";

enum Status {
    Loading,
    Ready,
    Error,
}

export interface Subject {
  facultyId: string;
  majorId: string;
  grade: integer;
  term: integer;
}

export async function fetchSubjects(
    faculty: string,
    major?: string,
    grade?: string,
    term?: string,
): Promise<Subject[]> {

    const params = new URLSearchParams();

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

    const res = await fetch(
        `${API_BASE}/subjects?${params.toString()}`,
        {
            headers: DEV_HEADERS,
        },
    );

    if (!res.ok) {
        throw new Error(`/subjects returned ${res.status}`);
    }

    return await res.json();
}

// private syncFormValue() {
//     const data = new FormData();
//     data.set("faculty", this.selectedFacultyId);
//     data.set("major", this.selectedMajorId);
//     this.#internals.setFormValue(data);

//     // 未選択があれば無効とする
//     if (this.selectedFacultyId === "" || this.selectedMajorId === "") {
//         this.#internals.setValidity(
//             { valueMissing: true },
//             "学部と系/コース/専攻を選択してください",
//         );
//     } else {
//         this.#internals.setValidity({});
//     }
// }

// private async loadFaclties() {
//     this.status = Status.Loading;
//     try {
//         this.faculties = await fetchFaculties();
//         this.status = Status.Ready;
//     } catch (e) {
//         console.error("学部一覧の取得に失敗", e);
//         this.status = Status.Error;
//     }
}