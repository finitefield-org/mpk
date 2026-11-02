# Inactive Go/Rust candidate context

The private candidate registry derives its retained Go fixed and Rust entries
from the actual validated revision-4 installed registry, and adds the frozen
practical-Go candidate entry. Its complete registry and entry identities are
pinned; a caller cannot authorize a changed registry, profile, contract,
descriptor or limit by recomputing hashes. Contexts require the full registry
reference, exact profile/entry, parameter envelope and foundation reference.
Practical Go admits only the frozen Linux/amd64, Go1.25.0, GOAMD64 v1 and
go1.22–go1.25 parameters. Retained Go and Rust parameter and selection checks
delegate to the existing compiled revision-4 validators. Request hashes cover
the complete context and selection; context linkage compares full canonical
values, including the distinction between Rust32 and Rust64.

The Rust context tests passed for Go fixed, practical Go, Rust32 and Rust64,
including changed parameters, wrong selections, stale request hashes, missing
fields and correctly rehashed caller registries. A separate test regenerated
all 133 frozen candidate configuration cases under the current Go/Rust scope:
18 candidates and 115 rejections matched. It also recomputed the frozen
limits, definition-inventory and descriptor hashes and checked all three raw
descriptor members and their sizes. The independent Python observer produced
the same candidate registry and practical request hashes. The existing three
runtime registry tests passed because the new consumer delegates retained
policy to that code. Strict Clippy and Rust format checks passed for the changed
context and tests. These are the affected configuration and retained-policy
checks; unrelated proof and source tests were not rerun.

The referenced descriptor explicitly has `candidate_only` activation and its
definitions are a recipe inventory. Matching those bytes is not both-checker
acceptance of the complete ordinary foundation. The candidate context is a
distinct private type and grants no installed profile, compiler/source
authority or theorem. Completion of the foundation, context/source binding,
the Go producer, common-artifact migration and T02-W01 remain open. Public
practical Go stays inactive. The cumulative `./scripts/check-fast.sh` gate is
deferred to T02-W04; no new validation archive is committed.
