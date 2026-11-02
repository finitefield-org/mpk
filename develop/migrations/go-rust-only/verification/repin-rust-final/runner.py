from pathlib import Path
import json,hashlib,sys,base64
sys.path.insert(0,'scripts');import successor_release_bundles as b
kind=sys.argv[1]
record=Path('develop/migrations/go-rust-only/verification')/('repin-'+kind+'-final');record.mkdir(exist_ok=False)
prior_candidate=b.read_canonical(Path('release/bundles/candidates/'+kind+'.json'))[0]
prior_registry=b.read_canonical(b.REGISTRY_PATH)[0]
old_binaries={r['name']:r['binary_sha256'] for r in [prior_candidate['frontend_bundles'][0]['main'],*prior_candidate['frontend_bundles'][0].get('subordinate_binaries',[])]}
needles=[prior_registry['registry_sha256'],*old_binaries.values()]
preimages=[]
for base in ['crates','go-tools','rust-tools','examples','fixtures','scripts','release/bundles']:
 for p in Path(base).rglob('*'):
  if not p.is_file() or p.suffix not in {'.rs','.go','.py','.json','.md','.sh'} or any(x in p.parts for x in ['vendor','target','__pycache__']):continue
  raw=p.read_bytes()
  if any(n.encode() in raw for n in needles) or str(p).startswith('release/bundles/'):
   preimages.append({'path':str(p),'sha256':hashlib.sha256(raw).hexdigest(),'bytes':len(raw),'base64':base64.b64encode(raw).decode()})
(record/'before.bytes.json').write_text(json.dumps({'schema':'mpk.original_bytes_set.v1','files':preimages},indent=2)+'\n')
if kind=='go':
 r=json.loads(Path('develop/migrations/go-rust-only/verification/build-go/result.json').read_text());assert r['status']=='passed' and r['byte_identical'] and r['whole_source_inputs_unchanged'];source=Path(r['outputs'][0]['go2vir']['path']).parent
 candidate=b.read_canonical(Path('release/bundles/candidates/go.json'))[0]
 # Actual emitted binary occupies bin/go2vir in the release inventory.
 b.refresh_frontend_candidate(candidate,source,{'bin/go2vir':'go2vir'})
 b.write_canonical(Path('release/bundles/candidates/go.json'),candidate)
elif kind=='rust':
 r=json.loads(Path('develop/migrations/go-rust-only/verification/build-rust-retry/result.json').read_text());assert r['status']=='passed' and r['byte_identical'] and r['whole_source_inputs_unchanged'];source=Path(r['outputs'][0]['rust2vir']['path']).parent
 candidate=b.read_canonical(Path('release/bundles/candidates/rust.json'))[0];b.refresh_frontend_candidate(candidate,source,{'bin/rust2vir':'rust2vir','bin/rust2vir-driver':'rust2vir-driver'});b.write_canonical(Path('release/bundles/candidates/rust.json'),candidate)
old=b.read_canonical(b.REGISTRY_PATH)[0];old_hash=old['registry_sha256'];old_raw=hashlib.sha256(b.REGISTRY_PATH.read_bytes()).hexdigest();b.update_descriptors();new=b.read_canonical(b.REGISTRY_PATH)[0];new_raw=hashlib.sha256(b.REGISTRY_PATH.read_bytes()).hexdigest()
source=Path('crates/mpk-vc/src/release_bundle_v1.rs');s=source.read_text()
import re
s=re.sub(r'(pub const ACTIVE_RELEASE_REGISTRY_SHA256: &str =\s*)"[0-9a-f]+"',lambda m:m[1]+'"'+new['registry_sha256']+'"',s)
for language,const in [('go','GO_FRONTEND_SHA256'),('rust','RUST_FRONTEND_SHA256'),('rust','RUST_DRIVER_SHA256')]:
 c=b.read_canonical(Path('release/bundles/candidates/'+language+'.json'))[0]['frontend_bundles'][0];val=c['subordinate_binaries'][0]['binary_sha256'] if 'DRIVER' in const else c['main']['binary_sha256'];s=re.sub(r'(pub const '+const+r': &str =\s*)"[0-9a-f]+"',lambda m:m[1]+'"'+val+'"',s)
source.write_text(s)
p=Path('scripts/successor_release_bundles.py');s=p.read_text().replace(old_hash,new['registry_sha256']).replace(old_raw,new_raw);p.write_text(s)
current_candidate=b.read_canonical(Path('release/bundles/candidates/'+kind+'.json'))[0]
new_binaries={r['name']:r['binary_sha256'] for r in [current_candidate['frontend_bundles'][0]['main'],*current_candidate['frontend_bundles'][0].get('subordinate_binaries',[])]}
changes={old_hash:new['registry_sha256'],**{sha:new_binaries[name] for name,sha in old_binaries.items()}}
# Preserve old epochs; update only current consumers' exact identities.
for root in ['crates','go-tools','rust-tools','examples','fixtures','scripts']:
 for p in Path(root).rglob('*'):
  if not p.is_file() or p.suffix not in {'.rs','.go','.py','.json','.md','.sh'} or any(x in p.parts for x in ['vendor','target','__pycache__']):continue
  try:s=p.read_text()
  except UnicodeError:continue
  ns=s
  for old_value,new_value in changes.items():ns=ns.replace(old_value,new_value)
  if ns!=s:p.write_text(ns)
p=Path('scripts/successor_release_bundles.py');s=p.read_text();s=re.sub(r'(if hashlib.sha256\(registry_bytes\).hexdigest\(\) != \(\n        )"[0-9a-f]{64}"',lambda m:m[1]+'"'+new_raw+'"',s);p.write_text(s)
b.validate_release_models()
(record/'changes.json').write_text(json.dumps(changes,indent=2)+'\n')
(record/'result.json').write_text(json.dumps({'status':'passed','old_registry':old_hash,'new_registry':new['registry_sha256'],'old_binaries':old_binaries,'new_binaries':new_binaries,'complete_preimages':len(preimages),'frontend_execution_claimed':False},indent=2)+'\n')
(record/'runner.py').write_bytes(Path(__file__).read_bytes())
print('repinned',kind,new['registry_sha256'])
