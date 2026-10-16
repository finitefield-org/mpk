# T01-W03 fresh construction and capacity-preserving append

W03 remains In progress. The predecessor is
`dc9a5a9c056f21c9c8f15694774b3937c193661a`. These are private conditional
ordinary proofs, not a public Go profile or completed production source/
ownership bindings.

## Construction and initialized capacity

The [construction producer](../../../../probes/go-practical/certificates/src/word/balanced/signed/bytes/bounds/slices/access/validity/reslice/copying/construction.rs)
retains full64 length/capacity operands, 512-bit descriptors and 128-bit
closed two-field value cells. Semantic workloads B=0/1/8/32/127/128/129 stay
separate from padding P=1/1/8/32/128/128/256. Both logical stores and every
padding cell remain quantified. Logical region names are not source pointer
bits, and no heap/region admission limit is frozen here.

The actual signed size checks require `0 <= length <= capacity`. Admitted
workloads also establish the private capacity/resource bound. The descriptor
uses a non-nil tag, zero offset and the computed source length/capacity;
zero-length construction therefore remains distinct from nil. The heap
lambda initializes **every element within capacity** to the full 128-bit
zero value. Initializing only the logical length would lose values observable
through a subsequent capacity-based re-slice. Cells outside the new region's
allocated capacity retain their old abstract values.

The new live mask sets the allocation's region to live. A fresh allocation
requires the **computed** old mask at that region to be false. For any old
live query region, equality with the allocation region contradicts that
freshness fact. Checked Boolean equality/congruence then proves that every
such old region's complete cell is unchanged. No free `NoAlias`/frame Boolean
replaces this derivation. Observer allocation failures and process OOM do not
become a source theorem; the source correspondence is scoped to the bounded
allocation workload and its declared resource exclusions.

## Append without reallocation

Let `H` be the heap immediately before append, and `D` and `S` the destination
and source descriptors. `C` canonicalizes all source-word bits:

```text
new_length = Sum64(C(length(D)), C(length(S)), 0)
no_reallocation = not Carry64(C(length(D)), C(length(S)), 0)
                  and new_length <=unsigned C(capacity(D))
start = Sum64(C(offset(D)), C(length(D)), 0)
end   = Sum64(C(offset(D)), new_length, 0)

H_after(r, q) =
  if r == region(D) and start <=unsigned C(q) <unsigned end
  then Read(H, S, (C(q) - start) mod 2^64)
  else H(r, C(q))
```

These are absolute element indexes, not machine addresses. Existing computed
validity and the actual capacity condition derive nonnegative result length,
validity, actual carry bounds and the physical interval. The returned
slice preserves the nil tag, region, offset and capacity while changing its
length. Appending zero elements preserves an incoming nil tag.

Each actual source index maps to `old_length + index`, which is proved
actually defined in both the result and the saved full-capacity destination
view. Checked association, monotonicity and cancellation connect that
physical cell to the source's **pre-append** 128-bit value, including overlap.
A saved original destination descriptor retains its old length. Its old
prefix is unchanged; saved full-capacity aliases see new elements. Capacity
tails after the result length, other regions and every actual query outside
the computed physical interval remain unchanged. A totalized read equation
alone does not discharge these source index conditions.

Insufficient existing capacity is a profile-domain obligation. Go may
successfully reallocate; it is not classified as a Go panic. Reallocating
append and its runtime growth policy remain deferred by the design. The
native reference checks the language's capacity bounds after growth without
guessing its exact policy, while retaining exact observed bytes for replay.

## Preallocate then append borrowed input

The [composed proof](../../../../probes/go-practical/certificates/src/word/balanced/signed/bytes/bounds/slices/access/validity/reslice/copying/construction/made/composed.rs)
connects the two operators at a common bounded span. Setting a new live bit
preserves an old source descriptor's computed validity. If that source index
is actually defined, its old region is live and the fresh-region theorem
proves that allocation preserves its complete value. The checked append
lemma then returns the value from **before either operation**.

Every formerly live region is preserved across the sequence, and empty
preallocation followed by empty append remains non-nil. The no-reallocation
condition is the actual computed expression for the constructed destination
and source. Production source/effect capture still has to prove generated
obligations; no borrowed-write permission or complete filter-loop theorem
is inferred from this conditional composition.

## Matrix and original-Go observations

The matrix has 276 candidates: 138 correct and 138 false mutations. Its
families cover make validity/nil/initialized capacity/fresh frames;
append metadata, result/alias indexes, pre-state reads, prefix/tail/other/
physical frames and overlap; and composed validity/nil/read/borrowed frames.
Each group is constructed twice from scratch and must be byte-identical.
B0 omits indexed/frame families without an observable bounded-source
counterexample. B1 omits post-state-source substitution, which needs multiple
overlapping cells to be false. Empty-domain true assertions are never
labeled semantic negatives.

