# Full-width bounds and value-preserving finite indexes

GO-PRAC-T01-W03 remains In progress. This private ordinary model discharges
bounded-index coverage and source-shaped signed-guard agreement needed by
the [finite string-byte ingredient](../byte-tails/README.md). Complete Go
source/operator/carrier bindings, remaining string laws, slice/descriptor/
control composition and aggregate resource closure remain due. No practical
Go profile is active; the full T01 gate stays deferred to W05.

## Claims and domains

Both source lengths and indexes are universally quantified 64-bit
Bool-addressed words. Capacities `B = 0/1/32/256` are private workload probes.
The premise is the actual full-width unsigned comparison `length <= B`.
The source-shaped index condition is the two's-complement signed expression
`0 <= index && index < length`. The model proves:

- The bounded length has a zero sign bit.
- That signed index condition equals the existing byte-mask condition
  `nonnegative(index) && unsignedLess(index, length)`.
- For positive B, an index satisfying that condition has exactly the same
  complete canonical value as the zero-extended finite low-bit embedding.
- For B = 0, the source-shaped index condition is false for every index.

The [order library](../../../../probes/go-practical/certificates/src/word/balanced/signed/bytes/bounds.rs)
proves comparison trichotomy/transitivity, equality substitution, sign/order
compatibility and zero/one bounds by checked Boolean cases and congruence.
The [coverage composition](../../../../probes/go-practical/certificates/src/word/balanced/signed/bytes/bounds/coverage.rs)
uses the actual length bound and index condition to derive the finite-fit
predicate internally. No caller supplies that predicate. Checked
projections and pointwise observations then construct the full canonical
index equality. Only the six bit-address coordinates are split when
relating the existing byte-model constants and embedding; operand values
are never enumerated. The B = 1 conclusion is the complete zero index.

The finite byte store has a total low-bit projection, but source indexing
uses the separate actual validity condition. Full-value coverage justifies
that finite representation at the ordinary model level. Applying these
claims to a captured Go operation still requires the production source and
carrier bindings. No extra Go entry restriction, checker rule, axiom,
conversion fuel, general function extensionality, record eta or unfinished
C# theorem is used.

## Evidence and false mutations

The [retained report](observations.json) contains 27 candidates, with two
identical fresh generations per capacity. Both checkers accept twelve
zero-axiom claims with matching logical hashes and reject fifteen mutations
at core type checking. The mutations change a result or erase a necessary
length/index premise; no fuel failure is counted. The complete canonical
certificate bytes accompany the report. Resource observations are actual
Darwin/arm64 child wall/CPU/RSS measurements, not frozen profile budgets or
native Linux release evidence. At capacity256, the full-index coverage
candidate has 243 declarations, 26,360 terms and 316,659 bytes. Its observed
Rust/Go resources are 5.676/13.218 seconds and 37,126,144/92,520,448 resident
bytes. The host reports Darwin27/arm64 and 10 CPUs; model/RAM queries are
unavailable. Independent private replays may overlap these measurements;
they are per-child resources rather than isolated-machine guarantees.

The [original Go observer](../../../../probes/go-practical/index_reference.go)
constructs actual strings and attempts indexes, recovering bounds panics
outside the verification closure. The [exact inputs](index-input.json) and
[outputs](index-output.json) are compared with independent 64-bit bit loops.
All 116 observations match. They include 15 bounded safe and 70 bounded
unsafe cases, four valid reads outside private coverage, 30 negative-index
panics and 15 maximum-positive-index panics. Cases cover lengths at, below
and above capacity, negative and extreme indexes, endpoints and successful
reads outside each private capacity.
Observer-only `defer`/`recover` does not admit those features in v1.

Every deliberate false claim has an explicit input/output counterexample,
recorded separately from its checker rejection. For example, B = 1,
length = 2, index = 1 gives a successful Go read and a full index value
different from its low-bit zero embedding. With the index condition erased,
length = 0 and index = -1 satisfies the length bound but fails full-value
coverage. For B = 0, erasing the length bound admits the real successful read
length = 1, index = 0. Changing the asserted length sign or negating the
claimed guard result has a counterexample at length/index zero. Type
failure alone is not evidence that a claim is false.

Replay and integrity commands:

```sh
python3 -B develop/probes/go-practical/run_index_probes.py --check
python3 -B develop/probes/go-practical/check_index_report.py
python3 -B -m unittest discover -s develop/probes/go-practical -p 'test_index_report.py' -v
```

`--check` actually runs Rust and Go proof verification and reproduces exact
native observation bytes. The separate report reader checks retained
identity and scope, including the existing `MPK-MODULE-CERT-0.1` domain hash
of accepted certificate bytes. `--write` refreshes measurements. All Go
builds use the pinned Go 1.25.0 host and disable VCS stamping; the standalone
observer cannot join the W02 original-package analyzer. The tests reject
lost full-width domains, fit oracles, missing premises/counterexamples,
crossed checker results, fuel counted as semantic rejection and premature
W03/profile/budget completion.

The separate final replay passes all 27 candidates through both checkers
and reproduces the exact native input/output bytes. All 32 adversarial
evidence tests pass. The shared-producer refresh also remeasures all 432
previous candidates; their canonical bytes and complete case indexes remain
unchanged. Their 189 affected evidence tests pass. Full T01 validation
remains deferred to W05.
