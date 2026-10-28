# Held source and export roots for the original Go owner

Status: scoped T02-W01 work; the practical Go profile remains inactive. The
supported source languages are Go and Rust. No public proof or launcher path
is activated by this checkpoint.

The private binding requires the independently matched application,
dependency and SDK capabilities, and the independently matched generated
export-cache capability, to be disjoint and to cover the complete original
export snapshot. Each observed directory entry, source identity and original
buffer must match that snapshot. Both match owners retain their directory
handles. A coherent source observation missing the SDK, exchanged or
overlapping capability sets, a changed export byte and an added unread cache
entry reject while the complete original owner stays intact.

Two original live runs opened all four roots before their respective captures,
matched 5,516 source objects and 181 generated export objects, retained both
matched root sets and imported the same 5,697-object, 182-package owner. In
each, the independent Rust observer also imported all 5,577 registered SDK
files (166,354,960 bytes) while holding those root owners. The final run checks
in both Go and Rust that the exact previously held export-cache path is absent
before binding; an additional read-only post-run check finds all four original
paths absent. These are observations at their stated times, not proof of
which paths a production launcher is authorized to select. The installed
compiler wrapper is private; its use with both live root owners has not been
established as installed-release authority.

An earlier live attempt retained complete diagnostics after the native Go
package-export command hit its existing 120-second limit. A subsequent
measurement copied and rehashed every SDK file, with original modes and paths,
on each filesystem: the host-shared directory took about 8.4 seconds and a
Linux volume about 1.4 seconds. This supports investigating storage overhead
but does not establish a sole cause. Both successful attempts use fresh empty
Linux volumes shared by Go and Rust; the original Go timeout, environment,
source files and cache generation remain unchanged. The failed attempt, its
four original outputs and both benchmark copies are retained.

The changed Go coordination and original-owner tests, Rust projection and
existing installed/compiler-root consumer, targeted negative cases, vet,
Clippy and format checks are recorded under their respective preflight and
downstream directories. One first Rust preflight exposed a duplicate lint
attribute and a native-name test on a case-insensitive host-shared temporary
directory. Its three passing tests and exact failure remain recorded; the
attribute was fixed and the failed test passed on a case-sensitive Linux
temporary filesystem, without removing any existing test case.

The successful runs retain every native evidence file losslessly as archived
hex chunks. Their original bytes, paths and modes can be restored with
`restore_evidence.py`. The failed diagnostics, actual coordination file modes,
and exact historical run drivers have separate verified restoration records.
The earlier inspection script was reconstructed by reversing the later patch;
its original execution did not record a source hash, so an earlier hash cannot
verify that reconstruction.
The first scoped review found one incidental match in a compressed input
archive's machine bytes; its contents had no findings. The original archive
and complete failed review are stored losslessly, and
`restore_review_incident.py /absolute/new-directory` restores their exact
original paths, bytes and modes without changing review rules.
The unchanged scoped review covers the changed code and retained evidence.
Local Linux containers on this host are not native Linux release acceptance.
The remaining source/program, installed/launcher and private-context authority,
W01 and T02–T11 work remain due. Per the repository instructions,
`./scripts/check-fast.sh` is deferred to final T02-W04.
