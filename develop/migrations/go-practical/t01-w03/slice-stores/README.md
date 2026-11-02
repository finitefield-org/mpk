# Descriptor/store sharing, array snapshots and full slice tails

GO-PRAC-T01-W03 remains In progress. These ordinary ingredients preserve
descriptor observations, shared backing and complete cell values. Complete
absolute-address and validity/access composition is recorded separately in
[the later access checkpoint](../slice-access/README.md). Ownership and
production Go source/operator bindings remain due. General signed slicing
is covered by [the later reslice checkpoint](../reslicing/README.md).
Overlapping `copy`, bounded `append`, remaining byte/control laws
and aggregate resources stay open. No practical Go profile is active; the
whole T01 gate remains deferred to W05.

## Representation and checked operations

The [private producer](../../../../probes/go-practical/certificates/src/word/balanced/signed/bytes/bounds/slices.rs)
encodes `(nil_tag, region_name, offset, length, capacity)` using ordinary
Boolean function data. Numeric fields and indices are complete 64-bit words.
The 512-bit descriptor carrier keeps all its unused packing bits quantified;
the five semantic observations include two Boolean tags and three words.
The two logical region names describe shared versus separate typed stores.
They are not Go pointer bits and do not admit pointer operations.

A cell has all 128 bits quantified, representing an ordinary pair of scalar
values. Source offset addition is the actual balanced modular64 bit adder,
with canonical operands. Heap reads canonicalize the complete physical word
key. Updates return the complete replacement cell at the matching
region/full-word key and preserve the old store elsewhere:

```text
Address(o,i)     = Sum64(Canonical(o), Canonical(i), false)
Read(H,r,i)      = H(r, Canonical(i))
Update(H,w,p,v)(r,q)
                = if RegionEqual(r,w) and WordEqual(q,Canonical(p))
                  then v else H(r,q)
```

Checked descriptor/constructor projections and read/write-definition lemmas
compose the operations. Actual whole-cell reads through two views with the
same region/offset observe a write through either view, even when their
length/capacity metadata differ. The opposite region is preserved. Both
source index guards appear in the corresponding final conditional claims;
the store algebra itself holds for its total ordinary operations.

`SliceDefined` is the local nil-and-signed-index guard. Descriptor validity
is a separate obligation: it must establish ordered nonnegative
length/capacity, a live typed region, and nonnegative offset plus capacity
within its actual storage. The present guard alone does not establish the
absolute address's validity. The initial finite-heap lift projects internal
queries through the low address bits. The later access checkpoint discharges
complete physical-index coverage from computed validity, the actual local
guard and the semantic storage bound. Production source/ownership binding
still must establish those computed conditions. No source input is truncated
or admitted by this private experiment, and no caller fit oracle closes the
source-binding obligation.

View assignment copies the descriptor and retains the region name. Full
re-slicing keeps the nil/region/offset/capacity observations and sets length
to that same capacity. A non-nil empty view may therefore expose existing
nonzero tail data. This is why the earlier unobservable string-tail mask
cannot discard slice capacity data.

Array value copying creates a separate-region snapshot of every complete
source cell. The checked snapshot-read and distinct-region frame laws prove
that updating the source array through its slice view preserves the copied
array's original value. The element model has no nested mutable references.
General slice `copy` and its overlapping-source semantics remain pending.

All laws use ordinary checked equality and Bool cases. No new axiom,
function extensionality, record eta, checker rule, conversion fuel or
unfinished C# theorem is introduced.

## Workloads and actual evidence

The [retained matrix](observations.json) covers capacities
0/1/8/32/127/128/129 and seven claim families. Empty storage has only the
three metadata families, avoiding a vacuous element-access positive. All
90 candidates have two identical fresh generations. Rust and Go accept
45 zero-axiom claims with matching hashes and reject 45 altered-result,
fresh-region-per-view, erased-separation, shared-array-copy and zeroed-tail
claims at core type checking. No current fuel outcome is counted as a
semantic negative. [Original positive type/fuel failures](../slice-initial/README.md)
remain preserved with exact sources.

