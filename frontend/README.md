# frontend

## フォーマットとリント

| 対象     | リンタ                  | フォーマッタ                                     |
| -------- | ----------------------- | ------------------------------------------------ |
| HTML     | superHTML(ビルド後のみ) | Prettier + `@awmottaz/prettier-plugin-void-html` |
| CSS/SCSS | stylelint               | Prettier                                         |
| TS       | oxlint + `tsc --noEmit` | oxfmt                                            |

```sh
bun run lint          # 全リンタ
bun run format        # 全フォーマッタ(上書き)
bun run format:check  # フォーマット検査(検査のみ)
bun run typecheck     # tsc --noEmit
bun run check         # 上記すべて
```

### 注意

- `lint:html`はPATH上の`superhtml`を使う
  - CIでは[GitHub Releases](https://github.com/kristoff-it/superhtml/releases)からバイナリを取得して配置している
- oxfmtでHTMLを整形しない
  - oxfmtは空要素に`/>`を付けるが，HTML5の規約に反する
  - HTMLの整形は`/>`を出力しないプラグインを入れたPrettierとする
- Prettierはプラグインのpeer dependency範囲があるために合わせて`~3.8.5`とする
