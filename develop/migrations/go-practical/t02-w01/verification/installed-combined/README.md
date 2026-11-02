# Test-only compiler owner with both original Go root sets

Status: a scoped T02-W01 checkpoint. Go and Rust are the supported source
languages, and the practical Go profile remains inactive.

The preceding live observer retained both independently matched root sets and
imported the complete original compiler inputs. This checkpoint additionally
passes those same held root owners into the private compiler binding. The local
observer is compiled with Rust's test configuration and uses the existing
test-only SDK snapshot constructor. It starts from all original SDK buffers,
checks them against the registered inventory, and retains the resulting test
compiler owner, complete export inputs and both root-handle sets together.
The observer compares the compiler record and release identity and verifies
that neither root set lost an object. It does not construct or authorize a
production installed-release root.

The actual Go run opens source roots before capture and the actual generated
export cache before its capture. It matches 5,516 source objects and 181 export
objects, checks the exact export cache path is absent before binding, and
binds the same 5,697-object, 182-package owner with all 5,577 registered SDK
files (166,354,960 bytes). The original root paths are also checked absent
after the run. Both the test-only observer binary and its predecessor are
retained losslessly with their exact modes and can be restored independently.

Ordinary Clippy, formatting and the test-only binary build passed. Passing
Rust's test configuration directly through Cargo Clippy also applied it to
unrelated dependency/test modules, producing existing unused-code diagnostics;
the complete failed output is retained. A separate attempt restricted Clippy
but reached a macOS-to-Linux link failure. The final check uses `cargo check`
with a forwarding wrapper that invokes the real Clippy driver and test
configuration only for this example; all other crates go through their normal
compiler unchanged. A separate deliberate lint error is rejected, confirming
the driver is active. The real example then passes with warnings denied. Only
under test configuration, the included root module allows unused imports
because this observer is built without a test harness; its actual root tests
were run separately in the preceding downstream check.

The complete native outputs, coordination files and original run driver are
retained with restoration checks. The archive precheck found one incidental
match in compressed machine bytes; its original archive and full precheck are
stored losslessly and `restore_archive_precheck.py /absolute/new-directory`
restores their paths, bytes and modes. Scanner rules and original source bytes
remain unchanged. The scoped review covers the original evidence and decoded
test binaries. Local Linux containers on this host are not native Linux release
acceptance. Launcher authorization, installed-release authority,
compiler/source semantics and application proof remain separate; W01 and
T02–T11 remain open. The cumulative `./scripts/check-fast.sh` gate is due at
final T02-W04.
