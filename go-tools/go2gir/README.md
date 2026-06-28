# go2gir

`go2gir` is the untrusted Go frontend command that will convert the accepted Go
subset into GIR.

GO-003 adds SSA construction for the target package. The command accepts one
package path, loads it through `golang.org/x/tools/go/packages` with pinned
settings, builds SSA with `golang.org/x/tools/go/ssa`, and emits deterministic
JSON containing both the package summary and source-function SSA dump. Feature
rejection and GIR emission are implemented by later milestones.

```sh
go run . ./testdata/samplepkg
```
