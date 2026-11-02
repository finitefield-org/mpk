"""Record terminal affected checks without closing T02-W01 or activating a profile."""
from pathlib import Path
import hashlib,json
ROOT=Path(__file__).resolve().parent;REPO=ROOT.parents[5]
def seal(p):
 raw=Path(p).read_bytes();return {'path':str(p),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()}
checks=['index-fixed','index-matrix','final-streams','paired','oracle','native-complete','native-comparison','final-lint','final-format','go-format-complete','fuel-diagnosis']
results={n:json.loads((ROOT/n/'result.json').read_bytes()) for n in checks};assert all(r['status']=='passed' for r in results.values())
assert (ROOT/'go-format-complete/stdout.log').read_bytes()==b''
pair=results['paired'];assert (pair['same_byte_streams'],pair['zero_axiom_acceptances'],pair['genuine_type_rejections'])==(28,26,2)
oracle=json.loads((ROOT/'oracle/structural-comparison.json').read_bytes());assert len(oracle['streams'])==25
normal=[json.loads(p.read_bytes()) for p in (ROOT/'final-streams/certificates').glob('*.sources.json')];assert len(normal)==25
maxima={k:max(r[k] for r in normal) for k in ['declarations','terms','bytes']};assert maxima['declarations']<=4096 and maxima['terms']<=131072 and maxima['bytes']<=2097152
summary=f'''The catalogue now derives string len/index sites from every original checked selected body and binds each private certificate to the complete original source buffers, package/root set, all source types/U, physical operation and operand nodes, actual result type and every contained ordinary call/callee. It distinguishes predeclared len from an ordinary function named len, arrays/slices and typed-int constant lengths. A single captured operand product has exact projections, actual Length/Read results, unchanged validity/index guards and full signed integer consequences. Error-free units retain all original source carriers/zeros without a fabricated source error key; runtime literal indexing retains the actual materialized string key. The error-bearing predecessor row remains byte-identical. Normal25 rows cover11 Length and14 Index instances across complete0/70-case units and separate actual Length/At/Literal selections. Both unchanged full cores agree on28 complete streams:26 zero-axiom mathematical acceptances and2 genuine type mismatches. Cross-site exact import rejects; the extra theorem is valid mathematics but fails exact import and the independent export-set oracle. Independent comparison reconstructs{oracle['new_operation_members']} site-specific members and{oracle['exact_source_recipe_members']} original source recipe members, plus the three generic Pair/Uncurry/PairApply declarations in every Index row. Original Go independently confirms all24 physical operation sites, including2 typed-int constant lengths, actual operand/result types, caller and child-callee origins, with full original error universes0/70. Maxima{maxima['declarations']} declarations/{maxima['terms']} terms/{maxima['bytes']} bytes fit unchanged4096/131072/2097152/1024 caps.'''
failure='''All original failures remain: observer-fixture syntax failure; Go rejection of returning typed-int len(\"a\") as byte; the incomplete two-package source packet and its real Scope rejection; and the full first Index CaptureResult fuel failures. No production constant-length typing fix was needed. The complete three-package packet fixes the observer input, while the old incomplete packet remains an explicit rejection test. Prefix diagnosis localizes the first mathematical failure to CaptureResult; no prefix is used as source evidence. Generic neutral-function PairApply and exact whole-function Capture/Result/ResultWord equations avoid repeatedly expanding the full64-bit source Read in reflexivity, without raising fuel or narrowing source types/U. Only affected failed Index roots are rerun. The original passing Length/control subtests and predecessor-byte check are retained with their failed parent result explicitly unchanged; one already-paired Index stream is reused by exact bytes. All43932 original Go output files/349835026 bytes and original modes remain losslessly archived and physically retained; no live verification was restarted.'''
remaining='''This is original operation/type binding and captured-value mathematics. It does not establish operand expression execution, operand-once sequencing, Go panic behavior, callee execution/effects/frame, arbitrary runtime-input validity or canonical source/IR/program/capture/context correspondence. Capture positions are argument positions, not a claim that all Go expression evaluations occur left to right. Constant-length sites remain inventoried with their actual int result; runtime-row elaboration for them remains due. All56 foundation coverage, all169 demands, native Linux release gates and remaining T02-T11 work remain due. R01/R06/W01 stay open and the practical profile stays inactive. The cumulative T02 gate is deferred to final W04: ./scripts/check-fast.sh. Go/Rust only; no other-language implementation, test or release prerequisite is added.'''
record={'schema':'mpk.go_practical.source_string_operations_checkpoint.v1','status':'scoped_checks_passed_complete_W01_due','basis_commit':'df37c071655e124d1770c18d27e2f9f9357feebf','normal_rows':25,'length_rows':11,'index_rows':14,'same_whole_streams':28,'zero_axiom_math_acceptances':26,'genuine_type_rejections':2,'site_specific_members':oracle['new_operation_members'],'source_recipe_members':oracle['exact_source_recipe_members'],'native_source_sites':24,'native_typed_int_constant_sites':2,'original_error_case_counts':[0,70],'maxima':maxima,'checks':{n:seal(ROOT/n/'result.json') for n in checks},'scope':summary,'original_failure_retention':failure,'remaining':remaining,'full_T02_gate':'deferred to final W04: ./scripts/check-fast.sh','T02_W01_complete':False,'public_profile_activated':False,'source_expression_execution_or_context_authority':False,'other_language_tests_executed':False,'native_Linux_release_acceptance_claimed':False}
(ROOT/'verification.json').write_text(json.dumps(record,indent=2)+'\n')
(ROOT/'README.md').write_text('# Original-source string operation bindings\n\n'+summary+'\n\n'+failure+'\n\n'+remaining+'''

Verification selects the new source-site catalogue, its composer and source
consumers, the affected runtime literal typing path and complete error-free
layout, with the existing error-bearing row preserved exactly. It also covers
a real ordinary function named len, both array/slice non-string len sites,
parenthesized nominal/alias strings, repeated/nested operations, actual child
calls, signed/unsigned index types, missing source int/byte/error keys and the
complete original three-package closure. No unrelated old matrix or broad
intermediate gate is repeated. Scoped Clippy and Rust/Go formatting pass.

- [Actual targeted results](verification.json)
- [Whole-stream checker agreement](paired/result.json)
- [Independent original recipe and public statement comparison](oracle/structural-comparison.json)
- [Original Go comparison](native-comparison/result.json)
- [Complete terminal subtest assembly](final-streams/result.json)
- [First mathematical failure diagnosis](fuel-diagnosis/result.json)
- [Scoped original-input review](input-review.json)
- [Complete original retention](native-original-relocations.json)
''')
p=ROOT.parents[1]/'requirements.json';r=json.loads(p.read_bytes())
for row in r['requirements']:
 if row['id']=='T02-W01-R01':row['source_string_operation_checkpoint']={'evidence':'verification/source-string-operations/verification.json','scope':summary,'remaining':remaining,'R01_complete':False,'R06_complete':False,'W01_complete':False,'public_profile_activated':False}
p.write_text(json.dumps(r,ensure_ascii=False,indent=2)+'\n')
p=REPO/'develop/docs/go-practical-implementation-traceability-ledger.md';assert '## 172.' not in p.read_text();p.write_text(p.read_text()+'\n\n## 172. T02-W01 original-source string operation bindings\n\nThe [operation checkpoint](../migrations/go-practical/t02-w01/verification/source-string-operations/README.md) binds the existing string mathematics to exact original operation sites and actual operand/result types.\n\n'+summary+'\n\n'+failure+'\n\n'+remaining+'\n')
p=ROOT.parents[1]/'README.md';p.write_text(p.read_text()+'\n\n## Original-source string operation bindings\n\nThe [operation checkpoint](verification/source-string-operations/README.md) binds every selected original string operation to its actual type-specific mathematical recipe.\n\n'+summary+'\n\n'+remaining+'\n')
print(json.dumps({'status':'recorded','normal_rows':25,'same_whole_streams':28,'members':oracle['new_operation_members'],'maxima':maxima}))
