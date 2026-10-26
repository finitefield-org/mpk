"""Independently verify complete locked graphs, actual builds and original preimages."""
from pathlib import Path
import hashlib,json,tarfile,tomllib
ROOT=Path(__file__).resolve().parent;REPO=ROOT.parents[6]
def seal(p):
 p=Path(p);b=p.read_bytes();return {'path':str(p.resolve()),'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest(),'mode':p.stat().st_mode&0o777}
def read(p):return json.loads(Path(p).read_bytes())
def main():
 out=ROOT/'inspection.json';assert not out.exists()
 before=read(ROOT/'before-resolved.json');assert len(before['cases'])==7 and all(r['exit_code']==101 for r in before['cases'])
 original=read(ROOT/'original-inputs.json')['inputs']+read(ROOT/'original-collectors.json')['inputs']
 for r in original:
  b=bytes.fromhex(r['raw_hex']);assert len(b)==r['bytes'] and hashlib.sha256(b).hexdigest()==r['sha256']
 locked=read(ROOT/'locked-final/result.json');assert locked['status']=='passed' and locked['inputs_unchanged'] and len(locked['cases'])==7
 graph_inputs=read(ROOT/'locked-final/inputs.json')
 for r in graph_inputs['inputs']:
  b=bytes.fromhex(r['raw_hex']);assert len(b)==r['bytes'] and hashlib.sha256(b).hexdigest()==r['sha256'];assert seal(r['path'])=={k:v for k,v in r.items() if k!='raw_hex'}
 for r in locked['cases']:
  assert r['exit_code']==0 and r['actual_private_codec_path_edge'] and '--locked'in r['command'] and '--offline'in r['command'] and '--no-deps'not in r['command']
  for k in ['stdout','stderr']:assert r[k]==seal(r[k]['path'])
  graph=read(r['stdout']['path']);pkg=next(p for p in graph['packages'] if p['name']=='mpk-go-zstd-blocks');vc=next(p for p in graph['packages'] if p['name']=='mpk-vc')
  assert Path(pkg['manifest_path'])==REPO/'third_party/ruzstd-go125/Cargo.toml' and pkg['source'] is None
  assert any(d['pkg']==pkg['id'] for n in graph['resolve']['nodes'] if n['id']==vc['id'] for d in n['deps'])
 builds=[]
 for name in ['native-linux','shared-native']:
  result=read(ROOT/name/'result.json');assert result['status']=='passed' and result['inputs_unchanged'] and not result['native_Linux_release_acceptance']
  receipt=read(ROOT/name/'inputs.json');assert result['inputs']==seal(ROOT/name/'inputs.json')
  archive=Path(receipt['complete_archive']['path']);assert receipt['complete_archive']==seal(archive)
  rows={str(Path(r['path']).relative_to(REPO)):r for r in receipt['inputs']}
  with tarfile.open(archive) as packed:
   members=packed.getmembers();assert len(members)==len(rows) and {m.name for m in members}==set(rows)
   for member in members:
    r=rows[member.name];b=packed.extractfile(member).read();assert member.isfile() and member.mode==r['mode'] and len(b)==r['bytes'] and hashlib.sha256(b).hexdigest()==r['sha256'];assert seal(r['path'])==r
  commands=read(ROOT/name/'commands.json');assert commands and all(r['exit_code']==0 for r in commands)
  for r in commands:
   for k in ['stdout','stderr']:assert r[k]==seal(r[k]['path'])
  binary=result.get('observer_binary',{}).get('path') or '/private/tmp/mpk-go-practical-policy-linux-target/debug/go-practical-release-native'
  assert Path(binary).is_file() and seal(binary)['sha256']==result['audit']['binary_sha256']
  if name=='native-linux':assert Path(binary).read_bytes().startswith(b'\x7fELF')
  else:assert result['complete_private_codec_files']==65
  builds.append({'name':name,'commands':len(commands),'source_files':len(rows),'binary_sha256':result['audit']['binary_sha256']})
 summary={'status':'passed','complete_locked_graphs':7,'original_failed_locked_graphs':7,'actual_observer_builds':builds,'original_preimages_retained':len(original),'source_and_command_output_seals_verified':True,'native_Linux_release_acceptance':False,'proofs_replayed':False}
 out.write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary))
if __name__=='__main__':main()
