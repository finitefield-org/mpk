# Source snapshots and retained-history definitions

The eleven metadata contexts replay the original W08 source contexts. Two
idempotency contexts contain seven complete snapshot nodes each; nine transition
contexts have no snapshot obligations. The three distinct certificate byte sets
are checked by both unchanged checkers with zero axioms and hash corruptions
rejected. Source-value equality covers all stored members, including values a
semantic binding projection might omit. The same complete equalities define the
W08 paired command/context `CanonicalFieldEncodingsEqual` relation.

The adapter also defines `HistoryCapacity4096`, `RetainedKeyPresent`,
`RetainedRecord`, and `RetainedKeysUnique` from the exact W08 state, command,
history, record, and key carriers. Lookup and uniqueness are bounded folds over
the complete 4096-element sequence representation. The retained-record value is
only specified when presence holds, matching every W08 use.
`AppendCompleteSnapshot` requires a one-element length increase and exact key,
complete command/context, and response fields in the new last record;
`PreserveRetainedHistoryOrder` compares every old record with the corresponding
new prefix record. Fourteen isolated member mutations are detected in 200
ordinary observations.

These are expected-value definitions, not proofs that source equality and
serialization helpers implement them. The adapter does not serialize canonical
JSON. The omitted-context source counterexample produces correct expected
relations while its application proof remains pending. Admission, replay, and
complete application assembly remain explicitly unresolved. No complete
application certificate is asserted. See
`../verification-logs/transition-snapshots/verification.json` and `review.md` in
that directory. Unit 6/W09 remain open; the full T06 gate stays with W12.
