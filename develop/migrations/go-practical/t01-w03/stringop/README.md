# T01-W03 guarded byte substring and bounded concatenation

W03 remains In progress. The predecessor is
`7eddc34ac28003c78eddce2223bf267bfba39bfc`. These are private conditional
ordinary proofs; production source/effect/operation bindings and the public
profile remain unchanged.

## Source-sized guards and complete bytes

The [operation producer](../../../../probes/go-practical/certificates/src/word/balanced/signed/bytes/bounds/slices/access/validity/reslice/stringops.rs)
retains all 64 source length/index bits and all eight bits of every byte.
For a source store S, actual signed bounds `0 <= lower <= upper <= length`
derive a nonnegative result length `upper-lower`, its bound by the old length,
and the actual old-source index `lower+i` for every defined result index i.
The result's full byte equals the old source's full byte at that index.
Subtraction, addition, restoration and order proofs are ordinary checked
identities; a free carry or index-fit assumption does not replace them.

Concatenation's length is the computed full64 sum of the two source lengths.
Their nonnegative signs imply no unsigned carry. An explicit result-bound
obligation derives the result's nonnegative sign and makes its bounded
carrier available. Every left source index remains defined; every right
source index maps to `left_length+i`. Addition/subtraction cancellation and
actual index checks establish both full-byte content equations. The result
bound is a required profile obligation, not a conclusion from two independent
input bounds and not a Go panic. The separate `concat-length` candidate
retains that exact obligation explicitly rather than claiming it is automatic.

Substring and concatenation defined-result reads are unchanged when unused
input tail bytes are zeroed. This follows from their actual source-index
relations and the checked byte-tail law. Equality of arbitrary total store
functions is not assumed. The pinned Go 1.25.0 specification/compiler inventory
is retained by the report; native operations are observed using that compiler.
This does not establish a production frontend's source/model linkage.

## Semantic bounds and finite backing

| Semantic bound B | Padding P | Byte bits in each quantified store |
| ---: | ---: | ---: |
| 0 | 1 | 8 |
| 1 | 1 | 8 |
| 31 | 32 | 256 |
| 32 | 32 | 256 |
| 33 | 64 | 512 |
| 255 | 256 | 2,048 |
| 256 | 256 | 2,048 |
| 257 | 512 | 4,096 |

All padding bytes and both concatenation stores remain quantified. Actual
semantic guards/bounds derive equality between the complete canonical source
index and its zero-extended finite reconstruction. Padding never raises the
admitted bound. No payload enumeration, checker rule, axiom, eta, function
extensionality or fuel change is used.

The matrix has 248 candidates: 124 correct and 124 false mutations. B0 has
five length/carry families and omits indexed false claims with empty domains;
each nonzero bound has 17 families. The latter add actual index lifting,
complete source/result finite-index coverage, full-byte reads and input-tail
preservation for substring and both concatenation arms. Each group is built
twice from scratch and must be byte-identical.

All false claims have explicit witnesses. Erasing the substring range guard
allows negative source endpoints whose raw modular difference exceeds B;
the actual source panics. Erasing a nonzero concatenation output bound allows
two bounded inputs whose combined Go string succeeds with length B+1.
Other witnesses remain within the bounded source domain, including B0's
nonvacuous length/carry controls. A checker resource failure is never evidence
that a claimed semantic equation is false.

## Original Go and independent reference

The [native observer](../../../../probes/go-practical/stringop_reference.go)
agrees with all 1,957
[independent full64 bit-loop and byte-loop cases](../../../../probes/go-practical/stringop_differential.py).
Categories overlap:

| Observation | Count |
| --- | ---: |
| Successful substring / substring panic | 902 / 1,055 |
| Empty substring / defined substring reads | 290 / 206 |
| Index panics after successful substring | 696 |
| Substring input-tail preservation | 902 |
| Successful concatenation / empty concatenation | 1,957 / 34 |
| Bounded inputs whose concatenation exceeds B without panic | 872 |
| Left / right source reads preserved by concatenation | 696 / 662 |
| Concatenation input-tail preservation | 1,957 |
| Invalid UTF-8 substring / concatenation containing NUL | 360 / 729 |
| Source reads outside B / without complete finite-index coverage | 8 / 3 |
| Minimum / maximum signed64 endpoint rows | 198 / 264 |

Strings contain arbitrary bytes, so invalid UTF-8 is preserved and slicing
may split a multibyte encoding. A byte length is not a character count.
Failed substring construction causes neither a subsequent index read nor a
subsequent index panic. Observed allocation, I/O and recovery remain outside
the proof closure. The small native workloads do not prove process allocation
availability or a theorem for strings of arbitrary machine-sized length.

## Local validation and remaining scope

The [retained report](observations.json) records source/checker identities,
canonical certificates, false witnesses and original-Go input/output bytes.

```sh
python3 -B develop/probes/go-practical/run_stringop_probes.py --write
python3 -B develop/probes/go-practical/run_stringop_probes.py --check
python3 -B develop/probes/go-practical/check_stringop_report.py
python3 -B -m unittest discover -s develop/probes/go-practical -p 'test_stringop_report.py' -v
```

The initial full matrix passes both unchanged checker implementations: 124
zero-axiom accepted candidates with matching accepted hashes and 124 false
mutations rejected at core type checking. The B256 concat-right-read example
has 550 declarations, 36,360 terms and 436,117 bytes. Observed Rust/Go wall
time is 4.718/7.703 seconds and peak resident size is
39,026,688/116,948,992 bytes. These are overlapping direct-child observations,
not isolated or aggregate budgets.

Initial measurement and a separate final `--check` replay pass all 248
candidates in both checkers and reproduce exact canonical certificates and
original-Go input/output bytes. All 537 selected tests pass: 64 new and 473
affected prior evidence tests, executed in disjoint batches. The new tests
cover actual range/index conditions, source/result bytes, NUL/invalid UTF-8,
output-bound versus source-panic domains, finite coverage, failed-parent
index attempts, both checker identities and evidence scope.

Shared-source pins require refreshing all 1,113 predecessor candidates. Their
canonical certificates, two standalone indexes and 15 complete case indexes/
outcomes remain unchanged. Production frontend/checkers, normative
specifications and registry remain at W01. The adjacent hardware receipt is
refreshed to the current slice-store report hash, preserving its original
model/RAM nulls. Unchanged expensive linear-word diagnostic proofs, unrelated
production suites and unchanged W01/W02 adversarial helpers are not rerun.
Offline locked release Clippy, current-file Rust formatting, pinned Go
formatting, Python/strict-JSON syntax and documentation/whitespace checks pass.
No current incomplete measurement attempt remains.

Three independent workload groups run concurrently within the new helper;
canonical case order and atomic incomplete-attempt records are retained.
Direct-child wall/CPU/RSS may overlap other private work. Neither the worker
count nor the 60-second child ceiling is an isolated or aggregate profile
budget. Retained checks validate integrity; `--check` executes both checkers
and reproduces exact canonical certificates and native bytes.

Production source/model/effect/operation bindings, checked prefix/suffix and
their exact library/helper binding, remaining control/aggregate and complete
application composition, and aggregate resource boundaries remain due.
W03 stays In progress, W04 has not started, and the full T01 gate remains
deferred to W05.
