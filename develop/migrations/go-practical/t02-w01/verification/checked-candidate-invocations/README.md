# Checked candidate call interfaces

The private CLI owner now generates and kernel-checks a mathematical interface
for every original call site in catalogue order. A different site, construction
failure, kernel rejection or nonzero axiom count prevents creation of the owner;
an original call-free closure yields an empty checked set. Source-bound and
graph-bound forms retain their corresponding candidate and contract owners.
These checks do not prove actual callee execution, effects, source/IR
correspondence or an application theorem.

To avoid retaining a large certificate and axiom report for every site, the
owner stores compact site identities, declaration counts and checked hashes.
It can regenerate one certificate from its retained original catalogue and
return it only when the complete certificate hash and site/module identities
match the checked receipt. This does not change a source, error, certificate or
checker limit and does not omit call sites.

The affected original-source test retains three selected roots, the fourth
transitive function, all three raw overlays and the complete original source.
The checked owner covers its one real cross-package call and reconstructs the
certificate with exactly the same site, module, declaration count and three
hashes previously accepted independently by Rust and Go. An invalid receipt
index rejects. The actual original test fixture retains 8,192 unselected boundary
declarations; no source package or error universe is replaced or reduced.

The complete live Go capture and Rust observer passed with all 182 original
packages, 5,516 source objects, 181 exports and 5,577 registered SDK files.
Their selected original root has no calls, so the checked call set is empty;
the same held candidate/graph owner also passes the existing entry check and
crossed-owner control. Its entry certificate is byte-identical to the earlier
independently Go-checked live certificate. An earlier attempt ended before this
binding when the original offline Go discovery process was killed; the fresh
full-input retry passed with the same 120-second command limit. Go 1.25 ran as
Linux/amd64 under local emulation and Rust ran on Linux arm64. The observer is
not the installed launcher and this is not native Linux release acceptance.

The targeted original-source test and complete live capture were selected for
the new checked owner and its held-root consumer. Strict Clippy passed for the
affected test and Linux observer, including its test configuration; changed
Rust formatting and diff checks passed. The complete existing invocation
regression also passed, including its 70-error case, as recorded in the
preceding checkpoint.
No unrelated suite or new validation archive was added.

R01 and T02-W01 remain open, and the practical profile stays inactive. The
cumulative `./scripts/check-fast.sh` gate is deferred to final T02-W04.
