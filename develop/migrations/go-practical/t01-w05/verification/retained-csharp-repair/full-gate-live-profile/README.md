# Live source320 gate process observation

The [observation](observation.json) records one short read-only sample of the
still-running source320 test process, PID 33771, owned by Cargo 61319 and gate
55665/session 31844. The original command requested one second at one millisecond.
Only two samples per thread were emitted. All ten test-worker stacks include
the unchanged test evaluator; the main harness waits for results. This small
sample does not establish throughput, a dominant bottleneck, absence of a
deadlock, test completion or resource acceptance.

The exact sampler output is preserved without normalization in
[raw-sample.json](raw-sample.json). Recover its original UTF-8 bytes from
`raw_utf8`; the original byte count and SHA-256 are included. The sampler reports
physical footprint 54.0G and peak 58.2G. Separate live host observations record
64 GiB physical RAM, swap statistics and actual process state. These are local
macOS observations, not the native Linux installed-release resource gate.
The executable hash and evaluator source bytes match the original source320
binary/Git epoch. The evaluator's identity memo has a fixed capacity of eight;
this observation supplies no evidence of an unbounded identity cache.

The current diagnostic continues unchanged. The next complete repaired retry
will set `RUST_TEST_THREADS=4` to reduce concurrent aggregate host memory
pressure. This changes only Rust libtest scheduling. Every test, original full
bound, assertion, debug assertion, overflow check, CLI build and accept/reject
check stays required. Production/checker limits and implementation are unchanged.
The planned retry setting is not an executed or passing gate, and no performance
gain is claimed. T01 remains incomplete and T02 remains unstarted.
