/**
 * APIで定義されている型に対するスマートコンストラクタもどきを提供する
 */

import type { components } from "./types";

declare const brand: unique symbol;

export type Faculty = components["schemas"]["Faculty"];
export type Major = components["schemas"]["Major"];
export type Subject = components["schemas"]["Subject"];
export type SubjectCreate = components["schemas"]["SubjectCreate"];
export type DocumentMetadata = components["schemas"]["DocumentMetadata"];

export type Grade = components["schemas"]["Subject"]["grade"] & { readonly [brand]: "Grade" };
export type Term = components["schemas"]["Subject"]["term"] & { readonly [brand]: "Term" };
export type Year = components["schemas"]["DocumentMetadata"]["year"] & { readonly [brand]: "Year" };
export type Num = components["schemas"]["DocumentMetadata"]["num"] & { readonly [brand]: "Num" };

export type FacultyId = components["schemas"]["Faculty"]["id"] & { readonly [brand]: "FacultyId" };
export type MajorId = components["schemas"]["Major"]["id"] & { readonly [brand]: "MajorId" };
export type SubjectId = components["schemas"]["Subject"]["id"] & { readonly [brand]: "SubjectId" };
export type ExamType = components["schemas"]["ExamType"];

// 共通の検証

/**
 * 整数変換と範囲検証を共通化する
 * `string`からの厳密な変換はJSの仕様上きりがないので，期待しないこと
 * @param min 省略可能な下限値
 * @param max 省略可能な上限値
 * @returns 検証を通らない場合は`undefined`
 */
function toBoundedInt(data: unknown, min?: number, max?: number): number | undefined {
    if (typeof data !== "number" && typeof data !== "string") return undefined;

    const num = Number(data);
    if (!Number.isInteger(num)) return undefined;
    if (min !== undefined && num < min) return undefined;
    if (max !== undefined && max < num) return undefined;

    return num;
}

/**
 * 必須の自由入力文字列を検証する
 * 空白のみの入力を弾くため，前後の空白を除去した値を返す
 * @returns 空文字の場合は`undefined`
 */
export function toRequiredString(data: unknown): string | undefined {
    if (typeof data !== "string") return undefined;

    const trimmed = data.trim();
    return trimmed === "" ? undefined : trimmed;
}

// Grade

export const GRADE_MIN = 1;
export const GRADE_MAX = 9;

/**
 * `Grade`型への検証を含むコンストラクタ
 * @returns 検証を通らない場合は`undefined`
 */
export function toGrade(data: unknown): Grade | undefined {
    return toBoundedInt(data, GRADE_MIN, GRADE_MAX) as Grade | undefined;
}

// Term

export const TERM_MIN = 1;
export const TERM_MAX = 4;

/**
 * `Term`型への検証を含むコンストラクタ
 * @returns 検証を通らない場合は`undefined`
 */
export function toTerm(data: unknown): Term | undefined {
    return toBoundedInt(data, TERM_MIN, TERM_MAX) as Term | undefined;
}

// Year

export const YEAR_MIN = 1949;

/**
 * `Year`型への検証を含むコンストラクタ
 * @returns 検証を通らない場合は`undefined`
 */
export function toYear(data: unknown): Year | undefined {
    return toBoundedInt(data, YEAR_MIN) as Year | undefined;
}

// Num

export const NUM_MIN = 1;

/**
 * `Num`型への検証を含むコンストラクタ
 * @returns 検証を通らない場合は`undefined`
 */
export function toNum(data: unknown): Num | undefined {
    return toBoundedInt(data, NUM_MIN) as Num | undefined;
}

// Id

/**
 * `FacultyId`型への検証を含むコンストラクタ
 * @returns 検証を通らない場合は`undefined`
 */
export function toFacultyId(data: unknown): FacultyId | undefined {
    return toRequiredString(data) as FacultyId | undefined;
}

/**
 * `MajorId`型への検証を含むコンストラクタ
 * @returns 検証を通らない場合は`undefined`
 */
export function toMajorId(data: unknown): MajorId | undefined {
    return toRequiredString(data) as MajorId | undefined;
}

/**
 * `SubjectId`型への検証を含むコンストラクタ
 * @returns 検証を通らない場合は`undefined`
 */
export function toSubjectId(data: unknown): SubjectId | undefined {
    return toRequiredString(data) as SubjectId | undefined;
}

// ExamType

/**
 * `ExamType`が取りうる全値
 * `Recordのキーに列挙型を設定すると自動で網羅性が検証される
 */
const EXAM_TYPE_SET: Record<ExamType, true> = {
    quiz: true,
    midterm: true,
    final: true,
    other: true,
};

/**
 * `ExamType`型への検証を含むコンストラクタ
 * @returns 検証を通らない場合は`undefined`
 */
export function toExamType(data: unknown): ExamType | undefined {
    if (typeof data !== "string") return undefined;
    // `EXAM_TYPE_SET`が`data`等できるか
    return Object.hasOwn(EXAM_TYPE_SET, data) ? (data as ExamType) : undefined;
}
