import argparse,base64,hashlib,io,json,pathlib,subprocess,tarfile
p=argparse.ArgumentParser();p.add_argument('commit');p.add_argument('output',type=pathlib.Path);a=p.parse_args();assert len(a.commit)==40 and all(c in '0123456789abcdef' for c in a.commit)
remote=r'''
import base64,hashlib,io,json,os,pathlib,subprocess,sys,tarfile
commit=sys.argv[1];repo=pathlib.Path('/root/mpk-w09-identity-proofs-'+commit[:8]);reports=repo.with_name(repo.name+'-linux')
s=json.loads((reports/'status.json').read_bytes());assert s['status']=='passed_targeted_tests_lint_format',s
base=repo/'develop/migrations/csharp-03/ordinary-foundation/verification-logs/identity-proofs';manifest=json.loads((base/'source-manifest.json').read_bytes());blobs={}
assert subprocess.check_output(['/usr/bin/git','rev-parse','HEAD'],cwd=repo,text=True).strip()==commit
assert subprocess.check_output(['/usr/bin/git','status','--porcelain'],cwd=repo,text=True).strip()==''
for group in ('source_hashes','fixture_hashes'):
 for name,h in manifest[group].items():
  data=(repo/name).read_bytes();assert hashlib.sha256(data).hexdigest()==h and subprocess.check_output(['/usr/bin/git','show',f'{commit}:{name}'],cwd=repo)==data,name;blobs[name]=h
harness={}
for name in ('tool-sources/run-targeted.py','tool-sources/launch-linux.py','source-manifest.json','expected-certificates.json'):
 f=base/name;data=f.read_bytes();assert subprocess.check_output(['/usr/bin/git','show',f'{commit}:{f.relative_to(repo)}'],cwd=repo)==data;harness[name]=hashlib.sha256(data).hexdigest()
for stage in s['stages']:assert stage['exit_code']==0 and hashlib.sha256((reports/(stage['stage']+'.log.txt')).read_bytes()).hexdigest()==stage['log_sha256']
for f,h in s['test_binaries'].items():assert hashlib.sha256(pathlib.Path(f).read_bytes()).hexdigest()==h
for n,h in s['exported_hashes'].items():assert hashlib.sha256((reports/'certificates'/n).read_bytes()).hexdigest()==h
pids={}
for name in ('supervisor_pid','process_pid'):
 pid=s[name];path=pathlib.Path('/proc')/str(pid)/'cmdline';pids[name]=dict(pid=pid,exists=path.exists(),cmdline=path.read_bytes().decode().replace('\0',' ') if path.exists() else '')
 assert not pids[name]['exists'],pids[name]
state=dict(status='passed_independent_terminal_linux_audit',commit=commit,clean_checkout=True,git_blob_files_verified=len(blobs),git_blob_hashes=blobs,harness_hashes=harness,test_binary_hashes=s['test_binaries'],process_observations=pids,exported_file_hashes=s['exported_hashes'],stage_results=s['stages'])
(reports/'final-audit.json').write_text(json.dumps(state,indent=2,sort_keys=True)+'\n')
buffer=io.BytesIO()
with tarfile.open(fileobj=buffer,mode='w:gz') as t:
 for f in sorted(reports.rglob('*')):
  if f.is_file():t.add(f,arcname=str(f.relative_to(reports)),recursive=False)
print(json.dumps(dict(audit=state,archive_base64=base64.b64encode(buffer.getvalue()).decode(),archive_sha256=hashlib.sha256(buffer.getvalue()).hexdigest())))
'''
a.output.mkdir(parents=True,exist_ok=True);command=['ssh','-i','/Users/kazuyoshitoshiya/ffvps.pem','-o','BatchMode=yes','-o','ConnectTimeout=30','root@162.43.92.154','python3','-',a.commit]
with (a.output/'fetch.json').open('wb') as o,(a.output/'fetch.stderr.txt').open('wb') as e:r=subprocess.run(command,input=remote.encode(),stdout=o,stderr=e)
assert r.returncode==0,r.returncode;data=json.loads((a.output/'fetch.json').read_bytes());raw=base64.b64decode(data['archive_base64']);assert hashlib.sha256(raw).hexdigest()==data['archive_sha256'];(a.output/'reports.tar.gz').write_bytes(raw)
with tarfile.open(fileobj=io.BytesIO(raw)) as t:
 for member in t.getmembers():
  assert member.isfile() and not pathlib.Path(member.name).is_absolute() and '..' not in pathlib.Path(member.name).parts
  f=a.output/member.name;f.parent.mkdir(parents=True,exist_ok=True);f.write_bytes(t.extractfile(member).read())
repo=pathlib.Path('/private/tmp/mpk-w09-packed-pattern-proofs');root=pathlib.Path('/private/tmp/mpk-w09-identity-proofs');state=json.loads((a.output/'status.json').read_bytes());m=json.loads((root/'source-manifest.json').read_bytes())
expected=json.loads((root/'local/expected-certificates.json').read_bytes());assert state['exported_hashes']==expected
for group in ('source_hashes','fixture_hashes'):
 for n,h in m[group].items():assert data['audit']['git_blob_hashes'][n]==h and hashlib.sha256((repo/n).read_bytes()).hexdigest()==h,n
for stage in state['stages']:assert hashlib.sha256((a.output/(stage['stage']+'.log.txt')).read_bytes()).hexdigest()==stage['log_sha256']
for n,h in expected.items():assert hashlib.sha256((a.output/'certificates'/n).read_bytes()).hexdigest()==h
assert state['identity_proofs']==2 and state['supplied_binding_sequents']==531 and state['remaining_binding_sequents']==456 and state['original_application_proofs_pending']==987
(a.output/'fetch.json').unlink()
print(json.dumps(dict(status='passed',commit=a.commit,git_blob_files_verified=data['audit']['git_blob_files_verified'],exported_files_verified=len(expected),stage_count=len(state['stages']),archive_sha256=data['archive_sha256'])))
