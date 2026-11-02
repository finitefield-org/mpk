# Retained aggregate preservation epochs

The [manifest](manifest.json) seals 716 distinct original/checkpoint/current
members against actual Git objects. It covers every one of the 225 original
pre-StepEight vectors in twelve families, their complete after-StepEight
successors from `09a14bdde6b14408e4f25e55cb787dd989577d7e`, all corresponding
current vectors and three metadata epochs per family. Five further members
bind the historical decimal parser, historical combined/component definitions
and the separate current combined/component definitions.

Of the 225 checkpoint vectors, 102 already exist unchanged in retained refresh
directories and are referenced there; 123 are exact Git-object copies here.
Existing archive/current bytes are untouched. These checkpoints precede the
later DomainCount correction, which retired helper names and changed its Result.
The original failure and compiler inputs remain in the
[diagnosis](../../verification/retained-csharp-repair/aggregate-archive-diagnosis/README.md).

The complete original comparison is preserved: 133 vectors reassociate exactly
8,192 StepTwo calls, 92 remain byte-identical, and all twelve metadata pairs
agree after removing only the original four cost fields. Every old declaration
and transitive dependency is compared. The structural comparator and permitted
aggregate normalization are unchanged; there is no helper-prefix exemption.
The test also checks the exact twelve-family/225-file inventory, every member
digest/size, current certificate structure and one rejected body mutation per
family. Explicitly supplied current generator output must still equal its
separately sealed current vector and metadata.

The decimal/JSON test retains its complete current component closure. It
separately checks the complete historical component closure and the original
decimal-parser reassociation, with current and historical body mutations
rejected. Both unchanged current producer owners already pass regeneration and
normal golden checks in the completed source320 lib target. The
[selected verification](../../verification/retained-csharp-repair/aggregate-archive-repair/verification.json)
distinguishes that completed-target observation from the original running whole
gate and from the actual repaired integration tests.

These are historical regression comparisons and current definition checks.
They provide no application/source proof, universal semantic equivalence or
new checker acceptance. No production implementation, existing fixture,
source capture, checker rule, resource limit or installed profile changes.
T01 still requires the complete repaired whole gate before T02 starts.
