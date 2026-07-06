import { Glob } from "bun";
import path from "node:path";

/** exitcodeの定義 */
enum GenCssExitCode {
    SassError = 1,
}

/** 全ページ共通のエントリ */
const COMMON_ENTRY = "src/styles/style.scss";

/** ページエントリの探索パターン（partialと区別するため pages/<name>/<name>.scss を採用） */
const pattern = new Glob("src/styles/pages/*/*.scss");

/** 出力ディレクトリ */
const OUTPUT_DIR = "src/temp";

/** sassのための`入力:出力`を作成 */
function buildEntries(): string[] {
    const entries = [`${COMMON_ENTRY}:${OUTPUT_DIR}/style.css`];

    // pages/<name>/<name>.scssのみ
    for (const file of pattern.scanSync()) {
        const name = path.basename(file, ".scss");
        const dir = path.basename(path.dirname(file));
        if (name !== dir) {
            continue;
        }
        entries.push(`src/styles/pages/${name}/${name}.scss:${OUTPUT_DIR}/${name}.css`);
    }

    return entries;
}

// --watch, --style=compressed等はそのままsassへ
const passthrough = process.argv.slice(2);

const { success } = Bun.spawnSync(["bunx", "sass", ...buildEntries(), ...passthrough], {
    stdout: "inherit",
    stderr: "inherit",
});
if (!success) {
    console.error("sass failed.");
    process.exit(GenCssExitCode.SassError);
}
