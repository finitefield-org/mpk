# Retained JSON boundary source epochs

The two original JSON files are exact Git objects from
`09a14bdde6b14408e4f25e55cb787dd989577d7e`, the commit that retained the
JSON string parser's previous pair pipeline. [epoch.json](epoch.json) pins
their source paths, tree, byte counts and hashes. Their current counterparts
stay in `develop/migrations/csharp-03/boundary-output/` and are unchanged.

All three source bodies, roots, complete contract bodies and native facts
agree across these epochs. The differences are the request/response IDs,
sidecar semantic contexts, sidecar self hashes and six corresponding raw
sidecar digests. The test helper pins all four original/current files and
compares the complete bodies after removing only those context-derived fields.
It first recomputes every native input digest from its actual raw input bytes.
Altered source, contract and root controls reject the correspondence.

The original fixture includes a lone UTF-16 surrogate in a contract value.
The first scalar-only Rust guard rejects that retained value before comparing
bodies; its source, binary identity and two failed observations are preserved
in the `json-boundary-repair/initial-scalar-json-guard` record. The corrected
guard uses the existing lossless practical JSON parser/writer and additionally
requires a lone-surrogate replacement control. The original fixture bytes are
unchanged. Rust guard completion is reported separately from the independent
Python source comparison and the still-pending cumulative passing gate.

The closed correspondence selects an original archive member when comparing
its definition closure and full expanded scan with a newly generated current
certificate. It does not rename an old member, rebind its historical metadata,
or supply source/application proof authority. Current golden files are generated
by the real 68-context reconstruction tests into separate Boolean-refreeze
directories; generation and normal golden verification are separate checks.
The Go T01 cumulative gate and both-checker definition checks remain separately
reported. All Go implementation/application proofs and CSharp's 987 application
proofs remain pending; neither practical profile is activated here.
