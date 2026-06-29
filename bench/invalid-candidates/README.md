# Invalid Candidate Benchmark

`bench/invalid-candidates` is the ALPHA-003 invalid proof-candidate benchmark.

The corpus contains 10,000 `mpk-api` batch candidate records in JSONL format.
Each record intentionally references an unregistered proof id, so the batch
checker must reject every candidate deterministically with `UNKNOWN_PROOF`
without mutating accepted session state.

Regenerate the checked-in artifacts from the repository root with:

```sh
MPK_UPDATE_INVALID_CANDIDATES=1 cargo test -p mpk-api --test invalid_candidates_benchmark
cargo test -p mpk-api --test invalid_candidates_benchmark
```
