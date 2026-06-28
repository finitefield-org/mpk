# go2gir

`go2gir` is the untrusted Go frontend command that will convert the accepted Go
subset into GIR.

GO-002 adds package loading with pinned settings. The command accepts one
package path, loads it through `golang.org/x/tools/go/packages`, and emits a
deterministic JSON package summary. SSA construction, feature rejection, and GIR
emission are implemented by later milestones.

```sh
go run . ./testdata/samplepkg
```
