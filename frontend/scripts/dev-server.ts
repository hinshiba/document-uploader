import { Glob, type HTMLBundle } from "bun";
import path from "node:path";

/** exitcodeの定義 */
enum DevServerExitCode {
    PageNotFound = 1,
}

/** gen-htmlが生成したページの場所 */
const PAGES_DIR = "src/temp";

/** ルートパスにも割り当てるページのファイル名 */
const INDEX_PAGE = "index.html";

// backendの`ServeDir`は`/upload.html`のように拡張子付きのURLで配信するため，devでもURLを揃える
// CLIの`bun --hot src/temp/*.html`は拡張子を落とした`/upload`にしかマップしないため，ルートを自前で組む
const routes: Record<string, HTMLBundle> = {};

for (const page of new Glob("*.html").scanSync(PAGES_DIR)) {
    // パスが変数でも`HTMLBundle`として解決され，バンドルとHMRの対象になる
    const bundle = (await import(path.resolve(PAGES_DIR, page))).default as HTMLBundle;

    routes[`/${page}`] = bundle;

    // `ServeDir`が`/`で`index.html`を返すのに合わせる
    if (page === INDEX_PAGE) {
        routes["/"] = bundle;
    }
}

if (Object.keys(routes).length === 0) {
    console.error(`no page found in ${PAGES_DIR}. run \`bun run gen:html\` first.`);
    process.exit(DevServerExitCode.PageNotFound);
}

const server = Bun.serve({
    development: { hmr: true },
    routes,
});

console.log(`start listening on ${server.url}`);
for (const route of Object.keys(routes).toSorted()) {
    console.log(`  ${route}`);
}
