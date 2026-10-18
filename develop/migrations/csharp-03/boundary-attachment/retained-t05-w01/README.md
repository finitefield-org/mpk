# Original T05-W01 input epoch

These two files are exact Git objects from
`b0c9a5d5a9616e107200731f62a5abac91828538`. Their raw hashes and sizes match the
unchanged original boundary-attachment conformance record. The live fixtures
were later rebound in the already approved Boolean-proof refreeze commit
`40469d298f1d69558713d02ea0b553a1dd583bf5`; they retain that commit's exact bytes.
The [epoch mapping](epochs.json) records both pairs of hashes without rewriting
either observation or either current fixture.

The retained-evidence test checks the old record against these original inputs
and separately pins the current refreeze inputs. Across all56 request/response
pairs it compares complete source bytes and sidecar bodies outside the exact
context/own-hash values, preserves every other native fact, and checks current
file digests and sizes against the actual current request bytes. Deliberately
malformed Unicode sidecars remain raw byte comparisons; only the independent
metadata values are parsed. No invalid case is removed or repaired.

This is historical test-evidence maintenance required by the Go T01 full gate.
It changes no C# language rule, source input, contract, limit, proof observation,
profile namespace or activation status. Current behavior is still exercised by
the live source/attachment tests. Old input hashes do not claim a new native run.
