"""Independent bit-loop arithmetic and pre-state snapshot for native Go copy."""
import json
import access_differential as access
from byte_differential import less_word
from order_differential import adder
from run_proof_pilot import json_bytes,sha,timed
from slice_differential import cell,ZERO

CAPACITIES=(0,1,8,32,127,128,129)
ARTIFACTS=("copy-input.json","copy-output.json")
MASK=(1<<64)-1
SCOPE="actual Go copy over fresh local arrays versus independent full64 bit loops and immutable pre-state cells; observer allocation/naive-loop/recovery establish no production Go capture or borrowed-write ownership"
WITNESS_SCOPE="bounded-source counterexamples to count, erased guards, zeroed cells and poststate-source mutations; raw invalid metadata/bindings and source construction/index failures remain separate"

def families(n):return ('count',)+(('destination-guard','source-guard','copied-read','frame','other-frame') if n else ())+ (('snapshot',) if n>=8 else ())
def label(f):return {'count':'count-one','destination-guard':'erased-copy-guard','source-guard':'erased-copy-guard','snapshot':'poststate-source'}.get(f,'zeroed-cell')
def view(o=0,length=0,cap=0,*,nil=False,region=False):return dict(offset=str(o),length=str(length),capacity=str(cap),nil=nil,region=region)
def row(b,n,d,s,i=0,*,live=(True,True)):return dict(profile=b,storage=str(n),destination=d,source=s,index=str(i),live=list(live))

def inputs():
 out=[]
 for b in CAPACITIES:
  one=min(b,1);half=b//2
  layouts=[(view(0,b,b),view(0,b,b)),(view(0,b,b),view(0,b,b,region=True)),(view(0,b,b),view(0,one,b)),(view(0,one,b),view(0,b,b)),
   (view(0,b,b),view(0,0,b)),(view(0,0,b),view(0,b,b)),(view(half,b-half,b-half),view(0,b,b)),(view(0,0,0,nil=True),view(0,b,b)),
   (view(0,b,b),view(0,0,0,nil=True)),(view(0,0,0,nil=True),view(0,0,0,nil=True))]
  if b:layouts.extend([(view(1,b-1,b-1),view(0,b-1,b)),(view(0,b-1,b),view(1,b-1,b-1))])
  for d,s in layouts:
   m=min(int(d['length']),int(s['length']))
   for i in dict.fromkeys((-1,0,m-1,m)):
    out.append(row(b,b,d,s,i))
  out.append(row(b,0,view(nil=True),view(nil=True)))
  if b:
   out.extend([row(b,1,view(0,1,1),view(0,0,1)),row(b,1,view(0,0,1),view(0,1,1)),row(b,1,view(0,1,1),view(0,1,1)),row(b,1,view(0,0,1),view(0,0,1)),
               row(b,1,view(0,1,1),view(0,1,1),live=(False,True))])
  if b>=8:out.append(row(b,3,view(1,2,2),view(0,2,3),1))
  for bad in (-(1<<63),-1,(1<<63)-1):
   out.extend([row(b,b,view(bad,0,0),view(0,0,b)),row(b,b,view(0,0,b),view(0,bad,b)),row(b,b,view(0,0,b),view(0,0,b),bad)])
  out.extend([row(b,b,view(0,1,0,nil=True),view(0,0,b)),row(b,b,view(0,0,b),view(0,1,0,nil=True)),row(b,b,view((1<<63)-1,1,(1<<63)-1),view(0,0,b))])
  for n,i in ((b+1,b),(access.padding(b)+1,access.padding(b))):out.append(row(b,n,view(0,n,n),view(0,n,n),i))
 return list({json.dumps(r,sort_keys=True):r for r in out}.values())

