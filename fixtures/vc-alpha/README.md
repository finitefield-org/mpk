# VC Alpha Corpus

`fixtures/vc-alpha` is the ALPHA-002 VC corpus.

The corpus records 1,056 branch verification-condition obligations generated
from the 33 ALPHA-001 branch cases expanded to 16 postconditions per case.
The artifacts are candidate theorem obligations only; they are not proof
evidence until later certificate generation and checker milestones.

Regenerate the checked-in artifacts from the repository root with:

```sh
MPK_UPDATE_VC_ALPHA=1 cargo test -p mpk-vc --test alpha_corpus
cargo test -p mpk-vc --test alpha_corpus
```
