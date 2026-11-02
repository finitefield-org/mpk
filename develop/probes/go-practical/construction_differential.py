"""Native allocation/append against independent bit loops and source snapshots."""
import json
import access_differential as access
import copy_differential as copied
from byte_differential import less_word
from order_differential import adder
from run_proof_pilot import json_bytes,sha,timed
from slice_differential import ZERO

CAPACITIES=(0,1,8,32,127,128,129)
ARTIFACTS=('construction-input.json','construction-output.json')
MASK=(1<<64)-1
ONES='ffffffffffffffff:ffffffffffffffff'
SCOPE='actual make/append, independent full64 bit loops, initialized capacity and immutable source snapshots; no runtime growth-capacity policy, production capture, borrowed-write permission or process OOM theorem'
WITNESS_SCOPE='bounded-source, source-make-panic, raw-invalid-fresh-binding and source-reallocates-not-panics witnesses remain distinct; no resource failure is false-claim evidence'

def families(n):
 a=[('make','validity'),('make','nil'),('append','validity'),('append','length'),('append','nil'),('composed','validity'),('composed','nil')]
 if n:a.extend([('make','zero'),('make','other-frame'),('make','fresh-frame'),('append','capacity'),('append','index'),('append','alias-index'),('append','read'),('append','prefix-frame'),('append','alias-read'),('append','tail-frame'),('append','other-frame'),('append','physical-frame'),('composed','read'),('composed','borrowed-frame')])
 if n>=8:a.append(('append','snapshot'))
 return a

def label(n,kind,f):
 return {('make','validity'):'erased-make-guard',('make','nil'):'nil-result',('make','zero'):'nonzero-initialization',('make','fresh-frame'):'erased-freshness',('append','validity'):'false-empty-validity' if not n else 'erased-capacity-guard',('append','length'):'extra-element',('append','nil'):'flipped-nil',('append','capacity'):'zero-capacity',('append','index'):'original-length-index',('append','alias-index'):'original-length-index',('composed','validity'):'false-empty-validity' if not n else 'erased-capacity-guard',('composed','nil'):'nil-result',('append','snapshot'):'poststate-source',('append','alias-read'):'stale-alias'}.get((kind,f),'zeroed-cell')

def make_row(b,n,l,k,i=0,*,r=False,q=True,fresh=True):
 live=[True,True];live[int(r)]=not fresh
 return dict(kind='make',profile=b,storage=str(n),length=str(l),capacity=str(k),fresh_region=r,query_region=q,live=live,index=str(i),destination=copied.view(),source=copied.view())
def append_row(b,n,d,s,i=0,*,q=False,live=(True,True)):
 return dict(kind='append',profile=b,storage=str(n),length='0',capacity='0',fresh_region=False,query_region=q,live=list(live),index=str(i),destination=d,source=s)

def composed_row(b,k,l=0,*,i=0,r=False,q=True,source=None,fresh=True):
 row=make_row(b,k,l,k,i,r=r,q=q,fresh=fresh);row['kind']='composed';row['source']=copied.view(0,k,k,region=not r) if source is None else source;return row

