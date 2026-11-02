# Complete live capture with closed error dispatch

The existing 182-package test application is captured with an added error type,
an entry that calls `error.Error()`, and a source contract limiting the error
input to that type. The original 5,515 other non-export objects remain
byte-identical; the Go file is exactly the original plus those declarations,
and the only other addition is the contract sidecar. All 181 exports and all
5,577 registered SDK files remain in scope.

With the original roots and module graph retained, the private combined owner
checks the selected entry, its one resolved concrete `Error` method and the
single dynamic call site. The call target's type and original source location
must match the closed error domain. Both the held-root owner and the test-only
SDK owner check entry, call and all nil/concrete value relations. A different
complete graph and a different original function reject.

Rust and Go checkers accepted the same certificate bytes with zero axioms:

| Component | Declarations | Certificate hash |
| --- | ---: | --- |
| Entry | 2,118 | `5f1b34ad42a2df0d22a9dd58f25f1e0a8420442ab4e63a70b40925b067a1a1b5` |
| Call interface | 2,133 | `32ddab0593cbb63c0fbf9f1a9e19199f487f64b91cac24d647e09234bbec3bb0` |
| Nil value relation | 149 | `fde86c02d16cfdc6a39fc054c4abfc7c1d0f2fe988a363cd015e4ae169d74e9d` |
| Concrete value relation | 153 | `9445a5359f7fafd81d3b1b4e72e84e93378331d7be71629f49b051c1636ab903` |

This checks mathematical interfaces and source-derived identities. It does
not prove the nil guard, method execution, a postcondition, source/IR
correspondence, a complete foundation or launcher authority. The SDK owner is
test-only; the Go frontend ran under local Linux emulation, not native Linux
release acceptance.

The full capture, source comparison and both checkers were selected to cover
the new nonempty error-dispatch closure. Go vet, changed-file formatting and
strict affected Linux observer Clippy, including its test configuration,
passed. The complete 70-case regression also passed; the affected existing
static-call case remains in progress. No validation archive is committed.
R01 and T02-W01 remain open, the practical profile remains inactive, and the
cumulative `./scripts/check-fast.sh` gate is deferred to final T02-W04.
