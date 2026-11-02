"""Independently inspect complete original compiler/executable binding evidence."""
from pathlib import Path
import hashlib,json,re
ROOT=Path(__file__).resolve().parent
INPUTS={}
def seal(path):
 p=Path(path);data=p.read_bytes();return {'path':str(p.resolve()),'bytes':len(data),'sha256':hashlib.sha256(data).hexdigest(),'mode':p.stat().st_mode&0o777}
def raw(path):
 p=Path(path);INPUTS[str(p.resolve())]=seal(p);return p.read_bytes()
def read(path):return json.loads(raw(path))
def typed(domain,v):return hashlib.sha256(domain.encode()+b'\0'+json.dumps(v,sort_keys=True,ensure_ascii=False,separators=(',',':')).encode()).hexdigest()
def record(v,domain):assert typed(domain,{k:x for k,x in v.items() if k!='content_sha256'})==v['content_sha256']
def main():
 out=ROOT/'inspection.json';assert not out.exists()
 verification=read(ROOT/'verification.json')
 epochs={p.parent.name:read(p) for p in ROOT.glob('*/result.json')}
 assert sorted(epochs)==verification['source_epochs']
 assert sorted(n for n,r in epochs.items() if r['status']=='failed')==verification['preserved_failed_epochs']
 for name,r in epochs.items():
  assert r['whole_sdk_unchanged'] and (r['consumed_inputs_unchanged'] or r['expected_formatter_mutation'])
  assert r['inputs']==seal(r['inputs']['path'])
  for key in ['stdout','stderr']:assert r[key]==seal(r[key]['path'])
 native=[json.loads(s) for s in raw(ROOT/'native/stdout.log').splitlines()]
 roots=[r for r in native if r.get('Action')=='pass' and r.get('Test') and '/' not in r['Test']]
 assert len(roots)==1 and not any(r.get('Action')=='fail' for r in native)
 oracle=read(ROOT/'fixtures/native.json');assert len(oracle)==187
 comparison=read(ROOT/verification['rust_epoch']/'observations/build-info/native-comparison.json')
 assert comparison['status']=='passed' and len(comparison['cases'])==len(oracle)
 for a,b in zip(oracle,comparison['cases']):
  assert a['name']==b['name']
  if a['accepted']:
   expected=dict(a['facts'])
   if a['form']=='text':expected['version_hex']=b'go1.25.0'.hex()
   assert b['facts']==expected and b['binding_accepted']==a['binding_accepted']
  else:assert b['rejected']
 assert sum(r['form']=='original' for r in oracle)==9
 log=raw(ROOT/verification['rust_epoch']/'stdout.log').decode()
 assert [int(n) for n in re.findall(r'test result: ok\. (\d+) passed; 0 failed; 0 ignored;',log)]==[1,5,6]
 directory=ROOT.parent/'compiler-inputs/native/native-originals/versioned-owned'
 packet=read(directory/'complete-original-module-graph-inputs.json')
 original=json.loads(bytes.fromhex(packet['compiler_inputs_hex']));export=json.loads(bytes.fromhex(packet['export_inputs_hex']))
 owner=read(ROOT/verification['rust_epoch']/'observations/compilers/whole-original-compiler-import.json')
 assert owner['files']==5577 and owner['bytes']==163457770 and owner['original_objects']==5697 and owner['original_packages']==182 and owner['independently_bound_executables']==9
 assert owner['compiler_inputs']==original
 for row in oracle:
  if row['form']=='original':
   data=raw(directory/'original-compiler-files'/(row['raw_sha256']+'.raw'));assert len(data)==row['bytes'] and hashlib.sha256(data).hexdigest()==row['raw_sha256']
  else:
   data=raw(ROOT/'fixtures'/row['file']);assert len(data)==row['bytes'] and hashlib.sha256(data).hexdigest()==row['raw_sha256']
 controls=[]
 for name in ['release','tool','cgo']:
  v=read(ROOT/verification['rust_epoch']/f'observations/compilers/coherent-executable-{name}.json')
  assert v['whole_original_source_and_Root_contract_unchanged'] and v['error']=='BuildInfo' and v['original_objects']==5697 and v['original_packages']==182
  data=raw(ROOT/verification['rust_epoch']/f'observations/compilers/changed-{name}.raw')
  entry=v['changed_entry'];assert len(data)==entry['size_bytes'] and hashlib.sha256(data).hexdigest()==entry['sha256']
  old=next(r['entry'] for r in packet['compiler_files'] if r['entry']['path']=='go/bin/gofmt')
  original_bytes=bytearray(raw(directory/'original-compiler-files'/(old['sha256']+'.raw')))
  before=bytes.fromhex(v['before_hex']);after=bytes.fromhex(v['after_hex']);at=v['byte_offset']
  assert original_bytes[at:at+len(before)]==before and len(before)==len(after)
  original_bytes[at:at+len(after)]=after;assert original_bytes==data
  wire=v['compiler_inputs'];record(wire,'MPK-PRIVATE-GO-COMPILER-IMAGE-INPUTS-1.0')
  expected=[entry if r['path']=='go/bin/gofmt' else r for r in original['inventory']['files']]
  assert wire['inventory']['files']==expected and len(expected)==5577
  assert wire['toolchain']['distribution_sha256']==typed('MPK-BUNDLE-CONTENT-0.1',wire['inventory'])
  for key,domain in [('envelope','CONTRACT'),('selection','SELECTION'),('capture','CAPTURE'),('exports','EXPORT')]:record(v[key],f'MPK-PRIVATE-GO-SOURCE-{domain}-INPUTS-1.0')
  assert v['envelope']['compiler_sha256']==wire['toolchain']['distribution_sha256']==v['exports']['compiler_sha256']
  assert v['selection']['source_contract_inputs_sha256']==v['envelope']['content_sha256']
  assert v['capture']['selection_inputs_sha256']==v['selection']['content_sha256']
  assert v['capture']['snapshot_inputs_sha256']==json.loads(bytes.fromhex(packet['capture_hex']))['snapshot_inputs_sha256']
  assert v['exports']['source_capture_inputs_sha256']==v['capture']['content_sha256']
  assert wire['source_export_inputs_sha256']==v['exports']['content_sha256']
  for key in ['metadata_hex','diagnostics_hex','packages','origins','go_binary_sha256','image_sha256']:assert v['exports'][key]==export[key]
  controls.append({'name':name,'error':v['error'],'changed_bytes':len(data)})
 old_controls=read(ROOT/verification['rust_epoch']/'observations/compilers/coherent-original-compiler-controls.json')
 assert len(old_controls['cases'])==30
 read(ROOT/verification['rust_epoch']/'observations/compilers/coherent-original-sdk-content-control.json')
 result={'status':'passed','affected_rust_roots':12,'native_roots':1,'native_cases':187,'actual_original_tools':9,'coherent_compiler_claim_controls':30,'coherent_binary_content_controls':3,'binary_controls':controls,'whole_original_owners':[{'objects':5697,'packages':182,'compiler_files':5577,'compiler_bytes':163457770}],'installed_context_authority':False,'proofs_replayed_in_this_checkpoint':False,'source_epochs':sorted(epochs),'inputs':list(INPUTS.values()),'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
 out.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items() if k not in ['inputs']}))
if __name__=='__main__':main()
