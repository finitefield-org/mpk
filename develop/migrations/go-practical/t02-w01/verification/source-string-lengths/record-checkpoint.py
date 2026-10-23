"""Record actual terminal scoped evidence; full T02/profile remains due."""
from pathlib import Path
import hashlib,json
ROOT=Path(__file__).resolve().parent;REPO=ROOT.parents[5]
def seal(p):
 b=Path(p).read_bytes();return {'path':str(p),'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()}
checks=['rust','paired','oracle','native','native-comparison','lint','registry','format-check','go-format-check','checker-protocol-fixed','checker-format-fixed']
results={name:json.loads((ROOT/name/'result.json').read_bytes()) for name in checks};assert all(r['status']=='passed' for r in results.values())
assert (ROOT/'go-format-check/stdout.log').read_bytes()==b''
paired=results['paired'];oracle=json.loads((ROOT/'oracle/structural-comparison.json').read_bytes())
normal=[json.loads(p.read_bytes()) for p in (ROOT/'rust/certificates').glob('source-length-*.sources.json')]
assert (len(normal),paired['same_byte_streams'],paired['zero_axiom_acceptances'],paired['genuine_type_rejections'])==(12,16,13,3)
maxima={k:max(r[k] for r in normal) for k in ['declarations','terms','bytes']};assert maxima['declarations']<=4096 and maxima['terms']<=131072 and maxima['bytes']<=2097152
summary=f'''The original-source string length catalogue returns the actual source builtin int carrier, preserving the entire64-bit header and balanced signed Int interpretation. It neither requires source uint8 nor fabricates a missing source int. Real string length-fit/validity proofs imply a clear sign bit and whole Int equality to the natural length, using unbounded Nat order and a balanced2^63 threshold without truncation, bounded length unrolling, axioms or changed caps. Every original source type/U and prior source-string declaration identity remains. Normal12 rows cover all4 builtin/nominal string types in each complete original0/4/70-error-case scope. Both unchanged full cores agree on16 streams:13 zero-axiom mathematical acceptances and3 genuine type mismatches. The accepted extra theorem still fails exact source import and the independent export-set oracle. Independent comparison checks{oracle['new_source_members']} source members,{oracle['generic_public_statements']} generic public statements,{oracle['closed_polynomial_checkpoints']} closed polynomial checkpoints and{oracle['exact_source_recipe_members']} original source recipe members, with{oracle['full64_bit_observations']} full64-bit observations. Original Go independently confirms4 dynamic int-returning len calls and all12 rows' original identities/U. Maxima{maxima['declarations']} declarations/{maxima['terms']} terms/{maxima['bytes']} bytes fit the unchanged4096/131072/2097152/1024 caps.'''
remaining='''This private typed representation/length bridge does not establish original CallExpr evaluation, operand-once execution, arbitrary runtime-input validity, source effects/frame or canonical capture/context/program correspondence. Complete56 foundation coverage, all169 demands, native Linux release gates and remaining T02-T11 work remain due. R01/R06/W01 remain open and the practical profile is inactive. The cumulative T02 gate remains final W04: ./scripts/check-fast.sh. Go/Rust only; no removed-language test or release prerequisite.'''
failure='''The initial formatter-driver path error remains with its full original inputs and actual failure output; correcting that path preceded every passing source check. No kernel test failed and no live verification was restarted. The original Go10983 output files/87657768 bytes and all modes remain losslessly archived and retained intact. No source, type/U, field, certificate, scanner rule, seal or exclusion is pruned or weakened.'''
record={'schema':'mpk.go_practical.source_string_lengths_checkpoint.v1','status':'scoped_checks_passed_complete_W01_due','basis_commit':'0e29b019d9807530aadad7c15cabdf6219b6159c','normal_rows':12,'same_whole_streams':16,'zero_axiom_math_acceptances':13,'genuine_type_rejections':3,'new_source_members':oracle['new_source_members'],'generic_public_statements':oracle['generic_public_statements'],'closed_polynomial_checkpoints':oracle['closed_polynomial_checkpoints'],'source_recipe_members':oracle['exact_source_recipe_members'],'full64_bit_observations':oracle['full64_bit_observations'],'maxima':maxima,'checks':{n:seal(ROOT/n/'result.json') for n in checks},'scope':summary,'original_failure_retention':failure,'remaining':remaining,'full_T02_gate':'deferred to final W04: ./scripts/check-fast.sh','T02_W01_complete':False,'public_profile_activated':False,'source_expression_or_context_authority':False,'other_language_tests_executed':False,'native_Linux_release_acceptance_claimed':False}
(ROOT/'verification.json').write_text(json.dumps(record,indent=2)+'\n')
(ROOT/'README.md').write_text('# Original-source Go int string lengths\n\n'+summary+'\n\n'+failure+'\n\n'+remaining+'''

Targeted tests cover the new catalogue/composer and their source consumers.
Complete prior source-string identities are compared inside each new stream;
unrelated old matrices are not rerun. Scoped Clippy and Rust/Go format pass.
Native facts never enter the Rust constructor. Concrete DAG observations
accelerate only exact prior accepted Nat definitions and supplement universal
laws checked by both cores.

The [current language documentation](../../../../go-rust-only/current-documentation/README.md)
also removes other-language continuation and release directions, preserving
complete prior bytes. Its scope, publication and installed-registry checks are
separate from full T02 and native release acceptance.

- [Actual targeted results](verification.json)
- [Whole-stream checker agreement](paired/result.json)
- [Independent complete matrix](oracle/structural-comparison.json)
- [Original Go comparison](native-comparison/result.json)
- [Scoped input review](input-review.json)
- [Complete original retention](native-original-relocations.json)
''')
p=ROOT.parents[1]/'requirements.json';r=json.loads(p.read_bytes())
for row in r['requirements']:
 if row['id']=='T02-W01-R01':row['source_int_string_length_checkpoint']={'evidence':'verification/source-string-lengths/verification.json','scope':summary,'remaining':remaining,'R01_complete':False,'R06_complete':False,'W01_complete':False,'public_profile_activated':False}
p.write_text(json.dumps(r,ensure_ascii=False,indent=2)+'\n')
p=REPO/'develop/docs/go-practical-implementation-traceability-ledger.md';assert '## 171.' not in p.read_text();p.write_text(p.read_text()+'\n\n## 171. T02-W01 original-source Go int string lengths\n\nThe [length checkpoint](../migrations/go-practical/t02-w01/verification/source-string-lengths/README.md) extends the typed source value bridge.\n\n'+summary+'\n\n'+failure+'\n\nThe [current documentation closure](../migrations/go-rust-only/current-documentation/README.md) removes remaining other-language continuation/release directions from the charter, summary, Rust handoff, roadmap, release gates and registry admission rules. Complete originals,22 resolved links, unchanged publication pins and the existing installed Go/Rust-only registry rejection test are retained.\n\n'+remaining+'\n')
p=ROOT.parents[1]/'README.md';p.write_text(p.read_text()+'\n\n## Original-source Go int string lengths\n\nThe [length checkpoint](verification/source-string-lengths/README.md) preserves every original source type/U and prior string identity.\n\n'+summary+'\n\n'+remaining+'\n')
print(json.dumps({'status':'recorded','same_whole_streams':16,'normal_rows':12,'full64_bit_observations':oracle['full64_bit_observations'],'maxima':maxima}))
