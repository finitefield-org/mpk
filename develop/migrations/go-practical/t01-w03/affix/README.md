# T01-W03 byte prefix/suffix and selected body correspondence

W03 remains In progress. The predecessor is
`170bf04ccde2203de1d2821a3ac3021cea0805df`. This private checkpoint adds
checked byte relations and source-shaped body correspondence; it does not
register a production adapter or activate a practical Go profile.

## Relations, safety and byte observations

The [producer](../../../../probes/go-practical/affix-certificates/src/affix.rs)
retains complete 64-bit lengths/indexes and eight-bit bytes. Both input lengths
are bounded by B. Their nonnegative signs are derived from those bounds,
without an independent caller sign or index-fit oracle. Let L be source
length and N pattern length. The ordinary definitions are:

```text
prefix = N <= L && Equal(N, source clipped to N, N, pattern)
suffix = N <= L && Equal(N, source shifted by L-N and clipped to N, N, pattern)
```

Equal uses complete byte equality at every finite address with length masking.
The source-shaped formulas use the actual signed length check and the ordinary
substring/equality model. Conditional restoration/cancellation proofs derive
substring length N. The length guard derives the actual substring range
checks; its false path short-circuits without evaluating the substring.
This safety result is retained separately from value correspondence.

Both relations accept the empty pattern and reject longer patterns. A true
source-shaped result derives the actual length guard. Every defined pattern
index then maps to an actually defined source index: i for prefix and
`L-N+i` for suffix. A checked fold projection recovers complete eight-bit
equality at that index. Semantic bounds derive full64 index reconstruction
before finite indexing; merely retaining low index bits is insufficient.
Defined-source read proofs preserve masking unused input tails. No new axiom,
checker rule, fuel limit, eta or function extensionality is used.

## Exact source identities and independent Go observations

The [original-Go observer](../../../../probes/go-practical/affix_reference.go)
compiles the original `strings` and `internal/stringslite` packages. It also
evaluates literal length/slice/equality helpers. It does not manufacture a Go
package by deleting unsupported declarations or replacing source bodies.
The retained AST receipt identifies exactly four Go 1.25.0 declarations:
`strings.HasPrefix`, `strings.HasSuffix`, and their two `internal/stringslite`
callees. Whole-file and formatted-declaration hashes, signatures and qualified
symbols are checked. The pinned toolchain inventory covers the original
package context. These identities and ordinary formula proofs are private
correspondence evidence; production capture/import/adapter binding remains due.

All 5,395 observations agree with the
[independent full64/byte-loop reference](../../../../probes/go-practical/affix_differential.py).
Overlapping categories include:

| Observation | Count |
| --- | ---: |
| Prefix / suffix matches | 1,055 / 1,061 |
| Empty / longer patterns | 316 / 1,988 |
| Different prefix and suffix results | 330 |
| Defined matched prefix / suffix reads | 207 / 213 |
| Full-byte rejection despite matching low bits | 664 |
| Patterns containing NUL | 1,533 |
| Invalid UTF-8 / multibyte source inputs | 2,265 / 918 |
| Inputs outside B / finite-projection counterexamples | 8 / 16 |
| Actual source reads without complete finite coverage | 8 |

The original library calls never panic in this corpus. Unconditionally
evaluating a substring with a longer pattern does panic. Optional index
attempts have their own definedness results and are not reported as library
call panics. Native recovery, AST capture, allocation and I/O are observer-only.

## Workloads and deliberately incorrect claims

Semantic bounds B=0/1/31/32/33/255/256/257 stay separate from padding
P=1/1/32/32/64/256/256/512. Both complete finite stores, including padding,
remain quantified. The matrix has 304 candidates: 152 correct and 152 false
mutations. B0 covers range, length, body correspondence, canonicalization,
empty and matched-length families for both relations. It omits indexed
negative controls with empty domains. Each nonzero bound adds longer-pattern,
actual matched-index, full-byte and complete-coverage families.

