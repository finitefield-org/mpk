# Policy Scan and Evidence v1 Specification

Status: normative and frozen for implementation.

This specification defines the untrusted policy scan document
`mpk.policy.scan.v1`, the untrusted policy evidence document
`mpk.policy.evidence.v1`, the generic `mpk policy` command-line contract, and
the structured reproduction recipes derived from one completed,
evidence-producing `policy verify` invocation. Neither policy document is proof
evidence by itself. Only canonical certificates and theory certificates
accepted by the configured source-free checkers are trusted evidence.

Policy v1 is the sole active policy boundary. There is no adapter for removed
predecessor policy documents at a parser or CLI boundary.

## 1. Conformance language, transport, and validation order

The terms MUST, MUST NOT, REQUIRED, and REJECT are normative. REJECT means
that the consumer returns no validated scan or evidence value and does not
promote any property or member to `mpk_verified`.

Every object and tagged union is closed. Unknown or inapplicable fields,
missing required fields, duplicate JSON object names, and `null` reject.
Strings and enum values are case-sensitive. Floating-point JSON numbers and
integral values outside `[-9007199254740991, 9007199254740991]` reject. A
`Sha256` is exactly 64 lowercase hexadecimal ASCII characters. JSON strings
contain only Unicode scalar values and receive no Unicode normalization.

Standalone scan and evidence files are exactly RFC 8785 JCS followed by one
ASCII LF:

```text
JCS(document) || 0x0a
```

The LF is transport framing and is not part of any repeated artifact hash.
A BOM, leading whitespace, pretty printing, CRLF, missing LF, more than one LF,
or trailing bytes rejects. In-memory conformance-vector values have
order-insensitive object keys; every array order specified below is semantic.

A scan parser validates in this first-error order:

1. `transport`: byte, UTF-8, JSON nesting, string, collection-count,
   duplicate-name, and number limits;
2. `shape`: exact schema and success/non-success root branch;
3. `scalar`: hashes, IDs, profile names, paths, issues, and enum values;
4. `order`: issue, helper-artifact, contract, and nested release-projection
   ordering;
5. `profile`: language/semantic-parameter pairing;
6. `release`: exact selected registry, frontend, toolchain, and limit-profile
   projection;
7. `source_linkage`: for `ir-lowered`, exact source-manifest, source-map, VIR,
   selection, input-set, contract, and helper repetition;
8. `canonical_size` and `canonical_transport`.

Evidence validation uses the same `transport`, `shape`, and `scalar` phases.
Its fourth `order` phase validates issue, nested release-projection, helper,
certificate, checked-declaration, declaration-dependency, theory-certificate,
checked-member-ID, property, member, note, and evidence-reference arrays. It
then validates:

1. `profile`: language/semantic pairing, registered strategy tuple, checker
   profile, and explicit axiom profile;
2. `release`: exact registry/frontend/toolchain projection;
3. `source_linkage`: exact successful internal scan, frontend-stage manifest,
   source map, and VIR repetition;
4. `manifest_lifecycle`: certificate manifest differs from the retained
   frontend manifest only by `vc_hash` and the derived manifest self-hash;
5. `vc_linkage`: complete canonical VC validation and repeated source,
   input-set, profile, and verification-limit identities;
6. `helpers`: exact source, contract raw/normalized, VIR, and VC helper rows;
7. `trusted`: certificate identity, the complete generated-declaration set
   and each declaration's non-dependency fields, theory-certificate,
   axiom-report, and checker-verdict projections; declaration dependency edges
   are deliberately deferred to `dependencies`;
8. `properties`: member/group/declaration bindings and status derivation;
9. `dependencies`: exact direct generated-declaration dependency edges,
   accepted containing declarations, and their complete transitive closure;
10. `recipes`: exact two recipes, argv order, fixed outputs, and invocation
    option repetition;
11. `canonical_size` and `canonical_transport`.

The first failing phase owns the stable code in section 12. A later VC hash,
recipe, or canonical-transport error cannot hide an earlier crossed strategy
tuple, manifest mutation, or unchecked declaration dependency.

After evidence validation succeeds, the `policy verify` route derives Markdown
in a `report` phase through checked append counters; an append that would cross
`markdown_bytes` fails before allocating or exposing a complete oversized
string. Once rendering completes within the limit, it validates the complete
derived result against section 9.1 and only then enters output publication.
`POLICY_LIMIT_MARKDOWN_BYTES` belongs to this phase. A standalone evidence
parser does not require a report destination, but the same renderer returns
this error rather than a partial string.

No field in either document may carry an absolute source, workspace, registry,
installation, bundle, toolchain, executable, home, temporary, sandbox, or
output path. A path-like string beginning with `/`, a drive-letter or UNC
prefix, `file:`, or the private rooted prefix `/mpk/` is forbidden. This rule
does not reject `/mpk/` occurring inside a non-path identity such as the Go
package `example.com/mpk/vector`. The only path values are portable
source-root-relative input paths and the fixed relative recipe output names
defined in section 9.

For the externally selected `selection` union, `shape` accepts either complete
closed Go or Rust object without yet consulting `source_language`. `scalar`
owns a language value other than `go` or `rust`, and `profile` owns a valid
language paired with the other branch. Status- and `kind`-tagged unions whose
tag is inside the object remain owned by `shape` as stated above.

## 2. Generic policy CLI contract

### 2.1 Exact routes and option sets

The scan route is:

```text
mpk policy scan <source-root>
  --language <go|rust>
  --semantic-profile <profile>
  --require-release-registry-id <registry-id>
  --require-release-registry-sha256 <sha256>
  --frontend-bundle <bundle-id>
  --toolchain-bundle <bundle-id>
  --target <target-id>
  --package <package>
  --function <function-id>
  --contract <normalized-relative-path> ...
  --json-out <output-path>
```

The verify route has the same source and release selection and additionally
requires verification profiles and two outputs:

```text
mpk policy verify <source-root>
  --language <go|rust>
  --semantic-profile <profile>
  --require-release-registry-id <registry-id>
  --require-release-registry-sha256 <sha256>
  --frontend-bundle <bundle-id>
  --toolchain-bundle <bundle-id>
  --target <target-id>
  --package <package>
  --function <function-id>
  --contract <normalized-relative-path> ...
  --strategy-profile <profile>
  --checker-profile <profile>
  --axiom-profile <profile>
  --evidence-json <output-path>
  --evidence-md <output-path>
  [--strict]
  [--update-fixtures]
```

