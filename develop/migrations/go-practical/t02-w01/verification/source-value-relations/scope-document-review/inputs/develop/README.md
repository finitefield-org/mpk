# MPK development

MPK accepts program sources in Go and Rust only. See the normative
[language scope](specs/LANGUAGE_SUPPORT_SCOPE_V1.md),
[semantic registry](specs/SEMANTIC_PROFILE_REGISTRY_V1.md),
[Go practical design](docs/09_go_practical_subset_design.md),
[Go work plan](docs/09_go_practical_subset_design-todo.md), and
[Go implementation ledger](docs/go-practical-implementation-traceability-ledger.md).

The practical Go profile is under development. Its ordinary source semantics,
full function/program correspondence, paired checked certificates and native
Linux gates must all be completed before activation. Continue the planned
feature priority and serial T/W order; removed-language releases are not
prerequisites.

`specs/` and `specs/vectors/manifest.json` own current contracts and conformance
vectors. `probes/go-practical/` contains development tools. `migrations/`
preserves original inputs, failures and historical receipts; these apply only
to their original bytes and do not confer current support or acceptance.
`docs/`, `tasks/` and `roadmap/` contain supporting design and tracking files.
Older multi-language plans are retired historical records. They do not
authorize another frontend, future language phase or release prerequisite.

Execute focused local validation during each W. At the final W of each T,
run `../scripts/check-fast.sh` over the cumulative changes. Release validation
uses the root README's local native Linux Go/Rust gates. Never create, run,
monitor or rely on GitHub Actions or workflow files.

The trusted acceptance boundary remains canonical certificates checked by the
Rust kernel and the independent Go reference checker. Source, contracts,
frontends, compilers, registries, VIR/VC, reports and AI output are untrusted.
