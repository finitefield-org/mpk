"""Rebind current fixture metadata against immutable original validity masks."""
from pathlib import Path
import base64,hashlib,json,sys,datetime
sys.path.insert(0,'develop/migrations/go-rust-only')
import migrate_fixtures as m
ROOT=Path.cwd();BASIS=m.ORIGINALS;RECORD=ROOT/'develop/migrations/go-rust-only/verification'/sys.argv[1]
RECORD.mkdir(parents=True,exist_ok=False)
m.DOMAINS.update({'mpk.rust.driver.request.v1':('request_fingerprint','MPK-RUST-DRIVER-REQUEST-1.0'),'mpk.rust.driver.v1':('payload_hash','MPK-RUST-DRIVER-PAYLOAD-1.0')})
original={};current={};raw_original={};before=[];valid=[];invalid=[];transports=[];lengths=[]
def nodes(node,path=()):
 if isinstance(node,dict):
  yield path,node
  for key,value in node.items():yield from nodes(value,(*path,key))
 elif isinstance(node,list):
  for i,value in enumerate(node):yield from nodes(value,(*path,i))
for source in sorted(BASIS.rglob('*.json')):
 rel=str(source.relative_to(BASIS));target=ROOT/rel
 if not target.is_file():continue
 try:o=json.loads(source.read_bytes());v=json.loads(target.read_bytes())
 except (ValueError,UnicodeError):continue
 original[rel]=o;current[rel]=v;raw_original[rel]=source.read_bytes()
 raw=target.read_bytes();before.append({'path':rel,'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'base64':base64.b64encode(raw).decode()})
 for path,kind,node in m.hash_nodes(o):
  field,domain=kind
  if field not in node:continue
  row=(rel,path,field,domain)
  (valid if node[field]==m.derived(node,field,domain) else invalid).append(row)
 for path,node in nodes(o):
  transport=node.get('transport');value=node.get('value')
  if isinstance(transport,dict) and isinstance(value,dict) and isinstance(transport.get('base64'),str):
   raw=base64.b64decode(transport['base64'],validate=True)
   assert hashlib.sha256(raw).hexdigest()==transport['sha256']
   assert len(raw)==transport['utf8_length']
   if raw==m.canonical(value)+b'\n':transports.append((rel,path))
  for key,field in [('fingerprint_preimage_utf8_length','request_fingerprint'),('payload_preimage_utf8_length','payload_hash')]:
   if key in node and isinstance(value,dict) and field in value:
    expected=len(m.canonical({k:v for k,v in value.items() if k!=field}))
    if node[key]==expected:lengths.append((rel,path,key,field))
(RECORD/'before.bytes.json').write_text(json.dumps({'schema':'mpk.original_bytes_set.v1','files':before},indent=2)+'\n')
changes=json.loads(Path(sys.argv[2]).read_text()) if len(sys.argv)>2 else {}
rounds=[]
for _ in range(40):
 count=0
 for rel in current:current[rel]=m.replace(current[rel],changes)
 for rel,path,field,domain in reversed(valid):
  node=m.locate(current[rel],path);new=m.derived(node,field,domain);old=node[field]
  if old!=new:changes[old]=new;node[field]=new;count+=1
 for rel,path in transports:
  node=m.locate(current[rel],path);raw=m.canonical(node['value'])+b'\n';t=node['transport'];new={**t,'base64':base64.b64encode(raw).decode(),'sha256':hashlib.sha256(raw).hexdigest(),'utf8_length':len(raw)}
  if t!=new:node['transport']=new;count+=1
 for rel,path,key,field in lengths:
  node=m.locate(current[rel],path);new=len(m.canonical({k:v for k,v in node['value'].items() if k!=field}))
  if node[key]!=new:node[key]=new;count+=1
 rounds.append(count)
 if count==0:break
else:raise AssertionError('typed fixture DAG did not converge')
def encode(rel,doc):
 if doc==original[rel]:return raw_original[rel]
 return m.canonical(doc)+(b'\n' if raw_original[rel].endswith(b'\n') else b'')
def resolve(rel,path):
 if not isinstance(path,str):return None
 candidates=[ROOT/path,(ROOT/rel).parent/path,ROOT/'fixtures/vir-go'/path]
 if 'vc_skeleton.json' in path:candidates += [p.with_name('vc.cert_skeleton.json') for p in candidates]
 for p in candidates:
  p=p.resolve()
  if p.is_relative_to(ROOT) and p.is_file():return p
 return None
refs=[]
for rel,doc in original.items():
 for path,node in nodes(doc):
  if not isinstance(node.get('path'),str) or not isinstance(node.get('sha256'),str):continue
  target=resolve(rel,node['path'])
  if target is None:continue
  basis=BASIS/target.relative_to(ROOT);raw=basis.read_bytes() if basis.is_file() else target.read_bytes()
  if hashlib.sha256(raw).hexdigest()==node['sha256']:refs.append((rel,path,target))
raw_rounds=[]
for _ in range(40):
 for rel,doc in current.items():(ROOT/rel).write_bytes(encode(rel,doc))
 count=0
 for rel,path,target in refs:
  node=m.locate(current[rel],path);raw=target.read_bytes();sha=hashlib.sha256(raw).hexdigest()
  if node['sha256']!=sha:node['sha256']=sha;count+=1
  for key in ['bytes','size_bytes']:
   if key in node and node[key]!=len(raw):node[key]=len(raw);count+=1
 raw_rounds.append(count)
 if count==0:break
else:raise AssertionError('raw fixture DAG did not converge')
for rel,path,field,domain in valid:
 node=m.locate(current[rel],path);assert node[field]==m.derived(node,field,domain),(rel,path)
for rel,path,field,domain in invalid:
 node=m.locate(current[rel],path);assert node[field]!=m.derived(node,field,domain),(rel,path)
changed=[{'path':r['path'],'sha256':hashlib.sha256((ROOT/r['path']).read_bytes()).hexdigest(),'bytes':(ROOT/r['path']).stat().st_size} for r in before if hashlib.sha256((ROOT/r['path']).read_bytes()).hexdigest()!=r['sha256']]
out={'status':'passed','basis_commit':'528b599aacfe439d4abd8f50e046a754c9a24435','changed':changed,'valid_typed_hashes':len(valid),'invalid_typed_hashes_preserved':len(invalid),'complete_canonical_transports_rebound':len(transports),'valid_preimage_lengths_rebound':len(lengths),'valid_raw_references':len(refs),'typed_rounds':rounds,'raw_rounds':raw_rounds,'selection_reason':'Registry/binary identity changes require full typed/transport/raw dependency rebinding. Immutable original masks preserve adversarial invalid hashes; semantically unchanged inputs retain exact original formatting, including whitespace conformance controls. Actual frontend corpus execution is a separate subsequent check.','frontend_execution_claimed':False,'program_proof_claimed':False,'finished_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat()}
(RECORD/'result.json').write_text(json.dumps(out,indent=2)+'\n');(RECORD/'runner.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps({k:v for k,v in out.items() if k not in {'changed','selection_reason'}}))
