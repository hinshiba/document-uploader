/**
 * APIで定義されている型に対するスマートコンストラクタもどきを提供する
 */

import type { components } from "./types";

declare const brand: unique symbol;

export type Faculty = components["schemas"]["Faculty"];
export type Major = components["schemas"]["Major"];
export type Subject = components["schemas"]["Subject"];
export type Grade = components["schemas"]["Subject"]["grade"] & { readonly [brand]: "Grade" };
export type Term = components["schemas"]["Subject"]["term"];
export type DocumentMetadata = components["schemas"]["DocumentMetadata"];

export const GRADE_MIN = 1;
export const GRADE_MAX = 9;

/**
 * `Grade`型への検証を含むコンストラクタ
 * @returns 検証を通らない場合は`undefined`
 */
export function toGrade(data: unknown): Grade | undefined {
    if (typeof data !== "number" && typeof data !== "string") return undefined;

    const num = Number(data);
    if (Number.isInteger(num) && GRADE_MIN <= num && num <= GRADE_MAX) {
        return num as Grade;
    }

    return undefined;
}
