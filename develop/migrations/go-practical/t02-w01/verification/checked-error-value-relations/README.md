# Checked Go error value relations

A private graph binding now derives the existing whole-carrier relation
catalogue from the selected candidate's complete original contract domain.
The CLI checks the catalogue's full type set, nominal underlying types, error
case order and tags against that domain, then generates and kernel-checks the
nil row and every concrete case with zero axioms. Callers cannot supply a
smaller set of rows or replacement certificates. Compact receipts allow a
row to be reconstructed only when its module and certificate hash match.

The complete original nil-only and small error inputs passed: respectively
one nil row and four rows for nil plus all three concrete error types. A
different complete original domain and an out-of-range reconstruction reject.
Both checkers accepted the same bytes for the nil-only row and the small
domain's nil and last concrete rows. These checks concern the entire encoded
values, including inactive fields; they do not establish Go equality or a
source or program theorem.

The first small test incorrectly expected one concrete case from the entry
annotation and failed before checking that case: the complete source closure
also contains two boxed types. The corrected test retains and checks all three.
The first full 70-case-and-nil run passed all 71 Rust kernel checks in 2,362
seconds. Go also accepted the same nil and last concrete certificates, with
zero axioms and matching hashes. Review then strengthened the check to compare
the source error type, generated module identities and verified certificate
identity on every row. The strengthened small-domain and crossed-catalogue
controls passed, and its full 70-case rerun passed all 71 rows in 2,467
seconds. Its nil and last concrete certificates are byte-identical to the
previously Go-checked streams. A full live capture
adds an error argument, a non-pointer error type and an actual contract listing
that type to the existing 182-package application. It retains all 5,577 SDK
files, 5,517 non-export objects and 181 exports. Independent byte comparison
checks 5,515 unchanged original objects; the remaining Go file is exactly the
original plus those declarations, and the only other addition is the sidecar.
The source roots and graph remain held while the private CLI checks both nil
and the concrete case. Both Go and Rust checkers accept the exact nil and
concrete certificates with zero axioms and respective certificate hashes
`86651b5c175dd54ea89c296fab255b7506c358455f7b8c9593db5d8c53c161f7`
and `1023aec7dbcba34a3d4c5cbdc08e57ea66f67b64a47339aa399b78924c327803`.
A crossed complete graph and a different original function reject. The SDK
owner used for the separate entry check is test-only; this is local emulation,
not native Linux release acceptance.

The affected nil/small tests, complete live capture, both checkers and the
source comparison exercise whole-domain coverage and the graph-bound consumer.
Go vet, changed-file formatting and strict affected Rust/observer Clippy,
including the test observer, passed. The pre-existing call-free and
postcondition-overlay cases also passed full-input reruns after the test-only
error mode was added; their certificates match the previously Go-checked bytes.
No validation archive is committed.

R01 and T02-W01 remain open; installed/launcher authority, source execution
and the complete foundation remain unproved. The practical profile is
inactive, and the cumulative `./scripts/check-fast.sh` gate is deferred to
final T02-W04.
