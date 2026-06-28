# go2gir

`go2gir` is the untrusted Go frontend command that will convert the accepted Go
subset into GIR.

GO-006 adds struct and fixed-array lowering to GIR. The command accepts one
package path, loads it through `golang.org/x/tools/go/packages` with pinned
settings, rejects features outside Go subset v0 with an exact JSON report,
builds SSA with `golang.org/x/tools/go/ssa`, and emits deterministic JSON
containing the package summary, source-function SSA dump, and GIR for supported
pure functions including struct field reads and fixed-array indexing. Canonical
GIR hashing is implemented by later milestones.

```sh
go run . ./testdata/samplepkg
```
