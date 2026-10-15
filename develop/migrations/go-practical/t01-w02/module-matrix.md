# T01-W02 Capture and Verification-Closure Matrix

Status: measured private Go 1.25.0 observations; not profile or proof acceptance.

Report SHA-256: `c4ce51fdf23a8b55b6dbe0e9675fd710f37040ba183e45c868d3eb43fbd1320f`.
Each row passed in two fresh original-source roots; dependency and source snapshots retained identical bytes.

| Case | Outcome | Bodies selected/dependent | Type-check-only bodies | Finding codes |
| --- | --- | --- | --- | --- |
| unused-unsupported-body | closed | 1 | 1 | — |
| unused-http-handler | closed | 1 | 1 | — |
| direct-helper | closed | 2 | 0 | — |
| external-helper | closed | 2 | 1 | — |
| unused-external-initializer | closed | 1 | 3 | — |
| unused-require-metadata | closed | 1 | 0 | — |
| imported-domain-type | closed | 1 | 2 | — |
| unused-init-and-generate | closed | 1 | 2 | — |
| reachable-global | rejected | 1 | 0 | GO_PROBE_GLOBAL_STATE |
| unproved-unreachable-branch | rejected | 2 | 0 | GO_PROBE_GLOBAL_STATE |
| reachable-map-helper | rejected | 2 | 0 | GO_PROBE_UNSUPPORTED_TYPE |
| indirect-call | rejected | 1 | 0 | GO_PROBE_INDIRECT_CALL, GO_PROBE_UNSUPPORTED_TYPE |
| interface-dispatch | rejected | 1 | 0 | GO_PROBE_INDIRECT_CALL, GO_PROBE_UNSUPPORTED_TYPE |
| recursive-call | rejected | 2 | 0 | GO_PROBE_CALL_CYCLE |
| unused-generic | closed | 1 | 1 | — |
| reachable-generic | rejected | 2 | 0 | GO_PROBE_GENERICS, GO_PROBE_UNSUPPORTED_TYPE |
| original-package-type-error | source_error | — | 1 | — |
| unmodeled-standard-call | rejected | 1 | 0 | GO_PROBE_UNMODELED_EXTERNAL_CALL |
| language-go122 | closed | 1 | 0 | — |
| language-go123 | closed | 1 | 0 | — |
| language-go124 | closed | 1 | 0 | — |
| language-go125 | closed | 1 | 0 | — |
| old-module-language | rejected | — | — | GO_PROBE_LANGUAGE_VERSION |
| future-module-language | rejected | — | — | GO_PROBE_LANGUAGE_VERSION |
| newer-module-patch | rejected | — | — | GO_PROBE_TOOLCHAIN_REQUIREMENT |
| newer-toolchain-directive | rejected | — | — | GO_PROBE_TOOLCHAIN_REQUIREMENT |
| file-language-downgrade | rejected | 1 | 0 | GO_PROBE_LANGUAGE_VERSION |
| explicit-build-tag | closed | 2 | 0 | — |
| explicit-linux-target | closed | 1 | 0 | — |
| local-replacement-in-capability | closed | 2 | 1 | — |
| local-replacement-escape | rejected | — | — | GO_PROBE_REPLACEMENT_ESCAPE |
| versioned-replacement | closed | 2 | 1 | — |
| missing-offline-module | source_error | — | — | — |
| root-workspace-rejected | rejected | — | — | GO_PROBE_BUILD_POLICY |
| vendor-mode-rejected | rejected | — | — | GO_PROBE_BUILD_POLICY |
| module-metadata | closed | 2 | 1 | — |
| closed-value-error-signature | closed | 1 | 1 | — |
| unused-assembly-input | closed | 1 | 0 | — |
| reachable-assembly-rejected | rejected | 1 | 0 | GO_PROBE_UNMODELED_EXTERNAL_CALL |

`closed` means the private early-exclusion closure probe completed. It is not a practical-Go admission or theorem verdict.
`source_error` means original package/module loading failed; no verification closure is admitted.
Multiple source initializers have separate identities and remain outside proof. Assembly files are retained as build inputs; reaching an unmodeled assembly body rejects.

The module/file raw version and language minor are retained separately. See [the snapshot contract](offline-snapshot-contract.md) and [the full observations](module-probes.json).
