# Candidate-bound original call interfaces

A bound candidate and its original contracts can now derive the existing
ordinary invocation catalogue while retaining their owners; the graph-bound
form also retains the independently checked module graph. Call sites and their
finite error dispatch come from the original selected/transitive source closure.
Their source layout uses the complete contract value domain, including types
mentioned only in annotations, rather than silently reverting to the smaller
pre-annotation domain. The selection, entry constraints and complete error
cases must agree across the two domains. Catalogue rows remain untrusted
mathematical candidates and do not establish callee execution or a program
proof.

The complete original selection test retains three selected roots, their fourth
transitive function, all three raw overlays and the original 8,192 unselected
boundary declarations. Its actual cross-package call from a selected root to
its original helper yields one call interface. Rust and Go both accept the same
480,845-byte interface certificate with 2,091 declarations, zero axioms and
matching hashes. The certificate hash is
`9838a820776a5d5a62f79d589127531704f0bd6b99c2931b3fa16ba4ffd60f9f`.

A separate original-source contract case refers to `ContractOnly`, a named
type not needed by the source body. The expanded catalogue retains that type's
actual carrier while preserving the same original call sites and error cases;
its source-layout foundation passes the Rust kernel with zero axioms. The
original function and its other declarations remain unchanged. A graph test
also retains two complete original owners of 5,697 and 5,667 objects, binds the
correct candidate/graph and rejects crossed ownership; its selected original
root has no call sites.

The new original-source, contract-only and full-owner graph tests passed.
Strict Clippy passed for those affected tests and the existing invocation
consumer; changed-file Rust formatting passed. The existing full invocation
suite passed all three tests, including the unchanged 70-error case (6,486.28
seconds in total). These tests were selected because
the existing catalogue now shares its construction routine with the bound
contract path. Unrelated Go suites and new validation archives were not added.

This does not prove source evaluation, single-execution ordering, callee
contracts/effects, source/IR correspondence, complete foundation or launcher
authority. R01 and T02-W01 remain open, and the practical profile stays inactive.
The cumulative `./scripts/check-fast.sh` gate is deferred to final T02-W04.
