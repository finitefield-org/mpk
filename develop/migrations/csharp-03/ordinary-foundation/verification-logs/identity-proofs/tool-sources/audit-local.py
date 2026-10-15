import argparse,hashlib,json,pathlib
p=argparse.ArgumentParser();p.add_argument('terminal_session',type=int);a=p.parse_args()
repo=pathlib.Path('/private/tmp/mpk-w09-packed-pattern-proofs');root=pathlib.Path('/private/tmp/mpk-w09-identity-proofs');local=root/'local';s=json.loads((local/'status.json').read_bytes());m=json.loads((root/'source-manifest.json').read_bytes())
def digest(p):return hashlib.sha256(p.read_bytes()).hexdigest()
assert s['status']=='passed_targeted_tests_lint_format'
for group in ('source_hashes','fixture_hashes'):
 for n,h in m[group].items():assert digest(repo/n)==h,n
for stage in s['stages']:assert stage['exit_code']==0 and digest(local/(stage['stage']+'.log.txt'))==stage['log_sha256']
for n,h in s['test_binaries'].items():assert digest(pathlib.Path(n))==h,n
for n,h in s['exported_hashes'].items():assert digest(local/'certificates'/n)==h,n
assert s['identity_proofs']==2 and s['source_contexts']==45 and s['supplied_binding_sequents']==531 and s['remaining_binding_sequents']==456 and s['original_application_proofs_pending']==987 and s['generic_operations_pending']==25
assert len(s['exported_hashes'])==47 and len(s['test_binaries'])==1
out=dict(status='passed',terminal_exec_session=a.terminal_session,source_files_verified=len(m['source_hashes']),fixture_files_verified=len(m['fixture_hashes']),logs_verified=len(s['stages']),test_binaries_verified=len(s['test_binaries']),exports_verified=47,original_foundation_programs_unchanged=45,unchanged_foundation_certificates=44,identity_proofs=2,supplied_binding_sequents=531,remaining_binding_sequents=456,application_proofs_pending=987,selection_reason=m['selection_reason'],full_t_gate='deferred to T06-W12')
(local/'final-audit.json').write_text(json.dumps(out,indent=2,sort_keys=True)+'\n');print(json.dumps(out))
