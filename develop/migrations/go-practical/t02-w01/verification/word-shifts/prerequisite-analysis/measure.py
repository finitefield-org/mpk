"""Read exact closed ordinary inputs to measure shift prerequisites, not authority."""
from pathlib import Path
import datetime,hashlib,json
root=Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
def seal(p):
 p=Path(p);b=p.read_bytes();return {'path':str(p),'sha256':hashlib.sha256(b).hexdigest(),'bytes':len(b)}
reader=root.parent.parent/'word-signed-division/prerequisite-analysis/measure.py'
ns={'__file__':str(reader.resolve())};exec(compile(reader.read_text().split('sources=[]',1)[0],str(reader),'exec'),ns)
Tables=ns['Tables'];sources=[]
for directory in ['word-conversions','word-arithmetic','word-foundation']:
 v=json.loads((root.parent.parent/directory/'verification.json').read_text());sources.append(Tables(v['fresh_component']['certificate']))
index={};hashes={}
for s,c in enumerate(sources):
 fp=c.fingerprints()
 for name,h in fp.items():
  if name in hashes:assert hashes[name]==h,name
  else:hashes[name]=h;index[name]=s
roots=list(sources[0].index)
for w in [8,16,32,64]:
 for n in ['ShiftLeft','ShiftRight','SignedShiftRight','Zero','Ones','Select','SignedCountDefined']:
  roots.append(f'Mpk.Go.Practical.Word.W{w}.{n}')
 for i in [0,*[1<<j for j in range(w.bit_length()-1)]]:
  roots.append(f'Mpk.Go.Practical.Word.Numeric.W{w}.ShiftConservation{i}')
 roots.append(f'Mpk.Go.Practical.Word.Numeric.W{w}.ZeroValue')
for n in ['PowerTwoAdd','PowerTwoZero','PowerTwoSucc',*[f'PowerTwoWeight{d}' for d in range(7)],'W64.DigitsValue']:
 roots.append('Mpk.Go.Practical.Word.Numeric.'+n)
selected=set();pending=list(roots)
while pending:
 name=pending.pop()
 if name in selected:continue
 selected.add(name);c=sources[index[name]]
 closure,_=c.closure([name]);pending.extend(closure-selected)
base=set(sources[0].index)
record={'schema':'mpk.go_practical.word_shift_dependency_planning.v1','status':'measured_original_named_closure_not_checker_authority','observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'sealed_original_inputs':[c.record for c in sources],'original_declaration_counts':[len(c.index)for c in sources],'shared_identity_conflicts':[],'complete_original_conversions':len(base),'selected_closed_named_union':len(selected),'additional_original_shift_prerequisites':len(selected-base),'remaining_slots_before_new_shift_laws':4096-len(selected),'fixed_roots':roots,'selected_names':sorted(selected),'reader_source':seal(reader),'scope':'Complete current conversion component plus fixed real all-width variable shift circuits, power-of-two fixed-left conservation and full64 digit expansion. Every original type/body/proof/level/family identity matches overlaps without rewrite/substitution. Small diagnostic reader neither generates nor accepts proof. Full unchanged production cores still must inspect actual closed proof terms, all source count binding/saturation and both right-shift interpretations within unchanged4096/131072/2097152 caps. No checker/fuel/axiom widening, source/application/profile activation or whole W01 completion.'}
(root/'result.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({k:record[k]for k in ['original_declaration_counts','complete_original_conversions','selected_closed_named_union','additional_original_shift_prerequisites','remaining_slots_before_new_shift_laws']}))