At the process boundary, `mpk`, `policy`, the route, and `source-root` are the
first four separate UTF-8 argv elements in that order. `source-root` is
nonempty and does not begin with `--`; a locator that would begin that way must
be given with an explicit relative or absolute directory prefix. Every
remaining token is an exact long option shown below, followed by a separate
value element when applicable. `--name=value`, short aliases, option
abbreviations, response files, and a `--` option terminator are not accepted.
After `source-root`, any element beginning with `-` that is not an exact
allowed or forbidden long-option token is `POLICY_CLI_UNKNOWN_OPTION`; any
other bare element is an extra positional argument.
Value options and Boolean flags may otherwise appear in any order, which does
not affect the normalized parsed value or recipes. The sole post-route argument
`help`, `-h`, or `--help` prints route help and exits 0 without validating
configuration, launching a frontend, or writing output; a help token mixed
with any other route argument is an unknown option or unexpected positional
argument under the normal rules.

There is exactly one positional `source-root`. Every shown value option is
mandatory. `--contract` is the only repeatable option and occurs at least once;
all other options and Boolean flags occur at most once. `--strict` and
`--update-fixtures` take no value. Every verify-only option shown above—both
flags, the three verification profiles, and the two evidence outputs—is
recognized only on `verify`; any of those names on `scan` rejects as
`POLICY_CLI_UNKNOWN_OPTION`. Conversely, `--json-out` is scan-only and is an
unknown option on `verify`.

The exact forbidden locator-option set is:

```text
--frontend
--frontend-helper
--driver
--removed-frontend
--toolchain-root
--toolchain-path
--registry
--registry-path
--release-registry-path
```

These names reject with `POLICY_CLI_FORBIDDEN_LOCATOR`. Other unrecognized
options reject with `POLICY_CLI_UNKNOWN_OPTION`. There is no `PATH`, adjacent
binary, environment, project toolchain file, registry path, or default bundle
fallback. The four release options are selections/assertions, never locators.

`--language`, `--semantic-profile`, registry assertions, bundle IDs, target,
package, function, and contracts are passed to one generic runner invocation.
`verify` consumes the exact retained internal scan result and canonical
frontend-stage manifest bytes. It does not start a second frontend, reread a
contract, or reconstruct an executable path.

### 2.2 Source selection and portable arguments

`source-root` is a CLI-only locator. It is resolved, opened, and retained by
the capture boundary, but its spelling never enters a policy artifact. The
recipe replacement is the literal `.`.

Before allocating input buffers, the generic pre-sandbox staging boundary
enforces inclusive operational maxima of 65,536 staged files, 33,554,432 bytes
per staged file, 536,870,912 staged bytes total, 65,536 visited directories,
262,144 examined directory entries, and the shared portable-path maximum of
1,024 bytes per relative path. It opens entries descriptor-relative without
following links, rejects staged inode aliases when the selected frontend
profile forbids them, checks file identity before and after one bounded read,
and re-enumerates the retained namespace. Go byte-bearing candidate
classification is its exact frozen filename set, so wrong-language sources and
build metadata consume no Go staged-file or staged-byte budget. Rust permits an
arbitrary portable `[lib].path`; its broader transport therefore retains every
regular file's bytes under the operational maxima, and the validated frontend
manifest later selects the exact Rust closure.

The private sandbox projection nevertheless preserves every enumerated
directory and every Go-noncandidate regular entry name. Directories retain
their directory kind; Go-noncandidate regular names are represented by empty
private placeholders because the Go profile excludes them before open. This
lets the frontend observe the retained directory-entry counts, excluded test
names, forbidden directory names, and candidate-name file kinds. Private
placeholders never enter a public source manifest and are bounded by the
examined-entry maximum rather than the byte-bearing staged-file maximum. The
two-pass comparison retains full stable metadata for traversed directories and
byte-bearing candidates. A Go placeholder, and a skipped `.git` directory,
contributes only its path and regular-file/directory kind; its excluded bytes,
size, identity, permissions, timestamps, and unvisited descendants cannot
affect acceptance or any public artifact.

The operational count and byte maxima are strictly larger than either
frontend closure profile's corresponding maxima. A candidate at a frontend
maximum plus one therefore reaches that frontend and retains its language
profile limit status/code. Generic staging is nevertheless a broader transport
inventory, not a semantic closure and not a replacement for frontend capture;
an input tree that exceeds an operational maximum fails artifact-free with
`POLICY_CLI_INPUT` before launch. Only the validated successful source manifest
selects the exact retained input closure.

The exact selection object is the union in `FRONTEND_PROTOCOL_V0.md`:

```json
{"package":"example.com/mpk/vector","function":"example.com/mpk/vector.Identity"}
```

for Go, and:

```json
{"package":"payment-policy","crate":"payment_policy","kind":"lib","function":"payment_policy::approved_reserve_cents"}
```

for Rust. For Rust, `crate` is the first canonical function-ID segment and
`kind` is the profile constant `lib`; for Go, `package` is the canonical import
path containing the selected function. A user cannot supply the derived Rust
fields separately.

For Go, the canonical function ID must begin with the exact selected package
followed by `.` and a nonempty function component. For Rust, it must contain a
nonempty first crate segment, `::`, and a nonempty remaining item path; that
first segment is copied to `selection.crate`. The Cargo package name is not
used to guess the crate segment because an explicit library target name may
differ from it. Violating either relational grammar is `POLICY_CLI_SCALAR`
before release lookup.

Every contract argument already satisfies the portable normalized input-path
grammar in `SOURCE_MANIFEST_V0.md`. Absolute paths, `.` or `..` components,
empty components, backslashes, drive/UNC prefixes, URI forms, and ASCII
case-fold collisions reject rather than normalize. After immutable capture,
contract paths are unique and sorted by UTF-8 bytes. Caller argument order
therefore cannot change frontend input, VIR, policy JSON, or recipe bytes.

`--json-out`, `--evidence-json`, and `--evidence-md` are normalized portable
paths relative to the CLI working directory, using the same component grammar
and 1,024-byte ceiling. They cannot be absolute, contain `.`/`..`, use a URI or
private path, or escape the retained working-directory root. The two verify
destinations are distinct under both byte equality and ASCII case folding.
Their caller spellings are never copied into scan/evidence JSON; recipes use
the fixed names in section 9.

### 2.3 CLI first-error precedence

CLI validation completes before frontend launch in this exact order:

1. route/help recognition;
2. option-token recognition, with a listed forbidden locator before the
   generic unknown-option result;
3. option arity, missing values, Boolean values, duplicate non-contract
   options, positional count, and duplicate identical contract arguments;
4. presence of every mandatory option and at least one contract;
5. scalar grammar for language, hashes, IDs, target, package, function,
   relational selection, portable contract paths, and output paths;
6. language/semantic-profile and target-derived semantic-parameter
   compatibility;
7. on verify, known strategy, checker, and axiom profiles followed by the exact
   strategy tuple in section 3;
8. registry ID/hash equality against embedded constants;
9. installed registry validation, followed by registered release tuple and
   bundle compatibility, then all-bundle metadata, selected-bundle snapshot,
   host, runtime, and sandbox validation;
10. route output collision and safe-write policy;
11. frontend launch.

