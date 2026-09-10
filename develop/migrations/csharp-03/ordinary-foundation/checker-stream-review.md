# W09 checker subprocess stream review (in progress)

During the live integer parser checker run, the boundary-exception-empty case
returned accepted, zero-axiom Rust JSON on stdout, but a Cargo unused-mut
warning on stderr was concatenated ahead of it. The harness's CombinedOutput
then failed JSON parsing. This is an observed execution/protocol failure, not a
proof rejection or a semantic parser failure. The run is not counted as passed.
The unnecessary mut was already removed from the new decimal formatter.

The harness now captures stdout and stderr separately and parses the complete
stdout as the sole verifier report. Stderr is retained with labeled stdout in
failure diagnostics. No JSON substring search, warning filtering or relaxed
exit/hash/verdict check is introduced. Missing stdout, polluted stdout, command
launch failure, a build failure exit code and internal/unknown rejection codes
continue to fail closed. Accepted report equality with the Go checker remains
unchanged, as do the two actual checker implementations and certificate bytes.

Subprocess regression tests exercise rejected and accepted stdout reports with
non-JSON stderr warnings, a report emitted only to stderr, stdout contamination,
and build failure with a deceptively well-formed stdout report. Existing hash,
verdict, exit-code and report agreement tests remain. The original observed
warning-plus-JSON log demonstrates the pre-fix failure; the new process tests
passed in the current five-case version (158.219 seconds). Current handles and results are in
`checker-stream-progress.json`. Preserve the existing live checker run until
terminal. The already observed failing case passed its independent recheck (27.122 seconds);
inspect any additional failures after the original run terminates. Six actual
decimal certificate cases have passed both acceptance and hash-corruption checks
through the corrected helper (92.206 seconds).
This is not W09 acceptance, a unit completion or a basis for a component commit.

The original integer corpus is now terminal: 53 cases passed and only the known
warning-contaminated case failed. Its successful targeted recheck covers both
acceptance and corruption rejection; every pinned case is accounted for. The
old whole-run verdict remains failed in the receipt. Direct review of the
stream fix found no additional issue. This closes the observed harness defect,
not W09 or an internal implementation unit.
