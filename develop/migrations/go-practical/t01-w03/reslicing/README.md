# General signed two-/three-index slice ingredients

GO-PRAC-T01-W03 remains In progress. These ordinary ingredients preserve
computed validity, exact capacities, source-shaped mapped index guards and
complete backing reads. Production Go capture/model/ownership/operation
bindings, overlapping `copy`, bounded construction/`append`, remaining
byte/control composition and aggregate resource closure remain due. W04 has
not started; the full T01 gate remains deferred to W05.

## Source operations and full-width arithmetic

The [model](../../../../probes/go-practical/certificates/src/word/balanced/signed/bytes/bounds/slices/access/validity/reslice/model.rs)
keeps source signed endpoints as complete 64-bit words. For an old
descriptor `(nil,region,offset,length,capacity)`, the actual guards are:

```text
two-index:   0 <= low <= high <= capacity
three-index: 0 <= low <= high <= maximum <= capacity
result:      (nil, region, offset+low, high-low, effective_maximum-low)
effective_maximum = old capacity for two-index; explicit maximum for three
```

Two-index high bounds use capacity, including high values above the old
logical length. Three-index capacity uses the explicit maximum; it cannot
retain the old capacity implicitly. Neither operation copies backing cells
or assigns a fresh region. Nil tags remain nil; a non-nil empty result
retains its non-nil tag. Equal endpoints and a result at the capacity end
are included, rather than excluded by a stronger caller condition.

The [modular arithmetic](../../../../probes/go-practical/certificates/src/word/balanced/signed/bytes/bounds/slices/access/validity/reslice/arith.rs)
proves full-word canonical sums, zero addition, commutativity, actual
subtraction carry and inverse addition. The
[associativity relation](../../../../probes/go-practical/certificates/src/word/balanced/signed/bytes/bounds/slices/access/validity/reslice/assoc.rs)
preserves the actual count of incoming/outgoing carries in six states.
Only local Boolean flags and bit addresses are split. Complete operand
values and collection payloads are never enumerated.

The [non-strict bounds](../../../../probes/go-practical/certificates/src/word/balanced/signed/bytes/bounds/slices/access/validity/reslice/numeric.rs)
include equality cases. The actual signed source checks derive endpoint
nonnegativity and unsigned ordering. Subtraction results are nonnegative
and ordered; they are not assumed to satisfy unrelated safety flags.
Checked inverse/associative identities connect the new offset-plus-capacity
to the original computed extent. Its actual carry and full sum form the
65-bit mathematical extent; this is no additional Go source integer type.

[Validity preservation](../../../../probes/go-practical/certificates/src/word/balanced/signed/bytes/bounds/slices/access/validity/reslice/preservation.rs)
consumes the old computed validity and actual range guard. Constructor
normalization facts are discharged from checked constructors and do not
become caller assumptions. Nil validity forces zero relevant endpoints;
non-nil validity retains a live typed region and derives the new offset,
length, capacity and extent conditions. No function extensionality, record
eta, new axiom, checker rule or conversion-fuel change is introduced.

## Defined mapped reads and preserved sharing

The [read composition](../../../../probes/go-practical/certificates/src/word/balanced/signed/bytes/bounds/slices/access/validity/reslice/readable.rs)
derives that every actually defined new index maps to a defined index in
the old descriptor **expanded to its existing capacity**. Mapping directly
to the old logical-length guard would reject valid tail exposure. The
relative `low+index` and full physical addresses retain all 64 bits and
derive their actual no-wrap conditions.

Checked full-word associativity and key congruence prove equality of the
whole 128-bit cell reads in the same region/backing. The
[earlier access closure](../slice-access/README.md) supplies complete numeric
coverage under the semantic storage bound. A totalized read equality alone
does not establish Go definedness, so mapped-guard claims are measured
separately. A copied-view implementation would also lose observations of
later updates through the original backing.

The proof domain bounds actual storage length by B=0/1/8/32/127/128/129.
Padding P=1/1/8/32/128/128/256 remains separate. Both complete stores and
their padding remain quantified: 32,768 bits at B128 and 65,536 at B129.
Source descriptor packing retains 512 bits; source numeric fields retain
64 and each cell retains 128. Logical regions are two names, not source
pointer bits. B0 emits validity cases only, avoiding a vacuous indexed-read
positive or an equivalent zero-capacity mutant mislabeled as false.

## Proof and actual-Go observations

The [matrix](observations.json) has 100 candidates across both source forms
and validity/capacity/mapped-guard/read families. Every group has two
identical fresh generations, and both checkers consume the same canonical
bytes. Both accept 50 zero-axiom claims with matching hashes and reject 50
false mutations at core type checking. No new fuel outcome is counted as
a semantic negative. Those outcomes remain separate from
the [initial failed-positive diagnostics](../reslice-initial/README.md).
Accepted observations bind to exact certificate bytes with the existing
`MPK-MODULE-CERT-0.1` hash domain; a timeout or fuel failure is not evidence
that a mathematical claim is false.

