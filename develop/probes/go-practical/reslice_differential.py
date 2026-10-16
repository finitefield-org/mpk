"""Signed source reslicing versus bit-loop metadata and separate Go outcomes."""
import json

import access_differential as access
from byte_differential import less_word
from order_differential import adder
from run_proof_pilot import json_bytes,sha,timed
from slice_differential import cell,ZERO

CAPACITIES=(0,1,8,32,127,128,129)
KINDS=("two","three")
FAMILIES=("validity","capacity","mapped-guard","read")
ARTIFACTS=("reslice-input.json","reslice-output.json")
MASK=(1<<64)-1
SCOPE="actual signed two/three-index operations and separate raw-model validity/bit-loop arithmetic; observer allocation/writes/recovery stay outside the proof closure and establish no production Go capture/ownership binding"
WITNESS_SCOPE="explicit source-operation-panic or bounded-source witnesses; source construction, raw invalid metadata and out-of-profile reads remain distinct"

def row(b,kind,n,o,length,cap,lo,hi,maximum,index,*,nil=False,region=False,dead=False):
 r=access.row(b,n,o,length,cap,index,nil=nil,region=region,dead=dead)
 r.update(kind=kind,low=str(lo),high=str(hi),maximum=str(maximum))
 return r

def inputs():
 result=[]
 for b in CAPACITIES:
  one=min(b,1);half=b//2
  parents=list(dict.fromkeys([(0,0,b),(0,b,b),(0,one,b),(one,0,b-one),(one,min(1,b-one),b-one),(half,b-half,b-half),(b,0,0),(0,0,one)]))
  for kind in KINDS:
   for o,length,cap in parents:
    specs=[(0,0,0),(0,length,cap),(0,cap,cap),(length,length,cap),(cap,cap,cap),(cap//2,cap,cap),(0,0,cap),(0,min(1,cap),min(1,cap))]
    if kind=="two":specs=[(lo,hi,cap) for lo,hi,_ in specs]
    for lo,hi,m in dict.fromkeys(specs):
     for index in dict.fromkeys((-1,0,hi-lo-1,hi-lo,-(1<<63),(1<<63)-1)):
      for region in (False,True):result.append(row(b,kind,b,o,length,cap,lo,hi,m,index,region=region))
   for lo,hi,m in [(-1,0,0),(1,0,b),(0,b+1,b),(0,1,0),(0,0,-1),(-(1<<63),0,b),(0,(1<<63)-1,b),(0,0,(1<<63)-1)]:
    if kind=="two":m=b
    result.append(row(b,kind,b,0,0,b,lo,hi,m,0))
   result.extend([row(b,kind,0,0,0,0,-1,0,0,0),row(b,kind,0,0,0,0,0,0,0,0,nil=True),row(b,kind,0,0,0,0,-1,0,0,0,nil=True),
                  row(b,kind,1,0,0,1,0,0,0,0),row(b,kind,1,0,0,1,0,1,1,0),row(b,kind,1,0,0,1,0,1,1,0,dead=True),
                  row(b,kind,1,0,2,1,0,1,1,0),row(b,kind,0,0,0,1,0,0,0,0,nil=True)])
   result.append(row(b,kind,0,0,0,0,-1,-1,-1,0,nil=True))
   for n,i in ((b+1,b),(access.padding(b)+1,access.padding(b))):result.append(row(b,kind,n,0,0,n,0,n,n,i))
 return list({json.dumps(r,sort_keys=True):r for r in result}.values())

def signed_le(x,y):
 x&=MASK;y&=MASK;sx=bool(x&(1<<63));sy=bool(y&(1<<63))
 return not less_word(64,y,x) if sx==sy else sx

def model(r):
 n,o,length,cap,lo,hi,m,index=(int(r[k]) for k in ("storage","offset","length","capacity","low","high","maximum","index"))
 if r["kind"]=="two":m=cap
 no,oc,_=adder(64,o&MASK,lo&MASK,False)
 nl,lc,_=adder(64,hi&MASK,(lo&MASK)^MASK,True)
 nk,kc,_=adder(64,m&MASK,(lo&MASK)^MASK,True)
 mapped,mc,_=adder(64,lo&MASK,index&MASK,False)
 physical,pc,_=adder(64,o&MASK,mapped,False)
 old=access.model({**r,"index":str(access.signed(mapped))})
 new=access.model({**r,"offset":str(access.signed(no)),"length":str(access.signed(nl)),"capacity":str(access.signed(nk))})
 guard=not bool((lo&MASK)&(1<<63)) and signed_le(lo,hi) and signed_le(hi,m) and signed_le(m,cap)
 resliced=old["constructed"] and guard
 read=resliced and index>=0 and index<access.signed(nl)
 full=old["constructed"] and not r["nil"] and access.signed(mapped)>=0 and access.signed(mapped)<cap
 original=old["read_defined"]
 seed=0x6a09e667f3bcc909 if r["region"] else 0
 reconstructed=physical&(access.padding(r["profile"])-1)
 value=cell(seed,physical) if read else ""
 replacement=cell(seed^0x510e527fade682d1,physical) if read and full else ""
 return {"valid_old":old["valid"],"guard":guard,"new_offset":str(no),"offset_carry":oc,"new_length":str(nl),"length_borrow":not lc,"new_capacity":str(nk),"capacity_borrow":not kc,
         "valid_new":new["valid"],"storage_bound":not less_word(64,r["profile"],n&MASK),"constructed":old["constructed"],"construction_panicked":old["construction_panicked"],
         "resliced":resliced,"reslice_panicked":old["constructed"] and not guard,"actual_nil":resliced and r["nil"],"actual_length":str(nl) if resliced else "","actual_capacity":str(nk) if resliced else "",
         "new_guard":not r["nil"] and index>=0 and index<access.signed(nl),"mapped":str(mapped),"mapped_carry":mc,"physical":str(physical),"physical_carry":pc,
         "covered":physical==reconstructed,"reconstructed":str(reconstructed),"modeled_value":cell(seed,reconstructed),"read_defined":read,"read_panicked":resliced and not read,"actual_value":value,
         "full_defined":full,"full_panicked":old["constructed"] and not full,"full_value":cell(seed,physical) if full else "","original_defined":original,"original_panicked":old["read_panicked"],"original_value":old["actual_value"],
         "replacement":replacement,"alias_after":replacement,"full_after":replacement}

def compare(cases,outputs):
 if cases!=inputs() or len(cases)!=len(outputs):raise ValueError("reslice observation input coverage")
 counts={k:0 for k in ("source_operations","operation_panics","construction_rejections","source_reads","index_panics","proof_domain_reads","tail_exposures","restricted_three_capacities","nil_results","nonnil_empty_results","alias_updates","out_of_profile_reads","padding_not_admission","uncovered_reads","valid_total_result_despite_source_panic")}
 for r,a in zip(cases,outputs):
  expected=model(r)
  if not isinstance(a,dict) or set(a)!=set(expected) or any(type(a[k]) is not type(v) for k,v in expected.items()):raise ValueError("reslice observation field/type mismatch")
  if a!=expected:raise ValueError("Go/bit-loop reslice mismatch")
  if a["valid_old"] and a["guard"] and not a["valid_new"]:raise ValueError("computed reslice validity failed")
  domain=a["valid_old"] and a["guard"] and a["new_guard"] and a["storage_bound"]
  if domain and not (a["read_defined"] and a["full_defined"] and a["covered"] and a["actual_value"]==a["modeled_value"]==a["full_value"]):raise ValueError("bounded complete reslice read/coverage failed")
  if domain and (a["mapped_carry"] or a["physical_carry"]):raise ValueError("defined reslice address wrapped")
  if a["read_defined"] and (a["actual_value"]!=a["full_value"] or a["alias_after"]!=a["replacement"] or a["full_after"]!=a["replacement"] or a["alias_after"]==a["actual_value"]):raise ValueError("source backing-sharing observation failed")
  counts["source_operations"]+=a["resliced"];counts["operation_panics"]+=a["reslice_panicked"];counts["construction_rejections"]+=not a["constructed"]
  counts["source_reads"]+=a["read_defined"];counts["index_panics"]+=a["read_panicked"];counts["proof_domain_reads"]+=domain
  counts["tail_exposures"]+=a["read_defined"] and not a["original_defined"]
  counts["restricted_three_capacities"]+=r["kind"]=="three" and a["resliced"] and int(a["actual_capacity"])<int(r["capacity"])-int(r["low"])
  counts["nil_results"]+=a["resliced"] and a["actual_nil"];counts["nonnil_empty_results"]+=a["resliced"] and not a["actual_nil"] and a["actual_length"]=="0"
  counts["alias_updates"]+=a["read_defined"] and bool(a["replacement"])
  counts["out_of_profile_reads"]+=a["read_defined"] and not a["storage_bound"]
  counts["padding_not_admission"]+=a["read_defined"] and r["profile"]<int(r["storage"])<=access.padding(r["profile"])
  counts["uncovered_reads"]+=a["read_defined"] and not a["covered"]
  counts["valid_total_result_despite_source_panic"]+=a["reslice_panicked"] and a["valid_new"]
 if not all(counts.values()):raise ValueError("lost reslice nonvacuity/boundary observations")
 return counts

def negative_witnesses(cases,outputs):
 out=[]
 for b in CAPACITIES:
  for kind in KINDS:
   for family in FAMILIES:
    if not b and family!="validity":continue
    if family=="validity":wanted=row(b,kind,0,0,0,0,-1,0,0,0);domain="source_operation_panic"
    elif family=="capacity":wanted=row(b,kind,1,0,0,1,0,0,0,0);domain="bounded_source"
    else:wanted=row(b,kind,1,0,0,1,0,1,1,0);domain="bounded_source"
    i=cases.index(wanted);a=outputs[i]
    if family=="validity":left,right=a["valid_new"],True
    elif family=="capacity":left,right=a["actual_capacity"],str((int(wanted["capacity"]) if kind=="three" else int(wanted["high"]))-int(wanted["low"]))
    elif family=="mapped-guard":left,right=a["original_defined"],True
    else:left,right=a["actual_value"],ZERO
    if left==right:raise ValueError("no reslice false-claim witness")
    label={"validity":"erased-range-guard","capacity":"wrong-capacity","mapped-guard":"original-length-guard","read":"zeroed-read"}[family]
    out.append({"case_id":f"reslice-{b}-{kind}-{family}-{label}","input_position":i,"domain":domain,"left":left,"right":right})
 return out

def run(executable,directory):
 cases=inputs();p=directory/ARTIFACTS[0];p.write_bytes(json_bytes(cases));measured=timed([executable,p])
 if measured["exit_code"]:raise RuntimeError("actual Go reslice observer failed")
 raw=measured.pop("stdout").encode();(directory/ARTIFACTS[1]).write_bytes(raw);outputs=json.loads(raw)
 return {"scope":SCOPE,"cases":len(cases),"outcomes":compare(cases,outputs),"witness_scope":WITNESS_SCOPE,"false_claim_witnesses":negative_witnesses(cases,outputs),"input_sha256":sha(p.read_bytes()),"output_sha256":sha(raw),"go_binary_sha256":sha(executable.read_bytes()),"go_measurement":measured}

def check(report,directory):
 a,b=((directory/name).read_bytes() for name in ARTIFACTS)
 if report["scope"]!=SCOPE or report["input_sha256"]!=sha(a) or report["output_sha256"]!=sha(b):raise ValueError("reslice observation scope/byte identity")
 cases,outputs=json.loads(a),json.loads(b)
 if a!=json_bytes(inputs()) or report["cases"]!=len(cases) or report["outcomes"]!=compare(cases,outputs):raise ValueError("reslice observation outcome coverage")
 if report["witness_scope"]!=WITNESS_SCOPE or json_bytes(report["false_claim_witnesses"])!=json_bytes(negative_witnesses(cases,outputs)):raise ValueError("missing, mistyped or misclassified reslice witnesses")
 return len(cases)
