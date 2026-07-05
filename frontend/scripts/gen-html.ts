import { Glob, spawnSync } from "bun";
import { mkdirSync, watch } from "node:fs";
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

/** 連続変更をまとめる待ち時間（ms） */
const DEBOUNCE_MS = 100;

// tera-cliがあるか確認
if (!spawnSync(["tera", "--help"]).success) {
    console.error(
        "tera cli not found. please\ncargo install --git https://github.com/chevdor/tera-cli",
    );
    process.exit(GenHtmlExitCode.TeraNotFound);
}

/** 全テンプレートページからHTMLを生成し，整形する */
function generate(): void {
    // 出力先はgitignore対象でクリーンな環境には存在しないため，先に用意する
    // For CI
    mkdirSync(OUTPUT_DIR, { recursive: true });

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
            // watch中は落とさず次の変更を待つ
            if (isWatch) return;
            process.exit(GenHtmlExitCode.TeraError);
        }
    }

    // CIで引っかからないように整形
    const { stderr, success } = Bun.spawnSync([
        "bunx",
        "prettier",
        "--write",
        "--ignore-path",
        ".prettierignore",
        OUTPUT_DIR,
    ]);
    if (!success) {
        console.error(`prettier failed.`);
        console.error(stderr);
        if (isWatch) return;
        process.exit(GenHtmlExitCode.FormatError);
    }
}

const isWatch = process.argv.includes("--watch");

// 初回生成
generate();

// --watch時はテンプレートの変更を監視して再生成
if (isWatch) {
    let timer: ReturnType<typeof setTimeout> | undefined;
    watch(TEMPLATES_DIR, { recursive: true }, (_event, filename) => {
        // 出力側のみの変更は無視（原理上ここには来ないが念のため）
        if (filename === null) return;
        clearTimeout(timer);
        timer = setTimeout(() => {
            console.log(`template changed: ${filename}. regenerating...`);
            generate();
        }, DEBOUNCE_MS);
    });
    console.log(`watching ${TEMPLATES_DIR} for changes...`);
}
