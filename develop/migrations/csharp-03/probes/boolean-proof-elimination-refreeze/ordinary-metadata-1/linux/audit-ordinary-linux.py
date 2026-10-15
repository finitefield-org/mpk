import hashlib,json,pathlib,subprocess
base=pathlib.Path(__file__).parent;repo=pathlib.Path('/private/tmp/mpk-w09-context-application-integration');dest=base/'ordinary-linux-final/archive-extracted';control=dest/'mpk-w09-refreeze-ordinary-40469d29-control';out=dest/'mpk-w09-refreeze-ordinary-40469d29-reports';manifest=json.loads((control/'manifest.json').read_bytes());state=json.loads((out/'status.json').read_bytes());local=json.loads((base/'ordinary-regeneration-1/status.json').read_bytes());digest=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
assert state['status']=='passed_selected_exact_public_source_linux_ordinary_owners'
assert state['source_commit']==manifest['source_commit']=='40469d298f1d69558713d02ea0b553a1dd583bf5'
assert len(state['stages'])==len(manifest['jobs'])==len(local['stages'])==7
assert (control/'manifest.json').read_bytes()==(base/'ordinary-linux-control/manifest.json').read_bytes()
assert (control/'run-linux.py').read_bytes()==(base/'ordinary-linux-control/run-linux.py').read_bytes()
rows=[]
for stage,lstage,job in zip(state['stages'],local['stages'],manifest['jobs']):
 assert stage['stage']==lstage['stage']==job['destination']
 assert stage['selector']==lstage['selector']==job['selector']
 assert stage['exit_code']==lstage['exit_code']==0 and stage['test_counts']==lstage['test_counts']==['1']
 assert digest(out/(stage['stage']+'.log.txt'))==stage['log_sha256']
 assert stage['generated_files']==lstage['generated_files']
 for path,sha in stage['generated_files'].items():
  a=out/'goldens'/stage['stage']/path;b=base/'ordinary-regeneration-1/goldens'/stage['stage']/path
  assert digest(a)==digest(b)==sha and a.read_bytes()==b.read_bytes()
  rows.append(dict(family=stage['stage'],path=path,raw_sha256=sha,bytes=a.stat().st_size))
paths=manifest['source_hashes'];tree=subprocess.check_output(['/usr/bin/git','-C',str(repo),'ls-tree','-r','--full-tree','-z',state['source_commit']]);oids={}
for row in tree.split(b'\0'):
 if not row:continue
 fields,path=row.split(b'\t',1);mode,kind,oid=fields.split();oids[path.decode()]=oid.decode()
proc=subprocess.Popen(['/usr/bin/git','-C',str(repo),'cat-file','--batch'],stdin=subprocess.PIPE,stdout=subprocess.PIPE)
for path,sha in paths.items():
 oid=oids[path];proc.stdin.write((oid+'\n').encode());proc.stdin.flush();header=proc.stdout.readline().decode().split();assert header[:2]==[oid,'blob'];size=int(header[2]);data=proc.stdout.read(size);assert len(data)==size and proc.stdout.read(1)==b'\n';assert hashlib.sha256(data).hexdigest()==sha,path
proc.stdin.close();assert proc.wait()==0
assert len(rows)==234 and sum(p['path'].endswith('.hex') for p in rows)==227
receipt={'schema':'mpk.csharp_practical.t01_w09.ordinary_linux_receipt.v1','status':'passed_independent_exact_public_source_and_all_generated_bytes_audit','source_commit':state['source_commit'],'public_source_git_blobs_checked':len(paths),'owner_tests':len(state['stages']),'generated_files':len(rows),'certificate_files':227,'metadata_files':7,'darwin_linux_exact_generated_bytes_equal':True,'files':rows,'archive_sha256':digest(base/'ordinary-linux-final/mpk-w09-refreeze-ordinary-40469d29-reports.tar.gz'),'original_application_proof_ids_pending':987,'selection_reason':manifest['selection_reason'],'full_t01_gate':'deferred_to_final_W10','full_t06_gate':'deferred_to_W12'}
(base/'ordinary-linux-final/audit.json').write_text(json.dumps(receipt,sort_keys=True,indent=2)+'\n');print(json.dumps({k:v for k,v in receipt.items() if k not in ['files','selection_reason']}))