The B128 three-index complete-read candidate has 594 declarations,
39,715 terms and 478,655 bytes. Its observed Rust/Go resources are
2.909/6.934 seconds and 42,893,312/143,851,520 resident bytes. Construction
groups and every checker retain wall/CPU/RSS and exact source/binary
identities. These ingredient observations do not freeze aggregate budgets.

The pinned [original-Go observer](../../../../probes/go-practical/reslice_reference.go)
and [independent bit-loop model](../../../../probes/go-practical/reslice_differential.py)
match all 4,261 runs in the [inputs](reslice-input.json) and
[outputs](reslice-output.json). Counts include 4,103 successful operations,
130 operation panics, 28 construction rejections, 692 source reads and
3,411 subsequent index panics. Overlapping categories include 658
proof-domain reads, 388 tail exposures, 602 restricted three-index
capacities, 14 nil results, 1,765 non-nil empty results and 692 observed
shared-backing updates. There are 22 reads outside the semantic bound,
eight readable padding cases outside admission and 14 uncovered source
reads. None is relabeled as a Go runtime capacity failure.

Seven source panics still produce a valid **total model descriptor**: a nil
slice with negative but equal variable endpoints has zero modeled length/
capacity, yet Go rejects the source operation. Output validity alone cannot
replace the actual source guard. The observer supplies variable int64
endpoints, so these are runtime input cases rather than rejected literals.
Raw invalid parent metadata and forged dead model bindings remain separate
from legal Go allocations and subsequent source/index panics.

All 50 false mutations have explicit typed witnesses. Erasing the range
guard admits a source operation panic with an invalid computed result.
Two-index capacity incorrectly replaced by new length, or three-index
capacity incorrectly retaining old capacity, disagrees with `cap` on a
valid empty result. A mapped original-length guard is false for an actually
defined read exposed beyond the old length. Zeroing the read loses a
nonzero full cell. These source/witness domains remain distinct from a
checker type failure.

Observer-only allocation, I/O, writes and panic recovery stay outside the
proof closure. They demonstrate empirical source behavior, not an admitted
borrowed-input write permission or a captured Go function theorem.

## Reproduction and remaining gate

```sh
python3 -B develop/probes/go-practical/run_reslice_probes.py --check
python3 -B develop/probes/go-practical/check_reslice_report.py
python3 -B -m unittest discover -s develop/probes/go-practical -p 'test_reslice_report.py' -v
```

`--check` actually executes both checkers and reproduces native bytes. The
separate retained-report reader checks integrity without proof execution.
`--write` intentionally refreshes complete evidence and source identities.
Debug `reslice-generate` / `reslice-trace` commands take
`REPO_ROOT OUTPUT CAPACITY`. Keep trace outputs and source snapshots outside
the current producer source tree. Go builds use the pinned Go1.25.0 host,
an explicit observer file, `-trimpath -buildvcs=false` and downloads disabled.

Resources are direct-child Darwin `os.wait4` wall/CPU/RSS observations in
bytes, possibly overlapping private replays. They freeze no aggregate
budget and provide no installed native Linux release receipt. Production
frontend/checker, normative specification and active registry remain at W01.
The adjacent [shared-producer host receipt](../local-resource-host.json)
identifies MacBookPro18,4 / Apple M1 Max, 64 GiB physical memory and
10 physical / 10 logical CPUs. It binds the refreshed slice-store report;
individual reports retain their original model/RAM nulls.
W03 remains In progress and the full authorized 11-stage objective stays
active. The complete T01 gate remains due in W05.

All 48 new adversarial evidence tests and 307 affected previous tests pass,
for 355 selected tests. The shared producer requires all 653 prior
candidates to be remeasured; their canonical bytes, two standalone indexes
and 12 complete case indexes/outcomes remain unchanged against predecessor
`94752d34168df9aa71dc420126bb25fdf29bace5`. Historical prose resources
describe their original checkpoints; refreshed reports carry current
source/binary identities and resources. Unchanged expensive linear-word
proofs and unaffected production suites are not needlessly replayed.

Validation targets the changed private endpoint/guard/descriptor/read
composition and its actual consumers. Offline locked release Clippy with
warnings denied, explicit formatting of the 17 current changed Rust files,
pinned Go formatting and changed Python syntax pass. Archived failed
sources are preserved verbatim. W01's production/regression identity audit
and W02's retained 39-case closure check pass. The full T gate remains
deferred to W05, separately from this targeted validation.

The separate final `--check` replay passes all 100 candidates through both
checkers, reproduces exact canonical certificates and matches the retained
original-Go input/output bytes. Eight changed documents/168 local links,
balanced fences, ordered ledger sections and Git whitespace checks pass.
No incomplete measurement attempt remains. W03 stays In progress, W04 has
not started and the complete T01 gate stays deferred to W05.
