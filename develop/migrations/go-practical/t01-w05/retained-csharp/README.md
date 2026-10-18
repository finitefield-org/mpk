# Retained CSharp epochs and current pending work

Go T01 keeps the CSharp checkpoint pending: T01-W09 is In progress, while
T01-W10 and T06-W09 are Blocked. All 987 application proofs remain pending.
The current Boolean kernel guards are independently exercised by the existing
feasibility tests. Historical reconstruction cannot finish that refreeze.

Three exact epochs have different purposes:

- `published-epoch.json`: commit `16946c6ae2a82004be03f49ba52726001dd3317f`,
  all 1,645 tracked files / 65,403,230 bytes. Both original Python helpers
  reproduce the original 709-vector freeze/publication without modification.
- `private-epoch.json`: commit `40469d298f1d69558713d02ea0b553a1dd583bf5`,
  115 exact inputs / 2,871,061 bytes. Its original Boolean freeze helper
  reconstructs its recorded candidate. The two 79-file receipt inventories
  bind these historical inputs; five core/checker inputs have since changed.
- `entry-epoch.json`: the three CSharp publication/status documents at the
  Go entry main commit `1c34ed5701df2e53fd515d49ad4d158cc728cb4b`.
  Their current bytes remain identical. The vector payload also remains
  identical to the older published epoch. Later status notes in the two
  prose specifications retain their entry bytes and original scope.

`develop/probes/go-practical/retained_csharp_epoch.py` verifies fixed commit,
tree, blobs, modes, raw SHA-256 and sizes before exporting regular files to
an isolated temporary directory. It verifies all inputs again after executing
the unchanged historical Python helpers with bytecode writes disabled. It
changes no Git ref or checkout. Run it outside the sandbox, as required for
all Git operations. Rust metadata tests read exact recorded historical blobs;
source/model generation code in the retained probes is unchanged.

The current helper still reports the known `bool_cases.rs` source-pin mismatch.
A dedicated test retains that precise pending state and verifies the inactive
CSharp registry boundary. The historical publication assertion uses its
original 16 evidence records and three specification members. The unpublished
Boolean evidence and core member remain part of the separate pending candidate.
Historical passing observations do not describe current application proofs.

`consumer-extension.json` retains the original inventory and its exact
147-path fingerprint at commit `1207d19c7f05bdc0ae3df285590d33f1e2e3849d`.
Twenty-nine explicit added references give the current 176-path fingerprint:
21 CSharp source/test paths, seven Go/core specification paths and the Boolean
fixture README. There are no deletions. Every added path has an exact raw-byte
pin, every current path remains classified, and added/deleted consumer controls
still fail. Original inventory bytes, cache corrections and search roots stay
unchanged. The active/private labels retain their existing classification
meaning and grant no new installed profile authority.

Current ordinary boundary document/fragment/UTF8 golden files live in new
`boolean-refreeze/` directories beside their retained predecessors. Generation
reconstructs all 68 original contexts in each family and creates three nonempty
outputs per family. Nine files share three byte-identical definition
certificates; each distinct byte sequence passes both current Rust and Go
source-free checkers with zero axioms and matching hashes. These are ordinary
definition certificates. Original application theorems remain pending.
Normal source/import/golden tests and the cumulative T01 gate are separate
checks recorded in `../verification/retained-csharp-repair/`.

The shared aggregate pipeline already groups four StepTwo operations into
one StepEight, then composes 2,048 groups. Its exact charged count is
4 + 2,048 = 2,052; the old 8,217-count assertion was stale. The source
implementation and the 16,384-cell capacity stay unchanged. The focused
reconstruction/import/mutation test passes with the current exact count.
