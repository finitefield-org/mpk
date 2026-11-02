# Original aggregate archive comparison failures

The unchanged source320 integration binary reproduces two still-queued
historical comparison failures: [aliased consumers](aliased-consumers-result.json)
in 0.155 seconds and [decimal/JSON sharing](decimal-shared-result.json) in
0.119 seconds, both exit 101. Their original stdout/stderr, executable identity
and all 477 certificate/metadata input observations are retained. Both fail on
the retired DomainCount Success helper. No implementation or assertion was
changed before these reproductions.

[Original input verification](original-input-epoch-verification.json) binds
all 474 aliased-consumer and three decimal/shared inputs to the exact source320
members in the sealed epoch manifest. The original full test source remains
recoverable in the earlier
[compiler source package](../json-calendar-diagnosis/compiler-source-snapshots.json).
This diagnosis's own three complete compiler inputs are preserved byte-for-byte
in [compiler-source-snapshots.json](compiler-source-snapshots.json); decode each
`raw_utf8` member to its recorded filename to restore module-relative paths.

[Initial complete comparisons](initial-comparison-result.json) confirm the
separate current component closure already passes. The historical old decimal
parser and first aliased example pass against their exact 09a14 checkpoint.
The [complete corpus diagnosis](corpus-result.json) then executes all 225
original vector comparisons and all twelve original non-cost metadata
comparisons against retained checkpoints: 133 exact aggregate reassociations,
92 byte-identical cases, zero failures. The shared structural comparator and
8,192-StepTwo normalization are unchanged. The raw per-vector and per-family
results are retained; this is a finite syntax audit, not a semantic or
application proof.

The [epoch manifest](../../../retained-csharp/aggregate-preservation-epochs/manifest.json)
and [selected repair](../aggregate-archive-repair/verification.json) preserve
those complete assertions while keeping current pins and current component
closure separate. The original no-fail-fast whole diagnostic continues with
its unchanged source320 binary; no isolated success can complete T01's gate.
