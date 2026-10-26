from pathlib import Path
import hashlib,json,subprocess,tomllib
repo=Path('/Users/kazuyoshitoshiya/ff/mpk');ev=repo/'develop/migrations/go-practical/t02-w01/verification/compiler-elf/build-wiring'
def seal(p):
 b=p.read_bytes();return {'path':str(p),'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest(),'mode':p.stat().st_mode&0o777}
def blocks(text):
 head,*parts=text.split('\n[[package]]\n')
 return head,{(v['name'],v['version'],v.get('source','')):part for part in parts for v in [tomllib.loads('[[package]]\n'+part)['package'][0]]}
root_head,root_packages=blocks((repo/'Cargo.lock').read_text())
old_cases=json.loads((ev/'before-resolved.json').read_bytes())['cases'];results=[]
for case in old_cases:
 manifest=repo/case['manifest'];lock=manifest.with_name('Cargo.lock');old_head,old_packages=blocks(lock.read_text())
 merged={**old_packages,**root_packages}
 lock.write_text(root_head+'\n[[package]]\n'+'\n[[package]]\n'.join(merged[k].strip()+'\n' for k in sorted(merged)))
 out=ev/('resolve-'+manifest.parent.name);out.mkdir()
 command=['cargo','metadata','--offline','--format-version','1','--manifest-path',str(manifest)]
 p=subprocess.run(command,cwd=repo,capture_output=True)
 (out/'stdout.log').write_bytes(p.stdout);(out/'stderr.log').write_bytes(p.stderr)
 row={'manifest':case['manifest'],'command':command,'exit_code':p.returncode,'stdout':seal(out/'stdout.log'),'stderr':seal(out/'stderr.log'),'lock':seal(lock)}
 if p.returncode==0:
  graph=json.loads(p.stdout);names={r['name'] for r in graph['packages']};assert 'mpk-go-zstd-blocks' in names
  allowed=set(root_packages)|set(old_packages)
  actual=tomllib.loads(lock.read_text())['package']
  assert all((r['name'],r['version'],r.get('source','')) in allowed for r in actual)
  original={k:tomllib.loads('[[package]]\n'+v)['package'][0] for k,v in merged.items()}
  for r in actual:
   if 'source' in r:assert r.get('checksum')==original[(r['name'],r['version'],r['source'])].get('checksum')
  row['resolved_packages']=len(graph['packages']);row['existing_registry_versions_and_checksums_retained']=True
 results.append(row)
(ev/'resolution.json').write_text(json.dumps({'status':'passed' if all(r['exit_code']==0 for r in results) else 'failed','root_and_original_dependency_versions_only':True,'cases':results},indent=2)+'\n')
print(json.dumps([{'manifest':r['manifest'],'exit_code':r['exit_code'],'diagnostic':Path(r['stderr']['path']).read_text()[-500:]} for r in results],ensure_ascii=False))
raise SystemExit(any(r['exit_code'] for r in results))
