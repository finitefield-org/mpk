# Nullable-reference scoped verification

All scoped checks pass. Four original-source null cases plus two distinct
normal-value partitions produced 30 executed result observations. Revalidated
exact-equivalence reuse covers 18 additional context observations, for 48 across
four source contexts. Two unique certificates cover four artifacts and pass
same-byte Rust/Go checking, zero axioms and corrupted-hash rejection.

`user-long-completed/` retains both original logs, the manifest and receipts.
Guarded/some passed in 41.17 seconds wall; conditional/some in 39.13 seconds.
Both are agent-owned if affected by future edits because they finish under one
minute. The prior bounded timeout remains diagnostic history; importing the
completed runs does not establish a code change or measured speedup.

Import checked all current source/input/certificate hashes and independently
revalidated the certificate/definition/SSA/oracle equivalence. Source-context
hashes are still distinct and were validated separately; reuse is not a native
control-flow proof. No test was rerun. Both pending lists are now empty and
the runner reports no pending checks. Unit 5 and W09 remain incomplete.
