# Full-scope obsolete-interface scanner prefilter

The cumulative T01 gate's initial scanner spent1,434 seconds on the repository
without reaching a verdict. It scans about5GiB across29,000 files using127
matchers:112 exact tokens,9 paths,4 qualified symbols and2 schema-qualified
fields. No input class or file is excluded by this optimization.

An exact token/path regex cannot match when its literal is absent. A qualified
symbol must contain its first component even when whitespace surrounds `::`.
Path-only findings remain checked independently of content. For decoded JSON,
unescaped keys/schema strings must occur literally; escaped input always retains
the full original structured check. Inherited schema and schema_version behavior,
boundary regexes, raw-text fallback, result offsets and excerpts are unchanged.
The lexer/parser and malformed-input behavior are not replaced with a text-only
schema check.

The [differential observation](comparison.json) compares the final source with
the exact script from commit `f1ada294`. All9 existing positive/negative cases
plus their manifest and3,206 generated cases agree, including path-only matches,
ASCII/Unicode boundaries, whitespace-separated symbols, inherited schema and
escaped keys/schema values. A4MiB absent-token sample takes about6.01 seconds
before and0.17 seconds after. This measurement is a sample, not a gate deadline
or a promise for all input shapes.

The [comparison tool](compare.py) loads only the Python helper definitions,
without running either full repository scan. Export the original script with
`/usr/bin/git show f1ada294:scripts/check-no-active-gir.sh` outside the sandbox
into a temporary file and pass that filename as its argument. The original
script is not copied into the active repository because the scanner's own
configuration strings are intentionally excluded only at the registered scanner
path. Its source digest and commit preserve the exact baseline.

Both superseded gate attempts have original raw logs, exit143, explicit
termination reasons and no claimed verdict in the neighboring initial directories.
The first prefilter attempt was replaced by the final JSON-aware condition.
The complete `./scripts/check-fast.sh` is rerun from the beginning against the
final source, including all repository files and all127 rules. No previous
partial attempt is counted as a passing gate.

## Findings and exact historical retention

The first completed scan reported132 occurrences in35 files. Twenty-seven files
belong to W04's frozen private candidate graph/models or preserved failed-source
snapshots. The final source-kind amendment uses go_file and requires a fresh
T02 source epoch, while keeping these old bytes intact. Seven additional hits
were coincidental short prose tokens in compressed C# proof receipts; their
original archive bytes remain untouched. The comparison tool's live prose was
renamed, and scanner diagnostics now use current interface-neutral terminology.

Fixture manifest v1 adds36 **exact sealed historical records**: the27 preceding
candidate files,7 compressed receipts and2 exact failed-scan output logs.
Each path/hash/size/class/source-epoch tuple is pinned independently in scanner
code and in the closed manifest. Changed bytes, a new/foreign path, a changed
epoch/class, omissions, duplicates or unsafe paths fail before scanning. The
existing12 retained-record declarations, all127 matchers and all4 scopes stay
unchanged. No directory or production file can be sealed. These records are
still inspected and hashed; they cannot silently become mutable exceptions.
The [seal controls](seal-controls.json) reject ten such changes without editing
any original input. Historical model inclusion grants no runtime/profile or
source-proof authority. Final production must reject the preliminary dialect.

The [classified original findings](../initial-vocabulary-findings/classification.json)
retain original failed output and the rationale for each individual sealed
file. This is an explicit historical-evidence boundary, not removal or relaxation
of any obsolete-interface matcher. Future live production and unlisted data
continue to be checked by all applicable rules.
