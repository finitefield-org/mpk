# T01-W03 sequential representative resource envelope

Predecessor checkpoint: `b6b283b0630314883403bfa159a60c82ff0eabfc`.
This measurement gives W04 exact observed inputs for its limit/budget freeze.
The complete T01 gate remains deferred to W05; no public profile is activated.

## Selection, scope and results

The [runner](../../../../probes/go-practical/measure_sequential_resources.py)
starts one direct child at a time. Fourteen selected positives cover upper
workloads and plus-one boundaries: halting exact sums at 128/129 elements,
three-index reads and overlap snapshots at 128, borrowed-input construction
at 128/129, substring/concat and affix reads at 256/257 bytes, coupled
struct/slice/byte reads at 256/128 and 257/129, and arbitrary-Nat invariant and
stopped-frame schemas. Each candidate already has a complete matrix and
separate proof replay; this measurement targets unresolved resource questions
rather than repeating every old kernel case.

Each fresh construction generates its complete candidate group and shared
library. That cost is not reported as an isolated single-theorem producer.
The selected canonical bytes must match the retained original. Rust and Go
then verify that selected certificate and reproduce its original accepted
hashes with zero axioms. Twelve rows use unchanged production checker
implementations; two explicitly use the private proposed Nat.induct extension.
Binary identities are checked against each original report before execution
and again afterward. All source, report and artifact identities remain fixed.

All fourteen measurements pass. Component maxima can come from different rows:

| Observed quantity | Maximum |
| --- | ---: |
| Complete-group construction wall time | 7.147 s |
| Rust checker wall time | 5.666 s |
| Go checker wall time | 13.009 s |
| Complete-group construction peak RSS | 466,616,320 bytes |
| Rust checker peak RSS | 72,499,200 bytes |
| Go checker peak RSS | 188,055,552 bytes |
| Canonical certificate size | 698,048 bytes |
| Certificate terms / declarations | 55,744 / 1,333 |

[The raw observations](observations.json) retain per-row CPU/wall/RSS,
certificate/term/declaration counts, complete checked case metadata and both
verifier reports. 195 source inputs and 22 predecessor artifact references
are pinned. The [identified hardware](../control-induction/local-resource-host.json)
is the Darwin/arm64 M1 Max host with 64 GiB RAM and ten physical/logical cores.
Other host processes remain uncontrolled. Sequential children do not imply
an isolated machine, native Linux release acceptance, or a resource guarantee
for every Go source program at these bounds.

These measurements support retaining byte/storage candidates 256/128 for
W04's exact freeze. The 257/129 cases are boundary probes and never extend
admission. W04 must separately fix transport/term/declaration caps, per-child
resource budgets and their production executors. A resource timeout remains
pending verification, not a source panic, proof of falsehood or success.
T02–T06 still own production capture/source/operation and actual application
composition; their final application gates must meet the frozen limits.

## Local checks

```sh
python3 -B develop/probes/go-practical/measure_sequential_resources.py --write
python3 -B develop/probes/go-practical/measure_sequential_resources.py --check
PYTHONPATH=develop/probes/go-practical python3 -B -m unittest test_resource_envelope -v
```

`--write` performs the selected fresh constructions/checks sequentially.
`--check` validates retained resource/evidence identities; it does not repeat
timing measurements or stand in for a new proof replay. All eighteen selected
resource-evidence adversarial tests pass in 2.454 seconds. They reject crossed
sources/artifacts/binaries, missing upper-bound rows, mixed private/production
checker scope, misreported construction cost, accepted axioms/hashes and
invented maxima or budgets. Unchanged production suites and unaffected old
adversarial tests are not rerun. Three new Python sources and the changed
document/JSON/whitespace checks cover this measurement-only change.
