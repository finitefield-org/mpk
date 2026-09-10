# W09 W03 integer/Boolean operation relations

Three independently captured source contexts contain six integer/Boolean
operation-definition occurrences and eight SSA use points. Existing scalar
bodies remain exact. The adapter connects W03 result/guard/failure formulas,
retains original subject order and CFG metadata, and explicitly records the
remaining structural/string data definitions. These are definitions, not proofs
of complete native bodies or application VCs.

Candidate reconstruction and mutation tests pass. Result-bit unit tests pass
1,068 cases. All three same-byte Rust/Go checker cases passed with zero axioms and matching
reports/hashes; hash corruption rejects. All eight source use points passed 80 result/guard cases; final runtime
certificate/metadata bytes match the checker pins. Shared-helper extraction
preserved all three pins. Exact coverage and retained failure logs are in `../unit-5-integer-data-progress.json` and its review. The three
candidates total 571,133 bytes (maximum 542,851). Units 3-8 remain open; no
original-unit completion or W09 acceptance is implied. The full gate remains
deferred to T06-W12.
