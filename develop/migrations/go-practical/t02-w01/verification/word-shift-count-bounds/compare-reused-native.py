"""Reuse exact old Bool/native circuits; check new finite count/stage counterparts."""
from pathlib import Path
import datetime,hashlib,json,sys
root=Path(__file__).resolve().parent.relative_to(Path.cwd().resolve());out=root/'finite-counterparts';out.mkdir(exist_ok=False)
def read(p):return json.loads(Path(p).read_text())
def seal(p):
 p=Path(p);b=p.read_bytes();return {'path':str(p),'sha256':hashlib.sha256(b).hexdigest(),'bytes':len(b)}
previous=root.parent/'word-shifts';v=read(previous/'verification.json');native=read(previous/'native/result.json');old_compare=read(previous/'native-comparison-final/result.json')
assert native['status']==old_compare['status']=='passed' and native['native_rows']==504 and native['two_native_outputs_byte_identical']
for key in ['original_source','owned_source','original_queries','owned_queries','complete_original_prepared_image']:assert seal(native[key]['path'])==native[key]
for command in native['commands']:
 assert command['exit_code']==0
 for key in ['stdout','stderr']:assert seal(command[key]['path'])==command[key]
for check in native['complete_original_image_checks']:assert check['status']=='passed' and check['files']==5577
assert (previous/'native/run1.stdout.log').read_bytes()==(previous/'native/run2.stdout.log').read_bytes()
for key in ['actual_Bool_model','model_metrics','independent_integer_oracle']:assert seal(old_compare[key]['path'])==old_compare[key]
original=read(previous/'query-origins.json')['original_input'];assert seal(original['path'])==original
qs=[q for q in read(original['path']) if q['operation'] in ['ShiftLeft','ShiftRight','SignedShiftRight']]
rows=read(previous/'native/run1.stdout.log');assert rows==read(old_compare['actual_Bool_model']['path']) and len(qs)==len(rows)==504
all_rows=[];guards=high_rows=stage_rows=explicit_powers=symbolic_powers=0
for q,row in zip(qs,rows,strict=True):
 assert q['id']==row['id'] and q['operation']==row['operation']
 w=q['width'];d=w.bit_length()-1;m=2**w;x=int(q['x_hex'],16);c=int(q['count_hex'],16);sgn=int(x>=m//2);sx=x-m*sgn
 lo=c%w;hi=c//w;ex=lo+w*max(hi-1,0);high=hi!=0;assert c==lo+w*hi and high==(c>=w) and (c<w)==(not high)
 assert row['low_count']==str(lo) and row['high_value']==str(hi) and row['count_high']==high
 if high:assert c==w+ex and row['large_count_excess']==str(ex)
 else:assert c==lo and row['large_count_excess'] is None
 prefixes=[]
 for i in range(7):
  n=sum(((c//(2**j))%2)*(2**j) for j in range(i));z=sum((1-((c//(2**j))%2))*(2**j) for j in range(i));assert n+z+1==2**i and 0<=n<2**i;prefixes.append({'bits':i,'value':str(n),'complement':str(z),'strict_bound':str(2**i)})
 defined=not q['signed_count'] or c<2**63;assert row['count_defined']==defined and row['native_count_panicked']==(not defined)
 result={'original_query':q,'count_prefixes':prefixes,'count_excess':str(ex),'source_defined':defined,'source_high':high}
 if not defined:guards+=1;assert row['result_hex'] is None and row['stage_hex'] is None;all_rows.append(result);continue
 high_rows+=high;stages=[int(h,16)for h in row['stage_hex']];assert len(stages)==d+1 and stages[0]==x
 n=D=R=0;stage_records=[]
 for i,word in enumerate(stages):
  p=2**n;sw=word-m*int(word>=m//2)
  if q['operation']=='ShiftLeft':assert word+m*D==p*x and p*sx==sw+m*(D+int(word>=m//2)-p*sgn)
  elif q['operation']=='ShiftRight':assert x==p*word+R and 0<=R<p
  else:assert x+m*(p-1)*sgn==p*word+R and sx==p*sw+R and 0<=R<p and int(word>=m//2)==sgn
  stage_records.append({'index':i,'count':str(n),'word_hex':row['stage_hex'][i],'discard':str(D) if q['operation']=='ShiftLeft' else None,'remainder':str(R) if q['operation']!='ShiftLeft' else None});stage_rows+=1
  if i<d and (c//(2**i))%2:
   k=2**i
   if q['operation']=='ShiftLeft':D=(2**k)*D+word//(2**(w-k))
   else:R+=p*(word%(2**k))
   n+=k
 assert n==lo
 actual=int(row['result_hex'],16);actual_s=actual-m*int(actual>=m//2)
 if high:
  assert actual==(m-1 if q['operation']=='SignedShiftRight' and sgn else 0)
  if c<=256:
   p=2**c;pe=2**ex;explicit_powers+=1
   if q['operation']=='ShiftLeft':fd=pe*x;assert actual+m*fd==p*x and p*sx==actual_s+m*(fd+int(actual>=m//2)-p*sgn);result['actual_discard']=str(fd)
   else:
    b=sgn if q['operation']=='SignedShiftRight' else 0;fr=x+m*(pe-1)*b;assert x+m*(p-1)*b==p*actual+fr and 0<=fr<p
    if b or q['operation']=='SignedShiftRight':assert sx==p*actual_s+fr
    result['actual_remainder']=str(fr)
  else:
   symbolic_powers+=1;result['symbolic_large_witness']={'kind':'left_discard' if q['operation']=='ShiftLeft' else 'filled_right_remainder','input':str(x),'modulus':str(m),'excess':str(ex),'fill':sgn if q['operation']=='SignedShiftRight' else 0};result['giant_power_materialized']=False
 else:
  p=2**c;assert actual==stages[-1]
  if q['operation']=='ShiftLeft':assert actual+m*D==p*x;result['actual_discard']=str(D)
  else:
   b=sgn if q['operation']=='SignedShiftRight' else 0;assert x+m*(p-1)*b==p*actual+R and 0<=R<p;result['actual_remainder']=str(R)
 result['stages']=stage_records;all_rows.append(result)
assert guards==24 and high_rows==309 and len(all_rows)==504
(out/'counterparts.json').write_text(json.dumps(all_rows,indent=2)+'\n')
record={'schema':'mpk.go_practical.full64_shift_finite_counterparts.v1','status':'passed','observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'old_native_execution':seal(previous/'native/result.json'),'old_actual_Bool_and_Python_comparison':seal(previous/'native-comparison-final/result.json'),'original_queries':original,'counterparts':seal(out/'counterparts.json'),'rows':504,'negative_native_count_panics':guards,'defined_high_counts':high_rows,'actual_stage_observations':stage_rows,'explicit_moderate_large_powers':explicit_powers,'symbolic_huge_powers':symbolic_powers,'unchanged_source_circuits_must_match_all3105_named_identities':True,'fresh_native_build_or_old_Bool_rerun':False,'new_Nat_Int_witness_or_theorem_bodies_executed':False,'scope':'The exact previously checked actual Bool/source stages and typed native Go outputs are reused because their original3105 named declaration identities remain exact in the new complete proof component. The new source/kernel proofs must independently be accepted by both unchanged cores; finite host counterparts confer no universal theorem authority. Prefix complements, exact small/large source counts, stage discards/remainders, signed modular-left and floor-right equations are checked with Python integers. All504 complete original rows and real24 negative-count panics remain. Huge counts retain their full values and symbolic witnesses without materializing enormous powers. Full source/profile/application authority remains due.','CSharp_tests_executed':False,'full_T02_gate':'deferred to T02-W04: check-fast.sh --skip-csharp'}
(out/'result.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({k:record[k]for k in ['status','rows','negative_native_count_panics','defined_high_counts','actual_stage_observations','symbolic_huge_powers']}))