The one-bit comparison mutation is refuted by `0x80` versus `0x00`, including
at B1. Prefix/suffix confusion is emitted only where the bound allows unequal
results; B0/B1 use different nonvacuous controls. Explicit witnesses distinguish
bounded source behavior, an erased source-success premise, optional index
panics, and a totalized model that erases the length guard. The last witness
uses zero-valued unobservable source padding: the original library returns
false, unguarded Go slicing panics, and the incorrect total model returns true.
Actual inputs outside B expose why finite projection cannot extend admission.

## Frozen predecessor and reproducible producer inputs

The standalone [crate](../../../../probes/go-practical/affix-certificates/Cargo.toml)
extends the predecessor through a deterministic private Rust build recipe.
Existing producer files and their source pins remain unchanged. Only generated
Rust candidate-producer sources are extended; no Go source is rewritten.
A separate [recipe replay](../../../../probes/go-practical/affix_build_sources.py)
checks all 46 generated inputs against the pinned predecessor, explicit
extension anchors and new source files. The report pins those inputs and both
checker binaries. Candidate generation runs twice from scratch per workload
and must reproduce exact canonical bytes.

The [initial diagnostics](../affix-initial/scope.json) retain an intended
positive rejection at core typing. Rewriting a sign subexpression before its
larger signed-order expression left a residual condition on a purported range
lemma. The stronger call correctly failed in both kernels. The correction
rewrites larger expressions first and checks the exact true range/length
targets before declaration. The [fuel diagnostic](../affix-initial/fuel/scope.json)
retains a separate positive construction that exhausted unchanged checker
fuel. An explicit checked projection equality and congruence over the fold
argument replace eager conversion of the closed fold. No checker setting or
semantic bound is changed. Neither diagnostic is false-claim evidence.

## Local verification and remaining work

```sh
python3 -B develop/probes/go-practical/run_affix_probes.py --write
python3 -B develop/probes/go-practical/run_affix_probes.py --check
python3 -B develop/probes/go-practical/check_affix_report.py
python3 -B develop/probes/go-practical/check_affix_initial.py
PYTHONPATH=develop/probes/go-practical python3 -B -m unittest test_affix_report -v
```

The first complete matrix passes both checkers: 152 zero-axiom positives
have matching accepted hashes and all 152 false mutations fail at core type
checking. No current fuel/resource outcome is counted as a false claim.
The separate final `--check` replay passes all 304 candidates through both
checkers and reproduces exact canonical certificates and all original-Go
input/output bytes. No incomplete measurement attempt remains.

All 81 new targeted tests pass: 70 report/native tests (20.367 s), ten
generation/diagnostic tests (0.103 s), and one adjacent hardware-receipt check
(0.039 s). They cover exact symbol/body identities, complete byte values,
computed guards, finite coverage, witness domains, generated-source closure,
archived failure classification and both checker identities. Eight archived
diagnostic rows pass integrity checks. Explicit formatting of four current
Rust files, pinned Go formatting, changed Python syntax and offline locked
release Clippy with warnings denied pass.

The B256 suffix matched-read certificate contains 626 declarations, 50,922
terms and 635,594 bytes. Observed Rust/Go wall time is 5.506/11.775 seconds;
peak resident size is 69,599,232/173,080,576 bytes. The adjacent
[hardware receipt](local-resource-host.json) binds this report and identifies
the Darwin/arm64 M1 Max host with 64 GiB of RAM and ten physical/logical cores.
It does not retroactively fill the report's original model/RAM null fields.

Three workload groups run concurrently within the runner. Direct-child
wall/CPU/RSS may overlap other private work; the worker count and 60-second
child ceiling are not an isolated or aggregate profile budget. Historical
predecessor proofs retain their existing execution evidence. Their inputs,
checkers and canonical artifacts are checked for preservation, so unrelated
old kernel measurements are not repeated for this independent extension.

Production capture/model/effect/operation bindings, registered exact symbol
adapters, remaining control/aggregate and complete application composition,
and aggregate boundary/resource closure remain due. W04 has not started. The
full T01 gate remains deferred to W05.
