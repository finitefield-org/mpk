from pathlib import Path
import hashlib,json,subprocess,tarfile
repo=Path('/root/mpk-w09-bool-cases-fd99c03c');control=repo.with_name(repo.name+'-control');reports=repo.with_name(repo.name+'-linux-2');prior=repo.with_name(repo.name+'-linux');h=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
state=json.loads((reports/'status.json').read_bytes());assert state['status']=='passed_exact_public_source_linux_tests_and_dual_checkers' and len(state['stages'])==82
assert state['cases']==34 and state['checker_stages']==68
assert subprocess.check_output(['/usr/bin/git','rev-parse','HEAD'],cwd=repo,text=True).strip()==state['source_commit']=='fd99c03c8e3535bcc31f0b9d1441d4b74de80ebf'
assert subprocess.check_output(['/usr/bin/git','status','--porcelain'],cwd=repo,text=True).strip()==''
manifest=json.loads((control/'source-manifest.json').read_bytes())
for group in ['source_hashes','fixture_hashes','document_hashes']:
 for p,sha in manifest[group].items():assert h(repo/p)==sha,p
for backend,row in state['binaries'].items():assert h(Path(row['path']))==row['sha256'],backend
files={}
for prefix,root in [(Path('.'),reports),(Path('initial-missing-embedded-input-compile'),prior)]:
 for p in root.rglob('*'):
  if p.is_file() and p.name!='go-checker':files[str(prefix/p.relative_to(root))]=(p,h(p))
metadata={'schema':'mpk.evidence_file_manifest.v1','files':{name:sha for name,(p,sha) in files.items()}}
manifest_path=control/'completed-linux-file-manifest.json';manifest_path.write_text(json.dumps(metadata,sort_keys=True,indent=2)+'\n')
archive=control/'completed-linux-evidence.tar.gz'
with tarfile.open(archive,'w:gz') as tar:
 for name,(p,sha) in sorted(files.items()):tar.add(p,arcname=name,recursive=False)
 tar.add(manifest_path,arcname='file-manifest.json',recursive=False)
print(json.dumps({'status':'prepared_only_after_all_linux_stages_passed','archive':str(archive),'archive_sha256':h(archive),'file_count':len(files),'source_clean_after':True,'binaries_exact':True}))
