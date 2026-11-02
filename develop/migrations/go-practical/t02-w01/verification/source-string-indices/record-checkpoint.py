"""Record actual terminal scoped evidence; complete T02/profile remains due."""
from pathlib import Path
import hashlib,json
ROOT=Path(__file__).resolve().parent;REPO=ROOT.parents[5]
def seal(p):
 raw=Path(p).read_bytes();return {'path':str(p),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()}
checks=['heterogeneous','remaining','paired-heterogeneous','paired-remaining','oracle-heterogeneous','oracle-remaining','native-comparison','lint-controls','format-check','go-format-check','native']
results={name:json.loads((ROOT/name/'result.json').read_bytes()) for name in checks};assert all(r['status']=='passed' for r in results.values())
assert (ROOT/'go-format-check/stdout.log').read_bytes()==b''
paired=[results[n] for n in ['paired-heterogeneous','paired-remaining']];oracles=[json.loads((ROOT/n/'structural-comparison.json').read_bytes()) for n in ['oracle-heterogeneous','oracle-remaining']]
normal=[json.loads(p.read_bytes()) for epoch in ['heterogeneous','remaining'] for p in (ROOT/epoch/'certificates').glob('*.sources.json') if not p.name.startswith('control-')]
accepted=sum(r['zero_axiom_acceptances'] for r in paired);rejected=sum(r['genuine_type_rejections'] for r in paired);assert (len(normal),accepted,rejected)==(131,132,3)
maxima={k:max(r[k] for r in normal) for k in ['declarations','terms','bytes']};assert maxima['declarations']<=4096 and maxima['terms']<=131072 and maxima['bytes']<=2097152
bits=sum(r['full64_bit_observations'] for r in oracles);bytebits=sum(r['complete_byte_bit_observations'] for r in oracles);laws=sum(r['new_exact_members'] for r in oracles);recipes=sum(r['exact_source_recipe_members'] for r in oracles)
summary=f'''The original-source string index catalogue binds every eligible builtin/nominal integer width and sign to full unsigned zero extension, actual balanced-Int interpretation and guarded source-uint8 reads. Definedness consumes the original strong string validity, actual nonnegative sign and complete mathematical index<size; no high bits are truncated. All original source types/U and every prior source-string declaration identity remain. Normal131 rows cover all71 integer types in the70-case unit,4 heterogeneous types and all56 pairs of4 string types with14 integer types in the new original-Go unit. Both unchanged full cores agree on135 complete streams:132 zero-axiom mathematical acceptances and3 genuine type mismatches. Independent comparison checks{laws} new complete definitions/statements and{recipes} original source recipe members, with{bits} full64 conversion-bit and{bytebits} full-byte bit observations. A coherent constant read is accepted as mathematics but fails exact source import and the independent body oracle. Original Go observations independently match all13 operand types,4 string types,4 full cases and the extra original numeric error case; aliases retain identity. Maxima{maxima['declarations']} declarations/{maxima['terms']} terms/{maxima['bytes']} bytes fit unchanged caps.'''
remaining='''These are private typed representation/definedness adapters, not ordinary proof of the original IndexExpr evaluation, operand-once ordering, signed Go len result, arbitrary runtime-input validity, source effects/frame or canonical capture/context. The full56 foundation/private registry/nativeLinux/all169 and remaining T02-T11 requirements remain due. Go/Rust only; R01/R06/W01 remain open and the practical profile is inactive. The cumulative T02 gate remains final W04: ./scripts/check-fast.sh.'''
failure='''The initial u16/u8 type-width compile failure remains with full original inputs/output; the source type key uses u8. Explicit whole-word/integer equalities isolate denotation conversion before Nat computation. The first proof launch was rejected before process creation when automatic approval review timed out; its single authorized retry ran. No live verification is restarted after an observation timeout. Original Go output10980 files and all bytes/modes remain losslessly archived and retained intact. No source pruning, caller roots/predicates, funext, axiom, new case/root/helper/parameter cap or unchanged matrix rerun is introduced.'''
record={'schema':'mpk.go_practical.source_string_indices_checkpoint.v1','status':'scoped_checks_passed_complete_W01_due','basis_commit':'5688494be9bc09e8d1955bc5872e715e3a076369','normal_rows':len(normal),'same_whole_streams':accepted+rejected,'zero_axiom_math_acceptances':accepted,'genuine_type_rejections':rejected,'new_exact_members':laws,'source_recipe_members':recipes,'full64_bit_observations':bits,'complete_byte_bit_observations':bytebits,'maxima':maxima,'checks':{n:seal(ROOT/n/'result.json') for n in checks},'scope':summary,'original_failure_retention':failure,'remaining':remaining,'full_T02_gate':'deferred to final W04: ./scripts/check-fast.sh','T02_W01_complete':False,'public_profile_activated':False,'source_expression_or_context_authority':False,'other_language_tests_executed':False,'native_Linux_release_acceptance_claimed':False}
(ROOT/'verification.json').write_text(json.dumps(record,indent=2)+'\n')
(ROOT/'README.md').write_text('# Original-source integer string indices\n\n'+summary+'\n\n'+failure+'\n\n'+remaining+'''

Targeted tests cover only the new catalogue/composer and their source/byte
consumers. The unchanged earlier source-string/word libraries are compared by
complete identities inside every new stream; unrelated old matrices are not
repeated. Scoped Clippy and Rust/Go format pass. Native comparison facts never
enter the Rust constructor. Concrete DAG observation accelerates only exact
prior accepted Nat and lookup definitions; the complete ordinary proofs are
checked by both cores. Finite observations supplement those universal laws.

- [Actual scoped results](verification.json)
- [Heterogeneous checker agreement](paired-heterogeneous/result.json)
- [Remaining checker agreement](paired-remaining/result.json)
- [Independent full matrix](oracle-remaining/structural-comparison.json)
- [Original Go comparison](native-comparison/result.json)
- [Scoped input review](input-review.json)
- [Complete original retention](native-original-relocations.json)
''')
p=ROOT.parents[1]/'requirements.json';r=json.loads(p.read_bytes())
for row in r['requirements']:
 if row['id']=='T02-W01-R01':row['source_integer_string_index_checkpoint']={'evidence':'verification/source-string-indices/verification.json','scope':summary,'remaining':remaining,'R01_complete':False,'R06_complete':False,'W01_complete':False,'public_profile_activated':False}
p.write_text(json.dumps(r,ensure_ascii=False,indent=2)+'\n')
p=REPO/'develop/docs/go-practical-implementation-traceability-ledger.md';assert '## 170.' not in p.read_text();p.write_text(p.read_text()+'\n\n## 170. T02-W01 original-source integer string indices\n\nThe [integer-index checkpoint](../migrations/go-practical/t02-w01/verification/source-string-indices/README.md) extends the typed source value bridge.\n\n'+summary+'\n\n'+failure+'\n\n'+remaining+'\n')
p=ROOT.parents[1]/'README.md';p.write_text(p.read_text()+'\n\n## Original-source integer string indices\n\nThe [index checkpoint](verification/source-string-indices/README.md) retains every original source type/U and prior string identity.\n\n'+summary+'\n\n'+remaining+'\n')
print(json.dumps({'status':'recorded','same_whole_streams':accepted+rejected,'normal_rows':len(normal),'full64_bit_observations':bits,'byte_bit_observations':bytebits,'maxima':maxima}))