def metadata(r,v):return access.model(dict(v,profile=r['profile'],storage=r['storage'],index=r['index'],live=r['live']))
def value(region,key):return cell(0x6a09e667f3bcc909 if region else 0,key)
def model(r):
 d,s=r['destination'],r['source'];n=int(r['storage']);i=int(r['index']);iu=i&MASK
 dm,sm=metadata(r,d),metadata(r,s);dl=int(d['length'])&MASK;sl=int(s['length'])&MASK;m=dl if less_word(64,dl,sl) else sl
 physical,carry,_=adder(64,int(d['offset'])&MASK,iu,False);relative,_,_=adder(64,physical,(int(d['offset'])&MASK)^MASK,True)
 key=physical;region=d['region']
 if less_word(64,relative,m):key,_,_=adder(64,int(s['offset'])&MASK,relative,False);region=s['region']
 modeled=value(region,key&(access.padding(r['profile'])-1))
 before=[[value(bool(rg),j) for j in range(n)] for rg in range(2)];after=[a[:] for a in before];naive=[a[:] for a in before]
 copied=dm['constructed'] and sm['constructed'];actual_count=min(int(d['length']),int(s['length'])) if copied else 0
 if copied:
  # The reference always reads the immutable pre-state; loop direction is irrelevant.
  for j in range(actual_count):after[int(d['region'])][int(d['offset'])+j]=before[int(s['region'])][int(s['offset'])+j]
  for j in range(actual_count):naive[int(d['region'])][int(d['offset'])+j]=naive[int(s['region'])][int(s['offset'])+j]
 def read(h,v,meta):return h[int(v['region'])][int(v['offset'])+i] if meta['read_defined'] else ''
 return dict(destination_valid=dm['valid'],source_valid=sm['valid'],count=str(m),count_nonnegative=access.signed(m)>=0,count_bounds=not less_word(64,dl,m) and not less_word(64,sl,m),copied_guard=i>=0 and less_word(64,iu,m),
  destination_guard=dm['guard'],source_guard=sm['guard'],physical=str(physical),physical_carry=carry,relative=str(relative),covered=physical==physical&(access.padding(r['profile'])-1),storage_bound=not less_word(64,r['profile'],n&MASK),modeled_value=modeled,
  destination_constructed=dm['constructed'],source_constructed=sm['constructed'],destination_construction_panicked=dm['construction_panicked'],source_construction_panicked=sm['construction_panicked'],copied=copied,actual_count=str(actual_count) if copied else '',metadata_preserved=copied,
  destination_read=dm['read_defined'],source_read=sm['read_defined'],destination_read_panicked=dm['read_panicked'],source_read_panicked=sm['read_panicked'],destination_before=read(before,d,dm),destination_after=read(after,d,dm),source_before=read(before,s,sm),source_after=read(after,s,sm),naive_destination=read(naive,d,dm) if copied else '',before=before,after=after)

def compare(cases,outputs):
 if cases!=inputs() or len(cases)!=len(outputs):raise ValueError('copy observation input coverage')
 keys=('source_operations','proof_domain_operations','copied_reads','outside_destination_reads','other_region_cells','right_overlap','left_overlap','different_regions','unequal_lengths','nil_operations','nonnil_empty_operations','construction_rejections','index_panics','invalid_binding_constructible','naive_forward_counterexamples','poststate_counterexamples','out_of_profile_reads','padding_not_admission','uncovered_reads')
 counts=dict.fromkeys(keys,0)
 for r,a in zip(cases,outputs):
  expected=model(r)
  if not isinstance(a,dict) or set(a)!=set(expected) or any(type(a[k]) is not type(v) for k,v in expected.items()):raise ValueError('copy observation field/type mismatch')
  if a!=expected:raise ValueError('Go/bit-loop snapshot copy mismatch')
  d,s=r['destination'],r['source'];n=int(r['storage']);i=int(r['index']);m=int(a['count']);do,so=int(d['offset']),int(s['offset'])
  valid=a['destination_valid'] and a['source_valid'];domain=valid and a['storage_bound'];read=domain and a['copied_guard']
  if valid and not (a['count_nonnegative'] and a['count_bounds'] and a['copied'] and a['actual_count']==a['count']):raise ValueError('valid computed copy count failed')
  if read and not (a['destination_guard'] and a['source_guard'] and a['destination_read'] and a['source_read'] and a['destination_after']==a['source_before']==a['modeled_value'] and a['covered'] and not a['physical_carry']):raise ValueError('bounded snapshot copied read failed')
  if domain and a['destination_read'] and not a['copied_guard'] and a['destination_before']!=a['destination_after']:raise ValueError('copy destination frame failed')
  if domain:
   for rg in range(2):
    for j in range(n):
     inside=rg==int(d['region']) and do<=j<do+m
     if not inside and a['before'][rg][j]!=a['after'][rg][j]:raise ValueError('copy whole heap frame failed')
  if a['relative']!=str(i&MASK):raise ValueError('copy full64 shift cancellation failed')
  overlap=d['region']==s['region'] and max(do,so)<min(do+m,so+m) and a['copied']
  counts['source_operations']+=a['copied'];counts['proof_domain_operations']+=domain;counts['copied_reads']+=read
  counts['outside_destination_reads']+=domain and a['destination_read'] and not a['copied_guard'];counts['other_region_cells']+=n if domain else 0
  counts['right_overlap']+=overlap and do>so;counts['left_overlap']+=overlap and do<so;counts['different_regions']+=a['copied'] and d['region']!=s['region'] and m>0
  counts['unequal_lengths']+=a['copied'] and d['length']!=s['length'];counts['nil_operations']+=a['copied'] and (d['nil'] or s['nil'])
  counts['nonnil_empty_operations']+=a['copied'] and not d['nil'] and d['length']=='0';counts['construction_rejections']+=not a['destination_constructed'] or not a['source_constructed'];counts['index_panics']+=a['destination_read_panicked'] or a['source_read_panicked']
  counts['invalid_binding_constructible']+=not valid and a['copied'];counts['naive_forward_counterexamples']+=a['destination_read'] and a['copied'] and a['destination_after']!=a['naive_destination']
  counts['poststate_counterexamples']+=read and a['destination_after']!=a['source_after'];counts['out_of_profile_reads']+=valid and not a['storage_bound'] and a['copied_guard'];counts['padding_not_admission']+=a['copied_guard'] and r['profile']<n<=access.padding(r['profile']);counts['uncovered_reads']+=a['destination_read'] and not a['covered']
 if not all(counts.values()):raise ValueError('lost copy nonvacuity/boundary observations')
 return counts

