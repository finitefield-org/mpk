# T02-W01 independent original lexical scopes

The [Rust scope builder](../../../../../../crates/mpk-vc/src/go_practical_type_keys/source_scopes.rs)
uses complete owned parsed source from candidate packages. It retains actual
declaration/name syntax nodes, binding kinds and local visibility starts. Package
and file scopes, function receivers/parameters/results, explicit blocks, implicit
if/for/range/switch scopes, case/communication clauses, type parameters and local
short declarations retain their distinct scopes. Variables, constants, functions
and imports remain blockers when they shadow a type or predeclared name. Local
constant/variable initializer references still see outer bindings; local type
declarations can refer to themselves. Parameter reuse in a short declaration
keeps the original parameter binding. The flat scope graph avoids recursive
scope destruction and derives bindings without a supplied AST or Go object map.

Import paths are decoded as exact Go literal bytes. Explicit aliases, actual
default package names and exported dot-import members are handled; a default
name is never guessed from the path's last component. Pinned Unicode Lu decides
export visibility. Missing default metadata blocks file-level lookup, and missing
dot metadata blocks lookup of possibly exported names, including potential
collisions with known file bindings. Inner local bindings remain resolvable.
Duplicate package/local/file-import bindings and short declarations introducing
no fresh name reject. Labels and selector fields have separate namespaces:
`lookup_identifier` returns a lexical lookup at an original Ident and does not
assert that every syntactic Ident position is itself a lexical use. These rules
follow the [Go scope specification](https://go.dev/ref/spec#Declarations_and_scope).

The [actual Go reference](../../../../../../go-tools/go2vir/practical_source_scopes_reference_test.go)
reconstructs an original three-package universe with seven source objects.
Its real compiler Uses objects provide117 lexical observations after explicitly
excluding selector members, fields and labels. Every use's actual binding kind,
definition source object and physical byte position matches independent Rust
lookup. The cases cover grouped constants/iota, package forward references,
cross-file declarations, Unicode/CRLF/line directives, shadowed initializer
references, recursive local types, aliases/dot/default/escaped imports, nested
functions and generics, method receiver parameters, and implicit control scopes.
The receiver's parameter Ident itself is recorded in Go Uses as well as denoting
its declaration; it binds to its own original parameter without exposing other
receiver occurrences before the normal scope start. Original application/init
bodies never execute. No ordinary source-evaluation proof is inferred.

Three [Rust test roots](../../../../../../crates/mpk-vc/tests/go_practical_source_scopes.rs)
pass, including missing-import uncertainty, variable shadow blockers, blank
imports, unknown/invalid locators, ownership after caller destruction and five
actual duplicate/short-declaration controls. After the original three-root run,
only the affected paired and import-blocker roots rerun. The unchanged duplicate
builder root's passing evidence is retained. Final strict library/test Clippy,
targeted Go vet and changed-file formats pass. The [receipt](verification.json)
records all21 executions across nine whole source epochs/6510 preimages.

Four original failures remain: an initial Rust private-helper visibility error;
the first native test's observation-count condition (83 uses, with no missing
use output reconstructed); an actual native dot-import/package declaration
collision in the expanded case; and the original partial Rust receiver-binder
comparison. Complete original source epochs, actual drivers and raw stdout/stderr
remain. All three native execution directories and all463 emitted Rust files are
losslessly retained with their original modes. The [literal byte audit](original-byte-audit/result.json)
compares every decoded file to its retained execution original; the unchanged-
rule [input review](input-review.json) scans all decoded originals and Go source
buffers without a new source, JSON, directory, seal or blanket exclusion. Its
first run incorrectly compared the53 root Go files to the complete189-input
frontend closure count. The [actual failure](initial-review-failure/result.json)
and driver/output remain; the corrected check compares the root set to that
closure's actual root members while still checking all189 complete inputs.
The next review used an obsolete checkpoint metadata key; its [original failure](second-review-failure/result.json)
also remains. Correcting that key leaves the actual source/type check results
unchanged.

Parser/lexer and existing importer behavior, pinned Unicode tables, all Go
production frontend189 inputs, frozen255 design members and ordinary448 paired
proof/core/evaluator/registry evidence remain unchanged and are retained without
unrelated runs. A private forwarding helper exposes the existing pinned Lu
predicate to the new scope builder without changing the importer's validation.
The graph's package/import metadata is still candidate metadata, not complete
module/build/version/capture authority. This lookup supplies no constant values,
type-expression denotation, nominal/alias correctness, source/carrier/storage/
permission/lifetime/state/program proof, full U/Plain/weights/counts/capture
budgets or56-family foundation/private descriptor/registry/context closure.
Those full requirements, nativeLinux/generalnamespace/selectedclosure/all169/
T02–T11 remain due. Go and Rust only; R01/R06/W01 remain open. The public practical
profile stays inactive and the T02 whole gate stays W04: `./scripts/check-fast.sh`.
