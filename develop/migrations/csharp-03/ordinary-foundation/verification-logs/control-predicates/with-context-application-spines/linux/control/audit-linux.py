import hashlib,json,pathlib,subprocess
root=pathlib.Path('/root/mpk-w09-context-application-f02dcb32-linux');launch=json.loads((root/'launch.json').read_text());status=json.loads((root/'status.json').read_text());repo=pathlib.Path(launch['repo']);control=pathlib.Path(launch['control'])
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
head=subprocess.check_output(['/usr/bin/git','rev-parse','HEAD'],cwd=repo,text=True).strip();assert head==launch['commit']=='f02dcb32ee0ff420a86083ea95067879455dec81'
assert subprocess.check_output(['/usr/bin/git','status','--porcelain'],cwd=repo,text=True).strip()==''
assert status['status']=='passed_targeted_tests_lint_format' and status['tests']==7 and len(status['stages'])==5 and all(s['exit_code']==0 for s in status['stages'])
m=json.loads((root/'source-manifest.json').read_text())
for group in ('source_hashes','fixture_hashes'):
 for n,h in m[group].items():assert sha(repo/n)==h,n
for n,h in launch['control_hashes'].items():assert sha(control/n)==h,n
assert launch['git_blob_files_verified']==1065
for s in status['stages']:assert sha(root/(s['stage']+'.log.txt'))==s['log_sha256']
for n,h in status['test_binaries'].items():assert sha(pathlib.Path(n))==h
exports={p.name:sha(p) for p in (root/'certificates').iterdir() if p.is_file()};assert len(exports)==7 and exports==status['exported_hashes']==json.loads((control/'expected-certificates.json').read_text())
counts={'context-units':3,'candidate-units':3,'substitution-unit':1}
for n,count in counts.items():assert f'test result: ok. {count} passed; 0 failed;' in (root/(n+'.log.txt')).read_text()
result=dict(status='passed_exact_public_source_linux_audit',source_commit=head,source_files=756,fixture_files=309,git_blob_files_verified_before_start=1065,clean_checkout=True,tests=7,logs=5,executed_test_binary_hashes=status['test_binaries'],matching_local_certificates=7,source_manifest_sha256=sha(root/'source-manifest.json'),runner_status_sha256=sha(root/'status.json'),launch_sha256=sha(root/'launch.json'),source_control_hashes=launch['control_hashes'],application_scope_pending=True,full_t_gate='deferred to T06-W12')
(root/'final-audit.json').write_text(json.dumps(result,indent=2,sort_keys=True)+'\n');print(json.dumps(result))
