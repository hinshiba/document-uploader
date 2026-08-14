# CSSガイドライン

## ファイル構成

- SCSSを採用する
- `var.scss`に変数を定義する
- `tags/`に要素セレクタの基本スタイルを定義する
  - カスタム要素も要素セレクタとして扱う
- `pages/<ページ名>/<セクション名>.scss`にページのセクションごとのスタイルを定義する
- `pages/<ページ名>/<タグ名>.scss`に`main`等のページごとに異なるコンテナ要素へのスタイルは書いてもよい
  - 全ページで同じになるものは`tags/`に置く
- 複数ページで同じ見た目になるものは`pages/`間で複製せず，`tags/`に置く

## エントリファイル

- 次の2種がエントリファイルであり，これ以外はpartialである
  - `style.scss`: 全ページで読み込む共通エントリ
  - `pages/<ページ名>/<ページ名>.scss`: そのページでのみ読み込むエントリ
    - ディレクトリ名と同名のファイルのみがエントリとして扱われる
    - ページ固有のスタイルが無い場合はディレクトリごと作らず，HTML側の`link`も書かない
- 次の要素のみ許可
  - `@use`
  - タグへの初期設定
    - `*`
    - `html`
    - `body`
    - これらは`style.scss`でのみ許可
- `@use`は次の順で書く
  - `var`
  - `tags/`
  - `pages/`
- `as *`は変数を取り込む`var`にのみ付ける
  - スタイルのみのファイルに付けると変数名が衝突する
- 拡張子と`./`は書かない
  - `@use "drop-area"`

## セレクタ

- クラスは原則として`section`と`article`にのみ付与する
  - `main`, `aside`, `nav`は文書に1つしか存在しないので，これらはクラス不要である
  - `div`に新しいクラスを付与することは最終手段
    - レイアウトのためだけの`div`は作らず，親のタグセレクタで指定する
  - `.active`等の状態/疑似クラス等はok
- スタイルを当てないクラスをHTML/TSで付与しない
- クラス配下の要素はタグセレクタで指定する
- IDでスタイルしない
- `!important`は禁止

### タグスタイルファイル

- ページ間で統一されるこちらを用いることが望ましい
- 次の構造はこちらのみで指定すること
  - 見出し
  - テキスト
    - `p`
    - `a`
    - `span`等
  - セクション等
    - 入れ子となる場合を除く
  - フォーム部品
    - `label`と入力欄の並べ方
- 親タグの配下に限定するのは，そこにしか現れない場合のみとする
  - `button`のように`form`外にも現れるものは，見た目をタグ単体で指定し，配置のみ親の配下で指定する

### セクションスタイルファイル

- タグに標準のスタイルでいい場合は付与しないこと
- ここで示す構造は特定のスタイルが必要になることがよくある
  - リスト
  - 表
  - フォーム
  - 大きさを制御する必要がある要素
    - `img`
    - `figure`
    - `div`
    - 入れ子となるセクション等

## ネスト

- クラス直下に配下のタグをネストして書く
- 擬似クラス・修飾クラスは`&`を用いて書く
  - `&:hover`
  - `&.active`等

## 命名

- ケバブケースを用いる

## 変数

- 次は`var.scss`に集約する
  - 色
  - フォント寸法
  - 複数のファイルで共有する寸法
    - 余白・間隔
    - コンテンツ幅
    - 境界線の太さ
- ファイル固有の寸法はローカル変数にする
- 1箇所でしか使わず意味が自明な値は，リテラルのままでよい

## 宣言

- `style.scss`のリセットで済むものを再指定しない
  - `margin`, `padding`, `box-sizing`
- 打ち消し合う宣言を書かない
  - 親の`text-align: center`と子の`text-align: left`等

## 例

大部分は省略している

```html
<!-- upload.html -->
<main>
  <section class="drop-area">
    <h2>ファイルをドロップ</h2>
    <div>
      <ul>
        <li>PDF</li>
        <li>PNG</li>
      </ul>
    </div>
    <label>ファイルを選択</label>
  </section>
</main>
```

```scss
// style.scss
@use "var" as *;
@use "tags/form";
```

```scss
// pages/upload/upload.scss
@use "drop-area";
```

```scss
// pages/upload/drop-area.scss
@use "../../var" as *;

.drop-area {

  div {
    border: $border-width solid $border-color;
  }

  ul {
    display: flex;
    gap: $space-md;
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