A failure through step 8, the tuple/bundle-compatibility part of step 9, or step
10 is a configuration failure: it exits 2, writes no policy JSON or Markdown,
and launches no frontend. A crossed known strategy tuple therefore rejects
before registry I/O or child launch. Failure in the installed-registry,
installed-bundle, host, runtime, or sandbox validation part of step 9 is the
artifact-free release `frontend-error` from `RELEASE_BUNDLES_V0.md`: it exits 1
and also does not create a scan document.

Because the build embeds only the registry ID/hash commitment, an unavailable
or invalid installed registry is observed before tuple membership can be
resolved. No duplicate build-embedded tuple allowlist is permitted to reverse
that release-boundary precedence.

Scan output uses create-new semantics and rejects any existing destination.
Verify validates both destinations before writing either. An existing
untracked destination always rejects. Existing tracked regular fixture files
may be replaced only when `--update-fixtures` is present. Both complete
payloads are generated and validated in memory before either destination is
published. The writer creates sibling temporary regular files with
create-new/no-follow semantics, writes and synchronizes both, revalidates both
destinations and parents, then publishes them in JSON-then-Markdown order. If
either publish or a required directory synchronization fails, it rolls back
every destination already published, restoring a retained sibling backup or
removing a newly created destination, before returning
`POLICY_CLI_OUTPUT`. A recoverable reported failure therefore has zero
committed writes. If the storage system also refuses rollback, the pair enters
the recovery-required state described below and is not a validated report. The
command never reports success or a policy-status result for a mixed pair.

This is a command-level rollback guarantee, not an impossible claim that two
independent pathnames have one crash-atomic filesystem commit point. An abrupt
process, kernel, power, or storage failure may leave private sibling temporary
or backup files. A later policy invocation refuses to publish over such state
until the implementation's recovery routine has either restored the complete
old pair or validated and completed the complete new pair; it never treats a
mixed pair as committed evidence. Symlinks, hard-link aliases, directories,
reparse points, output aliasing, and root escape reject. `--update-fixtures`
grants no permission to change source, contract, bundle, registry, or
non-output files.

## 3. Independent profiles and the strategy registry

The initial v1 strategy registry has exactly these two rows, in the shown
order:

| `strategy_profile` | `source_language` | `semantic_profile` | `axiom_profile` |
|---|---|---|---|
| `payment-policy-alpha` | `go` | `mpk.go.fixed.v0` | `zero-axiom` |
| `payment-policy-rust-alpha` | `rust` | `mpk.rust.checked.v0` | `mvp-theory` |

The row key is `strategy_profile`; every value is exact. A known strategy with
the other known language, semantic profile, or axiom profile is a crossed
tuple and rejects with `POLICY_PROFILE_TUPLE`. It is never normalized to an
unknown strategy. An unknown strategy rejects with `POLICY_PROFILE_UNKNOWN`.

`checker_profile` is a separate required field and is not embedded in the
strategy string or axiom profile. The exact checker-profile registry remains
`core-bootstrap`, `mvp-structural`, and `mvp-strict`, with proof-node support
defined by `CERT_V0.md`. Selecting one never changes the semantic profile or
axiom allowlist. Evidence may be proof-pending under a checker profile that
cannot accept a required proof node.

`axiom_profile` is one explicit selection. Evidence v1 has no
`retired_axiom_list`, implicit list, default, or category-wide widening.
The profile names and approved concrete axiom identities remain governed by
`AXIOM_POLICY_V0.md`. The Go strategy selects `zero-axiom`; the Rust strategy
selects `mvp-theory` without adding a Rust axiom category.

Scan has no `strategy_profile`, `checker_profile`, or `axiom_profile` field and
cannot claim a verification choice that the scan route did not use.

## 4. Shared successful-source identities

The success branch of scan and every evidence document repeat these fields
member-for-member from validated upstream artifacts:

| Field | Exact source and rule |
|---|---|
| `source_language` | frontend request/envelope, VIR, and manifest |
| `semantic_profile` | exact `mpk.go.fixed.v0` or `mpk.rust.checked.v0` pairing |
| `semantic_parameters` | exact profile object from `VIR_V0.md` |
| `selection` | exact section 2.2 union |
| `limit_profile` | exact selected source-manifest value, `mpk.vir.limits.v0` |
| `release_registry` | exact path-free source-manifest registry projection |
| `frontend` | exact path-free source-manifest frontend projection |
| `toolchain` | exact path-free source-manifest toolchain projection |
| `frontend_source_manifest_hash` | recomputed frontend-stage manifest hash |
| `input_set_hash` | exact source-manifest and VC input-set identity |
| `source_map_hash` | recomputed successful source-map hash |
| `source_ir_schema` | exactly `mpk.vir.v0` |
| `source_ir_hash` | recomputed VIR `vir_hash` |

`release_registry`, `frontend`, and `toolchain` reuse the exact closed types,
scalar rules, component ordering, and complete selected-descriptor projection
from `SOURCE_MANIFEST_V0.md`. Policy v1 does not create a looser duplicate
model and never records registry, executable, component, runtime, or bundle
paths.

For a successful source result, all fields above are required. Each value is
checked against the retained canonical frontend-stage manifest bytes, the
validated frontend envelope, source map, VIR, selected release descriptors,
and immutable captured input set. Equality to caller-supplied hashes alone is
insufficient.

## 5. `mpk.policy.scan.v1`

### 5.1 Common and status branches

Every scan root has these exact common fields:

| Field | Rule |
|---|---|
| `schema` | exactly `mpk.policy.scan.v1` |
| `frontend_status` | exact validated frontend status |
| `frontend_phase` | exact status-compatible frontend phase |
| `source_language` | exact request value |
| `semantic_profile` | exact request value |
| `semantic_parameters` | exact request object |
| `selection` | exact request union |
| `release_registry` | validated selected registry projection |
| `frontend` | validated selected frontend projection |
| `toolchain` | validated selected toolchain projection |
| `readiness` | exact value below |
| `rejected_features` | exact normalized frontend issue array |
| `diagnostics` | exact normalized frontend issue array |

`rejected_features` and `diagnostics` preserve the closed Issue shape, sorting,
truncation, and public-path rules of `FRONTEND_PROTOCOL_V0.md`. Policy scan does
not rewrite compiler prose or add line/column fields.

The exact readiness mapping is:

| `frontend_status` | `readiness` |
|---|---|
| `ir-lowered` | `ready` |
| `rejected` | `unsupported` |
| `source-error` | `source_error` |
| `frontend-error` | `frontend_error` |

Only a completely validated `ir-lowered` envelope can be `ready`. A malformed,
noncanonical, identity-mismatched, killed, or otherwise unvalidated frontend
result produces no scan artifact; it cannot be represented as ready by placing
an issue in a caller-constructed document.

