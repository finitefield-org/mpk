# Case-folded PGO lookup investigation

Status: investigation, no completion or source/program authority claim.
The Go frontend build consults the path `default.pgo`; the spelling stored in
an observed directory can differ on a filesystem that resolves names without
case sensitivity. A directory listing alone cannot determine whether the
compiler's lookup resolves such an entry. Linux containers can also use such a
filesystem via a host bind mount.

The reproduction retains the whole current Go test module, the complete
registered 269-file historical source epoch and registered SDK inventory.
It adds the exact actual SDK profile to the restored frontend as `DEFAULT.PGO`,
then compares the actual Go build plan with the auto-mode capture. The supplied
Go test is introduced through a test-only overlay; it does not replace any
original source. Its original failure, logs and complete test inputs remain
available as evidence.

Any correction must distinguish the mode (off or auto), the actual filesystem
lookup of `default.pgo`, the directory entry under its original spelling, and
the exact captured bytes. The independent Rust side must not treat a producer's
chosen path or claimed absence as authority: a held root must corroborate the
lookup and ensure that the exact referenced object is the captured file, while
preserving existing off-mode behavior. A case-sensitive filesystem with an
ignored differently spelled entry must remain accepted; these cannot be
handled by indiscriminately reading or rejecting all case-folded names.
The checks must continue retaining full original program and compiler inputs,
existing limits and no-follow root restrictions. Launcher selection, source
proof and native Linux release acceptance remain separate obligations.