Both finite stores and all padding bits remain quantified. Capacity128
keeps 32,768 backing bits; capacity129 keeps 65,536 and remains a private
boundary probe. It does not raise a Go profile limit. The capacity128
array-copy candidate has 273 declarations, 8,972 terms and 99,641 bytes.
Its observed Rust/Go resources are 0.219/0.509 seconds and
10,829,824/31,686,656 resident bytes. Actual child wall/CPU/RSS, exact
source/checker identities and construction resources are retained.
Measurements may overlap other private replays on the Darwin27/arm64,
10-CPU host; the report's sandbox metadata left model/RAM null. These are ingredient
observations rather than isolated-machine guarantees or frozen budgets.
An adjacent read-only query outside the sandbox now identifies the host as
MacBookPro18,4 / Apple M1 Max, with 64 GiB physical memory and 10 physical /
10 logical CPUs. The [exact hardware receipt](../local-resource-host.json)
retains its raw query and binds this measurement's exact report bytes. The
report's original model/RAM nulls are preserved. Refreshing it with `--write`
also requires an updated adjacent hardware receipt before its identity test.

The pinned [original-Go observer](../../../../probes/go-practical/slice_reference.go)
and [independent bit-loop/cell model](../../../../probes/go-practical/slice_differential.py)
match all 1,596 runs in the [exact inputs](slice-input.json) and
[outputs](slice-output.json). They include 28 nil and 600 non-nil empty
cases, 240 defined shared writes and array snapshots, 300 nonzero tail
observations, 514 restricted-capacity views and 1,356 rows with recovered
original-view bounds failures. Negative/extreme indexes, source offsets,
zero/full lengths, and restricted/full re-slicing are observed. A source
129-cell view really accepts index128; the private boundary is not reported
as a Go runtime panic.

The observer uses two independent backing arrays and an array value copy in
one run; each ordinary law models its two relevant region names separately.
The fixed 256-cell arrays support all private view bounds; indexing and
writes use the selected view, while observer frame comparisons inspect the
whole arrays. This does not establish a shared three-region source
binding. Observer-only allocation, I/O and panic recovery remain outside
the verification closure. These empirical results are separate from the
ordinary certificates and establish no captured Go function theorem.

Every false mutation has a concrete actual-Go/bit-loop witness. A wrong
fresh region for a copied view reads the other array instead of the newly
written cell. Erasing separation or sharing an array-copy region observes
the new value where the snapshot must retain the old one. Zeroing the tail
changes a successful read after full re-slicing. Metadata-result mutations
contradict the actual copied fields or nil/empty distinction. Type failure
alone does not establish falsity.

Replay and integrity commands:

```sh
python3 -B develop/probes/go-practical/run_slice_probes.py --check
python3 -B develop/probes/go-practical/check_slice_report.py
python3 -B -m unittest discover -s develop/probes/go-practical -p 'test_slice_report.py' -v
```

`--check` actually verifies every new canonical candidate in both checkers
and reproduces exact native bytes. The separate report reader checks source,
scope, input/candidate coverage, full domains, witness types and the accepted
certificate's existing `MPK-MODULE-CERT-0.1` domain hash. It also checks the
original diagnostic snapshots without turning resource findings into false
claims. `--write` intentionally refreshes measured reports. Go builds use
the pinned host compiler, disable VCS stamping and compile the observer's
explicit file, so it cannot join the W02 original-package analyzer.

The separate final `--check` replay passes all 90 candidates through both
checkers and reproduces exact native input/output bytes. All 40 new
adversarial evidence tests pass. They exercise source/cell/region/padding
loss, false alias/array/tail observations, nil/empty conflation, source and
checker crossing, missing or mistyped counterexamples, initial fuel/type
classification, extreme index failures and premature profile/W03 completion.
All 459 previous candidates are remeasured with the final shared producer;
their canonical bytes and complete case indexes/outcomes remain unchanged
against the predecessor checkpoint. Their 221 affected evidence tests pass,
for 261 selected tests including the 40 new tests. Unchanged expensive
linear-word proofs and unaffected production suites are not rerun. Full
T01 validation remains due in W05.
