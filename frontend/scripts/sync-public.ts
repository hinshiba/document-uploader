import { cpSync, existsSync, mkdirSync, readdirSync, rmSync } from "node:fs";
import path from "node:path";

/** exitcodeの定義 */
enum SyncPublicExitCode {
    SourceNotFound = 1,
}

/** コピー元（bun buildの出力先） */
const SOURCE_DIR = "out";

/** コピー先（backendの`ServeDir`が配信するディレクトリ） */
const TARGET_DIR = path.join("..", "backend", "public");

if (!existsSync(SOURCE_DIR)) {
    console.error(`${SOURCE_DIR} not found. run \`bun run build\` first.`);
    process.exit(SyncPublicExitCode.SourceNotFound);
}

// クリーンな環境には存在しないため，先に用意する
// Bun 1.3.14の`mkdirSync`はrecursive指定でも既存ディレクトリでEEXISTになるため，存在確認してから作る
if (!existsSync(TARGET_DIR)) {
    mkdirSync(TARGET_DIR, { recursive: true });
}

// 古いハッシュ付きファイルが残らないように中身を消してからコピーする
// ディレクトリ自体は残す（コピー先を間違えた場合の被害を抑えるため）
for (const entry of readdirSync(TARGET_DIR)) {
    rmSync(path.join(TARGET_DIR, entry), { recursive: true, force: true });
}

// Bunの`cpSync`はコピー先のディレクトリが存在するとEEXISTになるため，中身を1つずつコピーする
for (const entry of readdirSync(SOURCE_DIR)) {
    cpSync(path.join(SOURCE_DIR, entry), path.join(TARGET_DIR, entry), { recursive: true });
}

console.log(`copied ${SOURCE_DIR} to ${TARGET_DIR}.`);
