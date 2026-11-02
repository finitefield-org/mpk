"""Affected pinned-compiler frontend checks; no native-isolation acceptance claim."""
from pathlib import Path
import base64,datetime,hashlib,json,subprocess,sys,time,traceback
sys.path.insert(0,'scripts')
import rust_build_inputs as rust
ROOT=Path.cwd();MODE=sys.argv[1];TAG=sys.argv[2];assert MODE in {'update','check','protocol','contracts'}
OUT=ROOT/'develop/migrations/go-rust-only/verification'/('rust-frontend-'+TAG);OUT.mkdir(exist_ok=False)
PROJECT=ROOT/'rust-tools/rust2vir';vector=rust.load_vector();descriptor,cache=rust.check_build_inputs()
index=PROJECT/'testdata/positive/frontend-index.json';manifest=json.loads(index.read_bytes())
outputs={Path(row['path']).parent.as_posix() for case in manifest['cases'] for row in case['artifacts']}
output_paths=[PROJECT/'testdata/positive'/p for p in sorted(outputs)];output_paths.append(index)
source=[];generated=[]
for p in sorted(PROJECT.rglob('*')):
 if not p.is_file() or 'target' in p.relative_to(PROJECT).parts:continue
 raw=p.read_bytes();row={'path':str(p.relative_to(ROOT)),'sha256':hashlib.sha256(raw).hexdigest(),'bytes':len(raw),'base64':base64.b64encode(raw).decode()}
 writable=MODE=='update' and any(p==x or (x.is_dir() and p.is_relative_to(x)) for x in output_paths)
 (generated if writable else source).append(row)
for name,rows in [('source-inputs.bytes.json',source),('generated-before.bytes.json',generated)]:
 (OUT/name).write_text(json.dumps({'schema':'mpk.original_bytes_set.v1','files':rows},indent=2)+'\n')
(OUT/'runner.py').write_bytes(Path(__file__).read_bytes())
target=Path('/private/tmp/mpk-go-rust-scope/rust-selected-target');target.mkdir(exist_ok=True);target.chmod(0o777)
environment=rust.docker_build_environment(vector)
if MODE=='update':environment.append('MPK_UPDATE_RUST_POSITIVE_CORPUS=1')
tests=['contracts'] if MODE=='contracts' else ['positive_corpus'] if MODE in {'update','check'} else ['successor_protocol_identity','driver_protocol','frontend_envelope','fuzz_regressions','session','positive_corpus']
name='mpk-scope-rust-'+TAG
cmd=[rust.docker_path(),'run','--name='+name,'--pull=never','--platform=linux/amd64','--network=none','--ipc=none','--read-only','--user=501:501','--cap-drop=ALL','--security-opt=no-new-privileges','--security-opt=seccomp=unconfined','--pids-limit=256','--memory=12g','--memory-swap=12g','--ulimit=nofile=4096:4096']
for p in ['home','cargo-home','tmp','work']:cmd.append('--tmpfs=/mpk/'+p+':rw,nosuid,nodev,noexec,mode=1777')
mounts=[(PROJECT,'/mpk/frontend',True),(cache/'toolchain','/mpk/toolchain',True),(cache/'vendor','/mpk/vendor',True),(cache/'cargo-home-seed/config.toml','/mpk/cargo-home/config.toml',True),(cache/'native-sysroot','/mpk/native-sysroot',True),(cache/'native-runtime','/mpk/native-runtime',True),(cache/'native-runtime/lib64','/lib64',True),(cache/'native-runtime/lib/x86_64-linux-gnu','/lib/x86_64-linux-gnu',True),(cache/'native-runtime/lib/x86_64-linux-gnu','/usr/lib/x86_64-linux-gnu',True),(target,'/mpk/target',False)]
if MODE=='update':mounts += [(p,'/mpk/frontend/'+p.relative_to(PROJECT).as_posix(),False) for p in output_paths]
for host,guest,readonly in mounts:cmd.append(f'--mount=type=bind,src={host},dst={guest}'+(',readonly' if readonly else ''))
cmd.extend(['--workdir=/mpk/frontend',rust.RUNTIME_IMAGE,'/usr/bin/env','-i',*environment,'/mpk/toolchain/bin/cargo','test','--locked','--offline','--target','x86_64-unknown-linux-gnu','--jobs','1'])
for test in tests:cmd.extend(['--test',test])
cmd.extend(['--','--nocapture','--test-threads=1'])
start=time.time()
try:
 r=rust.run_bounded(cmd,stdout_limit=67_108_864,stderr_limit=2_097_152,docker_cleanup=(rust.docker_path(),name))
 result={'status':'passed' if r.returncode==0 else 'failed','exit_code':r.returncode}
 for stream,raw in [('stdout',r.stdout),('stderr',r.stderr)]:
  p=OUT/(stream+'.log');p.write_bytes(raw);result[stream]={'path':str(p.relative_to(ROOT)),'sha256':hashlib.sha256(raw).hexdigest(),'bytes':len(raw)}
except BaseException as error:
 result={'status':'failed','error':repr(error)};(OUT/'exception.log').write_text(traceback.format_exc())
finally:
 result.update(command=cmd,seconds=time.time()-start,mode=MODE,selected_tests=tests,source_inputs_unchanged=all(hashlib.sha256((ROOT/row['path']).read_bytes()).hexdigest()==row['sha256'] for row in source),build_inputs_sha256=descriptor['build_inputs_sha256'],native_release_acceptance=False,platform='local emulated Linux/amd64; exact frozen Rust compiler/linker/runtime bytes',selection_reason='Registry identity and generated driver/transports changed. Exercise the affected protocol/envelope/session/parser and twice-compiled positive source corpus; untouched arithmetic/lowering families are not rerun as a whole suite. Only declared generated artifact directories/index are writable in update mode; source bytes remain read-only.',finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat())
 if not result['source_inputs_unchanged']:result['status']='failed'
 (OUT/'result.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items() if k in {'status','exit_code','error','source_inputs_unchanged','seconds'}}),flush=True)
 if result['status']!='passed':raise SystemExit(1)
