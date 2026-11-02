# T01-W03 overlapping copy ingredients

W03 remains In progress. This is a private ordinary proof model, not a
production Go profile, captured-source theorem or permission to write through
a borrowed input. The predecessor is
`9f93641e86bccc77c7c0d38fdb48190828fd5522`.

## Operation and proof boundary

The [producer](../../../../probes/go-practical/certificates/src/word/balanced/signed/bytes/bounds/slices/access/validity/reslice/copying.rs)
constructs an explicit lambda for the heap after copying. Let `H` be the
immutable heap before the operation, `D` and `S` the destination/source
slice descriptors, and `C(x)` canonicalize all 64 source bits:

```text
m = unsigned_min(C(length(D)), C(length(S)))
relative(q) = (q - C(offset(D))) mod 2^64
H_after(r, q) =
  if r == region(D) and relative(q) <unsigned m
  then Read(H, S, relative(q))
  else H(r, q)
```

Every source read uses `H`, including when both descriptors share a backing
and their ranges overlap. A naive forward assignment loop does not implement
this relation for a right overlap. The source-after-copy heap also cannot
replace the source-before-copy heap in the resulting read equation.

Checked full-word complement/inverse/commutativity identities prove
`(offset + index) - offset = index` modulo 2^64. Constructor canonicalization
premises are discharged internally. These identities require no new checker
rule, axiom, conversion fuel, function extensionality or record eta.

Computed descriptor validity derives nonnegative lengths, including zero
length for nil descriptors. The min expression is nonnegative and bounded
by both lengths. The actual `0 <= index < m` condition then derives both
ordinary signed index guards and excludes nil on each indexed path. No
unrelated safety Boolean or caller canonicalization premise replaces them.

The heap relation yields complete 128-bit pre-state source reads at copied
destination indexes, unchanged destination reads beyond the copied prefix,
and unchanged reads throughout the other logical region. The bulk operation
returns a heap rather than changing descriptor nil/offset/length/capacity.
Actual Go observations separately check descriptor preservation. Production
source/operation/ownership bindings and aggregate closure remain due.

## Matrix and native observations

Private semantic storage bounds are B=0/1/8/32/127/128/129, with independent
padding P=1/1/8/32/128/128/256. Both logical regions and every complete
128-bit cell, including padding, remain universally quantified. Region
names are logical identifiers, not source pointer bits. Existing full-width
validity/access ingredients govern any future source-to-finite-store binding.

The matrix has 84 candidates: count bounds; destination/source definedness;
complete copied reads; destination and other-region frames; and overlapping
pre-state/source-after-copy discrimination. Each family has a correct claim
and one false mutation. B0 emits only the nonvacuous count family. B1 omits
the overlapping-source mutation because there are insufficient storage cells
to make that particular assertion false. B8/32/127/128/129 include it.

The [Go observer](../../../../probes/go-practical/copy_reference.go) constructs
fresh local arrays and invokes the actual built-in `copy`. The
[independent reference](../../../../probes/go-practical/copy_differential.py)
uses bit-loop arithmetic and an immutable pre-state cell snapshot, separately
from a diagnostic naive loop. All 341 source/model observations match:

| Observation | Count |
| --- | ---: |
| Actual source copy operations | 278 |
| Valid operations within the private semantic bound | 262 |
| Copied reads in the proof domain | 72 |
| Destination reads outside the copied prefix | 22 |
| Other-region cells checked in the proof domain | 16,595 |
| Right / left overlapping operations | 33 / 20 |
| Operations between different regions | 23 |
| Unequal-length operations | 108 |
| Nil / non-nil empty operations | 48 / 53 |
| Rows with a rejected source construction | 63 |
| Rows with a subsequent source/destination index panic | 253 |
| Constructible operations with invalid live bindings | 6 |
| Naive-forward-loop counterexamples | 12 |
| Source-after-copy counterexamples | 17 |
| Reads outside B / inside padding but outside B | 10 / 3 |
| Source reads without complete finite-address coverage | 7 |

These categories overlap. Copy itself does not panic on the nil/empty cases;
construction and subsequent index failures remain separate events. Invalid
raw descriptors and forged dead-region metadata are not admitted incoming
Go slices. Legal Go operations beyond B remain source operations rather than
invented profile-capacity panics. Native allocation, I/O, diagnostic writes
and recovery are observer-only operations. Every false proof mutation has
an explicit bounded-source witness in the retained report.

Both unchanged checkers accept 42 zero-axiom candidates and reject the 42
false mutations at core type checking, with matching accepted hashes. No
current resource failure is classified as a false claim. The B128 copied-read
candidate contains 615 declarations, 40,945 terms and 494,268 bytes. Its
observed Rust/Go wall time is 2.839/6.543 seconds and peak resident size is
66,191,360/131,547,136 bytes. Direct-child wall/CPU/RSS may overlap other
private replays; these values establish no isolated or aggregate budget.

## Local verification

The [retained observation report](observations.json) pins source inputs,
both checker binaries, raw certificate identities, resource observations
and native inputs/outputs with explicit false-claim witnesses.

```sh
python3 -B develop/probes/go-practical/run_copy_probes.py --write
python3 -B develop/probes/go-practical/run_copy_probes.py --check
python3 -B develop/probes/go-practical/check_copy_report.py
python3 -B -m unittest discover -s develop/probes/go-practical -p 'test_copy_report.py' -v
```

The runner requires two identical fresh constructions per group and sends
the same canonical bytes to independent Rust and Go checkers. An accepted
claim must have zero axioms and matching export/certificate/axiom-report
hashes. A rejected false mutation must fail at core type checking; a fuel
failure or timeout is never false-claim evidence. The retained report checker
checks evidence integrity; `--check` performs a separate actual replay and
reproduces canonical certificates and exact original-Go input/output bytes.
The 60-second direct-child ceiling is a private measurement guard, not a
frozen profile budget or an isolated machine measurement.

Debug `copy-generate` / `copy-trace` commands take
`REPO_ROOT OUTPUT CAPACITY`. Traces belong outside the current source tree.
The selected tests cover source/proof/witness identity, min versus destination
length, full-word arithmetic, nil/empty paths, overlap direction, pre-state
reads, both frames, padding admission and resource-result classification.

The shared producer records all current Rust inputs, so its new source
identity requires refreshing the 753 earlier candidate reports. Their
canonical certificates, two standalone indexes, 13 complete case indexes
and semantic outcomes must remain unchanged against the predecessor. The
unchanged expensive linear-word diagnostic proofs are not replayed.

Remaining work includes production Go capture/model/ownership/operation
bindings, bounded construction and capacity-preserving `append`, remaining
byte/control composition and aggregate boundary/resource closure. W04 has
not started. The complete T01 gate stays assigned to W05; it is not run as
an intermediate-W gate.
