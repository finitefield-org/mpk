# Candidate-bound source contracts and language versions

The inactive candidate context and original-source language checker now share
the same frozen Go 1.22 through Go 1.25 list. The canonical context and request
formats are unchanged. A source binding can derive contracts only from its
already validated original source selection. A graph binding retains the same
candidate, original source and verified graph while exposing that derived
module. This is candidate construction, not a checked foundation, launcher
authority or application proof.

The affected original three-file language fixture retains its unselected
Go 1.21 file. Selecting `Root` and its valid transitive callee succeeds;
selecting `Legacy` directly or reaching it through `CallsLegacy` rejects at
the actual old-language source location. The original sidecar fixture retains
all three raw overlays: its three selected roots and four derived functions,
including the two applicable overlays, exactly match the existing source-owned
contract module. The full 8,192-root boundary also derives all 8,192 original
functions and default partial contracts without reducing the input set.

The complete live capture and Rust observer passed on the original
182-package application, including its unselected HTTP handler, 5,516 source
objects, 181 export objects and all 5,577 registered SDK files. The validated
candidate and independently imported module graph retained their held root
owners while deriving the selected function's default partial `true` contract.
That capture has no contract sidecar; this is not a user property or proof.
The different complete 5,667-object owner resolves its same-named function but
is rejected when crossed with the held original owner. Go 1.25 ran as
Linux/amd64 under local emulation and Rust ran on Linux arm64. The observer and
installed-SDK owner are test-only; native Linux release acceptance and the
installed launcher are not established.

Validation was selected for the shared language list, candidate context and
original-source consumers: five context tests, two corpus tests covering 133
frozen vectors, four source-language tests covering the 150 original language
cases, the full-owner graph test, and all five source-binding tests. The
8,192-root contract derivation was rerun after its boundary assertions were
added. The complete live Go/Rust capture passed. Strict Clippy passed for the
affected Rust tests, Linux observers and the observer's test configuration;
changed-file Rust formatting and `git diff --check` passed. Go source did not
change, so unrelated Go suites were not repeated. No new validation archive
is committed.

T02-W01 and R01 remain open; the practical profile remains inactive. The
cumulative `./scripts/check-fast.sh` gate is deferred to final T02-W04.
