"""Outside sandbox: independently read retained whole original Git/source inputs."""
from pathlib import Path
import base64,datetime,hashlib,json,re,subprocess
root=Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
base='528b599aacfe439d4abd8f50e046a754c9a24435'
def seal(p):
 raw=Path(p).read_bytes();return {'path':str(p),'sha256':hashlib.sha256(raw).hexdigest(),'bytes':len(raw)}
def git(*args):return subprocess.check_output(['/usr/bin/git',*args])
assert git('rev-parse','HEAD').decode().strip()==base
receipts=[root/'inputs.json',*sorted(root.glob('source-epochs/*/inputs.json'))]
assert len(receipts)==6
objects={};observations=0;retained=0
with subprocess.Popen(['/usr/bin/git','cat-file','--batch'],stdin=subprocess.PIPE,stdout=subprocess.PIPE) as process:
 for path in receipts:
  r=json.loads(path.read_text());assert r['status']=='complete_original_inputs_retained' and r['base_commit']==base
  assert len(r['sources'])==964 and r['new_or_changed_complete_inputs_retained']==4
  for row in r['sources']:
   oid=row.get('original_git_blob')
   if oid:
    if oid not in objects:
     process.stdin.write(oid.encode()+b'\n');process.stdin.flush();fields=process.stdout.readline().decode().split();assert fields[:2]==[oid,'blob']
     raw=process.stdout.read(int(fields[2]));assert process.stdout.read(1)==b'\n'
     assert hashlib.sha1(b'blob '+str(len(raw)).encode()+b'\0'+raw).hexdigest()==oid
     objects[oid]=(hashlib.sha256(raw).hexdigest(),len(raw))
    assert objects[oid]==(row['original_git_blob_sha256'],row['original_git_blob_bytes'])
   if row['whole_current_bytes_equal_original_git_blob']:
    assert objects[oid]==(row['sha256'],row['bytes'])
   else:
    assert seal(row['complete_changed_input'])=={'path':row['complete_changed_input'],'sha256':row['sha256'],'bytes':row['bytes']};retained+=1
   if path.parent.name=='final':assert seal(row['path'])['sha256']==row['sha256'] and seal(row['path'])['bytes']==row['bytes']
   observations+=1
 process.stdin.close();assert process.wait()==0
assert retained==24 and observations==5784
checks=[]
for mode in ['selected','lint','format']:
 p=root/'final'/mode/'result.json';r=json.loads(p.read_text());assert r['status']=='passed' and r['whole_inputs_unchanged_during_execution']
 for field in ['stdout','stderr']:assert seal(r[field]['path'])==r[field]
 for row in r['sources']:assert seal(row['path'])['sha256']==row['sha256']
 if mode=='selected':assert len(r['passed_tests'])==14 and all('csharp' not in n.lower() for n in r['passed_tests'])
 checks.append(seal(p))
p=root/'final/linux/result.json';linux=json.loads(p.read_text());assert linux['status']=='passed' and linux['whole_inputs_unchanged_during_execution']
assert len(linux['checks'])==4 and all(r['exit_code']==0 for r in linux['checks'])
for check in linux['checks']:
 for field in ['stdout','stderr']:assert seal(check[field]['path'])==check[field]
 cmd=check['command'];assert '--network=none' in cmd and '--read-only' in cmd and '--cap-drop=ALL' in cmd
 assert cmd[cmd.index('--platform')+1]=='linux/amd64'
selected=(p.parent/'selected.stdout.log').read_text();policy=(p.parent/'policy.stdout.log').read_text()
linux_roots=re.findall(r'^test ([^ ]+) \.\.\. ok$',selected,re.M)
policy_roots=re.findall(r'^test ([^ ]+) \.\.\. ok$',policy,re.M)
assert len(linux_roots)==20 and len(policy_roots)==4
assert all('csharp' not in n.lower() for n in linux_roots+policy_roots)
assert 'actual non-native practical-Go preparation rejected before cgroup mutation' in (p.parent/'selected.stderr.log').read_text()
host=(p.parent/'host.stdout.log').read_text();assert 'Features' in host and 'rustc 1.95.0' in host
assert linux['native_Linux_release_acceptance_claimed'] is False
checks.append(seal(p))
for phase in ['initial/linux','indexed/linux','pidfd-regression/linux','probe-classification/regression']:
 p=root/phase/'result.json';r=json.loads(p.read_text());assert r['status']=='failed'
 if 'checks' in r:
  for check in r['checks']:
   for field in ['stdout','stderr']:assert seal(check[field]['path'])==check[field]
 else:
  assert r['whole_inputs_unchanged_during_execution']
  for field in ['stdout','stderr']:assert seal(r[field]['path'])==r[field]
