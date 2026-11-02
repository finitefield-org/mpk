# Original numeric-capacity historical comparison failure

The unchanged source320 integration binary reproduces the original assertion
failure in 1.056 seconds, exit 101, before any numeric-capacity test edit.
[result.json](result.json) pins that binary, the original test and all four
original certificate inputs. The complete stdout/stderr remain unchanged;
the stderr contains the original full name-set assertion, not a replacement
summary. [Git input verification](original-git-input-verification.json) checks
all five input byte strings against the exact source320 tree. The original
test source is also retained in [its raw copy](original-test-source.rs.txt).

[Current direct comparisons](current-direct-comparison-result.json) diagnose
the name mismatch. Each current family removes 27 old private DomainCount
helpers and changes seven retained bodies, with no retained type difference.
Six are the original JSON cell-limit grammar correction; the seventh is the
later DomainCount Result repair. This is a historical comparison-epoch
mismatch, not a new source execution or checker rejection.

[Historical candidates](historical-candidate-inputs.json) were obtained from
actual Git objects. [Exact definition comparisons](historical-direct-comparison-result.json)
confirm that both complete before/after pairs at 09a14 preserve all 4,387
declaration names/types and change exactly the original six reviewed grammar
bodies in each family. Each raw comparison output and its inputs are sealed.
The diagnostic binary source is recoverable byte-for-byte as `definition-diff.rs`
and `structural-comparator.rs` in the previous
[compiler source snapshot package](../json-calendar-diagnosis/compiler-source-snapshots.json).
The diagnostic compares syntax only; it establishes no semantic equivalence,
source proof or new checker acceptance.

The [sealed epoch manifest](../../../retained-csharp/json-numeric-capacity-epochs/manifest.json)
and [selected repair verification](../json-numeric-capacity-repair/verification.json)
keep the complete historical assertion separate from current source generation,
strict import, golden and mutation checks. The source320 cumulative diagnostic
continues with its unchanged original binary to collect remaining failures.
A complete repaired whole gate is still required before T01 completion/T02.