After atomically creating a valid scan document, `policy scan` exits 0 for all
four branches; `readiness`, not the process exit, preserves the child outcome.
Configuration and artifact-free release failures instead follow section 2.3
and create no scan document.

The `ir-lowered` root additionally has exactly the remaining shared fields from
section 4 plus `helper_artifacts`. The non-success branches omit all of
`limit_profile`, `frontend_source_manifest_hash`, `input_set_hash`,
`source_map_hash`, `source_ir_schema`, `source_ir_hash`, and
`helper_artifacts`. Those fields cannot be recovered from a prior invocation
or partial child output.

### 5.2 Helper artifacts

`helper_artifacts` is an array of the exact tagged union below. Every row has a
unique `id`. Rows are ordered first by the kind order `source`, `contract`,
`verification_ir`, `vc`, `ai_analysis`, `ci_status`, then by `id` UTF-8 bytes.

| `kind` | Other exact fields | Rule |
|---|---|---|
| `source` | `id`, `normalized_path`, `sha256` | one per source-kind manifest input |
| `contract` | `id`, `normalized_path`, `schema`, `raw_input_sha256`, `function_id`, `contract_hash` | one per normalized VIR contract |
| `verification_ir` | `id`, `schema`, `sha256` | exact VIR identity |
| `vc` | `id`, `schema`, `sha256` | exact VC identity |
| `ai_analysis` | `id`, `schema`, `sha256` | untrusted AI artifact identity |
| `ci_status` | `id`, `system`, `check`, `status`, `subject_sha256` | untrusted CI observation |

`id` is a 1-through-1,033-byte portable token matching
`[A-Za-z0-9][A-Za-z0-9._~:#/-]*`; it is an artifact-local reference, never a
filesystem path. The ceiling admits the 1,024-byte public function-ID maximum
plus the exact nine-byte `contract:` prefix, and also the seven-byte `source:`
prefix plus a maximum portable input path. `ai_analysis.schema`, `system`, and
`check` are `ProfileId` values from `RELEASE_BUNDLES_V0.md`. CI `status` is
exactly `success`, `failure`, or `pending` and `subject_sha256` identifies the
checked immutable subject bytes.

A scan success contains every and only:

- one `source` row per source-kind manifest input, with
  `id = source:` followed by `normalized_path`;
- one `contract` row per normalized contract selected into VIR; and
- one `verification_ir` row with `id = verification_ir`,
  `schema = mpk.vir.v0`, and `sha256 = source_ir_hash`.

It contains no `vc`, `ai_analysis`, or `ci_status` row. Source paths and hashes
match the manifest. A contract's `raw_input_sha256` is the digest of the exact
manifest contract input bytes, while `contract_hash` is the normalized VIR
contract self-hash repeated by its function and all `CallStatic` sites. These
two fields are never aliases and are both required even when their bytes happen
to compare equal. `schema` is the source-side `mpk.go.contract.v0` or
`mpk.rust.contract.v0`; `function_id` and `contract_hash` match exactly one VIR
contract. Contract IDs are `contract:` followed by the canonical function ID.

## 6. `mpk.policy.evidence.v1`

Evidence is produced only from an exact retained `ready` scan result. The root
has exactly these fields:

| Field | Rule |
|---|---|
| `schema` | exactly `mpk.policy.evidence.v1` |
| shared section 4 fields | exact internal scan and upstream repetition |
| `certificate_source_manifest_hash` | recomputed certificate-stage manifest hash |
| `source_vc_schema` | exactly `mpk.vc.v1` |
| `vc_hash` | recomputed VC v1 self-hash |
| `verification_limit_profile` | exactly `mpk.verify.limits.v0` |
| `strategy_profile` | exact registered strategy |
| `checker_profile` | exact independent checker profile |
| `axiom_profile` | exact strategy-compatible explicit axiom profile |
| `verification_options` | exact `VerificationOptions` |
| `helper_artifacts` | exact evidence helper array |
| `trusted_evidence` | exact section 7 object |
| `properties` | canonical nonempty section 8 array |
| `reproduction_recipes` | exactly two section 9 recipes |

`VerificationOptions` has exactly Boolean `strict` and `update_fixtures`.
These values record the verified invocation for recipe reconstruction. They do
not turn helper output into trusted evidence. A valid report with
`strict = true` may still contain proof-pending rows when the command writes
the complete untrusted evidence before returning its documented strict-mode
failure.

If verify's retained internal scan is valid but its readiness is not `ready`,
verify exits 1 and commits neither evidence output. It does not write a
standalone scan as a side effect; the user can run the explicit scan route to
persist that non-success branch. An unvalidated frontend result likewise
cannot be converted into evidence.

After both JSON and Markdown have validated and committed transactionally,
`policy verify` returns failure if either source-free checker has verdict
`rejected`, if any property is `unsupported`, or if `strict = true` and any
property is `proof_pending`. Otherwise it returns success, including a
non-strict report that explicitly contains proof-pending or helper-only rows.
A deterministic checker rejection is therefore reportable but never a
successful verification. This post-write status never converts those rows to
trusted evidence. A successful verification exits 0; every post-write failure
in this paragraph exits 1. A profile, linkage, checker crash/internal failure,
malformed-evidence, or output-transaction error occurs before commit and writes
neither report and exits 1, except for the exit-2 configuration failures in
section 2.3.

Evidence `helper_artifacts` preserves every source, contract, and
`verification_ir` row from the internal scan byte-for-byte, then adds exactly
one row with `id = vc`, `kind = vc`, `schema = mpk.vc.v1`, and
`sha256 = vc_hash`. Zero or more `ai_analysis` and `ci_status` rows may follow
in canonical order, but neither can affect a member or property's
`mpk_verified` status.

The generic CLI in section 2 has no AI/CI input option and therefore emits zero
such rows. A typed embedding may provide a complete immutable optional
observation set as a separate evidence-builder input; it hashes any AI bytes
before construction and normalizes CI records before ordering. Determinism is
over the verified invocation plus that explicit set. The two reproduction
recipes reproduce source capture and verification, not those out-of-band
untrusted observations, and a policy parser never upgrades a row merely
because its declared digest is well formed.
Complete contextual validation receives the same explicit set (empty for the
generic CLI) and rejects an extra, missing, or altered optional row with
`POLICY_HELPER_LINKAGE`; syntax-only decoding still treats every such row as
untrusted data.

The exact lifecycle equation is:

1. validate and retain the canonical frontend-stage manifest bytes;
2. validate the complete VC and recompute `vc_hash`;
3. require its source IR, input set, profile, and semantic parameters to equal
   the retained manifest and VIR;
4. copy every frontend manifest value except `source_manifest_hash`, add only
   `vc_hash`, and recompute `source_manifest_hash`;
5. retain that exact canonical value as the certificate-stage manifest even
   when proof search remains pending and no candidate certificate is emitted;
