import hashlib,json,pathlib
root=pathlib.Path('/private/tmp/mpk-w09-context-application-reports');repo=pathlib.Path('/private/tmp/mpk-w09-context-application-integration')
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
m=json.loads((root/'source-manifest.json').read_text());local=json.loads((root/'local/status.json').read_text());checks=json.loads((root/'checks-2/status.json').read_text());red=json.loads((root/'before-fix/status.json').read_text())
assert local['status']=='passed_targeted_tests_lint_format' and local['tests']==7 and len(local['stages'])==5 and all(x['exit_code']==0 for x in local['stages'])
for g in ('source_hashes','fixture_hashes'):
 for n,h in m[g].items():assert sha(repo/n)==h,n
for s in local['stages']:assert sha(root/'local'/(s['stage']+'.log.txt'))==s['log_sha256']
for n,h in local['test_binaries'].items():assert sha(pathlib.Path(n))==h
for n,h in local['exported_hashes'].items():assert sha(root/'local/certificates'/n)==h
assert red['exit_code']==101 and sha(root/'before-fix/test.log.txt')==red['log_sha256']
redlog=(root/'before-fix/test.log.txt').read_text();assert 'FAILED. 0 passed; 1 failed;' in redlog and 'Linkage' in redlog
binary=json.loads((root/'before-fix/binary.json').read_text());assert sha(pathlib.Path(binary['archive_path']))==binary['sha256']
assert sha(root/'before-fix/csharp_practical_ordinary_ownership_proofs.rs')==red['source_sha256']
assert checks['status']=='passed' and len(checks['stages'])==12
for backend,h in checks['binaries'].items():assert sha(pathlib.Path('/Users/kazuyoshitoshiya/mpk/target/release/mpk' if backend=='rust' else '/private/tmp/mpk-w09-scoped-construction/go-checker'))==h
accepted={}
for s in checks['stages']:
 c=s['case'];b=s['backend'];wrong='wrong' in c;assert s['exit_code']==(1 if wrong else 0)
 assert sha(root/'local/certificates'/(c+'.mpcert'))==s['input_file_sha256']
 for suffix,k in (('.json','report_sha256'),('.stderr.txt','stderr_sha256')):assert sha(root/'checks-2'/(c+'-'+b+suffix))==s[k]
 report=json.loads((root/'checks-2'/(c+'-'+b+'.json')).read_text());assert report['verdict']==('rejected' if wrong else 'accepted')
 if wrong:assert report['error_code' if b=='rust' else 'error_kind']==('KERNEL_CORE_CHECK' if b=='rust' else 'core_check')
 else:
  value=(root/'local/certificates'/(c+'.mpcert')).read_bytes();assert report['hashes']['certificate']==hashlib.sha256(b'MPK-MODULE-CERT-0.1\0'+value).hexdigest();assert report.get('axiom_count',0)==0
  if b=='rust':assert all(v==0 for v in report['axiom_report']['summary'].values())
  accepted[(c,b)]=(report['module'],report['declaration_count'],report['hashes'])
for c in ('true','false','nested-true','nested-false'):assert accepted[(c,'rust')]==accepted[(c,'go')]
old=repo/'develop/migrations/csharp-03/ordinary-foundation/verification-logs/control-predicates/with-context-refinement-candidates/context-certificates'
for n,h in json.loads((root/'unchanged-context-certificates.json').read_text()).items():assert sha(old/(n+'.mpcert'))==h and sha(root/'local/certificates'/(n+'.mpcert'))==h
for p,h in json.loads((root/'checks-2/inputs.json').read_text()).items():assert sha(pathlib.Path(p))==h
result=dict(status='passed_targeted_local_audit',source_files=756,fixture_files=309,tests=7,logs=5,checker_stages=12,fresh_checker_stages=7,reused_checker_stages=5,correct_certificates=4,wrong_certificates=2,unchanged_context_certificates=4,original_application_proof_ids_pending=987,application_scope_pending=True,full_t_gate='deferred to T06-W12',local_status_sha256=sha(root/'local/status.json'),dual_status_sha256=sha(root/'checks-2/status.json'),source_manifest_sha256=sha(root/'source-manifest.json'))
(root/'final-audit.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))
