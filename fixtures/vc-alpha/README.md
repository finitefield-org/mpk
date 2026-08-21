# VC Alpha Corpus

`fixtures/vc-alpha` is the active alpha-branch VIR/VC v1 corpus.

The corpus records 33 source functions, 33 VC v1 members, and 66 grouped
skeleton declarations. The artifacts are helper theorem obligations only;
they are not proof evidence unless a source-free checker accepts the matching
canonical certificate bytes.

Regenerate the checked-in artifacts from the repository root with:

```sh
./scripts/regenerate-go-vir-corpus.sh --update
./scripts/regenerate-go-vir-corpus.sh --check
```

Review every artifact and hash change before committing an update.
