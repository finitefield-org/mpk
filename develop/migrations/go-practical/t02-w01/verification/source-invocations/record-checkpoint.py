"""Record actual whole-obligation checks without claiming unchecked row wrappers."""
from pathlib import Path
import hashlib,json
ROOT=Path(__file__).resolve().parent;REPO=ROOT.parents[5]
def seal(p):
 b=Path(p).read_bytes();return {'path':str(p),'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()}
checks=['native-complete','derived','generation-linked','linkage','oracle-generation','native-comparison-generation','aggregation-interned','aggregate-union','aggregate-check-tools','aggregate-rust-pure','aggregate-go-pure','aggregate-rust-wide','aggregate-go-wide','aggregate-paired','paired-pure','go-wide-probe','aggregation-binary-path-repair','lint','final-format','final-go-format']
results={n:json.loads((ROOT/n/'result.json').read_bytes()) for n in checks};assert all(r['status']=='passed' for r in results.values())
assert (ROOT/'final-go-format/stdout.log').read_bytes()==b''
originals={name:json.loads((ROOT/name/'result.json').read_bytes()) for name in ['rust-entries','paired-followup']}
assert all(r['status'] in ['passed','failed'] for r in originals.values()),'never record completion while original verification is live'
if any(r['status']=='failed' for r in originals.values()):
 supersession=json.loads((ROOT/'individual-supersession/result.json').read_bytes());assert supersession['status']=='passed' and supersession['complete_replacement_obligations_checked_before_supersession']
paired=results['aggregate-paired'];oracle=json.loads((ROOT/'oracle-generation/structural-comparison.json').read_bytes());native=results['native-comparison-generation'];union=results['aggregate-union']
assert paired['complete_original_call_interfaces']==native['complete_calls']==oracle['normal_streams']==32
assert paired['complete_same_byte_union_streams']==2 and union['whole_original_declaration_comparisons']==81420
metadata=[json.loads(p.read_bytes()) for p in (ROOT/'generation-linked/certificates').glob('*.sources.json')]
row_maxima={key:max(row[key] for row in metadata) for key in ['declarations','terms','bytes']}
union_maxima={key:max(row[key] for row in paired['units']) for key in ['declarations','terms','bytes']}
for maxima in [row_maxima,union_maxima]:assert maxima['declarations']<=4096 and maxima['terms']<=131072 and maxima['bytes']<=2097152
lint=json.loads((ROOT/'lint/inputs.json').read_bytes());lint_rows={r['path']:r for r in lint['sources']}
changed=['crates/mpk-vc/src/go_practical_type_keys.rs','crates/mpk-vc/src/go_practical_type_keys/source_scalar_typing.rs','crates/mpk-vc/src/go_practical_type_keys/source_scalar_typing/call_sites.rs','crates/mpk-vc/src/go_practical_ordinary.rs','crates/mpk-vc/src/go_practical_ordinary/source_invocation.rs','crates/mpk-vc/tests/go_practical_source_invocation.rs']
for name in changed:
 r=lint_rows[name];assert seal(name)=={k:r[k] for k in ['path','bytes','sha256']} and Path(name).stat().st_mode&0o777==r['mode']
summary=('Every static, method and closed-error call is derived from the complete original checked Go body closure. '
'Original argument expression groups retain all tuple projections into the complete materialized parameter list, including receivers, '
'source/parameter types and untyped nil. Results, actual callee origins, receiver paths and the complete global error target universe remain source-owned. '
'A private ordinary interface couples one unbounded Nat invocation identity, complete evaluated arguments, before state, all results and post state. '
'State and invoke remain universally quantified. CaptureCall has one completion binder and one invoke application occurrence; '
'all five joint-field laws, every result projection, full result packing and the zero-result unit law retain their checked ordinary proofs.')
evidence=('All32 original call interfaces retain every source type/U and all65 input positions/16 heterogeneous results/void calls/method values and expressions. '
'One original expression forwards all16 result positions. Actual Go independently matches all32 callers, physical sites, full signatures, '
'154 argument groups,184 value projections, nil markers and possible targets in complete zero/70-case units. '
'All32 exact source-site imports and every original predecessor identity are checked without repeating kernel work. '
'An independent oracle reconstructs5760 source recipe members and1276 invocation definitions, law statements and proofs. '
'The complete original named declaration union is checked in two whole certificates by both unchanged real kernels, with zero axioms and identical hashes/counts. '
'Independent full AST comparison checks81420 original type/body/proof and constructor/recursor ownership identities, with no missing or extra declarations. '
'Cross-site import rejects; both kernels genuinely reject the stale-post law, while an accepted extra theorem fails source import and the export-set oracle. '
'Whole-union maxima3492 declarations/129452 terms/1310358 bytes and per-row maxima2946/125036/1210450 fit unchanged caps. '
'This establishes every original ordinary declaration obligation; it does not claim all32 separate original certificate wrappers were individually accepted.')
failure=('The first native comparison helper rejected builtin string after full original source/export/closure capture; its comparison-only replacement covers '
'all actual frozen type arms. The first Rust selection lacked explicit error input entries; the test derives all entries from complete actual signatures '
'and every nominal/struct/array/slice edge, keeping every root and global zero/70-case universe. '
'The first auxiliary Cargo fingerprint lookup and omitted sha2 link failures retain original drivers and actual diagnostics. '
'The first full union exceeded the term cap because its initial term indices were not normalized; the entire192749-term candidate remains. '
'Interning equal terms from the first original preserves every named type, body and proof while fitting the unchanged cap. '
'A temporary auxiliary compiler output path was reused; exact original failed tool bytes were recovered by the original compilation and verified against '
'its pre-execution SHA/length. Both tools remain at distinct paths and current bytes/modes are restored. '
'The first available Go batch also selected wide-000 after its Rust report arrived while the standalone probe was running; both genuine executions remain. '
'Later work excludes already-launched inputs by exact prelaunch receipts. No live verification is restarted on an observation timeout. '
'Both complete native original trees,43922 files/351161848 bytes with original modes, remain losslessly archived and physically retained.')
selection=('Checks cover the new source-call descriptor/interface and its direct source typing, domain, ordinary proof and native signature consumers. '
'All original candidates remain, and equivalent full named obligations share the same checked foundation instead of repeatedly checking it per call. '
'The original per-row runs retain their actual terminal statuses; incomplete individual wrapper checks are distinguished from the completed whole-obligation proof checks. '
'Existing execution matrices, unrelated kernel/frontend tests and removed-language tests are not rerun. Scoped Clippy/Rust/Go format pass on unchanged final production/test bytes. '
'No intermediate broad gate or decomposed equivalent is run.')
remaining=('This is a typed joint data interface. An ordinary proof must still bind invoke to the actual original Go callee. '
'Operand execution/once sequencing, callee preconditions/results/effects/frame, state/storage/alias semantics, and source/IR/program/canonical capture/context '
'correspondence remain due. Full56-family assembly, all169 demands, private descriptor/registry integration, native Linux release gates and T02-T11 remain due. '
'R01/R06/W01 stay open; the practical profile stays inactive. The cumulative T02 gate is deferred to final W04: ./scripts/check-fast.sh. '
'Go/Rust only; no other-language implementation, test or release dependency is added.')
record={'schema':'mpk.go_practical.source_invocation_checkpoint.v1','status':'scoped_whole_named_union_checks_passed_complete_W01_due','basis_commit':'75cc4973b1411337707b3c096215648309e827c8','original_call_interfaces':32,'complete_same_byte_union_streams':2,'whole_original_declaration_comparisons':81420,'source_recipe_members':5760,'invocation_members':1276,'complete_native_calls':32,'per_row_maxima':row_maxima,'whole_union_maxima':union_maxima,'individual_original_matrix_statuses':{n:r['status'] for n,r in originals.items()},'all32_individual_certificate_wrappers_accepted_claimed':False,'scope':summary,'verification':evidence,'original_failure_retention':failure,'test_selection':selection,'remaining':remaining,'checks':{n:seal(ROOT/n/'result.json') for n in checks},'original_individual_runs':{n:seal(ROOT/n/'result.json') for n in originals},'full_T02_gate':'deferred to final W04: ./scripts/check-fast.sh','T02_W01_complete':False,'public_profile_activated':False,'ordinary_source_ir_program_or_context_proof_authority':False,'other_language_tests_executed':False,'native_Linux_release_acceptance_claimed':False}
(ROOT/'verification.json').write_text(json.dumps(record,indent=2)+'\n')
links='''

- [Actual scoped results](verification.json)
- [Both kernels and all original source imports](aggregate-paired/result.json)
- [Independent complete named declaration union](aggregate-union/result.json)
- [Independent complete recipe comparison](oracle-generation/structural-comparison.json)
- [Original Go source call comparison](native-comparison-generation/result.json)
- [Complete native original preservation](native-original-relocations.json)
- [Exact original auxiliary tool recovery](aggregation-binary-path-repair/result.json)
- [Scoped source/input review](input-review.json)
'''
(ROOT/'README.md').write_text('# Original-source joint invocation interfaces\n\n'+summary+'\n\n'+evidence+'\n\n'+failure+'\n\n'+selection+'\n\n'+remaining+links)
p=ROOT.parents[1]/'requirements.json';r=json.loads(p.read_bytes());item=next(row for row in r['requirements'] if row['id']=='T02-W01-R01');item['source_joint_invocation_checkpoint']={'evidence':'verification/source-invocations/verification.json','scope':summary,'verification':evidence,'remaining':remaining,'R01_complete':False,'R06_complete':False,'W01_complete':False,'public_profile_activated':False,'not_an_ordinary_source_program_proof':True};p.write_text(json.dumps(r,ensure_ascii=False,indent=2)+'\n')
p=REPO/'develop/docs/go-practical-implementation-traceability-ledger.md';assert '## 174.' not in p.read_text();p.write_text(p.read_text()+'\n\n## 174. T02-W01 original-source joint invocation interfaces\n\nThe [joint invocation checkpoint](../migrations/go-practical/t02-w01/verification/source-invocations/README.md) binds all original source-call descriptors to complete ordinary joint records.\n\n'+summary+'\n\n'+evidence+'\n\n'+failure+'\n\n'+remaining+'\n')
p=ROOT.parents[1]/'README.md';p.write_text(p.read_text()+'\n\n## Original-source joint invocation interfaces\n\nThe [joint invocation checkpoint](verification/source-invocations/README.md) retains every original call, expression group, full signature and possible target.\n\n'+summary+'\n\n'+evidence+'\n\n'+remaining+'\n')
print(json.dumps({'status':'recorded','original_call_interfaces':32,'whole_union_streams':2,'ordinary_source_program_proof':False,'full_T_gate':'deferred W04'}))
