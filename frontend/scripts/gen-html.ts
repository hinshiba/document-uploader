import { Glob, spawnSync } from "bun";
import path from "node:path";

/** exitcodeの定義 */
enum GenHtmlExitCode {
    TeraNotFound = 1,
    TeraError = 2,
    FormatError = 3,
}

/** ページのテンプレートとなるHTMLが配置される場所 */
const pattern = new Glob("src/templates/pages/*.html");

/** ページの出力ディレクトリ */
const OUTPUT_DIR = "src/temp";

/** 参照元を含む完全なテンプレートパス */
const TEMPLATES_DIR = "src/templates";

// tera-cliがあるか確認
if (!spawnSync(["tera", "--help"]).success) {
    console.error(
        "tera cli not found. please\ncargo install --git https://github.com/chevdor/tera-cli",
    );
    process.exit(GenHtmlExitCode.TeraNotFound);
}

// 全テンプレートページから生成
for (const page_template of pattern.scanSync()) {
    const out_path = path.join(OUTPUT_DIR, path.basename(page_template));
    const { stderr, success } = spawnSync([
        "tera",
        "-t",
        page_template,
        "--include",
        "--include-path",
        TEMPLATES_DIR,
        "--escape",
        "--env-only",
        "-o",
        out_path,
    ]);
    if (!success) {
        console.error(`tera -t ${page_template} failed.`);
        console.error(stderr);
        process.exit(GenHtmlExitCode.TeraError);
    }
}

// CIで引っかからないように整形
const { stderr, success } = Bun.spawnSync(["bunx", "prettier", "--write", OUTPUT_DIR]);
if (!success) {
    console.error(`prettier failed.`);
    console.error(stderr);
    process.exit(GenHtmlExitCode.FormatError);
}
