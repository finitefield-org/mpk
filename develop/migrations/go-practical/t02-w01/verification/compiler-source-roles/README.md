# Independent whole original SDK source roles

Status: scoped checks passed. Go/Rust only; T02-W01 stays open and
the practical profile remains inactive.

The complete original compiler-input owner now derives the role of every prepared
SDK source from its actual owned path/bytes. It checks every source object's role
and reconstructs every directory containing a `.go` file, including excluded files.
The original std module/source-object and its declared Go requirement are retained.
This is source membership and classification, not target package/body admission,
installed SDK/context authentication or a library-body proof.

The fixed Go1.25.0 linux/amd64/CGO0 rules retain native ordering: reject a filename
mismatch before reading; diagnose unexpected import-header reads before false
constraints/test exclusions; apply exact filename/build constraints; retain binary-
only source; classify documentation/C imports using parser.ImportsOnly. Syntax
errors stay go_file for the actual checker. The import reader and parser have
different dispositions, so their checks stay separate. Import paths in this role
parser preserve arbitrary decoded bytes without go/build's later path validation.

The additional lexer demand state reads exactly the package/import grammar and
one following token, with Go scanner's immediate invalid UTF-8/NUL/BOM lookahead.
It does not parse unused later bodies, fabricate replacement Go files or use the
ordinary AST-node budget for role-only headers. Every existing lexer call leaves
the new internal demand state absent. Existing source/type/program parsing and
all source/bundle/kernel/certificate budgets stay unchanged. A complete original
40000-import header and its unused body remain in the native/Rust corpus.

Native observations cover all11004 sources in the original raw SDK and1017 raw
source package directories, plus388 complete filename/header/auxiliary probes.
The actual prepared compiler retains all5577 files/163457770 bytes, including its
5480 SDK sources and715 source package directories. Those prepared sources are
an exact subset of the raw observations; the additional raw SDK test/source files
are retained too. The [byte transport](source-byte-transport.json) reuses exact
original prepared content where identical and owns5415 additional unique raw
files. No observed source, unused body, excluded case or package is dropped to
make the prepared/raw counts agree. The original raw SDK's complete14496-file
receipt/archive remains unchanged.

The original182-package/5697-object HTTP application, complete dependency/cache/
archive/source universe, unexecuted init and Root contract remain intact. Three
unused SDK role controls change active/excluded/auxiliary metadata, coherently
reseal object/snapshot/capture/export/compiler links and successfully reimport
the complete source/export owner. Every source/compiler byte and application/
Root contract stays unchanged; the independent compiler importer must reject
SourceRole. Existing30 compiler claim/member controls, the unused-SDK-content
control and all3 executable build-info controls remain selected.

Affected verification selects23 Rust roots across the SDK-role/compiler/export/
lexer/build/package/auxiliary consumers, the full native observations, strict
affected Clippy, Go vet and changed-file formats. Each source epoch retains its
prelaunch consumed sources, original SDK/archive references and raw results.
Native Go-only epochs own the Go compiler/test/vendor and complete SDK inputs;
unrelated Rust sources are independent of those commands. Rust epochs retain the
full original source/package/type/error/proof inputs and all raw source bytes.
Prior32 proof streams and43-root/zero-or70-error universes retain their earlier
evidence; they are not relabeled as new kernel replay.

Strict standard-module semantics and remaining generic executable-reader cases,
complete installed compiler/root/capture/context/descriptor/registry/launcher/read
bindings, required foundation/source/program proofs, native Linux release gates
and remaining T02–T11 stay due. No frozen requirement, source universe, scanner
rule/seal/exclusion, premise or mathematical rule is removed or weakened.
R01/R03/R04/R05/R06/W01 remain open; final T02-W04 owns the cumulative
`./scripts/check-fast.sh` gate. No whole-workspace gate or removed-language tests
run here. The [native observations](native/observations/original-sdk-source-roles.json),
[verification driver](run.py) and [scoped review driver](review.py) retain exact
inputs and scope.

All23 selected Rust roots, the full11004-source/388-probe native comparison,
strict affected Clippy, Go vet and changed formats pass. All14 consuming epochs
are preserved:12 pass and the initial Rust/clippy epochs fail because the new
compiler helper was initially called through a module-private function. Internal
wrappers fix that visibility without changing existing consumers' visibility or
lexical behavior. All3 fully resealed source-role controls reject SourceRole;
existing30 compiler claim/member controls, the unused-source-content control and
all3 executable controls remain selected. See the [verification](verification.json)
and [inspection driver](inspect.py) for exact selected scope and remaining work.
