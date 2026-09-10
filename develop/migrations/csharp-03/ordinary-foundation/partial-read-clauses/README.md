# Ordinary partial contract reads (partial W09)

`sequence_index` and `tagged_payload` now lower to existing ordinary functions
with separate W03 definedness. The sequence case checks the complete i32 bit
pattern against both stored length and physical capacity before reading;negative
and high-bit indices cannot alias lower slots. The tag case requires the selected
arm and uses the existing masked single-payload projection. Validation's role-bound
error-sequence storage keeps its concrete payload type. These are operation and
condition definitions,not application proofs or type-admission claims.

The same-VIR relation cache shares sequence operations with the later structural
consumer. Sequence emission validates the full frozen operation entry and reuses
a complete existing read/range pair. An incomplete pair rejects. Both original
standalone output and all12 previous source contexts retain exact bytes/metadata.
The complete standalone sequence dependency closure matches the new source output.

The new capture has13 clauses:negative/valid/out-of-range/boundary indices,
an empty sequence,Some/None payload reads,and guarded payload reads. Source
expression hashes and W03 conditions are checked. Direct functions additionally
have2620 guard/storage observations,including length4095/4096,negative/high
indices,invalid length words,unknown/high-bit tags and nonzero inactive payloads.
Invalid representation probes do not establish public membership;zeroed failure
storage is not a normal contract result. New source payload coverage is
Option<Value>;complete source integration of the other eligible sum arms remains
part of the original unit acceptance.

Exact import rejects removal of definedness metadata. All three public consumers
still reject partial clauses until the original use-point owner can discharge
their conditions. `certificates.json` identifies the one candidate passed by both
unchanged checkers with zero axioms;its actual hash corruption rejects.
`../unit-4-partial-read-clauses-progress.json` retains commands/results and scope.
No internal unit/W09 completion,component-only commit/push or full gate is claimed.
The full gate remains deferred to T06-W12.