The [original-Go observer](../../../../probes/go-practical/construction_reference.go)
and [independent bit-loop reference](../../../../probes/go-practical/construction_differential.py)
agree on 2,806 cases. Key overlapping categories are:

| Observation | Count |
| --- | ---: |
| Successful make / make panics | 758 / 170 |
| Zero-capacity make / reads of initialized capacity tails | 67 / 211 |
| Make queries of old live regions / invalid fresh bindings | 187 / 6 |
| Make reads beyond B / uncovered zero-valued reads | 76 / 28 |
| Successful standalone append / valid bounded no-reallocation operations | 796 / 432 |
| Successful standalone append with reallocation | 354 |
| Standalone nil / non-nil empty append results | 44 / 84 |
| Bounded appended reads / saved-full alias reads | 128 / 128 |
| Old prefix / unused-capacity reads | 60 / 36 |
| Bounded physical frame queries | 141 |
| Right / left overlapping append observations | 150 / 50 |
| Post-state / naive-forward-loop counterexamples | 60 / 40 |
| Standalone operations with invalid live bindings | 12 |
| No-reallocation source reads outside B | 10 |
| Rows with a rejected source construction | 126 |
| Successful composed operations / bounded copied reads | 949 / 58 |
| Composed old-live frame queries / non-nil empty results | 126 / 119 |
| Successful composed reallocation / invalid bindings | 350 / 12 |

All 138 false mutations have explicit typed witnesses. The domains distinguish
bounded-source examples, actual make panics, forged fresh-region bindings
and successful source reallocation outside the no-reallocation condition.
Native recovery, I/O, allocation, seeding and alias probes are observer-only.
Alias probes use writes/restoration of whole values rather than source
pointer bits. Equal zero values beyond the bound do not establish full
address coverage or increase admission.

The reference initially predicted a full-view index attempt after an invalid
parent descriptor failed construction. It now gates that observation on the
original construction succeeding; the raw invalid nil metadata remains in
the corpus with neither a read nor a read panic. A selected regression test
protects that distinction. Explicit no-reallocation cases outside B were
added rather than allowing an empty coverage category. Neither correction
is a kernel rejection or false-claim proof result.

Both unchanged checkers accept all 138 correct candidates with zero axioms
and matching accepted hashes, and reject all 138 false mutations at core type
checking. No current resource failure counts as a semantic negative. The B128
composed-read candidate has 641 declarations, 44,978 terms and 545,819 bytes.
Its observed Rust/Go wall time is 4.448/10.922 seconds and peak resident size
is 64,831,488/139,575,296 bytes. These are direct-child observations with
possible overlap, not isolated or aggregate profile budgets.

## Local measurement and validation

The [retained report](observations.json) pins sources, both checker binaries,
raw certificate identities, native bytes, witnesses and resource observations.

```sh
python3 -B develop/probes/go-practical/run_construction_probes.py --write
python3 -B develop/probes/go-practical/run_construction_probes.py --check
python3 -B develop/probes/go-practical/check_construction_report.py
python3 -B -m unittest discover -s develop/probes/go-practical -p 'test_construction_report.py' -v
```

Initial measurement and the separate final replay pass all 276 candidates
in both checkers, reproducing exact certificates and native input/output
bytes. All 473 selected tests pass (70 new and 403 affected prior tests). The runner measures three
independent capacity groups concurrently, orders retained results by the
canonical matrix and atomically records incomplete attempts. This concurrency
is local to the runner, not a statement about total machine load. Direct-child
wall/CPU/RSS may overlap other private work. The 60-second child ceiling is a
measurement guard, not an isolated machine or profile budget.

A correct claim must pass both unchanged checkers with zero axioms and matching
accepted hashes. A false mutation must fail at core type checking. Resource
failures are never false-claim evidence. Retained checks validate integrity;
`--check` actually replays both checkers and exact native input/output bytes.
The 70 new tests cover size checks, nil/empty tags, initialized tails, computed
freshness, metadata, both overlap directions, saved aliases, all frames,
growth-policy scope, failed-parent observations and evidence/witness identity.

The 837 predecessor reports are refreshed because the shared producer pins
all current Rust input bytes. Their canonical certificates, two standalone
indexes and 14 complete case indexes/outcomes remain unchanged against the
predecessor. The adjacent hardware receipt is refreshed to the current
slice-store report without replacing its original model/RAM nulls. Unchanged expensive linear-word diagnostic proofs are not rerun.
Full production capture/model/ownership/operation bindings, remaining byte/
control and complete application proofs, and aggregate boundary/resource
closure remain due. W04 has not started. The full T01 gate remains deferred
to W05; no intermediate-W full gate is run.
