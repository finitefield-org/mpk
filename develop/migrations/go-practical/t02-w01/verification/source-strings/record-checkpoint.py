"""Record only actual terminal scoped results; W01/profile/full gate stay open."""
from pathlib import Path
import hashlib,json
ROOT=Path(__file__).resolve().parent
REPO=ROOT.parents[5]
def seal(p):
 b=Path(p).read_bytes();return {'path':str(p),'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()}
checks={name:json.loads((ROOT/name/'result.json').read_bytes()) for name in ['validity-parts','paired','oracle-names-fixed','lint-validity-parts','format-check','go-format-check','native','native-payloads']}
assert all(r['status']=='passed' for r in checks.values())
assert (ROOT/'go-format-check/stdout.log').read_bytes()==b''
paired=checks['paired'];oracle=json.loads((ROOT/'oracle-names-fixed/structural-comparison.json').read_bytes());assert oracle['status']=='passed'
metadata=[json.loads(p.read_bytes()) for p in sorted((ROOT/'validity-parts/certificates').glob('*.sources.json')) if not p.name.startswith('control-')]
normal=len(metadata);positive=paired['zero_axiom_acceptances'];negative=paired['genuine_type_rejections']
assert (normal,positive,negative)==(8,9,4)
maxima={k:max(r[k] for r in metadata) for k in ['declarations','terms','bytes']}
assert maxima['declarations']<=4096 and maxima['terms']<=131072 and maxima['bytes']<=2097152
summary=f'''The source string adapter checkpoint preserves all original source types and complete error universes in eight normal rows: nil-only, entry, heterogeneous, all70 cases, and every builtin/nominal string type in a fresh four-case original-Go unit. Whole source/math roundtrips, source-zero equality and validity, full64-bit length/all256 byte-slot observations, constructors and guarded byte observation laws are ordinary checked proofs. Both unchanged full cores agree on{positive+negative} streams: {positive} zero-axiom mathematical acceptances and{negative} genuine type rejections. A coherent constant-length adapter is accepted as mathematics but fails exact source import and the independent original-body oracle. Independent checks cover{oracle['source_recipe_members']} original source recipe members and{oracle['complete_bit_observations']} actual length/data-bit observations. Maxima are{maxima['declarations']} declarations/{maxima['terms']} terms/{maxima['bytes']} bytes under unchanged caps.'''
limits='''These are private representation adapters. They do not establish Go len result/int index typing, source expression evaluation, arbitrary runtime-input validity, source error dispatch safety/effects/frame, canonical capture/context authority or profile activation. The full56 foundation, private registry, nativeLinux/all169 and remaining T02-T11 work stay due. Go/Rust only; R01/R06/W01 remain open. The cumulative T02 gate stays final W04: ./scripts/check-fast.sh.'''
failures='''All original failures remain: binder-lifetime and report-API compile errors; MakeSize then MakeValid fuel failures; and the original named-string error-payload fixture rejected by the frozen source-domain rule. The initial independent observer also retained its name-table shadowing failure; only that observer is rerun after correction. The corrected independent four-case fixture uses string-valued struct fields and keeps the complete source types/U. MakeSize/ZeroSize use explicit equality transport, and MakeValid uses a general ValidOfParts witness. No cap increase, funext, source pruning, test restart on observation timeout or unrelated regression sweep occurs.'''
record={'schema':'mpk.go_practical.source_strings_checkpoint.v1','status':'passed_scoped_representation_adapters_full_W01_due','basis_commit':'82bd5eedf30c51a443df7bf7092407a41c265892','normal_rows':normal,'same_whole_streams':positive+negative,'zero_axiom_math_acceptances':positive,'genuine_core_type_rejections':negative,'maxima':maxima,'independent_source_recipe_members':oracle['source_recipe_members'],'independent_actual_bit_observations':oracle['complete_bit_observations'],'checks':{name:seal(ROOT/name/'result.json') for name in checks},'failure_scope':failures,'scope':summary,'remaining':limits,'full_T02_gate':'deferred to final W04: ./scripts/check-fast.sh','T02_W01_complete':False,'public_profile_activated':False,'native_Linux_release_acceptance_claimed':False,'other_language_tests_executed':False}
(ROOT/'verification.json').write_text(json.dumps(record,indent=2)+'\n')
text='''# Source-specific bounded strings\n\n'''+summary+'\n\n'+failures+'''\n\nThe verification scope is the new adapter/composer and its callers. Existing
unrelated ordinary producers and both cores are unchanged. Scoped Rust/Go
format and Clippy pass; the native Go fixture is the only frontend test run.
All original source epochs, failed outputs, whole streams, checker inputs and
binary identities remain receipt-bound. Native mode0600 outputs are preserved
losslessly with original paths/bytes/modes and separately retained intact;
the archives are decoded for content review. No scanner rule or exclusion is
changed. Prior historical/binary findings remain due for the final strict gate.

'''+limits+'''

- [Actual verification results](verification.json)
- [Same-byte Go/Rust checker agreement](paired/result.json)
- [Independent structure and bit observations](oracle-names-fixed/structural-comparison.json)
- [Scoped input review](input-review.json)
- [Lossless native-original retention](native-original-relocations.json)
- [Initial failed whole suite](first/result.json)
- [First declaration diagnosis](fuel-diagnosis/result.json)
- [Second declaration diagnosis](fuel-transport-diagnosis/result.json)
'''
(ROOT/'README.md').write_text(text)
requirements=ROOT.parents[1]/'requirements.json';r=json.loads(requirements.read_bytes())
for row in r['requirements']:
 if row['id']=='T02-W01-R01':row['source_string_representation_checkpoint']={'evidence':'verification/source-strings/verification.json','scope':summary,'remaining':limits,'T02_W01_complete':False,'public_profile_activated':False}
requirements.write_text(json.dumps(r,ensure_ascii=False,indent=2)+'\n')
section='''\n\n## 169. T02-W01 original-source bounded string adapters\n\nThe [source-string checkpoint](../migrations/go-practical/t02-w01/verification/source-strings/README.md) adds the private source-specific adapters.\n\n'''+summary+'\n\n'+failures+'\n\n'+limits+'\n'
ledger=REPO/'develop/docs/go-practical-implementation-traceability-ledger.md';assert '## 169.' not in ledger.read_text();ledger.write_text(ledger.read_text()+section)
p=ROOT.parents[1]/'README.md';p.write_text(p.read_text()+'''\n\n## Original-source bounded string adapters\n\nThe [string checkpoint](verification/source-strings/README.md) preserves every
original source type/U and adds builtin/nominal string adapters, whole
roundtrips, original zero/validity and constructor/guarded byte observations.
'''+summary+'\n\n'+limits+'\n')
print(json.dumps({'status':'recorded','whole_streams':positive+negative,'maxima':maxima}))
