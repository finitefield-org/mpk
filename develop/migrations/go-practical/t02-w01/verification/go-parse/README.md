# T02-W01 independent original Go syntax

The [Rust parser](../../../../../../crates/mpk-vc/src/go_practical_type_keys/parse.rs)
consumes the previously checked owned lexical source. It constructs a complete-
file syntax arena with original physical byte spans, scalar attributes and
explicit child-field roles. Expressions, selectors, indexing/slicing, literals,
composites and function literals; all type forms, generic parameters/constraints/
instantiations and aliases; statements, control clauses and declaration forms
are parsed. Function bodies are parsed rather than retained as unchecked token
blocks. Metadata keeps names, literal bytes, operators, assignment/alias flags,
channel direction and actual child topology. Canonical preorder sorts field
roles and preserves source order within each list; shared parameter-type nodes
remain shared. AST allocation order and pointer addresses are not identities.
Raw string spelling and enclosing physical spans retain CR bytes separately
from Go's normalized literal text. `//line` never redirects source reads.

The [real Go reference](../../../../../../go-tools/go2vir/practical_parse_reference_test.go)
uses pinned Go1.25 `go/parser` with AllErrors and SkipObjectResolution, plus the
existing original-token physical-end correction. Its graph excludes comment/
Doc fields, object-resolution scopes, derived File imports and build-version
metadata; those remain separate complete-capture obligations. The Rust syntax
arena applies the existing actual-node construction limit to the reachable
non-comment tree, not speculative disambiguation allocations. Complete capture
must still account for comments and all files under the frozen aggregate budget.
Node and worker resource failure never gives a partial admitted syntax tree.
The dedicated worker's virtual stack accommodates deep source syntax; it is
not a new source language or profile limit.

The [verification receipt](verification.json) records all2996 reference cases:
all2934 original Go-file objects from the retained universe, all34 pinned parser
testdata files and28 extra grammar/physical/error controls. Both implementations
accept2969 and reject27. Every5,731,905 accepted node's kind, physical span,
scalar attributes and ordered child-role/target graph agrees across50 node
kinds. The committed default corpus has62 edge cases:35 accepted/27 rejected,
with3134 exact nodes. Original bytes/returned-tree ownership, raw CR tags,
alias declarations and undeclared/duplicate-name examples are exercised.
The latter intentionally parse: syntax success gives no name or type witness.
Real source init functions and applications never execute.

A separate native/Rust depth test accepts0,1000 and20000 unary nestings and
rejects100000 at the pinned parser's nesting boundary. An owned flat arena and
iterative canonical traversal avoid recursive graph destruction. Go and Rust
run-time resource implementations differ; the checked boundary is evidence
for these actual cases, not a host-memory or native Linux release claim.
Two Go roots and three Rust roots, strict Clippy, targeted Go vet and changed-
file formats pass. The original full Rust source comparison takes433.43 seconds
and is executed by the agent. Unchanged lexer/source-key, ordinary448-case and
Go189-input frontend evidence is retained without unrelated repeats/rebuilds.
The full14,496-file/202,722,007-byte SDK remains byte-exact.

Twelve source epochs/8577 preimages retain20 actual executions and four failures.
The first formatter run rejected missing whitespace before Rust match guards;
its original partial formatting writes and inputs remain saved. The first
Clippy run rejected two unnecessary unwraps and an empty branch. The first
paired/default Rust syntax runs found a real grammar bug: a select-case colon
was consumed as a statement label. Restricting label recognition to label-
permitted contexts fixes both failures; all original source epochs, diagnostics
and every emitted candidate stay preserved. No missing AST/output is recreated.
Three [native](read-native.py)/[Rust](read-rust.py) lossless transports retain
12,165 actual original files/1,268,912,231 bytes, with2998 distinct original Rust
byte objects. The [full byte/mode audit](original-byte-audit/result.json)
compares decoded records literally against every retained original execution
file. [Input review](input-review.json) uses unchanged127 matchers/four scopes/
39 seals/51 exclusions; no new source, JSON, directory, seal or blanket exception
is added. The first audit treated a summary node-count integer as an AST list;
its original driver, source and raw diagnostic are [preserved](initial-review-failure/result.json).
The decoder now distinguishes summary counts from actual node arrays.
All255 frozen design inputs, every189 Go production frontend input,
ordinary proof/core/evaluator and registry bytes remain exact. The original Go
oracle/Rust comparison functions are unchanged after separate depth tests are
added; complete actual earlier harness inputs remain recorded.

Independent name/scope/import resolution, constant evaluation, source/type/
alias/nominal correctness and complete capture/version/module/context binding
are still required. Parsing supplies none of those witnesses or ordinary source-
carrier, path/lens, live-store/alias/permission/NoAccess/lifetime/state/program
proofs. Full U/Plain/weights/depth/counts, complete56-family foundation/private
descriptor/registry, native Linux/general namespace/selected closure/all169/
T02–T11 remain required. Go and Rust remain the only source languages and no
installed/public profile is activated. R01/R06/W01 stay open. The T02 whole gate
remains final W04: `./scripts/check-fast.sh`.
