"""Resolve every affected complete locked graph, including actual private path edges."""
from pathlib import Path
import hashlib,json,subprocess,tomllib,shutil
REPO=Path.cwd();ROOT=Path(__file__).resolve().parent

def seal(p):
 p=Path(p);b=p.read_bytes();return {'path':str(p.resolve()),'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest(),'mode':p.stat().st_mode&0o777}
def main():
 out=ROOT/'locked-final';out.mkdir(exist_ok=False)
 cases=json.loads((ROOT/'before-resolved.json').read_bytes())['cases']
 manifests=[REPO/r['manifest'] for r in cases]
 paths={Path(__file__).resolve(),REPO/'Cargo.toml',REPO/'Cargo.lock',REPO/'third_party/ruzstd-go125/Cargo.toml'}
 paths.update((REPO/'crates').glob('*/Cargo.toml'))
 for p in manifests:paths.update([p,p.with_name('Cargo.lock')])
 inputs=[{**seal(p),'raw_hex':p.read_bytes().hex()} for p in sorted(paths)]
 (out/'inputs.json').write_text(json.dumps({'inputs':inputs,'cargo':seal(shutil.which('cargo')),'selection_reason':'Resolve all seven complete independent Go/Rust observer/fuzz graphs offline/locked; package-only --no-deps output is insufficient evidence.'},indent=2)+'\n')
 results=[]
 for manifest in manifests:
  command=['cargo','metadata','--offline','--locked','--format-version','1','--manifest-path',str(manifest)]
  completed=subprocess.run(command,cwd=REPO,capture_output=True)
  name=manifest.parent.name;stdout=out/(name+'.stdout.log');stderr=out/(name+'.stderr.log');stdout.write_bytes(completed.stdout);stderr.write_bytes(completed.stderr)
  row={'manifest':str(manifest.relative_to(REPO)),'command':command,'exit_code':completed.returncode,'stdout':seal(stdout),'stderr':seal(stderr)}
  if completed.returncode==0:
   graph=json.loads(completed.stdout);codec=next(p for p in graph['packages'] if p['name']=='mpk-go-zstd-blocks');vc=next(p for p in graph['packages'] if p['name']=='mpk-vc')
   assert Path(codec['manifest_path'])==REPO/'third_party/ruzstd-go125/Cargo.toml' and codec['source'] is None
   node=next(n for n in graph['resolve']['nodes'] if n['id']==vc['id']);assert any(n['pkg']==codec['id'] for n in node['deps'])
   row.update(resolved_packages=len(graph['packages']),actual_private_codec_path_edge=True)
   lock=tomllib.loads(manifest.with_name('Cargo.lock').read_text())
   original=tomllib.loads(next(bytes.fromhex(r['raw_hex']).decode() for r in json.loads((ROOT/'original-inputs.json').read_bytes())['inputs'] if r['path']==str(manifest.with_name('Cargo.lock'))))
   allowed={(p['name'],p['version'],p.get('checksum')) for p in tomllib.loads((REPO/'Cargo.lock').read_text())['package']}
   if name=='fuzz':allowed.update((p['name'],p['version'],p.get('checksum')) for p in original['package'])
   assert all((p['name'],p['version'],p.get('checksum')) in allowed for p in lock['package'] if 'source' in p)
  results.append(row)
 unchanged=inputs==[{**seal(p),'raw_hex':p.read_bytes().hex()} for p in sorted(paths)]
 report={'status':'passed' if unchanged and all(r['exit_code']==0 for r in results) else 'failed','inputs_unchanged':unchanged,'cases':results}
 (out/'result.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({'status':report['status'],'complete_locked_graphs':len(results),'inputs_unchanged':unchanged}));raise SystemExit(report['status']!='passed')
if __name__=='__main__':main()
