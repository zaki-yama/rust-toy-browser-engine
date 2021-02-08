## Part 2: HTML

- `fn consume_char()`
  - 1文字進める
  - Rust の文字列はUTF-8のバイト列なので、次の文字列に進めるために単純に1バイト進めるだけだとマルチバイト文字列に対応できない
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
  - スタイルのコンフリクト時、どのスタイルをoverrideするかを決める方法
  - classよりid



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
  - DOM Treeを走査し各要素にマッチするruleを検索する
  - 複数見つかった場合は詳細度(specificity)の一番高いものを選ぶ
  - 今回実装したCSSパーサーはセレクタを詳細度の高い順に保持している(css > `parse_selectors()` 参照)ので最初の１個を選べばよい
- The Cascade
  - webページのオーナーが提供するスタイルシート：author style sheets
  - それに加えて、ブラウザが提供するデフォルトのスタイル：user agent style shetts
  - ユーザーがカスタムスタイルを追加するuser style sheets
  - 3つの "origins" のうちどれが優先されるかをカスケードという
    - 6つのレベルがある
  - このブラウザではカスケードは未実装。なので `<head>` タグとかも自分で明示的に消す必要がある
  - カスケードの実装はまあまあ簡単で、各ルールのoriginをトラックし、declarationを詳細度に加えoriginとimportanceでソートすればよい
- Computed Values
- Inheritance
- Style Attributes

## Part 5: Layout
- Part 4 で作成したStyle treeをinputにし、Layout treeを作る

### The Box Model
- content area: コンテンツが描画される矩形領域(rectangular section)
- width, height, ページ内でのpositionを持つ
- content area の周りに padding, borders, margins


### The Layout Tree
- layout tree: box の集まり(collection)
- box は dimensions と子の box を持つ
- box は Block, Inline, Anonymous のいずれかのノード
- style tree を走査しながら、display の値を見て layout tree に box を追加していく
  - 今回は display: none なら追加しないという実装
