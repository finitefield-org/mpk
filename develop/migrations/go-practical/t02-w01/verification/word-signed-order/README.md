# Actual signed word ordering and balanced integer comparisons

Status: scoped implementation checkpoint; T02-W01 remains In progress.
Entry: `63b680e9d3eee4bbd6c30e9869ed692714eaffa2`.

The [producer](../../../../../../crates/mpk-vc/src/go_practical_ordinary/word_numeric/signed_order.rs)
proves unconditional signed Less/LessEqual correspondence for all8/16/32/64-bit
words. It uses the actual existing word circuits and their actual top addresses,
full unsigned Nat values and established balanced Int projections. For unsigned
values a,b, modulus M and actual sign bits sx,sy, mathematical Int comparison is

```text
Less(a + M*Bit(sy), b + M*Bit(sx)).
```

When signs agree, checked additive translation gives ordinary unsigned order.
When they differ, the actual word bounds a<M and b<M prove the negative operand
is smaller. Genuine unbounded RightStrictGap supplies a computed Nat gap and
ordinary padding laws; no bound is supplied by a caller truth flag. The result
is exactly the existing source circuit's sign-Xor selection. LessEqual follows
by negating the reversed actual Less circuit and real Int.Less definition.
No operand enumeration, arithmetic opcode, axiom or extensionality rule appears.

The [fixed composer](../../../../../../crates/mpk-vc/src/go_practical_ordinary/composition.rs)
retains all3437 original order types/bodies/proofs/levels/families without
renaming or theorem substitution. It adds the two strict-gap proofs' complete
dependencies from the existing shift component; every included body is checked
again in the closed root with imports empty. Fingerprints and owning imports
supply no kernel/source authority. The complete old order raw SHA256
`c12b72e2f331427e24465ef60ef228b469d6c54c6d414bf005b4ba3a1a9380fa`
and complete4053-declaration shift raw SHA256
`119c384ddefd5afc6c42de2171df608d4b5b6329304912d3e9ce29860751e1b2`
remain exact under current generation.

The [component](final/tests/certificates/word-signed-order.mpcert) has3457
declarations,71954 terms,996816 bytes and zero axioms, within unchanged
4096/131072/2097152 caps and1024 conversion fuel. Twenty declarations beyond
the3437 original members include the shared gap laws, checked algebra steps,
padding/sign-choice lemmas and eight final ordering laws. The639 spare
declaration slots are this component's margin, not complete application budget
admission. Full56-family assembly and actual source/program statements remain due.

All three [affected Rust roots](final/tests/result.json), strict selected Clippy,
current format and an explicit ignored-diagnostic listing pass. The
[paired record](paired-completed/result.json) covers29 complete same-byte
streams in both unchanged cores: two zero-axiom acceptances and27 genuine
CORE_TYPE_MISMATCH rejections, with24 check mismatches and three lambda-domain
mismatches. Controls change actual SignedLess/SignedLessEqual into well-typed
unsigned functions, replace both true top addresses by the least significant
addresses, falsify mathematical/padding claims and erase real bound premises.
An honest extra valid definition passes both cores but fails exact component
ownership. No format, fuel or resource failure counts as a semantic negative.

[Finite counterparts](finite-counterparts/result.json) reuse all96 complete
original SignedLess/SignedLessEqual queries and actual decoded-certificate Bool/
typed Go results. All1892 original Bool component identities and3437 original
order identities match. Every original1640 result is cross-checked, including
all40 real native runtime failures; all original sources, queries, outputs and
before/after5577-file prepared-image checks stay pinned. Native compilation and
unchanged Bool evaluation are not repeated. Python checks the full input sign
bits, balanced cross sums, four sign branches and computed gap counterparts.
No new Nat/Int/theorem body executes in finite observations; universal proof
authority comes from both cores inspecting the complete proof terms.

The [first complete candidate](initial/construction/certificates/word-signed-order.mpcert)
fails in both cores. The [ignored direct diagnostic](diagnose-original/diagnostic/diagnostic.json)
locates declaration3448, SignedLessScalar. A computed Xor major must be
transported to its checked literal by an explicit ordinary equality before
Bool.cases. That Eq congruence repairs the recipe without changing conversion
or iota rules. All original source snapshots, complete bytes and raw diagnostics
remain. Direct prefix observations grant no encoded-prefix acceptance authority.

Three original observer mistakes also remain with complete source/raw output:
the Go core has human error_detail rather than the Rust nested JSON; native
panic detail differs from the Bool model's semantic reason; and Go's positive
protocol omits axiom_count. Corrected classification compares actual schemas,
retains every original runtime outcome and binds zero axioms through the exact
checked empty axiom-report hash and full Rust report for the same bytes. The
paired completion reuses28 complete Go executions and runs only the previously
unfinished canonical positive. Missing fields never default to proof authority.

All255 [frozen members](frozen-design-preservation.json) remain exact. Strict
input review uses the unchanged127-rule/four-scope scanner and39 historical
seals. Only four exact per-file hash/length original libtest stdout EOF
observations are retained; no source/JSON/directory/blanket exclusion is added.
Requirements preserve all R01-R07 scope and keep original package/export replay
pending. Complete signed equality and other remaining mathematical/source/
carrier/type/operation/storage/lifetime/state/program foundation, bounded
descriptor/private registry/context and retained-policy delegation remain due.
All169 application demands and T02-T11 stay open. C# tests stay skipped; the
full T02 gate stays W04: `./scripts/check-fast.sh --skip-csharp`.
No public profile, source/application proof, immutable application replay or
native Linux release is claimed.