assert 'no matching package named `serde_json` found' in (root/'initial/linux/policy.stderr.log').read_text()
assert 'cargo-clippy' in (root/'indexed/linux/lint.stderr.log').read_text()
assert 'go_policy_preserves_original_initial_pidfd_probe ... FAILED' in (root/'pidfd-regression/linux/pidfd-regression.stdout.log').read_text()
assert 'left: Err(Unavailable)' in (root/'probe-classification/regression/stderr.log').read_text()
assert 'right: Err(Killed)' in (root/'probe-classification/regression/stderr.log').read_text()
derived=json.loads((root/'derived-clone-controls.json').read_text())
assert set(derived['exact_required_clone_flags'].values())=={0x50f00,0x4111,0x5111,0x5100}
code=Path('crates/mpk-linux-sandbox/src/go_policy.rs').read_text()
constants={name:int(value,16) for name,value in re.findall(r'^const (THREAD|PROCESS|PROCESS_PIDFD|PIDFD_PROBE): u32 = (0x[0-9a-f_]+);',code,re.M)}
assert set(constants.values())==set(derived['exact_required_clone_flags'].values())
for p in (root/'pinned-go').glob('*.bytes.json'):
 r=json.loads(p.read_text());raw=base64.b64decode(r['base64'],validate=True);assert len(raw)==r['bytes'] and hashlib.sha256(raw).hexdigest()==r['raw_sha256']
prior=root.parent/'native-observation'
frozen=json.loads((prior/'frozen-design-preservation.json').read_text());assert len(frozen['inputs'])==255
for row in frozen['inputs']:assert seal(row['path'])==row
(root/'frozen-design-preservation.json').write_text(json.dumps({**frozen,'prior_complete_manifest':seal(prior/'frozen-design-preservation.json'),'observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat()},indent=2)+'\n')
frontend=prior/'compatibility/diagnostic/captured-inputs.json';r=json.loads(frontend.read_text());assert len(r['build_inputs'])==184
for obj in r['build_inputs']:
 raw=(Path('go-tools/go2vir')/obj['path']).read_bytes();assert len(raw)==obj['bytes'] and hashlib.sha256(raw).hexdigest()==obj['raw_sha256']
terminal=json.loads((root/'terminal-handles.json').read_text());assert terminal['status']=='passed' and terminal['all_actual_validation_handles_terminal'] and not terminal['live_matching_drivers']
for row in terminal['terminal_results']:assert seal(row['path'])=={k:row[k] for k in ['path','sha256','bytes']}
record={'schema':'mpk.go_practical.native_host_checkpoint.v1','status':'scoped_checks_passed','entry_commit':base,'observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),
 'checks':checks,'Darwin_affected_roots':14,'Linux_affected_roots':20,'Linux_BPF_roots':4,'original_input_epochs':len(receipts),'whole_original_input_observations':observations,'distinct_original_Git_blobs_actually_read':len(objects),'complete_changed_input_preimages':retained,
 'pinned_original_Go_source_members':3,'exact_pinned_clone_flag_combinations':4,'single_bit_clone_controls':256,
 'original_pidfd_probe_refusal_reproduced_then_fixed':True,'original_probe_resource_classification_failure_reproduced_then_fixed':True,
 'Go_native_preparation_is_source_free_and_opaque':True,'non_native_cpu_rejects_before_cgroup_state_mutation':True,'retained_profile_limits_and_error_classification_preserved':True,
 'full_private_original_input_context_execution_authority_complete':False,'native_Linux_release_acceptance_claimed':False,'actual_native_Go_syscall_budget_workload_acceptance_measured':False,
 'earlier_Go_frontend_inputs_unchanged':184,'frozen_members_unchanged':255,'terminal_handles':seal(root/'terminal-handles.json'),'all_actual_validation_handles_terminal':True,
 'CSharp_tests_executed':False,'agents_used':False,'GitHub_Actions_used':False,'T02_W01_complete':False,'R06_status':'pending','registered_context_or_source_program_application_proof':False,
 'full_T02_gate_passed':False,'full_T02_gate':'deferred to T02-W04: ./scripts/check-fast.sh --skip-csharp',
 'scope':'Private native frontend-host preparation, original frozen resource selection, source-free probe, finite Go syscall/thread/process policy, typed operational failure preservation. Four exact pinned Go clone combinations derive from complete original sources; unknown bits/network/namespace/privilege calls reject. No retained source-kind relabeling or private-context/source-proof activation.',
 'remaining':'Complete production immutable original-input/kernel lifecycle/context and consulted namespace/selected closure, full56-family/all6 common members/19 theorem source/carrier/nominal/type/operation/storage/lifetime/state/program foundation, compiler-derived complete operation/VC roots/private descriptor/registry/context/retained policies/all169 demands/T02-T11 remain required. Native Linux resource/syscall enforcement/full corpus fit and proof-child route remain mandatory T07 gates.'}
(root/'verification.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({k:record[k] for k in ['status','Darwin_affected_roots','Linux_affected_roots','Linux_BPF_roots','whole_original_input_observations','distinct_original_Git_blobs_actually_read','R06_status']}))