def negative_witnesses(cases,outputs):
 out=[]
 for b in CAPACITIES:
  for f in families(b):
   if f=='count':wanted=row(b,0,view(nil=True),view(nil=True));left,right=False,True
   elif f=='destination-guard':wanted=row(b,1,view(0,0,1),view(0,1,1));left,right=False,True
   elif f in ('source-guard','frame'):wanted=row(b,1,view(0,1,1),view(0,0,1));left,right=(False,True) if f=='source-guard' else (None,ZERO)
   elif f=='other-frame':wanted=row(b,1,view(0,0,1),view(0,0,1));left,right=None,ZERO
   elif f=='snapshot':wanted=row(b,3,view(1,2,2),view(0,2,3),1);left,right=None,None
   else:wanted=row(b,1,view(0,1,1),view(0,1,1));left,right=None,ZERO
   pos=cases.index(wanted);a=outputs[pos]
   if not (a['destination_valid'] and a['source_valid'] and a['storage_bound'] and a['copied']):raise ValueError('copy witness outside bounded source')
   if f=='count':left=a['count_nonnegative'] and 1<=int(wanted['destination']['length']) and 1<=int(wanted['source']['length'])
   elif f in ('destination-guard','source-guard'):left=a[f.replace('-','_')]
   elif f=='other-frame':left=a['after'][1][0]
   else:left=a['destination_after']
   if f=='snapshot':right=a['source_after']
   if left==right:raise ValueError('no copy false-claim witness')
   out.append(dict(case_id=f'copy-{b}-{f}-{label(f)}',input_position=pos,domain='bounded_source',left=left,right=right))
 return out

def run(executable,directory):
 cases=inputs();p=directory/ARTIFACTS[0];p.write_bytes(json_bytes(cases));m=timed([executable,p])
 if m['exit_code']:raise RuntimeError('actual Go copy observer failed')
 raw=m.pop('stdout').encode();(directory/ARTIFACTS[1]).write_bytes(raw);outputs=json.loads(raw)
 return dict(scope=SCOPE,cases=len(cases),outcomes=compare(cases,outputs),witness_scope=WITNESS_SCOPE,false_claim_witnesses=negative_witnesses(cases,outputs),input_sha256=sha(p.read_bytes()),output_sha256=sha(raw),go_binary_sha256=sha(executable.read_bytes()),go_measurement=m)

def check(report,directory):
 a,b=((directory/name).read_bytes() for name in ARTIFACTS)
 if report['scope']!=SCOPE or report['input_sha256']!=sha(a) or report['output_sha256']!=sha(b):raise ValueError('copy observation scope/byte identity')
 cases,outputs=json.loads(a),json.loads(b)
 if a!=json_bytes(inputs()) or report['cases']!=len(cases) or report['outcomes']!=compare(cases,outputs):raise ValueError('copy observation outcome coverage')
 if report['witness_scope']!=WITNESS_SCOPE or json_bytes(report['false_claim_witnesses'])!=json_bytes(negative_witnesses(cases,outputs)):raise ValueError('missing, mistyped or misclassified copy witnesses')
 return len(cases)
