# Halting Quote return-model feasibility

GO-PRAC-T01-W03 remains In progress. This is an ordinary private model and
checked arithmetic/control composition, not a Go source-to-certificate
application theorem or public profile admission. Complete source/operator,
string/slice/descriptor bindings, remaining byte/collection/control laws and
aggregate resource closure remain open. The full T01 gate is deferred to W05.

The [frame producer](../../../../probes/go-practical/certificates/src/word/balanced/signed/fold/continuation/returned.rs),
[halting fold](../../../../probes/go-practical/certificates/src/word/balanced/signed/fold/continuation/returned/run.rs)
and [root entry/composition](../../../../probes/go-practical/certificates/src/word/balanced/signed/fold/continuation/returned/quote.rs)
use only the same checked Bool/equality and natural-number foundations as
the preceding continuing-prefix measurements. No checker rule, conversion
fuel, axiom, unfinished C# theorem or source entry precondition is added.

## Model and checked conclusions

A frame contains the full 64-bit accumulator and 64 metadata bits in a
Bool-addressed cube. Observed metadata bits distinguish running, bad-input
and limit-exceeded states. Arbitrary incoming frames quantify all 128 bits;
the root constructs canonical frames. No record eta or function
extensionality is assumed. A stopped step returns the original entire frame,
so its whole-payload/metadata identity is proved, including unobserved bits.

For a running frame, the step checks the actual item's sign first, then
compares it with wrapping `limit - total`. A negative item returns the
bad-input frame; a failed remaining-bound check returns the limit-exceeded
frame; otherwise it adds the item and stays running. Both error frames have
zero total. That zero result is this Quote model's policy; it does not impose
a general zero-first-result restriction on Go multi-result/error functions.

The halting fold threads each actual low-prefix result into the high prefix.
Checked lemmas prove stopped absorption, inability to resume after a stop,
preservation of zero failure payloads, and exact successful payloads.
Success of the actual output derives success of both actual children and
every actual item check at the correct intermediate accumulator. These
checks and intermediate bounds are not supplied by the caller as oracles.
A failed low-prefix result is equal to the complete enclosing result, so
the later prefix cannot overwrite either its payload or error reason.

The root checks an abstract empty-name flag or negative limit, returning
bad-input before the fold. The empty flag is not yet bound to actual byte
length; the fixed count and store are not yet bound to a Go slice descriptor.
From actual root success alone, checked lemmas derive a nonnegative limit,
literal-zero entry, all actual item checks, a nonnegative result within the
limit, and the exact natural sum of the selected prefix. The natural theorem
quantifies an arbitrary initial natural, so it is not a ground evaluation.
From actual root failure, a separate theorem derives the zero result.

The closed-outcome theorem proves exhaustiveness and pairwise disjointness
of the three model observations. Exact entry-error and step-case equalities
retain the distinct error tags. This does not establish Go interface
admission, arbitrary `error` dispatch or a production closed-error binding.

## Workloads and evidence

The [measurement](observations.json) records exact inputs/source identities,
checker binaries, raw checker JSON, candidate bytes/hashes, two fresh
deterministic constructions and per-process wall/CPU/RSS resources. The
retained host is Darwin/arm64; the measurements are not frozen budgets.
Counts 0/1/3/8/32/127/128/129 retain all input and padded backing bits.
Count129 is an internal boundary probe, not Go capacity admission.

Each count covers full-frame absorption, zero failure result, successful
bounds, successful exact sum, entry error and closed outcome partition.
Counts above one also cover first-failure preservation across the actual
low/high boundary. Each valid claim has an altered counterpart: cleared
stop tag, arbitrary failure payload, unrelated success premise, confused
error reason, false outcome partition or overwritten low-prefix failure.
Resource failure cannot be counted as a semantic negative.

All 108 candidates have two identical fresh generations. Rust and Go accept
54 zero-axiom claims with identical export/certificate/axiom-report hashes
and reject 54 altered claims at core type checking. There are no fuel
failures in this matrix. At count128 the rooted exact-sum candidate has
1,008 declarations, 38,354 terms and 488,435 bytes, with all 8,192 backing
bits, 64 limit bits and the abstract empty flag quantified. Its retained
Rust/Go observations are 1.424/3.148 seconds and 65,372,160/136,167,424 peak
resident bytes. At count129 the padded store contains 16,384 quantified bits;
its rooted exact-sum candidate has 1,333 declarations, 45,409 terms and
587,794 bytes. These are observed child resources, not dedicated-host timing
guarantees or frozen application budgets.

The separate final `--check` replay also passes all 108 candidates in both
checkers and reproduces the exact original-Go input/output bytes. The 25
new adversarial evidence tests pass. Remeasurement of all 278 earlier
candidates preserves every canonical certificate and complete candidate
index; the 135 affected earlier evidence tests pass. Offline locked release
Clippy and private Rust formatting also pass. Unchanged production suites
are not rerun; the complete T01 gate remains due at W05.

The separate [Go input](quote-input.json) and [output](quote-output.json)
retain original-Go observations from the pinned compiler. The fixture has
no MPK dependency. These empirical observations cover success, both errors,
nil/empty input behavior and input preservation. They remain distinct from
the ordinary model certificates and supply no formal source/frame theorem.
All 769 observations match: 169 successes, 456 bad-input and 144
limit-exceeded returns. Actual failure decisions are zero, error reasons
remain distinct, and input/nil observations are preserved.

Replay and integrity commands:

```sh
python3 -B develop/probes/go-practical/run_returned_probes.py --check
python3 -B develop/probes/go-practical/check_returned_report.py
python3 -B -m unittest discover -s develop/probes/go-practical -p 'test_returned_report.py' -v
```

`--check` executes both independent checkers again and reproduces exact
candidate and Go observation bytes. The report check alone validates
retained evidence integrity. `--write` intentionally refreshes measurements.
Debug commands `returned-generate` / `returned-trace` take
`REPO_ROOT OUTPUT COUNT` and remain private feasibility tooling.
