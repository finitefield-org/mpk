# Nullable slot transport review

The `type` source retains its original nominal W04 slot declarations and native
SSA types. Its one nullable slot has an explicit, independently reconstructed
Option storage override. All 53 guarded slot joins compare the complete Option
carrier, including presence and payload; assignedness remains separate.

The changed source passes 2,758 runtime observations, including Some(Box(17)),
presence-only corruption, assignedness, disabled guards, physical payload and
inactive storage. Removing or changing the override rejects import. These
observations replace its former 2,016 observations, giving 38,901 across the
current edge corpus. The sixteen other contexts regenerate byte-identically.
Three historical preservation tests compare every old non-join declaration's
normalized term structure and retain original guard/phi/source metadata. Only
the specified slot join names and argument storage types may change.

The final candidate exactly matches the bytes accepted by both unchanged
checkers with matching reports and zero axioms; both reject hash corruption.
Targeted clippy and format checks pass. No unrelated runtime corpus is rerun.

This source has zero original W04 goal bindings. The tests therefore establish
storage transport, not interpretation of W04 logical observations. Node-entry
merges, non-transfer memory effects, exceptional execution and complete native
application proofs remain outstanding. The modeled SwitchExpressionException
edge remains explicitly pending. W09 and unit 5 remain incomplete; the full
gate is deferred to T06-W12.