6. require every candidate certificate that is emitted to attach
   those exact opaque source-manifest bytes;
7. record the two different self-hashes as
   `frontend_source_manifest_hash` and
   `certificate_source_manifest_hash`.

The two lifecycle hashes normally differ. Evidence never contains an ambiguous
`source_manifest_hash`, and scan never contains
`certificate_source_manifest_hash` or `vc_hash`.

## 7. Trusted evidence projections

`trusted_evidence` has exactly `certificates`, `theory_certificates`,
`axiom_report`, and `checker_verdicts`.

### 7.1 Certificates and checked generated declarations

`certificates` is exactly `[]` or a singleton whose `id` is `program`. The
singleton is the one canonical candidate program certificate containing every
generated declaration for this VC and submitted as identical bytes to either
configured source-free checker. Foundation modules remain hash-pinned imports
and are not additional rows. An empty array means neither checker received a
candidate. Merely appearing here is not an acceptance claim; the two verdict
rows below are the only acceptance projection. A `CertificateEvidence` has
exactly:

| Field | Rule |
|---|---|
| `id` | exactly `program` |
| `module` | canonical checked MPK module name |
| `certificate_hash` | recomputed `MPK-MODULE-CERT-0.1` hash |
| `export_hash` | recomputed module export hash |
| `axiom_report_hash` | recomputed attached axiom-report hash |
| `checked_declarations` | every and only generated `VC.Function.*` declaration in certificate order |

No certificate path or checker command is recorded. A
`CheckedDeclaration` has exactly:

| Field | Rule |
|---|---|
| `name` | exact source VC group declaration name |
| `declaration_hash` | recomputed `MPK-DECL-0.1` interface hash |
| `function_id` | exact source VC function |
| `group_id` | exact source group ID |
| `group_kind` | exact `contract` or `panic_free` |
| `member_ids` | exact source group member array |
| `dependencies` | exact direct generated dependency identities |

A dependency identity has exactly `name` and `declaration_hash`. Names retain
the source group's UTF-8 order. The hash equals the structurally checked,
resolved candidate declaration, not a caller assertion. `checked_declarations`
means that canonical decoding, import resolution, interface hashing, and exact
VC/skeleton reconstruction succeeded; proof acceptance still requires the
verdicts below. Rows retain certificate declaration order, which is the VC's
callee-first function order and contract-before-panic-free order. Generated
declaration names are unique evidence-wide and every source VC group appears
exactly once in the singleton candidate.

After `scalar` and `order` have checked dependency identity grammar, array
order, and uniqueness, the `trusted` phase deliberately leaves semantic
resolution and exact source comparison of those names and hashes to the later
`dependencies` phase. That phase also checks edge completeness against the
already validated complete declaration set and source VC. This phase ownership
makes a dependency-edge mismatch
`POLICY_DEPENDENCY_CLOSURE`, while a missing declaration or altered checker
verdict remains `POLICY_TRUSTED_EVIDENCE`.

### 7.2 Theory certificates, axiom report, and checker verdicts

`theory_certificates` is strictly increasing by `id`. Each row has exactly
`id`, `theory`, `format`, `theory_certificate_hash`, `checker_profile`, and
`checked_member_ids`. The `id` is evidence-wide and uses the portable artifact
ID grammar from section 5.2; `theory` and `format` use `ProfileId` from
`RELEASE_BUNDLES_V0.md`. Member IDs are strictly increasing and nonempty. The
profile equals the evidence root. The array contains every and only
independently checked theory certificate embedded in a listed candidate
certificate and reached by at least one structurally matched generated
declaration. Its hash is recomputed from the canonical embedded bytes.
`checked_member_ids` contains every and only VC member whose grouped proof
reaches that exact payload-bound theory certificate. Equal
canonical theory-certificate bytes embedded more than once are represented by
one row whose member array is the canonical union. A theory certificate over a
different payload, member ID, or checker profile is not reusable.

`axiom_report` is exactly one of:

```json
{"status":"not_generated"}
```

or:

```json
{
  "status":"checked",
  "axiom_report_hash":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  "category_counts":{
    "total_axiom_count":0,
    "core_axiom_count":0,
    "builtin_theory_axiom_count":0,
    "go_semantics_axiom_count":0,
    "external_axiom_count":0
  }
}
```

Counts use safe nonnegative JSON integers, their sum equation is exact, and
the hash equals every listed certificate's `axiom_report_hash`.
`not_generated` is required iff `certificates` is empty; `checked` is required
iff it is nonempty. A checked report is therefore required for any
`mpk_verified` member. Full concrete axiom identity approval is performed
against the recomputed report under the explicit root `axiom_profile`;
category counts alone cannot authorize an axiom.

`checker_verdicts` contains exactly two rows in this order:
`rust_fast_kernel`, then `reference_checker`. Each row has exactly `checker`,
`checker_profile`, `verdict`, and `certificate_ids`. The profile equals the
root. `verdict` is `accepted`, `rejected`, or `not_run`. Certificate IDs are
strictly increasing and unique. For `accepted` or `rejected`, they equal the
complete `certificates` ID array and are nonempty: `accepted` means that checker
accepted the candidate, while `rejected` means it rejected the candidate.
`not_run` requires an empty list. A nonempty `certificates` array requires at
least one non-`not_run`
verdict. For an `mpk_verified` member, the singleton containing certificate
therefore occurs in two complete accepted lists and contains the dependency
declarations as well. A checker row, certificate row, or checked-declaration
row alone is never sufficient.

## 8. Properties, member rows, and dependency closure

`properties` is strictly increasing by `id`; IDs are unique and use the
portable artifact ID grammar from section 5.2. A property has exactly `id`,
`description`, `status`, `members`, and `notes`. `description` and every note
are nonempty deterministic text of at most 4,096 UTF-8 bytes, contain no
control byte or machine-local path, and are not parsed as proof. They are
already normalized by the registered strategy; validators perform no Unicode
normalization.
`members` is nonempty and strictly increasing by `member_id`. A VC member may
appear in at most one property in one evidence document. `notes` is strictly
increasing by UTF-8 bytes and unique. The complete property set, IDs,
descriptions, member grouping, and notes equal the registered strategy's
deterministic classification of the validated VC and verification outcomes;
none is caller-supplied prose.

A `PolicyMemberRow` has exactly:

| Field | Rule |
|---|---|
| `member_id` | exact VC obligation ID |
| `function_id` | exact VC member function |
| `kind` | exact VC member kind |
| `group_id` | exact containing VC group |
| `declaration_name` | exact group declaration name |
| `declaration_hash` | recomputed candidate declaration interface hash |
| `status` | `mpk_verified`, `proof_pending`, `helper_only`, or `unsupported` |
| `evidence` | canonical closed reference array |

