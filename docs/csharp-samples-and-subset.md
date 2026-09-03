# C# サンプルと MPK 対応サブセット

MPK は一般的な C# プロジェクトをそのまま検証するものではありません。
検証対象は、結果を決定的に扱えるように意図して小さくした
`mpk.csharp.scalar.v0` サブセットです。このページでは、リポジトリ内の
C# サンプル相当のファイルが、そのサブセットに対応しているかを説明します。

正式な判定基準は
[C# Scalar Profile v0 Specification](../develop/specs/CSHARP_PROFILE_V0.md)です。
このページは、仕様を初めて読む人のための案内であり、仕様そのものを変更しません。

## 先に結論

現在、`examples/` にはコピーして実行できる一般向け C# サンプル一式はありません。
最もサンプルに近いのは、リリース回帰試験用の
[`fixtures/csharp/policy`](../fixtures/csharp/policy) です。

その中の contract は C# サブセットに対応しています。しかし、現在の
[`Required.cs`](../fixtures/csharp/policy/source/src/Required.cs) はメソッドを
式本体の `=> x` で書いているため、ソースとしては対応していません。
`mpk.csharp.scalar.v0` はメソッドに `{ ... }` のブロック本体を要求し、
式本体のメソッドを `CSHARP_SUBSET_DECLARATION` として拒否します。

| 対象 | 判定 | 理由 |
| --- | --- | --- |
| `Required.cs` の namespace、static class、`int` 引数と戻り値 | 対応 | いずれも許可された宣言・型です |
| `Identity(int x) => x` の式本体 | 非対応 | 許可されるのはブロック本体だけです |
| `contracts/identity.json` | 対応 | 正しいprofile、method ID、事後条件、無変更・正常終了・全域終了を指定しています |
| `scan.json`、`evidence.json`、certificate | サブセット判定には使えない | 下流処理用のfixtureであり、frontendの受理結果ではありません |
| `csharp2vir/fuzz/seeds` | 利用例ではない | parserや拒否動作を試すテスト入力です |

つまり、生成済みの evidence が accepted であっても、現在の `Required.cs` が
C# frontend に受理されることを意味しません。このfixtureのテストは、用意済みの
VIR、source map、manifestとソースのバイト列を結び付けて下流処理を確認します。
ソースがサブセット内かどうかは、別途 `csharp2vir` のfrontend gateで判定されます。

## サブセットに対応する最小例

`Required.cs` を次のようにブロック本体へ書き換えた形なら、ソース部分も現在の
C# サブセットに対応します。

```csharp
namespace Vector;

public static class RequiredChecks
{
    public static int Identity(int x)
    {
        return x;
    }
}
```

対応する contract の意味は「戻り値は入力 `x` と等しい」です。ソース内の
コメントや属性を契約として解釈するのではなく、メソッドごとに厳密なJSON
sidecarを用意します。

```json
{
  "schema": "mpk.csharp.contract.v0",
  "semantic_profile": "mpk.csharp.scalar.v0",
  "method": "Vector.RequiredChecks::Identity(i32)->i32",
  "requires": [],
  "ensures": [
    {
      "lhs": { "result": 0 },
      "op": "eq",
      "rhs": { "parameter": "x" }
    }
  ],
  "modifies": [],
  "abrupt_completion": "forbidden",
  "termination": "total"
}
```

実ファイルではJSONのキー、値、個数、型、method IDまで厳密に検査されます。
上の整形表示は読みやすくしたもので、正規化規則の代わりにはなりません。

## 使えるC#の範囲

サブセットで主に使えるものは次のとおりです。

- file-scopedまたはblock-scoped namespace
- 入れ子でない `public static class` または `internal static class`
- `public static` な検証対象メソッドと、`private static` / `internal static` な補助メソッド
- `bool`、`int`、`uint`、`long`、`ulong`
- 明示型のローカル変数、単純代入、ブロック、`if` / `else`、早期return、最終return
- 真偽演算、比較、ビット演算、シフト、条件演算子 `?:`
- 明示した `checked` / `unchecked` 文脈での整数演算と変換
- 左から右に評価される、同じソース集合内の直接static call

一方、一般的なC#であっても、次のものは現在のサブセット外です。

- 式本体メソッド、instance class、field、property、constructor、attribute
- `var`、配列、文字列、浮動小数点、`decimal`、nullable、任意のユーザー定義値型
- loop、`switch`、pattern matching、例外、`async` / `await`、iterator
- LINQ、I/O、時刻、乱数、reflection、P/Invoke、thread、task、外部ライブラリ呼び出し
- 再帰、循環するstatic call、未選択または到達不能な余分のメソッド
- 暗黙の型昇格に依存する混在型演算

「C#としてコンパイルできる」ことと「MPKのC#サブセットに入る」ことは別です。
Roslynによる通常の型検査に成功した後も、MPKは宣言、型、制御フロー、副作用、
contract、変換、overflow文脈を追加で検査します。

## 既存fixtureが示すもの、示さないもの

`fixtures/csharp/policy` は、C# のsemantic contextを持つVIRから、policy scan、
certificate、evidence、AI向けのsanitized requestまでが正しく連携することを示します。
特に、証明の判定をAIの説明ではなくsource-free checkerが担うことを確認するための
fixtureです。

ただし、次の用途には使えません。

- C#サブセット全体の機能一覧として使う
- 任意のC#コードが検証できると判断する
- `Required.cs` が現在のfrontendを通過した証拠として使う
- 一般利用者向けのend-to-end実行手順として使う

サブセット全体の受理・拒否例は
[`csharp-profile-v0.json`](../develop/specs/vectors/csharp-profile-v0.json) と、
実装テストの
[`csharp_subset_harness.cs`](../crates/mpk-cli/tests/csharp_subset_harness.cs) が
所有しています。これらは仕様・回帰試験用であり、一般向けサンプルの代替では
ありません。

## ローカルで確認する

C# frontendのサブセット試験は、固定済みのoffline toolchainを用いて実行します。

```sh
./scripts/build-csharp-frontend.sh --test-subset
```

必要なbuild-input cacheがまだない環境では、READMEに記載された明示的な
provisioningを先に行います。リリース全体を確認する場合は、native Linux環境で
次のローカルgateを使用します。

```sh
sudo ./scripts/check-java-frontend.sh
```

どちらも「既存の `Required.cs` が準拠サンプルである」という判定ではなく、固定
vectorに対してfrontendがサブセット境界を正しく受理・拒否することを確認します。

## 今後の実務向け拡張

式本体、`init` / `required` とobject initializerを含むimmutableなinstance data
model、配列と内部で具体化される上限付きsequence/canonical ordered map/set、
文字列とculture非依存の厳密なboundary codec、浮動小数点、decimal、nullable、
アプリケーション所有のclosed outcome、省略・明示的null・値を区別する
boundary presence、日付・時刻・duration・instant・GUID・Money、構造的な等値性と
順序、loop、switch/pattern、例外、明示的なserialization boundary、pureな
business state transitionを追加する提案は
[`C# Practical Subset Expansion Design`](../develop/docs/08_csharp_practical_subset_design.md)
にまとめています。アプリケーションのsource/runtimeにMPK依存は追加せず、
ユーザー定義generic、iterator、`async` / `await` は対象外です。この設計は
提案段階であり、現在のprofileや公開された受理範囲を変更するものではありません。
