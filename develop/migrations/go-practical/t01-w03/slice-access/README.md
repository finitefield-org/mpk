# Computed slice validity and complete physical-address coverage

GO-PRAC-T01-W03 remains In progress. These ordinary ingredients connect
descriptor validity and the actual local index guard to absolute-address
safety, semantic storage bounds and complete finite reads. Production Go
capture/model/ownership/operation bindings, general slicing, overlapping
`copy`, bounded construction/`append`, remaining byte/control laws and
aggregate resources stay open. No practical Go profile is active; the full
T01 gate remains due in W05.

The later [general reslice checkpoint](../reslicing/README.md) closes the
ordinary signed two-/three-index validity/capacity/mapped-read ingredients.
Production source/ownership binding and the remaining operation/aggregate
requirements still apply.

## Computed validity and actual access conditions

The [access producer](../../../../probes/go-practical/certificates/src/word/balanced/signed/bytes/bounds/slices/access.rs)
and [validity composition](../../../../probes/go-practical/certificates/src/word/balanced/signed/bytes/bounds/slices/access/validity.rs)
reuse the [descriptor/store representation](../slice-stores/README.md).
All numeric fields, storage length and indexes retain their full 64-bit
domains. A descriptor has nil/region/offset/length/capacity observations,
512 quantified packing bits and two logical region names. Cells retain all
128 bits; neither region is a source pointer operation.

For non-nil descriptors, the ordinary Boolean validity predicate computes:

```text
Live(region)
and offset >= 0 and length >= 0 and capacity >= 0 and storage_length >= 0
and unsigned(length) <= unsigned(capacity)
and Carry64(offset,capacity,false) = false
and Sum64(offset,capacity,false) <= unsigned(storage_length)
```

The last two conditions compare the actual 65-bit mathematical extent,
represented by `(computed carry, computed 64-bit sum)`, against storage.
The intermediate is not a new Go source integer type or an unrelated
caller-supplied no-wrap flag. Nil validity requires zero length/capacity;
its offset and region observations are inaccessible ghost data.

The separate local `SliceDefined` condition is non-nil plus the actual
source-shaped signed condition `0 <= index && index < length`. Its checked
composition extracts the live-region and nonnegative/order facts from
computed validity. It derives `index < capacity`, then proves that the
actual `offset + index` has no carry, is physically below storage length,
and remains nonnegative. No per-access safety or fit oracle replaces these
calculations.

An ordinary six-state relation links source/result ordering with the two
actual offset-addition carry chains. A checked local Boolean step preserves
that relation; low/high composition covers the complete 64-bit operands.
With `index < capacity` and the computed zero extent carry, it derives zero
index carry and strict order of the two resulting sums. The finite Boolean
cases concern flags and bit addresses, never enumeration of source values
or backing payloads.

Checked descriptor projections also prove that assignment copying and full
re-slicing preserve computed validity. Full re-slicing sets length to the
same capacity and retains nil/region/offset/capacity. Nil is unindexable at
every index. At semantic storage bound zero, validity makes the local guard
false. There is no vacuous zero-storage indexed-read positive.

## Semantic bounds and complete address reconstruction

The [embedding composition](../../../../probes/go-practical/certificates/src/word/balanced/signed/bytes/bounds/slices/access/validity/embedding.rs)
uses semantic bound `B` separately from power-of-two padding `P`:

| Semantic B | Padding P | Low address bits | Quantified backing bits, both regions |
| --- | --- | --- | --- |
| 0 | 1 | 0 | 256 |
| 1 | 1 | 0 | 256 |
| 8 | 8 | 3 | 2,048 |
| 32 | 32 | 5 | 8,192 |
| 127 | 128 | 7 | 32,768 |
| 128 | 128 | 7 | 32,768 |
| 129 | 256 | 8 | 65,536 |

The hypothesis remains actual `storage_length <= B`, including B127 and
B129. With the physical bound, it yields `absolute_address < B <= P`.
The [unsigned prefix reconstruction](../../../../probes/go-practical/certificates/src/word/balanced/signed/bytes/bounds/coverage.rs)
then proves equality of the **complete canonical 64-bit value** to its
zero-extended low-bit prefix. Its checked canonical-constructor lemmas
preserve all value bits without function extensionality or record eta.

Congruence at the full heap key transfers that equality to the complete
128-bit cell read. Both backings and their padding remain universally
quantified. A total finite-heap lift uses low address bits even outside its
admitted domain, so its self-equality alone would not prove full numeric
address coverage. The separate complete-value theorem closes that ordinary
gap. The false finite-read mutations zero a nonzero complete payload;
erasing a bound from a totalized self-read could leave a true statement.

These bounds are private feasibility workloads. Padded B127 storage does
not admit N128, and padded B129 storage does not admit N130 or N256. The
129 boundary does not activate a public Go capacity or freeze a budget.

## Evidence and distinct counterexample domains

The [matrix](observations.json) contains 104 candidates across nine
families: address safety, physical bounds, live region, complete value
coverage, finite reads, copy/full validity, nil exclusion and zero-storage
exclusion. Zero storage has only the four metadata/exclusion families.
Each group has two identical fresh generations; both checkers consume the
same canonical bytes. Both accept 52 zero-axiom claims with matching hashes
and reject 52 false mutations at core type checking. No new fuel failure is
counted as a semantic negative. Accepted observations bind to exact bytes with
the existing `MPK-MODULE-CERT-0.1` certificate hash domain. Core type
rejection and resource failures remain separate classifications.