All first six values are reconstructed from the validated VC, skeleton, and
candidate certificate interface; a policy classifier cannot choose another
group or declaration. The declaration hash is present even when a proof is
pending because the canonical grouped theorem interface is already fixed.

The evidence-reference union is:

| `kind` | Other exact fields |
|---|---|
| `checked_declaration` | `certificate_id` |
| `checked_theory_certificate` | `theory_certificate_id` |
| `helper_artifact` | `artifact_id` |
| `unsupported_feature` | `code` |

References are ordered by the kind order shown, then their single ID/code
field by UTF-8 bytes, and are unique. A stable unsupported code uses the Issue
code grammar. No reference contains a shell command, file locator, declaration
index, or unverified hash.

Member status rules are exact:

- `mpk_verified` has exactly one `checked_declaration` reference, zero or more
  checked-theory references, and no helper/unsupported reference. The named
  certificate contains the exact declaration name/hash/group/member row, both
  checker verdicts accepted it, and `axiom_report` is checked. Every
  checked-theory reference resolves exactly once and its
  `checked_member_ids` contains this row's exact `member_id`; every theory
  certificate reached by the accepted member proof is referenced exactly once.
- `proof_pending` has no trusted reference and at least the `vc` helper
  reference. A classifier match, solver answer, generated VC, or candidate
  proof is not sufficient to change this status.
- `helper_only` has no trusted or unsupported reference and at least one helper
  reference.
- `unsupported` has one or more unsupported-feature references and no trusted
  reference; helper references may accompany it.

The property status is derived from its member rows, never caller-selected. It
is `unsupported` if any member is unsupported, otherwise `proof_pending` if any
member is proof-pending, otherwise `helper_only` if any member is helper-only,
and otherwise `mpk_verified`.

For every `mpk_verified` member, validation starts at its containing checked
declaration and follows each direct generated dependency identity recursively.
Every edge must equal the exact source VC group dependency by both name and
recomputed hash, resolve to an accepted declaration in evidence, and occur
earlier in the singleton certificate's declaration order. The complete
transitive closure must be accepted by both checkers. A missing, extra,
renamed, hash-mismatched,
rejected, or `not_run` dependency makes the member and property invalid; it is
not downgraded silently to helper evidence. Foundation imports are checked by
certificate import hashes and are not duplicated in this generated-dependency
graph.

## 9. Structured reproduction recipes

`reproduction_recipes` contains exactly two recipes, label `scan` followed by
label `verify`. A recipe has exactly `label`,
`working_directory_role`, and `argv`. `working_directory_role` is exactly
`source_root`. `argv` is a nonempty array of UTF-8 strings; it is data passed
directly to an argument-vector process API and is never parsed as a shell
command.

The scan argv is exactly:

```text
mpk policy scan .
--language LANGUAGE
--semantic-profile SEMANTIC_PROFILE
--require-release-registry-id REGISTRY_ID
--require-release-registry-sha256 REGISTRY_SHA256
--frontend-bundle FRONTEND_BUNDLE
--toolchain-bundle TOOLCHAIN_BUNDLE
--target TARGET
--package PACKAGE
--function FUNCTION
(--contract NORMALIZED_CONTRACT)*
--json-out mpk-reproduction-scan.json
```

The verify argv has the same prefix through the complete sorted contract set,
then exactly:

```text
--strategy-profile STRATEGY_PROFILE
--checker-profile CHECKER_PROFILE
--axiom-profile AXIOM_PROFILE
--evidence-json mpk-reproduction-evidence.json
--evidence-md mpk-reproduction-evidence.md
[--strict]
[--update-fixtures]
```

Each line above represents one or two exact argv elements, not text with
substitution. Contracts are in normalized UTF-8 order. `--strict` is present
iff `verification_options.strict` is true; `--update-fixtures` is present iff
`verification_options.update_fixtures` is true. Caller source-root and output
spellings are replaced by the fixed values; all other values repeat the
validated invocation. No placeholder, `$` expansion, angle-bracket token,
raw executable option, retired `--removed-frontend`, or machine-local path is permitted.

Markdown is a deterministic derived view and does not add a command field to
JSON. To render one argv array for a POSIX shell display:

1. render a nonempty argument containing only ASCII
   `[A-Za-z0-9_@%+=:,./-]` unchanged;
2. render the empty argument as `''`;
3. otherwise wrap the argument in ASCII single quotes and replace each embedded
   single quote with the four-byte display sequence `'\''` (close quote,
   escaped quote, reopen quote);
4. join rendered arguments with one ASCII space, with no leading/trailing
   space or line wrapping.

The Markdown recipe section states that the working directory is the source
root and shows the rendered value in a `sh` code fence. Consumers on another
shell use `argv`; the rendering is not a free-form executable command or proof
evidence.

### 9.1 Deterministic Markdown evidence view

The evidence Markdown is UTF-8, uses LF line endings, and ends in exactly one
LF. It contains no trailing space and the renderer performs no line wrapping.
It is derived only after complete evidence validation and has these exact
level-two sections in order:

1. `Target and Profiles`;
2. `Source and Release Identities`;
3. `Verification Summary`;
4. `Properties`;
5. `Trusted Evidence`;
6. `Helper Artifacts`;
7. `Reproduction Recipes`;
8. `Trust-Boundary Notes`.

The title is exactly `# MPK Policy Evidence Report`. Profile and identity
sections render every corresponding root field, using `selection` labels
appropriate to its branch but no Go-only root target. Summary counts are
derived from property statuses. Properties retain their canonical array and
member order and render the bound group, declaration name/hash, status, and
reference kinds. Trusted evidence renders certificate/declaration hashes,
theory-certificate hashes, the axiom-report hash/counts, and both checker
verdicts; it never invents a certificate path or checker command. Helper rows
are explicitly labeled untrusted. The final notes state that only checker-
accepted certificate/theory bytes are trusted and that policy JSON, source,
contracts, VIR, VC, AI, CI, and Markdown are not proof evidence.

All document-provided text is Markdown-escaped before rendering. Backslash,
backtick, asterisk, underscore, braces, brackets, parentheses, hash, plus,
minus, period at list position, exclamation mark, angle brackets, ampersand,
and vertical bar are prefixed with one backslash in prose. Values rendered as
backtick code spans are limited to the scalar ID, hash, enum, and portable-path
grammars above, all of which forbid backticks and LF. No raw HTML is emitted.
Recipe code fences contain only the specification-rendered POSIX line and
therefore cannot be influenced by Markdown escaping. Re-rendering
the same evidence produces byte-identical Markdown or fails before any output
is committed.

Markdown is an untrusted presentation artifact, not JCS, a hash input, or an
interchange parser boundary. Conformance freezes the title, section/order and
content invariants, escaping, line discipline, and exact POSIX recipe lines
above; it intentionally does not make table/list punctuation a second
canonical protocol. The repository renderer's owner test may golden its full
chosen layout, but another renderer satisfying these invariants need not have
the same whole-document digest. A layout-only change cannot alter evidence
JSON, a recipe argv, or any verification status.

