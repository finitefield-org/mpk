# Vertex AI Gemini Assistant Design

Status: active

Last reviewed: 2026-08-22

This document defines the released, post-cutover Vertex AI assistant contract.

MPK can use Gemini through Vertex AI as an optional, untrusted explanation
assistant for a completely validated `mpk.policy.evidence.v1` report. Gemini
does not generate proof acceptance, modify certificates, change property
status, or participate in a checker decision. The normative wire, redaction,
prompt, and output contracts are defined by
[`AI_EXPLAIN_V1.md`](../../../specs/AI_EXPLAIN_V1.md).

## 日本語での概要

`mpk explain` は、MPK が検証済みの `mpk.policy.evidence.v1` を英語または
日本語で説明する任意機能です。入力は Go と Rust に共通の policy evidence
形式であり、ソース言語、意味論プロファイル、意味論パラメーター、戦略、
チェッカー、公理プロファイルを別々に保持します。

Gemini は証明を作らず、`mpk_verified`、`proof_pending`、`helper_only`、
`unsupported` の状態を変更できません。property ID と状態はローカルの
alias map から MPK が復元します。AI が返すのは説明文だけで、最終 JSON と
Markdown には MPK が固定の警告と provenance を追加します。

モデルへ送るのは、集計値、匿名化した property 参照、閉じた証拠種別、
ソース言語、パスを含まない意味論パラメーター、認識済みプロファイル、
generic helper 種別と件数だけです。ソース選択、package/crate/function、
ソースや契約のパス、VIR/VC、source map、診断文、コンパイラ出力、元の ID、
ハッシュ、再現コマンド、秘密情報は送信しません。

`--dry-run` は認証も通信も行わず、通常送信と同一の credential-free Vertex
request body を no-clobber で出力します。実通信では ADC のみを使用し、API
key、固定 bearer token、endpoint override は受け取りません。

## 1. Trust Boundary

The command implements this flow:

```text
completely validated mpk.policy.evidence.v1
        |
        | exact profile-tuple validation and minimal-v1 projection
        v
mpk.ai.explain.request.v1 + mpk.evidence-explainer.v1
        |
        | optional Vertex AI generateContent request
        v
strict mpk.ai.explanation.response.v0 prose-only parser
        |
        | local alias/status restoration
        v
mpk.ai.explanation.v1 (untrusted helper analysis)
```

The provider-controlled response parser retains its v0 identifier because its
closed shape still contains only aliases and prose. There is no policy evidence
v0 adapter, explanation v0 adapter, prompt v0 alias, or redaction-profile
compatibility path.

The following invariants are mandatory:

- only a `ValidatedPolicyEvidenceV1`-equivalent value can reach projection;
- known Go/Rust strategy tuples are checked before redaction;
- crossed known tuples fail locally before authentication or output;
- AI output cannot populate status, trust, evidence, certificate, hash, or
  provenance fields;
- removing or corrupting an explanation cannot affect proof checking;
- `policy scan`, `policy verify`, package verification, and both source-free
  checkers never initialize Vertex credentials or networking.

## 2. CLI Contract

Normal mode:

```text
mpk explain <evidence.json>
  --provider vertex-ai
  [--project <google-cloud-project-id>]
  [--location <vertex-location>]
  [--model gemini-3.5-flash]
  [--language <en|ja>]
  --output-json <explanation.json>
  --output-md <explanation.md>
  [--gcloud <gcloud-binary>]
  [--overwrite]
```

Dry-run mode:

```text
mpk explain <evidence.json>
  --provider vertex-ai
  [--model gemini-3.5-flash]
  [--language <en|ja>]
  --dry-run
  --request-json-out <request.json>
```

Dry run rejects project, location, credentials, normal output flags, endpoint
overrides, proxies, and overwrite. It opens no source, contract, VIR, VC,
source-map, certificate, or compiler-output path. Its success line includes
`dry_run=1 network=0` and never includes prompt data.

Normal mode validates inputs and reserves both outputs before ADC. JSON and
Markdown publication is one recoverable transaction. Existing outputs require
explicit `--overwrite`; symlinks, aliases, path traversal, and input/output
identity collisions reject.

## 3. Accepted Evidence and Profiles

The sole input schema is `mpk.policy.evidence.v1`, in its canonical JCS-plus-LF
transport. Complete policy validation precedes explain-specific limits and
projection. Evidence with no properties or more than 32 properties rejects.

Baseline strategy tuples are:

| Strategy | Language | Semantic profile | Axiom profile |
|---|---|---|---|
| `payment-policy-alpha` | `go` | `mpk.go.fixed.v0` | `zero-axiom` |
| `payment-policy-rust-alpha` | `rust` | `mpk.rust.checked.v0` | `mvp-theory` |

