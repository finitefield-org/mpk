# Owned physical original function selection inputs

The intact original-package owner now binds explicit physical function roots
to its complete source/contract handoff. Each root carries only the original
package key, source object identity and full declaration byte interval. No
native name, AST node index, type/signature graph, dispatch table or replacement
annotation list enters this input. One owner lock covers source/sidecar inventory
validation and exact native function membership. Input order is canonicalized;
duplicate roots, partial/interior intervals, foreign packages/sources, package
initializers, blank functions and counts above8192 reject.

The private envelope hashes every root field and the entire preceding
source/contract input identity. Returned source, annotation and canonical bytes
are independent of callers, preparation paths/images/donor buffers and owner
retirement. The preceding source/contract schema and source-only Bool seed
remain unchanged.

The independent Rust reader requires exact closed fields/hashes, strict order,
complete original sources/sidecars and physical package membership. It parses
the original source and resolves a root only to one complete named FuncDecl.
The resulting node identity comes from that independent parse. Contract
construction accepts no replacement selectors; it uses these exact stored roots,
the full stored annotations and the existing complete static/finite-error closure.
The existing named-root API and the new physical-root API share contract
construction. Both keep the original file-language guard.

Review found that the initial physical-root guard excluded receiver methods
named `init`. The existing source selection correctly distinguishes these from
package initialization. An actual pinned compiler
[builds the complete original package](method-init-go-validity/result.json),
including a separate init-panic source body, without executing it. The corrected
producer/importer admit the ordinary method and reject the package initializer.
A fresh intact owner consumes those exact original Go bytes. Physical and named
method contract derivations are equal. The
[original finding and source hashes](review-method-init-finding.json) remain.

[Verification](verification.json) records22 affected Rust roots,8 actual Go roots
and2 Go subtests, all passing. The original full Go run passed7 roots/2 subtests;
the additional ordinary-init method root passes separately. Both complete prior
source units retain all43 original physical roots each and byte-identical source
components/buffers. The existing consumers retain the complete zero/70-error
universes and literally replay every one of the32 original call-proof certificates.
The physical-root input test does not replace their entry cases with default
contracts. Same-name methods on distinct nominal receivers, a cross-package
callee, all captured raw annotations and8192 unused original functions remain.

All8192 distinct original functions are accepted at the input boundary and
independently produce8192 exact selected contract candidates/8192 expression
nodes. Count8193 rejects. There is no source/argument/result/type/case/root scope
reduction or new budget. Nine native selection controls and15 strict independent
controls reject omissions, crossings, stale/unknown fields, supplied node hints,
Boolean offsets, duplicate/reordered roots and non-function roots. Missing raw
source/sidecar inputs reject. A coherently changed real root is a different
untrusted candidate, demonstrating that a self-hash does not authenticate intent
or an installed launcher/capture.

Strict affected Clippy, current Go vet, changed-file Rust/Go formats and a Go1.23
zero-test compile pass. The five new input tests took404.39 seconds together;
the complete22-root command including compilation took597.81 seconds. These are
host input/contract-candidate checks, not a complete program certificate or
native Linux resource acceptance. An optional sampling attempt found that the
same worker had already finished; its [observation](performance/observed-result.json)
is retained and the worker was not restarted. Unchanged kernels and full
foundation matrices are not rerun. The cumulative T02 gate stays final W04:
`./scripts/check-fast.sh`.

[Independent inspection](inspection-final/result.json) checks five complete
original packets, all hashes/bytes/native package membership, exact native
function positions, root ordering, both unchanged original source units, the
full8192 boundary and the init distinction. [Snapshot storage](snapshot-storage/manifest.json)
retains every one of76800 original files/617338538 bytes with exact paths and
modes. [Transport storage](archive-storage/manifest.json) retains all17 complete
original prelaunch containers/313050954 bytes. Duplicate local originals remain
physically unchanged. Exact restore:

```sh
python3 -B develop/migrations/go-practical/t02-w01/verification/selection-inputs/preserve-originals.py --restore
python3 -B develop/migrations/go-practical/t02-w01/verification/selection-inputs/preserve-transports.py --restore
```

Normal Rust tests read the small tracked original packets directly.
[Input review](input-review.json) scans all decoded originals under the existing
127 matchers/four scopes/39 seals/51 exclusions. No review rule, semantic premise,
axiom, kernel or proof cap changes. Original stdout/stderr keep their exact bytes.

This is private owned input linkage and independent contract analysis. Full
canonical capture/context/descriptor/registry authority, source/IR/state/storage/
frame/callee/program proofs, all56 foundation families/all169 demands, native
Linux release gates and remaining T02-T11 remain due. R01/R06/W01 stay open;
the practical profile stays inactive. Go and Rust remain the only source languages.
