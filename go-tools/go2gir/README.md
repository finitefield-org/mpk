# go2gir

`go2gir` is the untrusted Go frontend command that will convert the accepted Go
subset into GIR.

GO-004 adds unsupported-feature detection. The command accepts one
package path, loads it through `golang.org/x/tools/go/packages` with pinned
settings, rejects features outside Go subset v0 with an exact JSON report, then
builds SSA with `golang.org/x/tools/go/ssa` and emits deterministic JSON
containing both the package summary and source-function SSA dump. GIR emission
is implemented by later milestones.

```sh
go run . ./testdata/samplepkg
```
