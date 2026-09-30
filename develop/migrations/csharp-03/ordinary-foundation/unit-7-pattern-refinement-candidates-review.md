# Exact pattern refinement candidates — W09 checkpoint

The candidate linker requires a theorem for every reconstructed path, with a
name bound to the complete program hash, source sequent, native edge and original
named refinement type. It preserves the original module, source manifest,
imports, level/term prefixes and every original declaration body. Canonically
sorted name indices may move, so names are compared by their resolved value.
Auxiliary ordinary definitions and theorem candidates are allowed; required
theorems must still reference their exact original refinement definition.

Canonical decoding and the existing practical profile enforce byte, node,
declaration and binder limits and exclude axioms, theory certificates and
proof-node entries. The returned object is explicitly untrusted: proof and
application checks remain pending, even when structural linkage succeeds.
Neither a kernel dependency nor proof acceptance is added to production mpk-vc.

Two units check complete universal refinements with free variables and complete
premises. They reject a helper-only certificate, a missing theorem, another
source context, substituted theorem types and changed original source bodies.
A required theorem using an ordinary auxiliary lemma is accepted by the kernel
with zero axioms. A wrong proof of the correct declared type passes structural
linkage but fails kernel core checking. The unchanged Go and Rust binaries both
accept the same correct fixture bytes and reject the same wrong fixture bytes;
their export, certificate and axiom-report hashes agree for the correct proof.

The affected scope units and original-context integration pass eight unique
tests. The integration checks all 113 reconstructed paths across 18 contexts,
rejects every helper-only original certificate and preserves all 36 original
metadata/certificate files byte for byte. This replay preceded the auxiliary
helper policy extension. The recorded source comparison shows that only the
candidate linker and its unit fixture changed afterward; generation bodies,
required theorem names/types and the integration consumer were unchanged.
The two candidate units, Clippy with test targets and format were checked again
after that extension. This is why the expensive generation replay was retained.

Receipts, raw logs, source manifests, same-byte fixture checks and test-selection
reasons are under
`verification-logs/control-predicates/with-pattern-refinement-candidates/`.
The previous packed generation checkpoint separately passed its requested
Linux replay at fixed source commit `130a3ca4`; its terminal receipts are under
`with-packed-pattern-proof-types/server-linux/final/`. Linux verification of
the candidate-linkage source passed eight targeted tests, Clippy and format at
fixed public commit `a45fd22f325463aa1753d693acac812da144fc00` on the requested
Linux server. The terminal audit verifies 1,150 distinct Git blobs, a clean
checkout and all log/test-binary hashes; receipts are in `server-linux/final/`.

The next candidate-generation checkpoint builds proof terms from every complete
scope projection and targets every original named refinement definition. Its
Boolean normalizer uses supplied ordinary equality proofs, congruence and
transitivity. It rejects unknown computations and contradictory facts; a missing
path proof fails the whole generation route. The resulting candidate still
requires both kernels and keeps application checks pending.

Congruence shifts free branch values under newly introduced binders. Context
proofs remain inside their enclosing theorem and never become open auxiliary
declarations. Expanded sharing costs saturate, and the context mode keeps sharing
disabled even at saturation. A small shared DAG exercises this boundary without
exceeding the ordinary node/binder/byte limits. Closed modes retain their small
sharing thresholds, so child costs remain below the threshold and cannot reach
the new saturation case. Their existing original ownership and closed-default
pin replays pass unchanged.

The three candidate units, two contextual Boolean units, the existing free
selector unit and both affected original consumers pass (eight unique tests,
with the consumers retained after the context-cost boundary fix). Clippy with
test targets and format pass. Both unchanged checkers accept the actual generated
small universal candidate and the true/false contextual fixtures with zero
axioms and matching hashes, and reject the wrong proof fixtures at core checking.
The shared-DAG certificate is checked separately; its terminal or pending status
is recorded explicitly in the checkpoint receipts. Linux replay of the latest
candidate-generation source is still pending.

An independent original-source capability probe reconstructs `is_binding` and
requests all seven required refinements. Generation currently returns `Linkage`,
so all seven source proofs remain pending. Its successful diagnostic process
only confirms that observation; it is not proof acceptance. The temporary probe
source is preserved with its receipt, and the original test source was restored
byte for byte afterward.

No original application refinement or execution proof is supplied by this
linkage API or its small fixtures. All 987 application proofs remain pending.
Unit 7 and W09 are In progress, W10–W12 remain Blocked, and the full T06 gate
`./scripts/check-fast.sh` is deferred to the final T06-W12 work item.