def inputs():
 out=[]
 for b in CAPACITIES:
  for l,k in dict.fromkeys(((0,0),(0,min(b,1)),(min(b,1),min(b,1)),(0,b),(b//2,b),(b,b),(max(b-1,0),b),(b+1,b),(0,b+1),(0,access.padding(b)+1))):
   for i in dict.fromkeys((-1,0,l-1,l,k-1,k)):
    for r in (False,True):
     for q in (False,True):out.append(make_row(b,b,l,k,i,r=r,q=q))
  for l,k in ((-1,0),(-(1<<63),0),((1<<63)-1,0),(0,-1),(0,-(1<<63)),(-1,(1<<63)-1)):
   out.append(make_row(b,b,l,k))
  out.append(make_row(b,0,0,0))
  if b:out.extend([make_row(b,1,0,1,q=False),make_row(b,1,0,0),make_row(b,1,0,1,q=False,fresh=False)])
 # Reuse raw/source boundaries but retain append's distinct output/guard semantics.
 for old in copied.inputs():
  for q in (False,True):out.append(append_row(old['profile'],int(old['storage']),old['destination'],old['source'],int(old['index']),q=q,live=old['live']))
 for b in CAPACITIES:
  for n in dict.fromkeys((b+1,access.padding(b)+1)):
   out.append(append_row(b,n,copied.view(0,0,n),copied.view(0,n,n,region=True),n-1))
  out.append(append_row(b,0,copied.view(nil=True),copied.view(nil=True)))
  if b:
   out.extend([append_row(b,1,copied.view(),copied.view(0,1,1,region=True)),append_row(b,1,copied.view(0,0,1),copied.view(0,0,1,region=True)),
    append_row(b,1,copied.view(0,0,1),copied.view(0,1,1,region=True)),append_row(b,1,copied.view(0,1,1),copied.view(0,0,1,region=True)),
    append_row(b,1,copied.view(0,0,1),copied.view(0,0,1,region=True),q=True)])
  if b>=8:
   for i in (-1,0,1,2,3):
    for r in (False,True):
     out.extend([append_row(b,3,copied.view(0,1,3,region=r),copied.view(0,2,3,region=r),i,q=r),append_row(b,4,copied.view(1,1,3,region=r),copied.view(1,2,3,region=r),i,q=r),
      append_row(b,4,copied.view(0,0,4,region=r),copied.view(1,3,3,region=r),i,q=r),append_row(b,4,copied.view(1,0,3,region=r),copied.view(0,3,4,region=r),i,q=r)])
 for b in CAPACITIES:
  for k in dict.fromkeys((0,min(b,1),b,b+1,access.padding(b)+1)):
   for l in dict.fromkeys((0,min(k,1),k)):
    for length in dict.fromkeys((0,min(k,1),k)):
     for i in dict.fromkeys((-1,0,length-1,length)):
      for r in (False,True):out.append(composed_row(b,k,l,i=i,r=r,q=not r,source=copied.view(0,length,k,region=not r)))
  out.extend([composed_row(b,0,-1,source=copied.view(nil=True)),composed_row(b,0,source=copied.view(nil=True))])
  if b:out.extend([composed_row(b,1,q=False),composed_row(b,1,source=copied.view(0,1,1),q=False),composed_row(b,1,fresh=False,q=False)])
 return list({json.dumps(r,sort_keys=True):r for r in out}.values())

def make_model(r):
 n,l,k,i=(int(r[x]) for x in ('storage','length','capacity','index'));guard=0<=l<=k;fresh=not r['live'][int(r['fresh_region'])]
 key=i&MASK;modeled=copied.value(r['query_region'],key&(access.padding(r['profile'])-1))
 if r['query_region']==r['fresh_region'] and less_word(64,key,k&MASK):modeled=ZERO
 live=r['live'][:];live[int(r['fresh_region'])]=True
 meta=access.model(dict(profile=r['profile'],storage=str(k),offset='0',length=str(l),capacity=str(k),index=str(i),nil=False,region=r['fresh_region'],live=live))
 before=copied.value(r['query_region'],i) if 0<=i<n else ''
 read=guard and 0<=i<l;full=guard and 0<=i<k
 return dict(kind='make',guard=guard,fresh=fresh,model_valid=meta['valid'],storage_bound=not less_word(64,r['profile'],k&MASK),old_storage_bound=not less_word(64,r['profile'],n),query_covered=key==key&(access.padding(r['profile'])-1),query_live=r['live'][int(r['query_region'])],new_query_live=live[int(r['query_region'])],model_query_after=modeled,borrowed_before=before,borrowed_after=before,
  constructed=guard,make_panicked=not guard,actual_nil=False,actual_length=str(l) if guard else '',actual_capacity=str(k) if guard else '',actual_values=[ZERO]*l if guard else [],full_values=[ZERO]*k if guard else [],read_defined=read,read_panicked=guard and not read,full_read_defined=full,full_read_panicked=guard and not full,actual_value=ZERO if read else '',full_value=ZERO if full else '')

def append_model(r,actual_capacity):
 d,s=r['destination'],r['source'];n=int(r['storage']);i=int(r['index']);iu=i&MASK
 def metadata(v):return access.model(dict(v,profile=r['profile'],storage=r['storage'],index=r['index'],live=r['live']))
 dm,sm=metadata(d),metadata(s);dl,sl=int(d['length'])&MASK,int(s['length'])&MASK;do=int(d['offset'])&MASK
 total,lc,_=adder(64,dl,sl,False);start,sc,_=adder(64,do,dl,False);end,ec,_=adder(64,do,total,False);mapped,_,_=adder(64,dl,iu,False)
 nr=not lc and not less_word(64,int(d['capacity'])&MASK,total);new=metadata(dict(d,length=str(access.signed(total))))
 inside=r['query_region']==d['region'] and not less_word(64,iu,start) and less_word(64,iu,end)
 key=iu;region=r['query_region']
 if inside:
  j,_,_=adder(64,iu,start^MASK,True);key,_,_=adder(64,int(s['offset'])&MASK,j,False);region=s['region']
 model_value=copied.value(region,key&(access.padding(r['profile'])-1))
 before=[[copied.value(bool(rg),j) for j in range(n)] for rg in range(2)];after=[a[:] for a in before];naive=[a[:] for a in before]
 op=dm['constructed'] and sm['constructed'];capacity=0;values=[];full=[];probe=False;shares=False
 if op:
  if not isinstance(actual_capacity,str) or not actual_capacity.isdecimal():raise ValueError('append observed capacity type')
  capacity=int(actual_capacity);newlen=int(d['length'])+int(s['length'])
  if nr and capacity!=int(d['capacity']) or not nr and (capacity<newlen or capacity<=int(d['capacity'])):raise ValueError('append observed capacity violates specified bound; runtime growth policy stays unmodeled')
  old_prefix=before[int(d['region'])][int(d['offset']):int(d['offset'])+int(d['length'])]
  source=before[int(s['region'])][int(s['offset']):int(s['offset'])+int(s['length'])]
  values=old_prefix+source
  if nr:
   for j,x in enumerate(source):after[int(d['region'])][int(d['offset'])+int(d['length'])+j]=x
   for j in range(int(s['length'])):naive[int(d['region'])][int(d['offset'])+int(d['length'])+j]=naive[int(s['region'])][int(s['offset'])+j]
   full=after[int(d['region'])][int(d['offset']):int(d['offset'])+capacity]
  else:full=values+[ZERO]*(capacity-newlen)
  probe=int(d['capacity'])>0 and capacity>0;shares=probe and nr
 def read(h,v,meta):return h[int(v['region'])][int(v['offset'])+i] if meta['read_defined'] else ''
 fullmeta=metadata(dict(d,length=d['capacity']));fullmeta['read_defined']=dm['constructed'] and fullmeta['guard'];fullmeta['read_panicked']=dm['constructed'] and not fullmeta['guard'];mi=access.signed(mapped);result_read=op and 0<=i<len(values);added_read=op and 0<=mi<len(values)
 alias_read=op and 0<=mi<int(d['capacity']);naive_at=naive[int(d['region'])][int(d['offset'])+mi] if op and nr and 0<=mi<int(d['capacity']) else ''
 return dict(kind='append',destination_valid=dm['valid'],source_valid=sm['valid'],storage_bound=not less_word(64,r['profile'],n),total=str(total),length_carry=lc,start=str(start),start_carry=sc,end=str(end),end_carry=ec,no_reallocation=nr,model_valid=new['valid'],inside_write=inside,model_query_after=model_value,
  destination_constructed=dm['constructed'],source_constructed=sm['constructed'],destination_construction_panicked=dm['construction_panicked'],source_construction_panicked=sm['construction_panicked'],appended=op,actual_nil=op and d['nil'] and int(s['length'])==0,actual_length=str(len(values)) if op else '',actual_capacity=actual_capacity if op else '',values=values,full_values=full,old_metadata_preserved=op,probe_observed=probe,shares_old=shares,
  destination_read=dm['read_defined'],destination_read_panicked=dm['read_panicked'],source_read=sm['read_defined'],source_read_panicked=sm['read_panicked'],full_read=fullmeta['read_defined'],full_read_panicked=fullmeta['read_panicked'],result_read=result_read,result_read_panicked=op and not result_read,appended_read=added_read,appended_read_panicked=op and not added_read,
  destination_before=read(before,d,dm),destination_after=read(after,d,dm),full_before=read(before,dict(d,length=d['capacity']),fullmeta),full_after=read(after,dict(d,length=d['capacity']),fullmeta),source_before=read(before,s,sm),source_after=read(after,s,sm),result_at=values[i] if result_read else '',appended_at=values[mi] if added_read else '',alias_at=after[int(d['region'])][int(d['offset'])+mi] if alias_read else '',naive_at=naive_at,mapped=str(mapped),before=before,after=after)

def composed_model(r,actual_capacity):
 k,l,i=(int(r[x]) for x in ('capacity','length','index'));key=i&MASK;s=r['source'];region=r['fresh_region'];guard=0<=l<=k
 sm=access.model(dict(s,profile=r['profile'],storage=str(k),index=str(i),live=r['live']))
 total,carry,_=adder(64,l&MASK,int(s['length'])&MASK,False);nr=not carry and not less_word(64,k&MASK,total);mapped,_,_=adder(64,l&MASK,key,False);mi=access.signed(mapped)
 live=r['live'][:];live[int(region)]=True
 vd=access.model(dict(profile=r['profile'],storage=str(k),offset='0',length=str(access.signed(total)),capacity=str(k),index=str(i),nil=False,region=region,live=live))
 model_value=copied.value(r['query_region'],key&(access.padding(r['profile'])-1))
 if r['query_region']==region and less_word(64,key,k&MASK):model_value=ZERO
 if r['query_region']==region and not less_word(64,key,l&MASK) and less_word(64,key,total):
  delta,_,_=adder(64,key,(l&MASK)^MASK,True);source_key,_,_=adder(64,int(s['offset'])&MASK,delta,False)
  model_value=ZERO if s['region']==region and less_word(64,source_key,k&MASK) else copied.value(s['region'],source_key&(access.padding(r['profile'])-1))
 before=[[copied.value(bool(rg),j) for j in range(k)] for rg in range(2)];op=guard and sm['constructed'];values=[];full=[];probe=False;shares=False
 if op:
  if not isinstance(actual_capacity,str) or not actual_capacity.isdecimal():raise ValueError('composed observed capacity type')
  cap=int(actual_capacity);values=[ZERO]*l+before[int(s['region'])][int(s['offset']):int(s['offset'])+int(s['length'])]
  if nr and cap!=k or not nr and (cap<len(values) or cap<=k):raise ValueError('composed observed capacity violates specified bound; runtime growth policy stays unmodeled')
  full=values+[ZERO]*(cap-len(values));probe=k>0 and cap>0;shares=probe and nr
 source_value=before[int(s['region'])][int(s['offset'])+i] if sm['read_defined'] else '';borrowed=before[int(r['query_region'])][i] if 0<=i<k else ''
 read=op and 0<=mi<len(values)
 return dict(kind='composed',guard=guard,fresh=not r['live'][int(region)],source_valid=sm['valid'],source_constructed=sm['constructed'],source_construction_panicked=sm['construction_panicked'],storage_bound=not less_word(64,r['profile'],k),query_live=r['live'][int(r['query_region'])],model_valid=vd['valid'],model_query_after=model_value,made=guard,make_panicked=not guard,appended=op,total=str(total),length_carry=carry,no_reallocation=nr,actual_nil=False,actual_length=str(len(values)) if op else '',actual_capacity=actual_capacity if op else '',values=values,full_values=full,source_read=sm['read_defined'],source_read_panicked=sm['read_panicked'],source_before=source_value,source_after=source_value,mapped=str(mapped),result_read=read,result_read_panicked=op and not read,result_value=values[mi] if read else '',probe_observed=probe,shares_preallocation=shares,borrowed_before=borrowed,borrowed_after=borrowed,before=before,after=before)

def compare(cases,outputs):
 if cases!=inputs() or len(cases)!=len(outputs):raise ValueError('construction observation input coverage')
 keys=('make_operations','make_panics','make_zero_capacity','make_tail_reads','make_live_frames','make_invalid_fresh_bindings','make_outside_bound_reads','make_uncovered_zero_reads','append_operations','append_no_reallocation','append_reallocates_without_panic','append_nil_results','append_empty_nonnil','append_copied_reads','append_alias_reads','append_prefix_reads','append_tail_reads','append_physical_frames','append_right_overlap','append_left_overlap','append_poststate_counterexamples','append_naive_counterexamples','append_invalid_binding_constructible','append_outside_bound_reads','construction_rejections','composed_operations','composed_reads','composed_borrowed_frames','composed_non_nil_empty','composed_reallocates_without_panic','composed_invalid_binding')
 counts=dict.fromkeys(keys,0)
 for r,a in zip(cases,outputs):
  if not isinstance(a,dict):raise ValueError('construction observation field/type mismatch')
  expected=make_model(r) if r['kind']=='make' else (composed_model(r,a.get('actual_capacity')) if r['kind']=='composed' else append_model(r,a.get('actual_capacity')))
  if not isinstance(a,dict) or set(a)!=set(expected) or any(type(a[k]) is not type(v) for k,v in expected.items()):raise ValueError('construction observation field/type mismatch')
  if a!=expected:raise ValueError('Go/bit-loop construction mismatch')
  i=int(r['index']);n=int(r['storage'])
  if r['kind']=='composed':
   domain=a['guard'] and a['fresh'] and a['source_valid'] and a['storage_bound'] and a['no_reallocation']
   if domain and (not a['appended'] or not a['model_valid'] or a['actual_nil']):raise ValueError('composed validity failed')
   if domain and a['source_read'] and not (a['result_read'] and a['result_value']==a['source_before']==a['source_after']):raise ValueError('composed pre-state source read failed')
   if domain and a['query_live'] and a['borrowed_before'] and a['model_query_after']!=a['borrowed_before']:raise ValueError('composed borrowed frame failed')
   counts['composed_operations']+=a['appended'];counts['composed_reads']+=domain and a['source_read'];counts['composed_borrowed_frames']+=domain and a['query_live'] and bool(a['borrowed_before']);counts['composed_non_nil_empty']+=a['appended'] and a['actual_length']=='0' and not a['actual_nil'];counts['composed_reallocates_without_panic']+=a['appended'] and not a['no_reallocation'];counts['composed_invalid_binding']+=a['appended'] and (not a['fresh'] or not a['source_valid'])
   continue
  if r['kind']=='make':
   if a['guard'] and (not a['model_valid'] or not a['constructed'] or a['actual_nil']):raise ValueError('make validity/nil law failed')
   if a['fresh'] and a['query_live'] and a['borrowed_before'] and a['storage_bound'] and a['old_storage_bound'] and a['model_query_after']!=a['borrowed_before']:raise ValueError('computed fresh-region separation failed')
   counts['make_operations']+=a['constructed'];counts['make_panics']+=a['make_panicked'];counts['make_zero_capacity']+=a['constructed'] and a['actual_capacity']=='0';counts['make_tail_reads']+=a['full_read_defined'] and not a['read_defined'];counts['make_live_frames']+=a['constructed'] and a['fresh'] and a['query_live'] and bool(a['borrowed_before']);counts['make_invalid_fresh_bindings']+=a['constructed'] and not a['fresh'] and a['query_live'] and a['model_query_after']!=a['borrowed_before'];counts['make_outside_bound_reads']+=a['full_read_defined'] and not a['storage_bound'];counts['make_uncovered_zero_reads']+=a['full_read_defined'] and not a['query_covered'] and a['full_value']==ZERO
   continue
  d,s=r['destination'],r['source'];valid=a['destination_valid'] and a['source_valid'];domain=valid and a['no_reallocation'] and a['storage_bound'];m=int(a['total']);mapped=int(a['mapped'])
  if valid and a['no_reallocation'] and not a['model_valid']:raise ValueError('computed append validity failed')
  copied=domain and a['source_read']
  if copied and not (a['appended_read'] and a['appended_at']==a['source_before']==a['alias_at']):raise ValueError('append source/result/alias index or pre-state read failed')
  if domain:
   for rg in range(2):
    for key in range(n):
     inside=rg==int(d['region']) and int(a['start'])<=key<int(a['end'])
     if not inside and a['before'][rg][key]!=a['after'][rg][key]:raise ValueError('append complete physical frame failed')
   if 0<=i<n and r['live'][int(r['query_region'])] and not a['inside_write'] and a['model_query_after']!=a['before'][int(r['query_region'])][i]:raise ValueError('append query frame failed')
  overlap=a['appended'] and a['no_reallocation'] and d['region']==s['region'] and max(int(a['start']),int(s['offset']))<min(int(a['end']),int(s['offset'])+int(s['length']))
  counts['append_operations']+=a['appended'];counts['append_no_reallocation']+=domain;counts['append_reallocates_without_panic']+=a['appended'] and not a['no_reallocation'];counts['append_nil_results']+=a['appended'] and a['actual_nil'];counts['append_empty_nonnil']+=a['appended'] and not a['actual_nil'] and a['actual_length']=='0';counts['append_copied_reads']+=copied;counts['append_alias_reads']+=copied and bool(a['alias_at']);counts['append_prefix_reads']+=domain and a['destination_read'];counts['append_tail_reads']+=domain and a['full_read'] and i>=m;counts['append_physical_frames']+=domain and 0<=i<n and not a['inside_write'];counts['append_right_overlap']+=overlap and int(a['start'])>int(s['offset']);counts['append_left_overlap']+=overlap and int(a['start'])<int(s['offset']);counts['append_poststate_counterexamples']+=copied and a['appended_at']!=a['source_after'];counts['append_naive_counterexamples']+=copied and a['appended_at']!=a['naive_at'];counts['append_invalid_binding_constructible']+=not valid and a['appended'];counts['append_outside_bound_reads']+=valid and a['no_reallocation'] and not a['storage_bound'] and a['source_read'];counts['construction_rejections']+=not a['destination_constructed'] or not a['source_constructed']
 if not all(counts.values()):raise ValueError('lost construction nonvacuity/boundary observations: '+repr([k for k,v in counts.items() if not v]))
 return counts

def negative_witnesses(cases,outputs):
 out=[]
 for b in CAPACITIES:
  for kind,f in families(b):
   domain='bounded_source'
   if kind=='composed':
    if f=='validity' and b:r=composed_row(b,1,1);domain='source_reallocates_not_panics'
    elif f in ('validity','nil'):r=composed_row(b,0,source=copied.view(nil=True))
    else:r=composed_row(b,1)
   elif kind=='make':
    if f=='validity':r=make_row(b,b,-1,0);domain='source_make_panic'
    elif f=='nil':r=make_row(b,0,0,0)
    elif f=='zero':r=make_row(b,1,0,1,q=False)
    elif f=='fresh-frame':r=make_row(b,1,0,1,q=False,fresh=False);domain='raw_invalid_fresh_binding'
    else:r=make_row(b,1,0,0)
   elif f in ('validity','length','nil'):
    if f=='validity' and b:r=append_row(b,1,copied.view(),copied.view(0,1,1,region=True));domain='source_reallocates_not_panics'
    else:r=append_row(b,0,copied.view(nil=True),copied.view(nil=True))
   elif f in ('capacity','tail-frame','physical-frame'):r=append_row(b,1,copied.view(0,0,1),copied.view(0,0,1,region=True))
   elif f=='prefix-frame':r=append_row(b,1,copied.view(0,1,1),copied.view(0,0,1,region=True))
   elif f=='other-frame':r=append_row(b,1,copied.view(0,0,1),copied.view(0,0,1,region=True),q=True)
   elif f=='snapshot':r=append_row(b,3,copied.view(0,1,3),copied.view(0,2,3),1)
   else:r=append_row(b,1,copied.view(0,0,1),copied.view(0,1,1,region=True))
   pos=cases.index(r);a=outputs[pos]
   if kind=='composed':
    if f=='validity':left,right=a['model_valid'],bool(b)
    elif f=='nil':left,right=a['actual_nil'],True
    elif f=='read':left,right=a['result_value'],ZERO
    else:left,right=a['model_query_after'],ZERO
   elif kind=='make':
    if f=='validity':left,right=a['model_valid'],True
    elif f=='nil':left,right=a['actual_nil'],True
    elif f=='zero':left,right=a['full_value'],ONES
    elif f=='fresh-frame':left,right=a['model_query_after'],a['borrowed_before']
    else:left,right=a['borrowed_after'],ZERO
   else:
    if f=='validity':left,right=a['model_valid'],bool(b)
    elif f=='length':left,right=a['actual_length'],str(int(a['total'])+1)
    elif f=='nil':left,right=a['actual_nil'],not r['destination']['nil']
    elif f=='capacity':left,right=a['actual_capacity'],'0'
    elif f in ('index','alias-index'):left,right=a['destination_read'],True
    elif f=='snapshot':left,right=a['appended_at'],a['source_after']
    elif f=='alias-read':left,right=a['alias_at'],a['full_before']
    elif f=='prefix-frame':left,right=a['destination_after'],ZERO
    elif f=='tail-frame':left,right=a['full_after'],ZERO
    elif f in ('other-frame','physical-frame'):left,right=a['after'][int(r['query_region'])][int(r['index'])],ZERO
    else:left,right=a['appended_at'],ZERO
   if left==right:raise ValueError('no construction false-claim witness '+repr((b,kind,f)))
   out.append(dict(case_id=f'construct-{b}-{kind}-{f}-{label(b,kind,f)}',input_position=pos,domain=domain,left=left,right=right))
 return out

def run(executable,directory):
 cases=inputs();p=directory/ARTIFACTS[0];p.write_bytes(json_bytes(cases));m=timed([executable,p])
 if m['exit_code']:raise RuntimeError('actual Go construction observer failed: '+m['stderr'])
 raw=m.pop('stdout').encode();(directory/ARTIFACTS[1]).write_bytes(raw);outputs=json.loads(raw)
 return dict(scope=SCOPE,cases=len(cases),outcomes=compare(cases,outputs),witness_scope=WITNESS_SCOPE,false_claim_witnesses=negative_witnesses(cases,outputs),input_sha256=sha(p.read_bytes()),output_sha256=sha(raw),go_binary_sha256=sha(executable.read_bytes()),go_measurement=m)

def check(report,directory):
 a,b=((directory/f).read_bytes() for f in ARTIFACTS)
 if report['scope']!=SCOPE or report['input_sha256']!=sha(a) or report['output_sha256']!=sha(b):raise ValueError('construction observation scope/byte identity')
 cases,outputs=json.loads(a),json.loads(b)
 if a!=json_bytes(inputs()) or report['cases']!=len(cases) or report['outcomes']!=compare(cases,outputs):raise ValueError('construction observation coverage')
 if report['witness_scope']!=WITNESS_SCOPE or json_bytes(report['false_claim_witnesses'])!=json_bytes(negative_witnesses(cases,outputs)):raise ValueError('missing, mistyped or misclassified construction witnesses')
 return len(cases)
