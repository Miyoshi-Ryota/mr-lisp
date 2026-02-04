# memo
このメモでは本文中の誤りやミス、その他向上させられる点について、気づいた点をまとめておく。

## 誤り
* parserにおいてListを返すべきところでListDataになっている
* pareserにおいてmatchのtokenがborrow checkerに引っかかる
* parserにおいて、tokenの列をreverseが抜けている。
* test_area_of_a_circle()などのPatil, Vishal. Lisp interpreter in Rust (p. 72)あたりのテストケースについて、2.0.1のリポジトリのコードでもテストが通らない。ListのListについて、再帰的に評価しない実装になっているため。beginキーワードを導入するなりトップレベルのコードについて特別な対応をする必要があった。
* テストコード中の(fib 10)のexpectが55じゃなくて、89になっている。（テストコードが間違っている）

## 誤りかもしれない
* parserにおいて、BoolのObjectを生成しない => bool値を扱うことができないまま進んでいっている…気がする。

## 補足説明ポイント
* 初登場時のObject::ListDataとListの違いの説明が明確でなく、ListDataが何者かわからないまま進んでいった。その後のparserの実装において、本文中のコードでListDataとListの取り違えがある。わかっていないと修正が難しいが説明がなくわからないまま進んでいっているので多少の困難がある。

## 補足
* 2.0.1ではlexerでエラーを返さないのでエラー定義自体無駄。