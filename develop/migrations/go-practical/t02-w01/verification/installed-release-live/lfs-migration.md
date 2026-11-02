# Git LFS migration of a retained input

The existing `module-observations/rust-original/inputs.tar.gz` is required by
the recorded review and exceeds GitHub's 100 MiB ordinary Git blob limit. It
is tracked through Git LFS so its exact 150,916,960 bytes remain available at
the same path on an LFS-enabled checkout. Its SHA-256 is
`420b1d663e28f0315b9508b8882b6b31ab25689727a2c6b57cf66d5d8daa081d`.

The unpushed branch history was migrated with only that path selected and with
`origin/main` excluded. The source archive was restored from the local LFS
object and rehashed; the complete pre-migration and post-migration trees differ
only in that Git representation and the new `.gitattributes` entry. Before
rewriting, the original branch tip was retained locally at
`refs/codex/go-practical-before-lfs` (`d26ad589109fb56300dcdad19f981bc609f1264f`).
The accompanying commit map records the old and new commit IDs. Local QEMU
checks do not constitute native Linux release acceptance.
