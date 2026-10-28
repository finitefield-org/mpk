# Go analysis generations with the same build recipe

Status: W01 remains in progress. This observation compares the retained earlier
registered frontend source epoch with the separately retained later analysis
source epoch. It does not identify the live checkout or activate a profile.

The later archive restores all 331 captured files and their receipt modes,
including all 328 frontend files. The earlier archive retains all 269 source
files, including 242 frontend files. The complete captured inputs contain 196
and 213 source objects respectively. Independent byte checks compare every
captured object and the entire 184-object and 201-object build-input lists:
178 common inputs are byte-identical, six are changed, 17 are added and none
are removed. Both observed frontend roots lack `default.pgo`.

Two independent empty-cache, offline builds of the complete later source use
the exact same normalized command, environment, registered Go 1.25 SDK and
release flags as the two earlier registered-source builds. Each pair produces
identical complete streams. The earlier stream is the registered frontend,
9,004,378 bytes, SHA-256
`ccc4a984d41b5da7b3a6838cbfaeee3eb23f675a2e4d7fb5d4f9fcd7e597d538`.
The later stream is 9,004,642 bytes, SHA-256
`67b24a896ff32004cd71dabeee5e47c3e0475d5081973a63911d47d1c0e06c14`.
The output streams differ even with the same build recipe; the comparison does
not identify a particular causal source change. Both later streams are retained
losslessly as separate hex files. This is local Linux emulation, not a native
release receipt or source/program theorem.

The scoped review checks all 409 captured source objects, 600 archived files,
7,004 ZIP members and both decoded later binary streams. It finds no active
source matches and records two incidental historical matches in the binary
machine bytes. The original findings and full binary bytes remain recoverable;
scanner rules and exclusions are unchanged. The first review attempt failed
before scanning because a local script shadowed Python's standard `inspect`
module. That exact failure and the renamed unchanged script are retained.

An additional inspection initially tried to overwrite the already retained
first observation and stopped at the no-overwrite assertion. The unchanged
first record, exact failed diagnostic, driver and successful independent
follow-up remain separate. Existing Go/Rust source-owner and root tests were
not repeated because their inputs and implementations did not change. The
complete T02 `./scripts/check-fast.sh` gate remains deferred to final W04;
trusted launcher roots, registered practical frontend identity, export-root
authority and Go source proofs remain open.
