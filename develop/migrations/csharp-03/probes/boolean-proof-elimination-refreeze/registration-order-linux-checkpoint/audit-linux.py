from pathlib import Path
import hashlib,json,re
b=Path(__file__).parent;out=b/'linux-evidence';expected=json.loads((b/'linux-control/expected.json').read_bytes());manifest=json.loads((out/'file-manifest.json').read_bytes())
h=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
for path,sha in manifest['files'].items():assert h(out/path)==sha,path
state=json.loads((out/'status.json').read_bytes());assert state['status']=='passed_exact_public_source_linux_tests_and_dual_checkers';assert state['source_commit']=='fd99c03c8e3535bcc31f0b9d1441d4b74de80ebf';assert state['cases']==34 and state['checker_stages']==68 and len(state['stages'])==82
pairs=[]
for case,wanted in expected.items():
 inp=out/(case+'.mpcert');data=inp.read_bytes();assert h(inp)==wanted['input_sha256'];reports=[]
 for backend in ['rust','go']:
  label=case+'-'+backend;stage=next(row for row in state['stages'] if row['stage']==label);report_path=out/(label+'.json');stderr=out/(label+'.stderr.txt');assert h(report_path)==stage['report_sha256'] and h(stderr)==stage['stderr_sha256'] and h(inp)==stage['input_sha256']
  rep=json.loads(report_path.read_bytes());local=wanted['reports'][backend];assert rep['verdict']==local['verdict'];good=rep['verdict']=='accepted';assert stage['exit_code']==(0 if good else 1)
  if backend=='rust' or good:assert rep['hashes']['certificate']==hashlib.sha256(b'MPK-MODULE-CERT-0.1\0'+data).hexdigest()
  if good:
   for key in ['module','hashes','declaration_count','axiom_count','axiom_report']:
    if key in local:assert rep[key]==local[key],(label,key)
   assert rep.get('axiom_count',0)==0
   if backend=='rust':
    assert all(v==0 for v in rep['axiom_report']['summary'].values());assert rep['axiom_report']['entries']==[];assert rep['axiom_report']['declaration_dependencies']==[]
   reports.append(dict(module=rep['module'],hashes=rep['hashes'],declaration_count=rep.get('declaration_count',0)))
  else:assert rep['error_code' if backend=='rust' else 'error_kind']==('KERNEL_CORE_CHECK' if backend=='rust' else 'core_check')
 if reports:assert reports[0]==reports[1],case
 pairs.append({'case':case,'verdict':wanted['reports']['rust']['verdict'],'input_sha256':h(inp),'matching_local_reports':True})
nonchecker=[s for s in state['stages'] if not s['stage'].endswith(('-rust','-go'))];assert len(nonchecker)==14 and all(s['exit_code']==0 for s in nonchecker)
for s in nonchecker:assert h(out/(s['stage']+'.log.txt'))==s['log_sha256']
go=(out/'go-core-defeq-tests.log.txt').read_text();top=len(re.findall(r'^--- PASS:',go,re.M));sub=len(re.findall(r'^    --- PASS: TestBoolCasesCertificatesMatchRust/',go,re.M));assert top==16 and sub==31
receipt={'status':'passed_exact_public_fix_source_linux_selected_tests_lint_format_and_68_dual_checker_stages','source_commit':state['source_commit'],'rust_selected_tests':state['rust_tests'],'go_top_level_tests':top,'go_shared_fixture_subcases':sub,'shared_positive_cases':10,'shared_negative_cases':21,'predecessor_cases':3,'cases':34,'checker_stages':68,'build_test_lint_format_stages':14,'case_reports':pairs,'linux_source_clean_after':state['source_clean_after'],'linux_binaries':state['binaries'],'full_t01_gate':'deferred_to_final_W10','full_t06_gate':'deferred_to_W12','first_launch_failure':'linux-launch-first-failure.json','initial_missing_embedded_input_compile_exit_code':101, 'git_blob_files_verified':json.loads((out/'launch.json').read_bytes())['git_blob_files_verified'], 'successful_unchanged_prefix_stages_reused':6,'application_proof_ids_pending':987}
(b/'linux-final-audit.json').write_text(json.dumps(receipt,sort_keys=True,indent=2)+'\n');print(json.dumps({k:v for k,v in receipt.items() if k not in ['case_reports','linux_binaries']}))
