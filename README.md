## Part 2: HTML

- `fn consume_char()`
  - 1 文字進める
  - Rust の文字列は UTF-8 のバイト列なので、次の文字列に進めるために単純に 1 バイト進めるだけだとマルチバイト文字列に対応できない
  - -> `char_indices()` というメソッドを使うといい

```rust
// Return the current character, and advance self.pos to the next character.
fn consume_char(&mut self) -> char {
    let mut iter = self.input[self.pos..].char_indices();
    let (_, cur_char) = iter.next().unwrap();
    let (next_pos, _) = iter.next().unwrap_or((1, ' '));
    self.pos += next_pos;
    return cur_char;
}
```

## Part 3: CSS

- simple selector のみ
  - combinators で結合されたセレクタのチェインは対象外
- Specificity 詳細度
  - スタイルのコンフリクト時、どのスタイルを override するかを決める方法
  - class より id

## Part 4: Style

- DOM と CSS ルールを入力として、各ノードに適用する CSS プロパティの値を決定する処理
  - Style Tree と呼ぶ <- CSSOM と同じかな？
- 仕様でいう [Assigning property values, Cascading, and Inheritance](https://www.w3.org/TR/CSS2/cascade.html)
- Selector Matching

  - simple selector しかサポートしてないので簡単
  - simple selector がある要素にマッチするかどうかは、その要素だけを見ればいいので

- 🤔 `fn matches_simple_selector()`

`if selector.tag_name.iter().any(|name| elem.tag_name != *name)` というようにイテレータが登場する理由がわからない。 `tag_name` って１個のタグ名じゃないの？

- Building the Style Tree
  - DOM Tree を走査し各要素にマッチする rule を検索する
  - 複数見つかった場合は詳細度(specificity)の一番高いものを選ぶ
  - 今回実装した CSS パーサーはセレクタを詳細度の高い順に保持している(css > `parse_selectors()` 参照)ので最初の１個を選べばよい
- The Cascade
  - web ページのオーナーが提供するスタイルシート：author style sheets
  - それに加えて、ブラウザが提供するデフォルトのスタイル：user agent style shetts
  - ユーザーがカスタムスタイルを追加する user style sheets
  - 3 つの "origins" のうちどれが優先されるかをカスケードという
    - 6 つのレベルがある
  - このブラウザではカスケードは未実装。なので `<head>` タグとかも自分で明示的に消す必要がある
  - カスケードの実装はまあまあ簡単で、各ルールの origin をトラックし、declaration を詳細度に加え origin と importance でソートすればよい
- Computed Values
- Inheritance
- Style Attributes

## Part 5: Boxes

- Part 4 で作成した Style tree を input にし、Layout tree を作る

### The Box Model

- content area: コンテンツが描画される矩形領域(rectangular section)
- width, height, ページ内での position を持つ
- content area の周りに padding, borders, margins

### The Layout Tree

- layout tree: box の集まり(collection)
- box は dimensions と子の box を持つ
- box は Block, Inline, Anonymous のいずれかのノード
- style tree を走査しながら、display の値を見て layout tree に box を追加していく
  - 今回は display: none なら追加しないという実装

## Part 6: Block Layout

- block boxes のレイアウトを実装する
  - headings や paragraphs など、垂直方向に stack する
- 実装するのは normal flow のみ：float や absolute、fixed positioning はスコープ外

### Traversing the Layout Tree

- ある block のレイアウトは、それを含む block (containing block) の dimensions に依存する
  - normal flow における block boxes の場合、単に親
  - root はブラウザのウィンドウサイズ (または "viewport")
- (前 Part の復習) block の width は親に依存し、height は子に依存する
- ...ということは、widths の計算には木構造をトップダウンに走査し、heights を計算するにはボトムアップで走査する必要がある

### Calculating the Width

- 子が親の width を変えることはできないので、自身の width が親の width にフィットしているかを確認する必要がある
  - CSS の spec では constraints のセットで表現されている
- 計算アルゴリズム
  - 要素の(margin, padding も含めた)トータルの width を計算する -> `total`
  - トータルの width が親(containing_block)の width を超えていた場合、
    - margin_left または margin_right が auto なら 0 をセットする
  - underflow = containing_block.content.width - total;

### Positioning

- width の計算よりはシンプル
- 要素の margin, border, padding の幅を計算し、親(containing_block)の content の x 座標から margin, border, padding の left 部分だけずらした位置が content の x 座標
- y 座標も同様だが、垂直方向にスタックする挙動にするためには、要素を追加するたびに ↓ の `containing_block.content.height` が更新されてないといけない

```rust
d.content.y = containing_block.content.height + containing_block.content.y +
              d.margin.top + d.border.top + d.padding.top;
```
