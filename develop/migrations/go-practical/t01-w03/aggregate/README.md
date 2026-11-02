# T01-W03 coupled struct, slice-address and byte observations

W03 remains In progress. The predecessor is
`f54d8bbbbee66d00c615f9894827e2b3ab34099a`. The private
[producer](../../../../probes/go-practical/aggregate-certificates/src/aggregate.rs)
combines full value cells, actual descriptor addressing and matched bytes.
It does not implement production Go capture, admission or source lowering.

## Ordinary carriers and conditional claims

Let B be the byte bound, S the backing-storage element bound, and
P = nextPowerOfTwo(max(1,B,S)). The shared internal padding P never raises
either semantic bound. Let d = log2(P), r = max(6,d+3). A complete cell is
`Bool^2 -> Bool^r -> Bool`. Tags 00/01/10 select the Amount row, SKU-length
row and SKU-byte row. The first 64 bits of the first two rows are complete
source words; the byte row retains all eight bits at every finite address.
Every other field and padding bit remains arbitrary. A heap quantifies both
logical regions, complete 64-bit addresses and complete cells.

Computed descriptor validity, an actually defined line index and storage
length <= S derive complete physical-address reconstruction. Validity derives
no-wrap, sign, live-region and physical-storage facts; no caller fit or carry
oracle is added. Equality of complete addresses lifts through the arbitrary
heap to equality of the complete selected cell, including padding. Amount
and SKU-length projections then retain all 64 bits. A value assignment to
Amount preserves the entire SKU-length/byte subframe, including padding,
through explicit product projections. It does not mutate the input heap.

For byte claims, the original cell's actual source-shaped prefix/suffix
success and actual pattern-index guard are the premises. Source and pattern
lengths are bounded by B; their signs are derived. Whole-cell transport then
connects that original source byte to the cell read through the finite
physical-address projection. Both prefix and suffix conclude complete
eight-bit equality with the actual pattern byte. This composes the two
index spaces and both semantic bounds in one checked theorem.

The recorded finite observable footprint is `2 * P * cellBits`. It does not
claim that the original heap has only that many quantified bits: the original
64-bit-addressed heap remains arbitrary at every address. At B256/S128,
P256 and 8,192-bit cells give a 4,194,304-bit finite footprint. At B257/S129,
padding rises to P512 and 16,384-bit cells. The proof uses symbolic functions
and congruence; it does not enumerate cell or collection bit patterns.

## Workloads and Go observations

The nine B/S pairs are 0/0, 1/1, 32/8, 31/31, 32/32, 33/33, 255/127,
256/128 and 257/129. They exercise the proposed byte and slice workloads,
plus both sides of the proposed maximum boundaries. There are 98 candidates:
49 correct claims and 49 false mutations. The zero-storage workload has only
the nonvacuous value-frame pair; it has no indexed false claims with an empty
domain. Nonzero pairs add whole-cell, Amount, SKU-length and both matched-byte
families. Every false claim has a concrete source-domain witness. The bit
flips in complete cell/word claims change the low Amount or length bit; matched
byte mutations deny an actually equal pair.

The [original-Go observer](../../../../probes/go-practical/aggregate_reference.go)
uses ordinary value structs, actual slice views/reads, local Amount assignment
and the original strings package. The
[independent reference](../../../../probes/go-practical/aggregate_differential.py)
uses full64 bit-loop arithmetic and complete byte comparisons. All 732
observations match: 664 profile-domain inputs, 685 successful line reads,
38 actual index panics, nine invalid descriptors, 68 outside-domain inputs,
and nine whole-cell finite-projection counterexamples outside the domain.
The overlapping corpus has 128 matched prefix and 128 matched suffix reads,
676 negative Amount observations, 190 NUL and 366 invalid-UTF8 source values,
seven empty views, three nil views and 595 valid nonzero offsets. Actual
source success outside a profile bound remains distinct from admission.
Native allocation, recovery, I/O and reflection are observer-only.

## Reproducible extension and initial rejected positives

The [standalone crate](../../../../probes/go-practical/aggregate-certificates/Cargo.toml)
extends only generated private Rust candidate code. Frozen predecessor sources
remain unchanged. An independent
[recipe replay](../../../../probes/go-practical/aggregate_build_sources.py)
checks all 47 generated inputs. No Go package is rewritten or reduced to stubs.
The complete source and both checker identities surround each measurement;
each workload has two identical fresh candidate generations.

The [initial diagnostic](../aggregate-initial/README.md) retains exactly three
actually checked intended positives rejected at core typing by both kernels.
One untrusted proof used a negation-elimination helper in the wrong direction;
it supplied a false-comparison proof where a true-negation proof was required.
The corrected construction uses explicit checked negation congruence.
The first pointwise Amount update also lacked a checked equality between its
function-valued field expressions. Explicit product-field updates make the
projection path direct. The final symbolic claims and all source widths,
semantic bounds, checker rules and fuel limits remain unchanged. These failed
positive candidates are diagnostic evidence, not semantic-negative claims.

## Local verification

```sh
python3 -B develop/probes/go-practical/run_aggregate_probes.py --write
python3 -B develop/probes/go-practical/run_aggregate_probes.py --check
python3 -B develop/probes/go-practical/check_aggregate_report.py
python3 -B develop/probes/go-practical/check_aggregate_initial.py
PYTHONPATH=develop/probes/go-practical python3 -B -m unittest test_aggregate_report -v
```

The complete matrix passes both unchanged production checker implementations:
49 correct claims have zero axioms and matching accepted hashes; 49 false
mutations fail at core type checking. The separate final `--check` replay
passes all 98 candidates and reproduces exact canonical certificates and all
original-Go input/output bytes. The earlier replay process was interrupted;
its missing handle and absent process were verified before a fresh full replay.
The final completed replay is the evidence used here.

All 37 new targeted tests pass: 32 report/native tests (1.586 s), four
source-generation/initial-diagnostic tests (0.078 s), and the adjacent hardware
receipt test (0.012 s). The selection follows the independent producer and its
new evidence/native consumers. Existing predecessor source files, binaries,
1,665 canonical certificates, two standalone indexes and 17 complete case
indexes/outcomes remain unchanged. Their kernel measurements and unrelated
adversarial suites are not rerun. The W01 identity audit and W02 retained
39-case closure report also pass. Explicit formatting of the three current
Rust files, pinned Go formatting, changed Python syntax and offline locked
release Clippy with warnings denied pass.

The B256/S128 suffix matched-read certificate has 649 declarations, 53,098
terms and 661,754 bytes. Observed Rust/Go wall time is 5.289/11.990 seconds,
with peak resident size 69,533,696/178,896,896 bytes. The adjacent
[hardware receipt](local-resource-host.json) binds the report to the
Darwin/arm64 M1 Max host with 64 GiB of RAM and ten physical/logical cores.
Its original report model/RAM nulls are retained. The three concurrent groups
and 60-second child ceiling remain private settings, not an isolated profile
budget. These indexed laws do not establish an entire collection-program
resource guarantee or production source-to-certificate application.

The [W03 exit audit](../feasibility-exit-audit.md) identifies the remaining
ordinary loop/control induction and isolated resource review before W04.
Production bindings remain assigned to T02–T06. The complete T01 gate remains
deferred to T01-W05; no practical profile is active.
