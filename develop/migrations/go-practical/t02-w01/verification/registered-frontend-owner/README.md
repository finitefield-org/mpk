# Registered Go frontend retained with the SDK and roots

The private release owner retains the complete registered Go frontend and SDK
from the same validated release. It checks the frontend's entire inventory,
executable modes, bytes and hashes against the pinned registry, and checks its
Go build metadata. It can retain the frontend alongside the original compiler
inputs or alongside the existing complete source and export root owners. This
does not establish that the registered frontend ran the capture or produced
the input, nor does it authorize launcher-selected roots.

The complete original frontend executable (9,004,378 bytes), all 5,577 SDK
files, and the original 182-package, 5,697-object capture passed with test-only
sealed snapshots. Two independently resealed frontend controls, changing the
binary and its executable mode respectively, reject against the pinned
inventory. Existing coherent wrong-host and crossed Darwin-owner controls also
reject. This test does not open an installed release root.

The full live 182-package error-dispatch application also passed with the
registered frontend, SDK and both held root sets retained by the test-only
owner. Its 5,517 source objects and 181 exports stay in scope; a different
complete graph and a different original function reject. All entry, call,
error-value and string certificates are byte-identical to the previously
Go-checked streams. The complete no-string static-call application also passes
with the registered frontend and both held roots; its existing entry, call and
value certificates remain byte-identical to Go-checked streams.

Linux-target Clippy passed for the affected library, installed-compiler test
and test-configured live observer. The first observer Clippy run found four
unnecessary references; those were removed and the scoped check passed. The
full original-snapshot test was selected for the new identity boundary; the
live cases were selected for the root and checked-component consumers.

An installed-root check using local Linux x86-64 emulation retained all four
registered Go/Rust bundles. The new release snapshot and its binding to the
complete original inputs passed, as did the existing SDK snapshot and binding.
Uninstalled execution, conflicting host, crossed Darwin inputs, changed
frontend bytes or mode, changed SDK, a missing Rust bundle member and a
separately valid alternate registry all reject at the appropriate boundary.
The original bundle bytes and registry were restored and revalidated after the
controls. The first legacy binding attempt returned `Release` while another
full capture was running; an independent snapshot diagnostic accepted, and the
same legacy binding passed on a separate retry after that capture completed.
The cause of the first rejection is not established.

The first temporary preparation driver failed on its own archive path. Another
attempt reproduced both registered Rust binaries exactly but stopped at the
existing no-hard-link check; independent copies and the unchanged inventory
checks then validated both complete Rust bundles. An initial download attempt
also lacked a trusted Python CA configuration; the verified retry used the
system trust store and retained the original full archive hashes. No validation
archives are committed.

W01, the complete foundation, source execution and application proofs remain
open; the public practical profile remains inactive. The full T02
`./scripts/check-fast.sh` gate is deferred to final W04. Local Linux emulation
does not establish native Linux release acceptance.
