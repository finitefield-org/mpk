# AI v4 request and explanation design

This inactive W04 design adds `mpk.ai.explain.request.v4` and
`mpk.ai.explanation.v4`. Both use raw SHA256 of their entire canonical narrow
JSON value, without a trailing LF or own hash field. Practical C#'s reserved
v3 roots and the active v2 consumers retain their meanings. The
[catalogue](schema.json) and [executable model](../../../../probes/go-practical/ai_contracts.py)
define closed required fields, nested shapes, limits, category allowlists and
the two new projection/redaction recipe IDs. The shared inactive specification
will own those IDs; this record does not register or publish them.

## Source authority and redaction

Input authority is the complete independently validated source/context,
selection, policy evidence and receipt, including the actual ordinary proof
state. The model reuses the previous source/policy validation rather than
trusting an ID, hash, caller status or AI-supplied evidence object. The final
local report retains full context, contracts, selection/evidence/receipt refs,
request refs, the selected explanation language and the private alias map's
restored identities. English/Japanese fixture prose follows that selection.

The provider receives only the sanitized request. Its field set contains public
profile facts, closed strategy/checker/axiom tuples, coarse source state,
global status counts, checker summary, helper counts, batch coordinates and
property aliases/categories/statuses/evidence kinds. No source text, contracts,
captured paths, module/function/member/declaration identities, private property
IDs, source/evidence/certificate/selection hashes, release IDs, diagnostics or
foundation/limits digests enter that payload.

Semantic parameters use a separate exact allowlist for each profile. In
particular, C# preprocessor symbols and Go limits/foundation references stay
local. Fixed target widths, checked-overflow policy, compiler/language versions
and other enumerated public profile facts may be explained. Category strings
come only from the closed catalogue; source descriptions and notes are never
copied. Unrecognized retained categories stay `unrecognized`.

Retain the existing complete checker/theory summary: both verdicts, certificate
candidate count, checked theory-certificate count, normalized theory formats,
and nullable axiom counts for total/core/builtin-theory/Go-semantics/external.
Counts come from independently validated evidence; absence is not a fabricated
zero report. The [initial report review](verification/initial-report-review/diagnostic.json)
preserves the first wire draft before language and these legacy fields were
restored. Future materialized Go uses the same exact evidence-summary contract.

`mpk.evidence.ai_projection.v2` and `mpk.evidence.redaction.v2` describe the new
common projection. New-family profile-owned AI contracts bind those recipes,
display language and `proof_authority:false`/`source_access:false`; the Go
contract additionally binds its foundation descriptor locally. Reusing a new
recipe ID in an old profile contract is forbidden. Every migrated consumer
must validate the exact new context-selected contract; old compiled v0
contracts and `minimal-v1` remain unchanged.

## Complete batching and truthful state

The old request limit of 32 properties is preserved per request. Order is
status, category, evidence-kind bitset, then original source order. Split this
whole ordered list into contiguous batches of at most 32. Aliases use global
ordinals (`property-0001` onward), so different batches have disjoint aliases.
An equal-size response from another batch therefore rejects. Never truncate
an obligation list or silently summarize only its first batch.

The two current Go authorities contain 85 and 84 unelaborated demands: all 169
appear exactly once, in three batches each. Their status remains
`proof_pending`, with `ordinary_elaboration_pending` as source state. There
are no generated ordinary members, checker acceptances or axiom counts for
those Go demands. Retained checked Rust properties retain their actual
`mpk_verified` statuses. That status comes from policy, not the explanation.

Actual retained assembly errors are distinct from compiler rejection, false
theorem and checker rejection. With no properties they receive one diagnostic
batch with a zero-length property response and `not_run` checker fields. Its
item schema avoids an empty enum; zero cardinality forbids fabricated items.

Materialized Go will use exact policy-owned property statuses and ordinary
target/evidence bindings with this same complete projection/batching algorithm.
The current model has no such Go authority and cannot invent it. The remaining
ordinary-program/policy migration design must close that binding before W04
freeze. Actual source/elaborator/application proofs and compiled integration
remain T02–T07 requirements.

## Provider and local report boundary

