# go2gir

`go2gir` is the untrusted Go frontend command that will convert the accepted Go
subset into GIR.

GO-001 initializes only the command-line entry point. The command accepts one
package path and emits a deterministic JSON acknowledgement. Package loading,
SSA construction, feature rejection, and GIR emission are implemented by later
milestones.

```sh
go run . ./example/package
```