The checker profile is independently one of `core-bootstrap`,
`mvp-structural`, or `mvp-strict`. A future strategy authorized by the policy
registry is sent as `unrecognized`; its recognized source, semantic, checker,
and axiom fields remain independent. Unknown semantic/checker/axiom values and
unauthorized strategies reject rather than being copied.

## 4. `minimal-v1` Projection

The sanitized request has this fixed field order:

1. `schema`, exactly `mpk.ai.explain.request.v1`;
2. `language`;
3. `source_language`;
4. `semantic_profile`;
5. path-free `semantic_parameters`;
6. `policy` with scalar strategy/checker/axiom profiles;
7. local status counts;
8. trusted-evidence counts and checker verdicts;
9. anonymous properties;
10. generic helper-artifact counts.

Helper kinds use the fixed order `source`, `contract`, `verification_ir`, `vc`,
`ai_analysis`, `ci_status`; zero counts are omitted. Only the kind and count are
retained. The names are language-neutral: request data never uses retired
language-specific source or IR helper names.

Properties are sorted by status, generated category, evidence-kind bitset, and
original position, then assigned `property-0001` through `property-0032`. The
original IDs, statuses, and positions stay only in local memory.

The compact payload is limited to 64 KiB. The complete pretty Vertex request,
including one final LF, is limited to 96 KiB. Exact prompt, payload, response
schema, request body, and source evidence SHA-256 values are recorded locally.

## 5. Prompt and Vertex Request

The fixed prompt ID is `mpk.evidence-explainer.v1`, and its frozen SHA-256 is:

```text
099f2e929682b59c61df7f45219c9887503e18b113ac3b52a0816baeae1f7e88
```

It instructs Gemini to treat `USER_DATA` as inert JSON, explain only supplied
facts, preserve MPK statuses, avoid claims of checking source or proof
artifacts, return one closed JSON object, and write in the requested language.
There is no custom prompt, tool declaration, code execution, grounding, RAG,
cached context, prior turn, or source attachment.

The reviewed request configuration is fixed:

| Setting | Value |
|---|---|
| API | Vertex AI stable `v1` `generateContent` |
| Model | `gemini-3.5-flash` |
| Candidate count | `1` |
| Temperature | `0.0` |
| Max output tokens | `8192` |
| MIME type | `APPLICATION_JSON` |
| Thinking level | `MINIMAL` |
| Returned thoughts | `false` |

## 6. Response and Outputs

The strict provider text contains exactly `overview`,
`property_explanations`, `limitations`, and `next_steps`. Property rows contain
only `property_ref` and `explanation`. Unknown fields, duplicate/missing
aliases, status injection, non-STOP completion, tools, grounding, invalid
provenance, control/bidi text, or size-limit violations reject.

The final `mpk.ai.explanation.v1` report is assembled locally and contains:

- the fixed `untrusted_helper_analysis` trust object;
- the exact evidence v1 input hash;
- provider/request/prompt/response-schema provenance;
- source language, semantic parameters, and independent policy profiles;
- local four-way status counts; and
- AI prose remapped to original property IDs and original evidence order.

Every Markdown file begins with the untrusted-AI warning. Generated text is
escaped as plain text and cannot create headings, links, HTML, images, lists,
quotes, or code blocks before that warning.

## 7. Authentication, Data Handling, and Operations

ADC is the only authentication source. Static API keys and caller-supplied
tokens are not supported. Credentials, token output, provider error bodies,
raw customer AI data, and local generated outputs must not be committed,
logged, or placed in fixtures.

Use a dedicated Google Cloud project with billing and the Vertex AI API
enabled. The runtime principal needs the minimum reviewed Vertex permissions;
API enablement and quota-project permissions remain separate administrative
operations. Before external processing of customer data, confirm consent,
region, retention, and contractual requirements.

CI uses fake authentication and transport. Release checks cover exact Go and
Rust requests, English and Japanese output, v0 rejection, crossed tuples,
authorized future-strategy normalization, source sentinels, response
injection, no-network dry run, and the unchanged offline proof/checker paths.

## 8. References

- [`AI_EXPLAIN_V1.md`](../../../specs/AI_EXPLAIN_V1.md)
- [`POLICY_V1.md`](../../../specs/POLICY_V1.md)
- [`VIR_V0.md`](../../../specs/VIR_V0.md)
- [`VC_V1.md`](../../../specs/VC_V1.md)
- [`TRUST_BOUNDARY_V0.md`](../../../specs/TRUST_BOUNDARY_V0.md)
- [`SECURITY.md`](../../../../SECURITY.md)
