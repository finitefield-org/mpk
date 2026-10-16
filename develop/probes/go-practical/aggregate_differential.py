"""Original structs/slice views and strings versus an independent bit/byte model."""
import json
import random
from pathlib import Path
from affix_differential import add,equal_word,read,signed,sub
from run_proof_pilot import json_bytes,sha,timed

POINTS=((0,0),(1,1),(32,8),(31,31),(32,32),(33,33),(255,127),(256,128),(257,129))
ARTIFACTS=('aggregate-input.json','aggregate-output.json')
MASK=(1<<64)-1
SCOPE='original Go value structs, actual slice views and strings versus independent full64 and byte loops; private feasibility, no production source-to-certificate claim'
WITNESS_SCOPE='bounded original-Go field/frame and matched-byte witnesses; profile bounds, physical coverage and source panics remain distinct'
def depth(n):return max(n-1,0).bit_length()
def padding(b,s):return 1<<depth(max(b,s))
def families(b,s):return ['layout-frame']+(['full-cell','amount','sku-length'] if s else [])+(['prefix-read','suffix-read'] if b and s else [])
def identifier(b,s,f,m):return f"aggregate-{b}-{s}-{f}-{'altered' if m else 'valid'}"
def inputs():
 rng=random.Random(0x4147475245474154);rows=[]
 for b,s in POINTS:
  def row(value=b'',pattern=b'',i=0,j=0,offset=0,*,nil=False):
   cells=[{'amount':str(signed(rng.getrandbits(64))),'sku':bytes([n&255]*min(b,3)).hex()} for n in range(s)]
   if cells:cells[min(offset+max(i,0),s-1)]={'amount':str(-(1<<63)),'sku':value.hex()}
   return {'byte_bound':b,'slice_bound':s,'padding':padding(b,s),'backing':cells,'frame':{'amount':'9223372036854775807','sku':value.hex()},'replacement':'-9223372036854775808','nil':nil,'offset':str(offset),'length':str(s-offset),'capacity':str(s-offset),'index':str(i),'pattern':pattern.hex(),'byte_index':str(j)}
  if not s:
   for nil in (False,True):
    for i in (0,-1, -(1<<63)):rows.append(row(i=i,nil=nil))
  else:
   values=list(dict.fromkeys([b'',b'\x80'[:b],b'\x00'[:b],b'\xff'[:b],b'\xc3('[:b],b'\xe3\x81\x82'[:b],b'a\x00x'[:b],bytes(rng.randrange(256) for _ in range(b))]))
   for value in values:
    patterns=list(dict.fromkeys([b'',value,value[:1],value[-1:],value+b'\x00',b'\x00'[:b]]))
    for pattern in patterns:
     for j in dict.fromkeys((0,len(pattern)-1,len(pattern))):rows.append(row(value,pattern,j=j,offset=int(s>1)))
   for i in (-1,s,s-1,-(1<<63),(1<<63)-1):rows.append(row(b'\x80',b'\x80',i))
   rows.append(row(b'\x00'*(b+1),b'\x00'*b))
  # Actual source beyond finite padding: low-address projection aliases cell 0.
  outside=row();outside['backing']=[{'amount':str(n),'sku':''} for n in range(padding(b,s)+1)]
  outside.update(offset='0',length=str(len(outside['backing'])),capacity=str(len(outside['backing'])),index=str(padding(b,s)))
  rows.append(outside)
  bad=row();bad['offset']='9223372036854775807';rows.append(bad)
 return rows
def relation(source,pattern,suffix):
 if len(pattern)>len(source):return False
 start=len(source)-len(pattern) if suffix else 0
 return all(equal_word(8,source[start+j],pattern[j]) for j in range(len(pattern)))
def model(r):
 cells=r['backing'];o,l,k,i,j=[int(r[n]) for n in ('offset','length','capacity','index','byte_index')]
 valid=(l>=0 and k>=l and o>=0 and o<=len(cells) and k<=len(cells)-o) if not r['nil'] else l==0 and k==0
 pattern=bytes.fromhex(r['pattern']);frame=bytes.fromhex(r['frame']['sku']);absolute=add(o,i)
 profile=valid and len(cells)<=r['slice_bound'] and len(pattern)<=r['byte_bound'] and len(frame)<=r['byte_bound'] and all(len(bytes.fromhex(c['sku']))<=r['byte_bound'] for c in cells)
 defined=valid and not r['nil'] and i>=0 and i<l
 a=dict(descriptor_valid=valid,profile_input=profile,line_read=defined,line_panicked=valid and not defined,absolute=str(absolute),finite_absolute=str(absolute&(r['padding']-1)),amount='0',sku='',source_length=0,prefix=False,suffix=False,pattern_read=False,pattern_byte=-1,prefix_read=False,prefix_byte=-1,suffix_read=False,suffix_byte=-1,frame_sku_preserved=True,frame_amount=r['replacement'],input_unchanged=True)
 if defined:
  cell=cells[absolute];value=bytes.fromhex(cell['sku']);a.update(amount=cell['amount'],sku=cell['sku'],source_length=len(value),prefix=relation(value,pattern,False),suffix=relation(value,pattern,True))
  pr,pv=read(pattern,j);sr,sv=read(value,j);tr,tv=read(value,signed(add(sub(len(value),len(pattern)),j)))
  a.update(pattern_read=pr,pattern_byte=pv,prefix_read=sr,prefix_byte=sv,suffix_read=tr,suffix_byte=tv)
 return a
