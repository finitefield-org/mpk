# Checked candidate entry parts

The private CLI owner now checks the entry catalogue against every original
selected and transitive function contract. It generates all catalogue layouts
without a caller-supplied subset, requires the finished parts to cover those
layouts exactly once in canonical order, and checks each complete part with
the Rust kernel. Any construction, coverage, kernel or nonzero-axiom failure
prevents creation of the checked owner. Incomplete packing snapshots remain
untrusted construction data; they do not receive kernel reports. The checked
owner retains the same candidate, original source/contract and graph binding.
This checks entry mathematics, not source execution, a complete foundation,
launcher authority or an application theorem.

The original source-entry fixtures passed with every function: the zero-error
unit has 47 functions in one part, while the 70-error unit has 117 functions in
two parts covering 107 and 10 functions. All three parts passed the Rust kernel
with zero axioms. The first contains 3,199 declarations; the wide parts contain
3,830 and 3,412 declarations and are 1,350,988 and 1,324,010 bytes. The Go
checker independently accepted the same zero-error part with identical hashes.
Both wide parts are byte-for-byte identical to the original parts previously
accepted by the recorded Go checker; those Go reports and the current Rust
reports agree on module, counts and all hashes. A fresh Go recheck of both wide
parts was also started and remains in progress; it is not yet reported as a
completed run.

The later contract-coverage check retains the original selection with three
roots, all three raw sidecars and four selected/transitive functions. Its own
four original layouts match; a separate complete original project with four
layouts rejects when crossed. This additional check does not change the entry
packing or kernel-verification code exercised by the earlier full 47/117-function
tests; those unchanged tests were not repeated.

The final live Go capture and Rust observer passed with the complete original
182-package application, 5,516 source objects, 181 exports and 5,577 registered
SDK files. While both root sets were held, the observer checked the candidate's
entry part and rejected the other complete owner when crossed. The prior live
Go checker output is reusable by exact certificate bytes: the final live
certificate matches it byte-for-byte, and the Go and current Rust reports agree
on all hashes, 2,063 declarations and zero axioms. There is no user contract
sidecar in that live capture; the default `true` contract is not a user property
or program proof. Go 1.25 ran as Linux/amd64 under local emulation and Rust ran
on Linux arm64; this is not native Linux release or installed-launcher acceptance.

The original entry-fixture tests, the targeted contract-coverage control and the
complete live capture were selected for the new packing/checking and retained
graph consumer. Strict Clippy passed for the affected test and Linux observer,
including the observer's test configuration; changed-file formatting and diff
checks passed. No unrelated suite or new validation archive was added.

R01 and T02-W01 remain open and the practical profile stays inactive. The
cumulative `./scripts/check-fast.sh` gate remains deferred to final T02-W04.
