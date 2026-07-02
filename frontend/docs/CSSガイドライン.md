# CSSガイドライン

## ファイル構成

- SCSSを採用する
- `style.scss`をエントリとする
- `tags/`に要素セレクタの基本スタイルを定義する
- `pages/<ページ名>/<セクション名>.scss`にページのセクションごとのスタイルを定義する
- `pages/<ページ名>/<タグ名>.scss`に`main`等のページごとに異なるコンテナ要素へのスタイルは書いてもよい

## エントリファイル

- 次の要素のみ許可
  - `@use`
  - タグへの初期設定
    - `*`
    - `html`
    - `body`
- `@use`は次の順で書く
  - `tags/`
  - `pages/`

## セレクタ

- クラスは`section`と`article`にのみ付与する
  - `main`, `aside`, `nav`は文書に1つしか存在しないので，これらはクラス不要である
  - `div`に新しいクラスを付与することは最終手段
  - `.active`等の状態/疑似クラス等はok
- クラス配下の要素はタグセレクタで指定する
- IDでスタイルしない
- `!important`は禁止

## ネスト

- クラス直下に配下のタグをネストして書く
- 擬似クラス・修飾クラスは`&`で書く（`&:hover`，`&.active`）
- ネストはできるだけ浅く保つ

## 命名

- クラス名は役割・内容を表す
- ケバブケースを用いる

## 変数

- 色・フォント寸法は`_var.scss`に集約する
- ファイル固有の寸法はローカル変数にする

## 例

大部分は省略している

```html
<!-- upload.html -->
<main>
  <section class="drop-area">
    <h2>ファイルをドロップ</h2>
    <ul>
      <li>PDF</li>
      <li>PNG</li>
    </ul>
    <label>ファイルを選択</label>
  </section>
</main>
```

```scss
// style.scss
@use "pages/upload/drop-area" as *;
```

```scss
// pages/upload/drop-area.scss
@use "../../_var" as *;

.drop-area {

  div {
    border: 2px $border-color;
  }

  ul {
    display: flex;
    gap: 20px;
  }

  label {
    background: $primary-color;
    cursor: pointer;

    &:hover {
      background: $primary-hover-color;
    }
  }
}
```
