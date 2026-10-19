"""Diagnostic named metadata comparison of all exact original prerequisites."""
from pathlib import Path
import datetime,hashlib,json
root=Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
def seal(p):
 p=Path(p);b=p.read_bytes();return {'path':str(p),'sha256':hashlib.sha256(b).hexdigest(),'bytes':len(b)}
reader=root.parent/'word-signed-division/prerequisite-analysis/measure.py';ns={'__file__':str(reader.resolve())}
exec(compile(reader.read_text().split('sources=[]',1)[0],str(reader),'exec'),ns)
Tables=ns['Tables'];current=Tables(seal(root/'applied-count-cases/rust-construction/certificates/word-shifts.mpcert'));union={};refs=[]
for directory in ['word-conversions','word-arithmetic','word-foundation']:
 record=json.loads((root.parent/directory/'verification.json').read_text())['fresh_component']['certificate'];t=Tables(record);fp=t.fingerprints();refs.append(record)
 for n,v in fp.items():
  if n in union:assert union[n]==v,n
  union[n]=v
retained=[];new=[]
for n,v in current.fingerprints().items():
 if n.startswith('Mpk.Go.Practical.Word.Shifts.'):new.append(n)
 else:assert union[n]==v,n;retained.append(n)
assert len(current.index)==3105 and len(retained)==2669 and len(new)==436
old=Tables(refs[0]);assert len(old.index)==1882 and set(old.index).issubset(retained)
planning=json.loads((root/'prerequisite-analysis/result.json').read_text());assert set(retained)==set(planning['selected_names'])|{'Std.Bool.and','Std.Bool.if','Std.Bool.not','Std.Bool.or'}
record={'schema':'mpk.go_practical.word_shifts_retained_identity_observation.v1','status':'matched_structural_identities_not_checker_authority','observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'certificate':current.record,'complete_original_reference_certificates':refs,'reader_source':seal(reader),'reader_extraction':'Only definitions before sources=[]; original main is not executed or changed.','all_retained_original_named_identities':2669,'complete_prior_conversions':1882,'new_shift_declarations':436,'retained_names':sorted(retained),'new_shift_names':sorted(new),'scope':'Complete original named type/body/proof/level/family identities match without rename/substitution. Diagnostic metadata gives no authority; both unchanged production cores inspect every actual term. Full source/program/profile/application work remains due.'}
p=root/'retained-identities.json';assert not p.exists();p.write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({k:record[k]for k in ['status','all_retained_original_named_identities','complete_prior_conversions','new_shift_declarations']}))