## 10. Package and release policy ownership

`policy verify` validates and records one checker, strategy, semantic, and
axiom profile. It does not read, reproduce, or override package-manifest
`checker_profile` or `retired_axiom_list` fields.

The source-free package/release gate owns the later cross-check. Given
validated evidence, canonical certificate bytes, the recomputed axiom report,
the active release selection, and a validated package manifest, it requires:

1. active checker profile equals evidence `checker_profile` and package
   `checker_profile`;
2. active axiom profile equals evidence `axiom_profile` and is a member of the
   package manifest's `retired_axiom_list`;
3. the recomputed complete axiom report is allowed by that exact active
   profile;
4. release strategy/language/semantic/axiom selection equals the evidence
   registry row;
5. certificate, declaration, checker, and manifest hashes recompute.

Failure blocks release and cannot be repaired by editing evidence. The package
allowlist remains package policy, not an evidence field. Evidence's explicit
single `axiom_profile` cannot broaden it.

## 11. Deterministic limits

Policy v1 uses checked unsigned counters before allocating complete values.
The inclusive limits are:

| Limit ID | Maximum |
|---|---:|
| `json_transport_bytes` | 268,435,456 including LF |
| `markdown_bytes` | 268,435,456 |
| `json_nesting` | 256 |
| `string_bytes` | 1,048,576 per decoded string |
| `array_elements_default` | 262,144 per array |
| `object_members_default` | 262,144 per object |
| `helper_artifacts` | 65,536 |
| `certificates` | 1 |
| `checked_declarations` | 262,144 |
| `theory_certificates` | 262,144 |
| `properties` | 262,144 |
| `member_rows` | 262,144 |
| `references_per_member` | 4,096 |
| `recipe_argv_elements` | 65,536 across both recipes |

`json_nesting` is the maximum number of simultaneously open JSON object or
array containers: the policy root object has depth one and entering a nested
container adds one; scalar values add none. `string_bytes` counts the UTF-8
encoding of the decoded scalar value, not its escaped transport spelling.
Array counts include every syntactic element, and object counts include every
syntactic name/value member, including one whose name will later be diagnosed
as a duplicate.

`array_elements_default` applies to every policy-owned array that has no
smaller field-specific limit in this table and no smaller limit inherited from
its reused upstream type. `object_members_default` applies before closed-shape
validation to every JSON object, including an object that will later reject for
unknown names. Aggregate counters such as `member_rows` and
`recipe_argv_elements` apply in addition to each container's own ceiling.
Exact schema cardinalities such as the two recipes and two checker verdicts
remain smaller shape constraints; the defaults exist to bound streaming
allocation before shape validation.

Evidence `verification_limit_profile = mpk.verify.limits.v0` fixes the 256 MiB
policy JSON and rendered-Markdown ceilings consistently with `VC_V1.md`.
These values are not command-line options. Scan uses the same policy JSON
transport ceiling and repeats the release-selected VIR `limit_profile`; it
does not claim the later verification profile.

A limit failure emits no partial policy artifact. Strict-mode failure due to a
valid proof-pending report is not a limit failure and follows section 6.

## 12. Stable codes

| Code | Meaning |
|---|---|
| `POLICY_JSON_DUPLICATE_KEY` | duplicate policy object name |
| `POLICY_JSON_INVALID` | invalid UTF-8, JSON, number, string/nesting limit, or trailing framing |
| `POLICY_SCAN_SCHEMA` | wrong scan schema |
| `POLICY_EVIDENCE_SCHEMA` | wrong evidence schema |
| `POLICY_SHAPE` | missing, unknown, retired, or wrong-union field |
| `POLICY_SCALAR` | malformed hash, ID, enum, profile token, text, or portable path |
| `POLICY_ORDER` | noncanonical or duplicate array entry |
| `POLICY_PROFILE_UNKNOWN` | well-formed but unregistered semantic, strategy, checker, or axiom profile |
| `POLICY_PROFILE_TUPLE` | known crossed language, semantic profile/parameters, strategy, or axiom tuple |
| `POLICY_RELEASE_LINKAGE` | registry, frontend, toolchain, or limit projection mismatch |
| `POLICY_SOURCE_LINKAGE` | envelope, selection, manifest, map, input-set, or VIR mismatch |
| `POLICY_MANIFEST_LIFECYCLE` | final manifest changed more than `vc_hash` and its self-hash |
| `POLICY_VC_LINKAGE` | VC schema/hash/source/profile/input/limit mismatch |
| `POLICY_HELPER_LINKAGE` | helper kind, raw/normalized contract, source, VIR, or VC mismatch |
| `POLICY_TRUSTED_EVIDENCE` | certificate identity/declaration-set, theory, axiom-report, or checker projection mismatch |
| `POLICY_MEMBER_LINKAGE` | property member/group/declaration row mismatches VC/skeleton |
| `POLICY_DEPENDENCY_CLOSURE` | required generated declaration dependency is not accepted exactly |
| `POLICY_PROPERTY_STATUS` | member/property status or allowed evidence-reference set is wrong |
| `POLICY_RECIPE` | recipe count, label, working-directory role, argv, or fixed output mismatch |
| `POLICY_CANONICAL_TRANSPORT` | received policy bytes are not exact JCS plus LF |
| `POLICY_LIMIT_JSON_BYTES` | policy JSON transport exceeds 256 MiB |
| `POLICY_LIMIT_MARKDOWN_BYTES` | rendered Markdown exceeds 256 MiB |
| `POLICY_LIMIT_COLLECTION` | another section 11 collection limit is exceeded |

CLI configuration errors use:

| Code | Meaning |
|---|---|
| `POLICY_CLI_FORBIDDEN_LOCATOR` | listed raw executable/toolchain/registry option |
| `POLICY_CLI_UNKNOWN_OPTION` | another unknown option |
| `POLICY_CLI_ARGUMENT` | duplicate, missing-value, positional, or Boolean-value error |
| `POLICY_CLI_REQUIRED` | mandatory option or contract is missing |
| `POLICY_CLI_SCALAR` | malformed value, selection relationship, normalized path, or output path |
| `POLICY_CLI_OUTPUT` | output alias, existing-file, tracked-fixture, or safe-write violation |

Registry assertion, bundle selection, and installed-release codes retain the
more specific `FRONTEND_REGISTRY_*` and `FRONTEND_BUNDLE_*` codes from
`RELEASE_BUNDLES_V0.md` after policy CLI structure/profile validation.

## 13. Conformance vectors and test ownership

`develop/specs/vectors/policy-scan-v1.json` has schema
`mpk.policy.scan.conformance.v1` and exact top-level fields `schema`,
`spec_schemas`, `dependencies`, `owner_test`, `linkage_contexts`, `fixtures`,
`scan_cases`, and `limit_cases`.