def compare(cases,outputs):
 if cases!=inputs() or len(cases)!=len(outputs):raise ValueError('aggregate input coverage')
 keys=('profile_inputs','line_reads','source_index_panics','invalid_descriptors','outside_bound','projection_counterexamples','matched_prefix_reads','matched_suffix_reads','negative_amounts','nul_inputs','invalid_utf8_inputs','empty_views','nil_views','nonzero_offsets')
 counts=dict.fromkeys(keys,0)
 for r,a in zip(cases,outputs):
  expected=model(r)
  if not isinstance(a,dict) or set(a)!=set(expected) or any(type(a[k]) is not type(v) for k,v in expected.items()) or a!=expected:raise ValueError('Go/aggregate bit-loop observation mismatch')
  for kind in ('prefix','suffix'):
   if a[kind] and a['pattern_read'] and (not a[kind+'_read'] or a[kind+'_byte']!=a['pattern_byte']):raise ValueError('aggregate matched full-byte law')
   counts['matched_'+kind+'_reads']+=a[kind] and a['pattern_read']
  if a['line_read']:
   actual=r['backing'][int(a['absolute'])];projected=r['backing'][int(a['finite_absolute'])]
   if a['profile_input'] and actual!=projected:raise ValueError('bounded aggregate projection changed a whole cell')
   counts['projection_counterexamples']+=actual!=projected
  value=bytes.fromhex(a['sku'])
  counts['profile_inputs']+=a['profile_input'];counts['line_reads']+=a['line_read'];counts['source_index_panics']+=a['line_panicked'];counts['invalid_descriptors']+=not a['descriptor_valid'];counts['outside_bound']+=not a['profile_input'];counts['negative_amounts']+=int(a['amount'])<0;counts['nul_inputs']+=0 in value;counts['empty_views']+=int(r['length'])==0;counts['nil_views']+=r['nil'];counts['nonzero_offsets']+=int(r['offset'])>0 and a['descriptor_valid']
  try:value.decode('utf8')
  except UnicodeDecodeError:counts['invalid_utf8_inputs']+=1
 if not all(counts.values()):raise ValueError('aggregate nonvacuity coverage '+repr(counts))
 return counts
def negative_witnesses(cases,outputs):
 witnesses=[]
 for b,s in POINTS:
  for f in families(b,s):
   choices=[(i,r,a) for i,(r,a) in enumerate(zip(cases,outputs)) if (r['byte_bound'],r['slice_bound'])==(b,s) and a['profile_input'] and (f=='layout-frame' or a['line_read']) and (not f.endswith('-read') or a[f.removesuffix('-read')] and a['pattern_read'])]
   if not choices:raise ValueError('aggregate false claim lacks source-domain witness')
   i,r,a=choices[0]
   if f=='layout-frame':actual=len(bytes.fromhex(r['frame']['sku']));wrong=actual^1
   elif f=='sku-length':actual=a['source_length'];wrong=actual^1
   elif f in ('amount','full-cell'):actual=int(a['amount'])&MASK;wrong=actual^1
   else:actual=True;wrong=False
   if actual==wrong:raise ValueError('vacuous aggregate false witness')
   witnesses.append({'case_id':identifier(b,s,f,1),'input_index':i,'input_sha256':sha(json_bytes(r)),'actual':actual,'altered':wrong})
 return witnesses
def run(executable,directory):
 cases=inputs();raw=json_bytes(cases);input_path=directory/ARTIFACTS[0];input_path.write_bytes(raw)
 measurement=timed([executable,input_path])
 if measurement['exit_code']!=0:raise ValueError('aggregate native observer failed')
 output=measurement.pop('stdout').encode();outputs=json.loads(output);(directory/ARTIFACTS[1]).write_bytes(output)
 return {'scope':SCOPE,'cases':len(cases),'counts':compare(cases,outputs),'witness_scope':WITNESS_SCOPE,'false_claim_witnesses':negative_witnesses(cases,outputs),'input_sha256':sha(raw),'output_sha256':sha(output),'go_binary_sha256':sha(Path(executable).read_bytes()),'go_measurement':measurement}
def check(record,directory):
 raw=(directory/ARTIFACTS[0]).read_bytes();out=(directory/ARTIFACTS[1]).read_bytes();cases=json.loads(raw);outputs=json.loads(out)
 if record['scope']!=SCOPE or record['witness_scope']!=WITNESS_SCOPE or record['cases']!=len(cases) or record['input_sha256']!=sha(raw) or record['output_sha256']!=sha(out):raise ValueError('aggregate native scope/byte identity')
 if record['counts']!=compare(cases,outputs) or record['false_claim_witnesses']!=negative_witnesses(cases,outputs):raise ValueError('aggregate native witnesses/counts')
 return len(cases)