Each response has exactly `overview`, `property_explanations`, `limitations`
and `next_steps`; property explanations contain only `property_ref` and prose.
All aliases for that batch must occur exactly once. Arbitrary output ordering
is allowed; the local assembler restores original source order, identities
and statuses using its own alias map. Unknown fields, status/proof/context/
hash/provenance injection, missing or duplicate aliases and missing batches
reject before any report is returned. Provider failures or incomplete batches
cannot turn into a complete explanation or change proof state.

Use the existing prose-only response schema v0 and prompt template v1. The
prepared provider body retains the existing Vertex JSON field names and
generation settings. These are inert fixture bytes: this task contacts no
provider. Prompt bytes come from the pinned existing source; the substituted
JSON remains data. Sending must remain an explicit CLI/user action through
the existing provider adapter, with existing authentication, endpoint, retry
and resource rules. Neither design validation nor a scheduled proof run
implicitly authorizes a network send.

Generated prose uses byte limits, not Unicode character counts: overview
2,000 bytes, each item 500 bytes, each generated list at most 10 items, total
prose at most 32KiB per batch. Reject empty/whitespace-only text, Unicode
controls except LF, and bidi controls. Canonical requests are at most 64KiB,
pretty provider bodies at most 96KiB, raw prose responses at most 1MiB and the
complete local explanation at most 2MiB. Container depth is at most 128.
Reject a workload that exceeds a complete-document budget; batching never
raises proof/domain bounds or removes required properties. Provider bodies
include their LF in the raw body hash; MPK request/report refs exclude LF.

Each local provider observation has the exact fields in the catalogue:
batch index, local origin, request-body SHA, response-byte SHA and metadata.
The model's origin is `offline_fixture` and metadata is required null. The
production `provider_observed` arm has a required closed metadata object:
existing provider/project/location/requested-model and model-version/response-
ID/create-time/STOP/attempt/token fields, checked by the exact existing native
validators. The local transport adapter, with its independently retained
request/configuration and observed response, creates that arm. A provider
prose field or imported JSON cannot assert that a call happened.

All reports retain `untrusted_helper_analysis`, `proof_evidence:false` and
`source_access:false`, plus the existing disclaimer. Even prose claiming all
source is verified cannot alter the source state, local summary, properties,
checker verdicts or any certificate. The UI must present local statuses as
the authority and prose as explanation. An imported report must regenerate
in full from the same source/evidence, batch requests, separately retained
provider bytes and local observations; shape and hash consistency alone are
insufficient. AI output is never a policy receipt or proof input.

## Evidence and ownership

The [bundles](bundles.json) cover eight source authorities, five profiles and
English/Japanese: 16 local explanations, 24 complete request batches. The
[vectors](vectors.json) cover root fields, versions, language/context/recipe
crossings, batch/property completeness, status/proof promotion, canonical
transport and generated prose limits. The [independent hashes](hash-vectors.json)
cover all 40 request/report roots in both Rust and Go, each observed twice.

The [native AI test receipt](verification/native-ai-checks.json) binds exact
unchanged consumer/test/fixture sources and two real Linux test bodies, each
run twice. It exercises deterministic redacted requests, untrusted reports,
wrong profile/contract/status/hash/alias/prose controls and frozen fixture
ownership over GoFixed, Rust and C#. Java's new wire model is covered here;
its existing native AI test requires its Java setup and is **not** claimed as
executed in this record. All four native preservation/release paths remain
mandatory at activation. Linux runs here use local amd64 emulation on arm64,
not native installed-release or process-tree/cgroup evidence.

T07-W01 owns `crates/mpk-cli/src/go_practical_ai_explain.rs`, its CLI/report
integration and `crates/mpk-cli/tests/go_practical_ai_explain.rs`. Preserve
the existing `csharp_ai_explain.rs` and `java_policy_verify.rs` semantics under
the reviewed common-family migration. T01-W05 publishes the inactive schema/
recipe definitions and vectors; it does not enable public selection or send
requests. Final migration review must also distinguish historical probe source
pins from the new normative package/production baseline rather than relabeling
old observations after source changes.

The [verification record](verification/verification.json) gives selected-test
reasons and raw local logs. The [current metadata checker](../../../../probes/go-practical/check_ai_metadata.py)
retains the immediate API predecessor and all 143 inventoried consumers.
Release roots, ordinary-program/source-binding signatures and final migration
review remain W04 work. W05 owns its cumulative T01 gate. No practical Go profile is activated,
no Go demand is discharged and the full T02–T11 objective remains required.
