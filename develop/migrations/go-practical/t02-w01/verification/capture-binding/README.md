# T02-W01 capture/root/compiler binding

Status: private component, targeted checks passed; W01 remains in progress.
This checkpoint adds an owned connection between independently compared Linux
source-root handles and the existing complete installed-Go compiler/export
owner. It does not activate a practical Go profile.

For each matched capability, `contains_exact_capabilities` compares the complete
recorded directory-entry sets, source identities and original bytes with the
same capability in the full compiler/export capture. The binding requires all
non-export capabilities to be matched. An unmatched
`original-go-package-exports` capability remains in the full owner and is
exempt only when all of its captured objects are export data. Matching does not
authorize launcher-selected roots, authenticate a prior capture epoch, bind an
unmatched export root or establish a theorem about the source program.

The integration test restores all 16,880 retained original evidence files,
all 74 original application/dependency entries, and the exact 5,577 registered
SDK files (166,354,960 bytes). It compares the original 5,516 captured objects
under application, dependency and SDK roots with those roots, and binds the
result to the full original owner with 5,697 objects, 182 packages, one Root
function selection and zero contract overlays. The SDK image is sealed by a
test-only helper and the roots are restored test roots: neither impersonates an
installed root or proves launcher authority. The 181 extra export objects and
the complete original owner are retained; the separate export root is not
claimed authenticated.

The positive test and four adversarial controls pass: a source-byte change
with recomputed source and snapshot identities; an extra unread directory
entry with recomputed snapshot identities; an omitted required source root; and
reclassifying an unmatched export object as a source. The first two modified
captures successfully match their independently opened, modified roots before
the original full owner rejects them. The omitted-root case is a negative
attack, not a replacement for the complete original input.

The directly affected existing source-root integration test passes with its
213 original objects and native-name controls. The new complete-owner test,
Linux-target test compilation, Linux-target Clippy and changed-file formatting
pass locally. The first Clippy attempt rejected a duplicate test-only
`dead_code` attribute; the original failure is retained and the corrected test
and Clippy reruns pass. A separate fixture audit initially expected 216 files;
the full 213 objects reference 209 distinct byte files plus four records. The
original count failure and corrected independent byte audit are retained.

The scoped review scans all changed files, 5,239 archived input files and
10,506 embedded ZIP members with no findings; scanner rules are unchanged.
The previous complete live-evidence review covers all original files; the 24
exactly recoverable evidence chunks remain unchanged. Local Linux arm64 test
execution and Linux x86_64 cross-target compilation are not native Linux
release acceptance. The T02-wide `./scripts/check-fast.sh` gate remains deferred
to the final T02-W04; no Go theorem, full private context or W01 completion is
claimed.
