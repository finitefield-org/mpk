# Original source file languages

The independent source reader now derives file language facts from every
complete original module and active Go source buffer. It retains the full
`go.mod` requirement, including patch/prerelease spelling, checks it against
the pinned Go1.25.0 compiler before reducing it to a language minor, and reads
leading modern build constraints from the original bytes. Decimal version
comparison does not truncate large version numbers.

Go1.25 file-version rules allow a versioned build constraint to lower the
module language. The effective version is the greater of that constraint's
structural minimum and Go1.21. A missing versioned constraint retains the
exact module configuration spelling; a module without `go` uses the native
Go1.16 default. Negation and Boolean branches use the pinned constraint
algorithm, including impossible branches, leading-zero tags, Unicode tags and
the native parser's1,000-entry bound. Legacy `+build` comments do not assign
a file version. The preserved [primary sources](spec-reference/inputs.json)
are exact files from the pinned compiler distribution.

The owned source/contract consumer checks every selected and transitive body
against the frozen Go1.22-Go1.25 range. An older unused original file remains
in the complete package. Selecting that older function directly or calling
it from a newer function rejects at the original function byte location.
Source-component identities bind facts to the same complete package set;
foreign source contexts reject. Native version observations are comparison
oracles and do not enter the independent constructor.

This is source-derived file-version metadata and a selected-body guard.
Complete module/toolchain/role validation, build-file selection and
canonical capture/context/descriptor/registry authority remain separate work.
The lower-level candidate analysis APIs do not gain installed context or
program-proof authority. No schema, kernel, axiom or checker cap changes.
The practical profile remains inactive; R01/R06/W01, all56 foundation families,
all169 program demands, native Linux release gates and remaining T02-T11
work stay open. Go and Rust remain the only source languages.

[Verification](verification.json) selects the changed reader, owned contract
consumer and direct existing source-input/contract-input consumers. All11
Rust roots and2 native Go roots pass. All150 original Go parser/types cases
are compared:138 metadata cases agree and12 invalid cases reject. Both
complete original zero/70-error units remain unchanged, and all32 original
call-proof certificate bytes replay. Strict affected Clippy, Go vet and
changed-file Rust/Go format checks pass. Unchanged kernels, large foundation
matrices and unrelated frontends are not repeated. The cumulative T02 gate
is deferred to final W04: `./scripts/check-fast.sh`.

Diff review found one further header boundary: a prefix lookalike followed
by trailing whitespace must remain an ordinary comment. Trimming the entire
comment before removing the prefix matches Go. Eighteen added native cases
cover trailing ASCII/Unicode whitespace and a real Unicode-separated directive.
All132 predecessor cases remain byte-exact within the150-case final corpus.
The final11 Rust roots, corrected native probe, strict Clippy and formats pass.
The [initial complete review](review-before-header-fix.json) and its
[exact changed input preimages](review-predecessor-preimages/manifest.json)
remain retained separately from the corrected final review.

The original Clippy attempt failed on the empty-string comparison style;
using `is_empty` fixes it, and final lint and all11 affected tests pass.
The first independent inspection started while the same Rust worker was
still compiling and correctly refused its running result. Its
[original driver, running result and diagnostic](inspection/observed-result.json)
remain; the worker was not restarted. The
[final independent inspection](inspection-header-final/result.json) passes and
checks all original packet hashes/bytes/native package membership, pinned
source references, actual native file versions and original rejection offsets.
The initial transport collector encountered the separate primary-source
inventory rather than a worker archive receipt; its
[original attempt](archive-storage-preflight/observed-result.json) remains.
The corrected collector preserves every worker container; all primary-source
files remain included in worker archives and current input review.

[Snapshot storage](snapshot-storage/manifest.json) losslessly retains all10970
original files (87177162 bytes), including5485 original source objects, their
exact paths and modes. [Transport storage](archive-storage/manifest.json)
and [followup storage](archive-storage-followup/manifest.json) retain all19
complete original prelaunch containers (348922973 bytes).
The duplicate local originals remain physically unchanged. Exact restore:

```sh
python3 -B develop/migrations/go-practical/t02-w01/verification/source-languages/preserve-originals.py --restore
python3 -B develop/migrations/go-practical/t02-w01/verification/source-languages/preserve-transports.py --restore
python3 -B develop/migrations/go-practical/t02-w01/verification/source-languages/preserve-followup-transports.py --restore
```

Normal Rust tests consume the small tracked original packets directly.
[Input review](input-review.json) scans complete decoded original source
members under the unchanged127 matchers/four scopes/39 seals/51 exclusions.
Only duplicate exact storage copies are ignored; source review scope and
proof/test demands are retained. Producer stdout/stderr keep their exact
bytes, including original final blank lines.
