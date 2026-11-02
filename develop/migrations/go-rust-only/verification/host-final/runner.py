from pathlib import Path
import base64,datetime,hashlib,json,subprocess,time
ROOT=Path.cwd();OUT=ROOT/'develop/migrations/go-rust-only/verification/host-final';OUT.mkdir(exist_ok=False)
paths=[p for p in (ROOT/'crates').rglob('*') if p.is_file() and p.suffix in {'.rs','.toml'}]+[ROOT/'Cargo.toml',ROOT/'Cargo.lock']+[p for p in (ROOT/'fixtures').rglob('*.json') if p.is_file()]
inputs=[]
for p in sorted(paths):
 b=p.read_bytes();inputs.append({'path':str(p.relative_to(ROOT)),'sha256':hashlib.sha256(b).hexdigest(),'bytes':len(b),'base64':base64.b64encode(b).decode()})
(OUT/'inputs.bytes.json').write_text(json.dumps({'schema':'mpk.original_bytes_set.v1','files':inputs},indent=2)+'\n');(OUT/'runner.py').write_bytes(Path(__file__).read_bytes())
vc=['release_bundle','rust_calls','rust_positive_corpus','rust_runtime_safety','safety_profiles','semantic_profile_registry_runtime','successor_go_migration','successor_rust_migration','successor_source_artifacts','successor_vc','go_vir_corpus']
cli=['frontend_runner','policy_cli','policy_scan','successor_frontend_protocol','successor_go_protocol','successor_rust_protocol','successor_go_release','successor_rust_release','successor_ai_explain','successor_policy_verify']
def test(package,targets):
 c=['cargo','test','--locked','-p',package]
 for t in targets:c += ['--test',t]
 return c
commands=[('vc',test('mpk-vc',vc)),('cli',test('mpk-cli',cli)),('api',['cargo','test','--locked','-p','mpk-api','--lib']),('clippy',['cargo','clippy','--locked','-p','mpk-cli','-p','mpk-vc','-p','mpk-api','--all-targets','--','-D','warnings'])];rows=[]
for label,command in commands:
 start=time.time();r=subprocess.run(command,capture_output=True);row={'label':label,'command':command,'exit_code':r.returncode,'seconds':time.time()-start}
 for stream,data in [('stdout',r.stdout),('stderr',r.stderr)]:
  p=OUT/(label+'.'+stream+'.log');p.write_bytes(data);row[stream]={'path':str(p.relative_to(ROOT)),'sha256':hashlib.sha256(data).hexdigest(),'bytes':len(data)}
 rows.append(row);print(label,r.returncode,flush=True)
 if r.returncode:break
unchanged=all(hashlib.sha256((ROOT/x['path']).read_bytes()).hexdigest()==x['sha256'] for x in inputs)
result={'status':'passed' if len(rows)==len(commands) and all(r['exit_code']==0 for r in rows) and unchanged else 'failed','checks':rows,'source_inputs_unchanged':unchanged,'native_release_acceptance':False,'selection_reason':'The final registry/binary/artifact identities affect shared Go/Rust dispatch, typed imports, VC/certificate links, rejection, API sessions, and their CLI owners. Select only these consumer targets plus required lint; unchanged kernel/foundation/math suites are deferred to the cumulative T02 final W gate.','full_gate_deferred_to':'GO-PRAC-T02-W04: ./scripts/check-fast.sh','finished_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat()}
(OUT/'result.json').write_text(json.dumps(result,indent=2)+'\n');print(result['status'],flush=True);raise SystemExit(result['status']!='passed')