The B128 complete-value candidate has 420 declarations, 21,701 terms and
248,640 bytes. Its observed Rust/Go resources are 2.657/5.028 seconds and
22,593,536/68,157,440 resident bytes. Both backings retain 32,768 bits;
B129 retains 65,536 bits including padding. Construction groups and every
checker run retain actual wall/CPU/RSS observations and executable/source
identities. These are ingredient measurements, not aggregate budgets.

The pinned [standalone Go observer](../../../../probes/go-practical/access_reference.go)
and [independent bit-loop model](../../../../probes/go-practical/access_differential.py)
retain [exact input bytes](access-input.json) and
[exact output bytes](access-output.json). They compare computed raw-model
validity, carry/sum and full physical addresses, then separately attempt
actual Go three-index slice construction and indexed reading. A malformed
raw representation is not presented as an incoming legal Go slice.

False claims have explicit witnesses in separate domains:

- Erasing descriptor validity from physical bounds admits raw length2 /
  capacity1 / storage1 / index1. The local guard is true, but the raw
  representation is invalid and Go construction panics. No legal incoming
  slice is blamed for that representation error.
- Erasing validity from live-region claims admits a forged dead-region
  binding. Go can still construct and read its real live allocation; the
  supplied model binding is false. That is not a Go index panic.
- Erasing storage bounds admits legal Go N=P+1 / index=P reads. The full
  address P differs from reconstructed zero. At B0, N1/index0 is also a
  valid Go access outside the private proof domain.
- Altered safety/copy/full/nil results and zeroed finite reads contradict
  bounded observations. Boolean witnesses must remain Boolean, rather
  than numeric zero/one.

Source construction failures, subsequent index panics, valid reads outside
the semantic bound, and those inside padding are recorded separately.
These finite observations are empirical; they establish no captured-source
function theorem. Observer-only allocation, `math/bits`, I/O and recovery
do not enter the ordinary proof closure.

All 1,307 observations match: 157 proof-domain reads, 174 actual source
reads, 98 rejected constructions and 1,035 subsequent index panics. There
are 21 raw-invalid tuples with a true local guard, seven constructible
forged dead bindings, 11 source reads outside the semantic bound, four
readable padding cases outside admission and seven source reads whose
full address differs from its reconstruction. These counts overlap where
their conditions overlap. Every one of the 52 false mutations has an
explicit witness with the correct domain and exact Boolean/value types.

## Reproduction and remaining scope

```sh
python3 -B develop/probes/go-practical/run_access_probes.py --check
python3 -B develop/probes/go-practical/check_access_report.py
python3 -B -m unittest discover -s develop/probes/go-practical -p 'test_access_report.py' -v
```

`--check` executes both checkers on every candidate and reproduces exact
native bytes. The separate report reader checks retained integrity without
proof execution. `--write` intentionally refreshes the measurements and
source identities. Debug `access-generate` / `access-trace` commands take
`REPO_ROOT OUTPUT CAPACITY`; trace outputs are kept outside current source
trees. The observer builds only its explicit file with Go 1.25.0,
`-trimpath -buildvcs=false`, local toolchains and downloads disabled.

All 46 new adversarial evidence tests pass. They cover lost source/value/
descriptor/region/padding domains, erased premises, source versus raw-model
failures, full-word versus modulo reconstruction, nil/empty boundaries,
nonzero complete cells, witness types, crossed source/checker identities,
fuel classification and premature completion. The affected 261 earlier
evidence tests also pass, for 307 selected tests.

The separate final `--check` replay passes all 104 candidates through both
checkers, reproduces their exact canonical bytes and matches the retained
original-Go input/output bytes. No incomplete measurement attempt remains.
Offline locked release Clippy with warnings denied, explicit current Rust
formatting, pinned Go formatting, changed Python syntax, documentation links
and Git whitespace checks pass.

All 549 earlier candidates are remeasured because the shared producer is
pinned by every retained report. Their canonical certificate bytes, two
standalone indexes, 11 complete case indexes/outcomes and raw hashes remain
unchanged against predecessor `f6d56227394536083e703c20101bbbe66171369c`.
Historical prose resource examples describe their original checkpoints;
refreshed reports retain the current source/binary identities and resources.
Unchanged expensive linear-word proofs and unaffected production suites are
not needlessly replayed. W01's source/regression identity audit and W02's
retained 39-case closure check pass. These targeted checks cover the actual
private producer and affected evidence, rather than the complete T gate.

Resources use Darwin `os.wait4` direct-child wall/CPU time and RSS in bytes.
Independent private replays may overlap. Model/RAM remain null in reports;
the adjacent [development-host receipt](../local-resource-host.json)
identifies the local machine separately. This is no isolated-machine,
native Linux release or frozen profile-budget claim.

No axiom, checker rule, conversion fuel, record eta, function
extensionality, smaller source domain, stronger Go entry condition or
unfinished C# theorem is introduced. Computed ordinary validity/access
closure leaves production admission responsible for establishing the same
conditions from captured Go values and ownership. W03 stays In progress;
W04 has not started, and the complete T01 gate stays deferred to W05.
