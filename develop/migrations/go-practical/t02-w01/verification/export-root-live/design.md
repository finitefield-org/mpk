# Live Go export-cache observation: design boundary

T02-W01 remains in progress. The earlier live comparison holds the original
application, dependency and SDK roots before source capture. Its complete
5,697-object owner contains 181 additional export objects, but that comparison
cannot authenticate the separate export-cache root. Reconstructing only the
captured export files and inventing unread cache entries would not be valid.

The scoped observer opens the actual Go-generated cache after the native
compiler command finishes and before the export session reads any files. Go
then seals its original export snapshot and hands the observer independently
owned copies of the snapshot inputs while the original cache is still live.
The Rust observer checks the entire reported namespace and every captured byte
against its held directory descriptor in two passes. A test-only mutation of
those copies must not alter the original replay or compiler owner; the original
full 182-package application, all source and export objects and all 5,577 SDK
files remain in the existing test.

The observation is tied to this actual test capture. It neither authorizes a
launcher-selected path nor proves that the compiler or its outputs implement
Go semantics. A further private owner must compare the complete observed
export capability, along with all non-export capabilities, to the same
independently imported compiler/export owner. No source, type, error or proof
universe is reduced; no practical profile or native release is activated.
