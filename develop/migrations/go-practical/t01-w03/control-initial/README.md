# Original-checker natural-induction limitations

Two actual observations are retained separately:

1. The initial intended-positive invariant candidate assumed a dependent
   motive argument on the old Nat recursor. Both unchanged checkers rejected
   it at core typing. Its exact source snapshots, input hashes, binaries,
   candidate and raw observations are retained here. Only that one executed
   row is claimed; generated downstream candidates are not claimed checked.
2. The corrected candidate declares the proposed separate `Nat.induct` type.
   Both unchanged checkers reject its use because the new generated recursor
   has no supported reduction rule. The exact candidate and raw results are
   retained under [current-interface](current-interface/observations.json).

Neither result is evidence that the intended invariant is false or that
production supports general induction. The later
[private prototype](../control-induction/README.md) adds a checked canonical
interface in mirrored Rust/Go sources, preserving production files and the
old Nat.rec type/equations. `check_control_initial.py` checks these two
limitation records without promoting them to semantic negatives or W03
completion.