A linkage context is a vector-only projection of a validated request, release
selection, frontend result, source manifest, source map, VIR, and normalized
contracts. A fixture has exactly `id`, `linkage_context`,
`canonical_transport_utf8_length`, `canonical_transport_sha256`, and the exact
scan `input`. The length and digest cover `JCS(input) || 0x0a`; they are vector
assertions, not public scan self-hash fields.
Cases contain exactly one of `input_from`, `transport_from`, `json_text`, or
`construction`. `transport_from` has exactly `fixture` and `encoding`; the
closed encoding set is `two_space_indent_with_final_lf` and
`jcs_without_final_lf`. It submits those exact bytes without normalizing them.
Construction has `base` and ordered RFC 6901/RFC 6902 `add`, `remove`, or
`replace` patches; vector-only `swap` has exactly `op`, `path`, `first`, and
`second`. Patches never repair another field implicitly. `expect` contains
`outcome` and, for rejection, exact `phase` and `code`.
`input_from` and the final value of `construction` are submitted as
`JCS(value) || 0x0a`; `json_text` is submitted as its exact UTF-8 bytes.

`develop/specs/vectors/policy-evidence-v1.json` has schema
`mpk.policy.evidence.conformance.v1` and exact top-level fields `schema`,
`spec_schemas`, `dependencies`, `owner_test`, `linkage_contexts`, `fixtures`,
`evidence_cases`, and `limit_cases`. Its contexts additionally freeze exact VC
members/groups, candidate declaration hashes, direct generated dependencies,
certificate/checker acceptance, and both manifest lifecycle hashes.
An evidence fixture has the same five exact fixture fields and digest meaning
as a scan fixture. Construction and expectation shapes are the same as the
scan vector. The
vector-only operation `copy` has exactly `op`, `from`, and `path` and inserts a
deep copy, permitting duplicate/member-reuse attacks.

In both schema vectors, a limit case has exactly `id`, `limit`, `counter`,
`below`, `at`, and `above`. Each boundary object has exactly `count` and
`outcome`, plus `phase` and `code` only when rejecting. These are isolated
checked-counter tests: `accept` means that counter accepts the supplied count,
not that all independent schema maxima can coexist in one linked policy
artifact. The owner feeds the count before allocating a complete collection,
string, or nested value; JSON and Markdown byte cases feed the complete-root
or complete-render counter. The union of the scan and evidence vectors covers
every section 11 limit ID, and an owner test fails if a table limit has no
boundary case or if `limit - 1`, `limit`, and checked `limit + 1` are not the
three exact counts.

`develop/specs/vectors/policy-recipes-v1.json` has schema
`mpk.policy.recipes.conformance.v1` and exact top-level fields `schema`,
`spec_schema`, `owner_test`, `invocations`, `cli_cases`, `recipe_cases`,
`render_cases`, and `output_cases`. Invocation values are vector-only parsed
CLI projections. Each invocation has exactly `id`, `route`, `argv`, and
`parsed`; `parsed` is the applicable closed scan/verify CLI value projection
from section 2. Scan `parsed` has exactly `source_root`, `source_language`,
`semantic_profile`, `registry_id`, `registry_sha256`, `frontend_bundle_id`,
`toolchain_bundle_id`, `target_id`, `package`, `function`, sorted `contracts`,
and `json_out`. Verify replaces `json_out` with exactly `strategy_profile`,
`checker_profile`, `axiom_profile`, `evidence_json`, `evidence_md`, `strict`,
and `update_fixtures`; its other fields are identical. A CLI case has exactly
`id`, one of `input_from` or
`construction`, and `expect`. Its construction has `base` and `operations`.
The closed operation shapes are:

- `remove_option`, `remove_option_value`, and `remove_all_options`: exactly
  `op` and `name`;
- `append_option` and `replace_option`: exactly `op`, `name`, and `value`;
- `append_flag`: exactly `op` and `name`;
- `remove_source_root`: exactly `op`;
- `append_positional`: exactly `op` and `value`.

Operations apply in array order to a copy of the base invocation's exact
`argv`. `remove_option` removes the first exact name and its immediately
following value; `remove_all_options` removes every such name/value pair from
last to first; `remove_option_value` removes only the value immediately after
the first name. `append_option` appends name then value, `replace_option`
replaces only the value after the first name, and `append_flag` appends its name
as one element. `remove_source_root` removes argv index three, and
`append_positional` appends its value as one bare element. A construction whose
required occurrence/value is absent is an invalid vector, not a CLI test.
CLI `expect` has exactly `outcome` and `launch_count`, plus `code` only for
rejection. The owner compares an accepted `input_from` result with that
invocation's complete `parsed` object. For an accepted construction, it derives
the expected parsed object by applying the successful option edit to the base
projection and then the specified contract sorting; checking only outcome or
launch count is insufficient.

A recipe case has exactly `id`, `invocation`, and `expect.recipes`, freezing
both complete recipe values. A render case has exactly `id`, `argv`, and
`expected_posix`. An output case has exactly `id`, `route`,
`update_fixtures`, `destinations`, optional `injected_failure`, and `expect`;
a destination has exactly `path` and `state`. When present,
`injected_failure` is exactly `publish_markdown` or
`sync_output_directory`, models that reportable I/O failure after JSON or both
new payloads have been published respectively, and requires the writer to run
its normal rollback. The closed state set is `absent`, `tracked_regular`,
`untracked_regular`, `symlink`, `directory`, `hardlink_alias`, and
`reparse_point`; `hardlink_alias` means the destination inode aliases another
modeled destination or non-output file. These cases use a synthetic filesystem
model rather than touching the owner test's working tree. Every rejected output
case requires `committed_writes = 0`; this counts destinations whose new
payload remains committed when the modeled command returns, so a destination
whose retained old payload was restored does not count. The model does not
inject an abrupt process/host failure or failure of the rollback mechanism
itself; either would enter section 2.3's recovery-required state instead.
`expect` has exactly `outcome`, `launch_count`, and `committed_writes`, plus
`code` only for rejection. Accepted cases and injected publication failures
have `launch_count = 1`; every destination/alias preflight rejection has
`launch_count = 0`.

All vector objects are closed. Owning tests reject unknown vector or case
fields, enforce unique IDs across every case array, execute each case exactly
once, and fail if a declared case is skipped. The owning tests are:

- `crates/mpk-cli/tests/policy_schema_v1.rs` for scan/evidence models and their
  two vectors;
- `crates/mpk-cli/tests/policy_profiles.rs` for profile and package/release
  cross-checks; and
- `crates/mpk-cli/tests/policy_recipes_v1.rs` plus
  `crates/mpk-cli/tests/policy_report.rs` for CLI, recipes, safe output, and
  POSIX display.
