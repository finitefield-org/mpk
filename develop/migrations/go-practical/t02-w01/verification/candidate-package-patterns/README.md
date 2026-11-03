# Candidate package patterns

The Go 1.25 command treats `all`, `std`, `cmd`, `tool` and `work` as
meta-package patterns. Candidate package derivation rejects all five exact
spellings before invoking the compiler. Paths such as `example.com/tool` and
`example.com/work` remain ordinary package paths.

The registered Go 1.25 toolchain was run offline against real module roots
named for all five patterns. Even when invoked as `go list .` from each root,
Go rejected the resulting import path as non-importable. A separate original
module contained real `p...q` and `p123q` source packages: `go list .` from
`p...q` resolved that one package, while passing its import path as a command
argument expanded to both packages. The full 183-package registered-SDK
candidate source/export test and the existing 8,192-root package-derivation
test also passed. These tests were selected to cover the added guard, ordinary
path components, actual Go pattern semantics and the shared capture path.

Literal `...` import paths remain unsupported by the candidate capture. A
correct general solution must retain the original main-module environment,
exact selected roots, every native observation and the existing resource
limits even when multiple selected packages are in different modules. Merely
filtering a wildcard's extra results would not establish that correspondence.
Native argument-size handling also remains open. Go vet, changed-file
formatting and Go 1.23 compile-only compatibility passed locally; these are not
a native Linux release gate. W01 remains incomplete, and the T02-wide
`./scripts/check-fast.sh` gate is deferred to final T02-W04. No validation
archive is committed.
