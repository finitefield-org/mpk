# Finite string bytes and unobservable tails

GO-PRAC-T01-W03 remains In progress. This experiment establishes ordinary
canonical-index, finite byte-observation and string tail laws. It does not
publish a Go profile or prove a captured Go function. Complete Go source/
carrier/operator bindings, remaining byte laws, slice/descriptor/control
composition and aggregate resource closure remain due. The full T01 gate
stays deferred to W05.

The [private producer](../../../../probes/go-practical/certificates/src/word/balanced/signed/bytes.rs)
uses the existing checked Bool/equality and genuine-natural foundations.
No new axiom, checker rule, conversion fuel, record eta or function
extensionality is introduced. It does not depend on unfinished C# proofs.

## Full source words and complete finite backing

Source lengths and indices retain all 64 value bits. A word is a Bool-addressed
cube; treating such a function's syntax as an index identity would be
unsound. Checked canonical-word projections, bit observations, idempotence
and congruence construct a canonical value from those bits. Pointwise bit
equality therefore gives equality of the canonical constructors. A byte read
uses that canonical address, proving value-based index coherence without
assuming general equality of pointwise-equal functions.

The generic store interface is a total function from canonical word values
to complete eight-bit values. Final claims instantiate that interface with
a complete finite Bool-addressed backing: 8, 8, 256 and 2,048 quantified bits
at logical capacities 0, 1, 32 and 256. The zero-capacity carrier retains one
ignored byte; both logical lengths are constrained to zero. Comparisons
quantify two independent complete stores. Payloads and padding are never
grounded, reduced to one bit, or enumerated.

The finite lift gives a total model projection through the low address bits.
Its checked read law preserves every byte bit. This totalization is **not**
permission for an out-of-range Go index: the separate defined-index theorem
requires the actual nonnegative-and-less-than-length guard. Under the
explicit length bound, the production binding still must prove that its
source-defined index is covered by the finite backing and that its guard
agrees with the captured Go operators.
The empty workload intentionally has no defined-index positive, avoiding a
vacuous indexing demonstration. Complete production admission and binding
of the guard and length/capacity relation remain pending.

## Tail normalization and observations

For a backing `s`, length `l` and source index `i`, the model defines

```text
Read(s, i)       = s(Canonical(i))
Active(i, l)     = nonnegative(Canonical(i)) and unsignedLess(Canonical(i), l)
Read(Mask(l,s),i)= if Active(i,l) then Read(s,i) else zeroByte
```

Checked canonical-index idempotence discharges the repeated canonical read
in that last equation. A checked typed conditional lemma proves that masking
twice has the same byte observations as masking once. Defined indexing
preserves the original full byte, using the actual guard as its premise.
Replacing that premise with an unrelated true equality rejects.

String equality observes both complete lengths and every active byte.
Bytewise order composes continuation functions in source index order:
different active bytes decide by unsigned eight-bit order, the shorter
active sequence precedes the longer one, and equal active bytes continue.
The fold is balanced and its congruence consumes checked pointwise byte/
continuation equalities. It never requires equality of the backing functions
or enumerates their contents. Thus whole equality and ordering observations
are unchanged by tail normalization, including empty and equal-prefix
strings. This proves those ordinary observer laws, not all Go string
operations or a production source/model equivalence theorem.

Remaining byte obligations include full representation validity/source
binding, literal/length/content relations, substring and concatenation,
prefix/suffix relations and their exact-symbol library bindings. Slice
tails are deliberately outside this canonicalization result: re-slicing
can expose data beyond logical slice length.

## Measured proof and original-Go evidence

The [retained report](observations.json) covers six families at capacities
0/1/32/256, excluding empty defined-index cases. All 46 candidates have two
identical fresh generations. Rust and Go accept 25 zero-axiom claims with
matching export/certificate/axiom-report hashes and reject 21 altered
payload/guard/index-coherence/comparison claims at core type checking. There
are no fuel failures in this matrix. A separate `--check` replay repeats
both checkers and reproduces all candidate and original-Go observation bytes.

The capacity256 ordering claim has 150 declarations, 2,811 terms and 31,722
bytes. Its retained Rust/Go observations are 0.357/0.857 seconds and
5,439,488/14,057,472 peak resident bytes on Darwin/arm64. Construction and
every checker's wall/CPU/RSS and exact source/binary identities are retained.
These are ingredient measurements, not a frozen application budget or
native Linux release evidence.

The [ordinary Go observer](../../../../probes/go-practical/byte_reference.go)
constructs strings from exact arbitrary bytes and separately observes logical
and padded equality/order, defined indexing and unchanged inputs. The
[exact input](byte-input.json) and [output](byte-output.json) cover 2,791
cases, including NUL, invalid UTF-8, multibyte text, empty/equal-prefix/full
strings and negative/extreme/end-point 64-bit indices. An independent
bit-loop model matches all observations: 646 defined and 2,145 undefined
indices, 224 equal and 1,249 less-than results. In 138 cases equal strings
have different padding; in 337 cases comparing padding would change the
observed order. These observations are empirical and distinct from the
ordinary certificates; they establish no captured-source function proof.

The observer actually attempts each Go index and recovers its bounds panic
outside the proof closure; the undefined cases are not merely a repeated
source guard. This observer-only recovery does not admit `defer`/`recover`
in the practical v1 verification closure.

Replay and integrity commands:

```sh
python3 -B develop/probes/go-practical/run_byte_probes.py --check
python3 -B develop/probes/go-practical/check_byte_report.py
python3 -B -m unittest discover -s develop/probes/go-practical -p 'test_byte_report.py' -v
```

The report check validates evidence integrity without proof execution.
`--write` intentionally refreshes actual measurements. Debug commands
`bytes-generate` / `bytes-trace` take `REPO_ROOT OUTPUT CAPACITY`. All Go
builds disable VCS stamping; the observer builds only its explicit source
file, so it cannot join the original-package W02 analyzer.

The integrity reader also recomputes the existing `MPK-MODULE-CERT-0.1`
domain hash over each accepted candidate's exact retained bytes. Matching
Rust/Go results from a different candidate cannot be substituted merely
because their module and declaration count match. The separate replay still
executes proofs; the integrity check is not a substitute for that execution.

Two acceptances are one-cell controls. At capacities 0 and 1 a total model
read returns the same byte without any index relation. The [original wrong
witnesses](../byte-initial/README.md) failed type checking even though their
claims are true; they are not false-claim evidence. The corrected witnesses
use reflexivity, and the other 44 candidate bytes/case rows remain unchanged.
