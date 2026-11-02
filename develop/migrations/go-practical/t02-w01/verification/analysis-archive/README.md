# Go analysis archive and vendor bytes: in progress

The private Rust owner checks the complete bytes of the three pinned analysis
module ZIPs and compares every supplied vendor file with its original archive
member. It retains the manifest, archive bytes, supplied source objects and
explicit differences. It checks module/root ownership, necessary bounds on the
supplied files and ZIP namespaces, and package coverage; an absent archive is
reported separately from a resource-limit failure. These checks do not
authenticate the input capabilities, prove that the supplied file list is a complete namespace, parse
Go source syntax, or establish a source-to-binary or practical-context binding.

The [native Go capture](native/result.json) uses the full original frontend and
retains all 213 captured source objects, three ZIPs and 133 vendor files. The
[independent inspection](inspection.json) recomputes every source identity,
compares all captured originals with prelaunch bytes, and checks all 1,751 ZIP
members. It retains 126 Go sources and seven auxiliary files. The one original
uncompiled `TODO` difference remains explicit; neither its bytes nor the archive
member are normalized. All 29 original declared packages remain represented.

Five controls compare Go's actual capture with the Rust owner using complete
copies of the original tree. A changed Go file, the only file of a package and
an unknown asset reject; a changed `TODO` is retained as a difference. A sixth
control corrupts a complete ZIP and is checked by the native archive parser.
Removing one file from a package that still contains other files is
accepted by both checks. That last observation is why this owner **must not**
be treated as proof of a complete namespace. Every original input is retained
independently of these controls; no application package or source stub is used.

The [affected Rust tests](archive-selected/result.json) pass the new two roots,
the original vendor-manifest root and all four original module-data roots. A
follow-up [error-class check](control-inspection.json) reruns only the two
changed test roots, confirms all 1,044 other prior inputs are unchanged, and
preserves the earlier full regression. Scoped Clippy, Go vet and changed-file
format checks pass. An initial missing-archive classification was corrected
before the final tests; its original source preimage is retained in the earlier
check archives.

Complete namespace/read authority, the shared capture budget, Go syntax and
embedded-input checks, the original module manifest/checksum linkage,
source-to-binary build authority, private context and application proofs remain due. The practical profile and
T02-W01 remain inactive and open. Local Linux emulation is not native Linux
release acceptance. The whole T02 gate `./scripts/check-fast.sh` is deferred to
the final T02-W04.
