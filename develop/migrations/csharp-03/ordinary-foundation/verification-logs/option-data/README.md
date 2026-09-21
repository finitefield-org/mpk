# Option data scoped verification

All twelve original-source certificate pairs pass the ordinary Rust/Go checkers
with zero axioms and hash-corruption rejection. All twelve runtime contexts
pass (177 result observations), including the user-run nested-string value-or
partition: 235.32 test seconds, 235.39 seconds wall. That partition remains
user-owned on future affected runs because it exceeds one minute.

`user-long-completed/` retains the original log, manifest and receipts. Import
verified all current source/input/certificate hashes without rerunning tests.
`verification.json` binds this evidence and the earlier short-check receipts.
`pending-long-checks.json` now has empty checker and runtime lists; the runner
reports no pending checks. Its former one-case manifest is preserved in the
completed directory. The initial bounded timeout remains diagnostic only.

The initial candidate generation used the first test build; the later successful
runtime partitions also regenerate and compare all twelve final fixture pairs.
The preserved sixteen outcome and seven source-value predecessor candidates
regenerate byte-identically. Clippy, affected formatting and inventory passed.
No unrelated/full gate ran; W09 and internal unit 5 remain incomplete.
