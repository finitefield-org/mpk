"""Portable exact-byte reconstruction; native provisioning/acceptance remains pending.

No production guard is changed or overridden. Every final SDK file must match
its existing immutable descriptor, including two clean builds of both tools.
"""
from pathlib import Path
import base64,datetime,hashlib,io,json,shlex,shutil,subprocess,sys,tarfile,time,traceback
sys.path.insert(0,'scripts')
import rust_build_inputs as r
ROOT=Path.cwd();OUT=ROOT/'develop/migrations/go-rust-only/verification/sdk-portable-bytes-retry';OUT.mkdir(exist_ok=False)
PARENT=r.cache_parent();STAGE=PARENT/'.portable-sdk-stage-go-rust';WORK=PARENT/'.portable-sdk-work-go-rust-retry'
assert STAGE.is_dir() and not STAGE.is_symlink();WORK.mkdir();DOWNLOADS=WORK/'downloads';DOWNLOADS.mkdir()
BACKUP=Path('/private/tmp/mpk-go-rust-scope/download-archive-backup')
V=r.load_vector();EXPECTED,TRANSPORT=r.read_descriptor(V);LIMITS=r.FROZEN_RESOURCE_LIMITS
commands=[];downloads=[];phases=[]
def sha(path):
 h=hashlib.sha256()
 with path.open('rb') as f:
  for block in iter(lambda:f.read(1024*1024),b''):h.update(block)
 return h.hexdigest()
raw_inputs=[]
for p in [Path('scripts/rust_build_inputs.py'),Path('develop/specs/vectors/rust-build-inputs-v0.json'),Path('release/build-inputs/rust/build-inputs.json')]:
 b=p.read_bytes();raw_inputs.append({'path':str(p),'sha256':hashlib.sha256(b).hexdigest(),'bytes':len(b),'base64':base64.b64encode(b).decode()})
(OUT/'inputs.bytes.json').write_text(json.dumps({'schema':'mpk.original_bytes_set.v1','files':raw_inputs},indent=2)+'\n')
(OUT/'runner.py').write_bytes(Path(__file__).read_bytes())
original_download=r.download
original_run=subprocess.run
def download(url,expected,destination,maximum=2147483648):
 cached=BACKUP/destination.name;assert cached.is_file() and sha(cached)==expected,destination.name
 shutil.copyfile(cached,destination);original_download(url,expected,destination,maximum)
 row={'url':url,'sha256':sha(destination),'expected_sha256':expected,'bytes':destination.stat().st_size,'reused_complete_original_archive':str(cached)}
 assert row['sha256']==expected;downloads.append(row);(OUT/'downloads.json').write_text(json.dumps(downloads,indent=2)+'\n')
r.download=download
def save_command(command,result,seconds,label=None):
 index=len(commands);row={'command':command,'exit_code':result.returncode,'seconds':seconds,'label':label}
 for stream,data in [('stdout',result.stdout),('stderr',result.stderr)]:
  if data is None:continue
  if isinstance(data,str):data=data.encode()
  # Binary artifacts remain complete in the owned case-sensitive work area.
  p=(WORK/f'command-{index:03d}.stdout.tar') if stream=='stdout' and label else (OUT/f'command-{index:03d}.{stream}.log')
  p.write_bytes(data);row[stream]={'path':str(p),'sha256':hashlib.sha256(data).hexdigest(),'bytes':len(data)}
 commands.append(row);(OUT/'commands.json').write_text(json.dumps(commands,indent=2)+'\n')
 return row

def observed_run(command,*args,**kwargs):
 start=time.time();result=original_run(command,*args,**kwargs);save_command(command,result,time.time()-start);return result
subprocess.run=observed_run

def unpack_tar(data,destination,allowed_prefixes):
 destination.mkdir()
 with tarfile.open(fileobj=io.BytesIO(data),mode='r:') as archive:
  for member in archive.getmembers():
   name=member.name.rstrip('/');parts=Path(name).parts
   assert name and not name.startswith('/') and '..' not in parts and '\\' not in name
   assert any(name==p or name.startswith(p+'/') for p in allowed_prefixes),name
   target=destination/name
   if member.isdir():target.mkdir(parents=True,exist_ok=True);continue
   assert member.isfile() and not member.issym() and not member.islnk(),name
   target.parent.mkdir(parents=True,exist_ok=True);source=archive.extractfile(member);assert source is not None
   with target.open('xb') as f:shutil.copyfileobj(source,f)
   assert target.stat().st_size==member.size;target.chmod(member.mode & 0o777)

def compile_portable(label,recipe,source=None,cargo=False):
 name='mpk-sdk-bytes-'+label;memory=LIMITS['cgroup_memory_bytes'];tmp=LIMITS['writable_allocated_bytes'];inodes=LIMITS['writable_inodes']
 command=[r.docker_path(),'run',f'--name={name}','--pull=never','--platform=linux/amd64','--network=none','--ipc=none','--read-only',f'--user={r.OUTER_SANDBOX_ID}:{r.OUTER_SANDBOX_ID}','--hostname=mpk-build','--cap-drop=ALL','--security-opt=no-new-privileges','--security-opt=seccomp=unconfined',f'--pids-limit={LIMITS["cgroup_tasks"]}',f'--memory={memory}',f'--memory-swap={memory}',f'--ulimit=nofile={LIMITS["open_files_per_process"]}:{LIMITS["open_files_per_process"]}','--ulimit=fsize=-1:-1','--ulimit=nproc=-1:-1',f'--tmpfs=/mpk:rw,exec,nosuid,nodev,size={tmp},nr_inodes={inodes},mode=1777']
 for host,guest in [(STAGE/'toolchain','toolchain'),(STAGE/'native-sysroot','native-sysroot'),(STAGE/'vendor','vendor'),(STAGE/'cargo-home-seed/config.toml','cargo-home-seed.toml')]:command.append(f'--mount=type=bind,src={host},dst=/mpk/{guest},readonly')
 if source:command.extend([f'--mount=type=bind,src={source},dst=/mpk/frontend,readonly','--workdir=/mpk/frontend'])
 setup=f'umask 022; mkdir -p /mpk/home /mpk/cargo-home /mpk/tmp /mpk/target /mpk/work; cp /mpk/cargo-home-seed.toml /mpk/cargo-home/config.toml\n'
 if cargo:
  step=shlex.join(['/usr/bin/env','-i',*r.docker_build_environment(V),*recipe])
  script=setup+step+' > /mpk/tmp/build.stdout\ntest ! -s /mpk/tmp/build.stdout\ntar -C /mpk -cf - target/x86_64-unknown-linux-gnu/release/cargo-fuzz cargo-home\n'
 else:
  script=setup+'mkdir -p /mpk/tmp/libfuzzer-build/objects\n'+'\n'.join(shlex.join(step) for step in recipe)+'\ntar -C /mpk/tmp/libfuzzer-build -cf - libfuzzer.a\n'
 command.extend([r.RUNTIME_IMAGE,'/bin/sh','-ceu',script])
 start=time.time();result=r.run_bounded(command,stdout_limit=LIMITS['stdout_bytes'],stderr_limit=LIMITS['stderr_bytes'],docker_cleanup=(r.docker_path(),name));save_command(command,result,time.time()-start,label)
 assert result.returncode==0,(label,result.returncode,result.stderr.decode(errors='replace')[-3000:])
 destination=WORK/(label+'-result');unpack_tar(result.stdout,destination,['target/x86_64-unknown-linux-gnu/release/cargo-fuzz','cargo-home'] if cargo else ['libfuzzer.a'])
 if cargo:r.validate_post_run_cargo_home(destination/'cargo-home',V)
 return destination/('target/x86_64-unknown-linux-gnu/release/cargo-fuzz' if cargo else 'libfuzzer.a')

def phase(name,operation):
 start=time.time();operation();row={'name':name,'seconds':time.time()-start};phases.append(row);(OUT/'phases.json').write_text(json.dumps(phases,indent=2)+'\n');print(json.dumps(row),flush=True)
try:
 r.validate_project_templates(V)
 expected_files={item['path']:item for component in EXPECTED['components'] for item in component['files']}
 present=[]
 for source in sorted(STAGE.rglob('*')):
  assert not source.is_symlink(),source
  if not source.is_file():continue
  relative=source.relative_to(STAGE).as_posix();assert relative in expected_files,relative
  wanted=expected_files[relative];digest=sha(source)
  assert digest==wanted['sha256'] and source.stat().st_size==wanted['size_bytes'],relative
  present.append({'path':relative,'sha256':digest,'bytes':source.stat().st_size})
 (OUT/'authenticated-existing-files.json').write_text(json.dumps({'status':'passed','files':present,'missing_expected_files':sorted(set(expected_files)-{x['path'] for x in present}),'source_epoch':'Failed preceding portable materializer retained these original complete files; each full file is authenticated against the original immutable descriptor before reuse.'},indent=2)+'\n')
 print('Authenticated existing materialized files',len(present),flush=True)
 outputs=[compile_portable('cargo-fuzz-'+str(i),V['valid_descriptor']['cargo_fuzz']['build_argv'],source=STAGE/'tool-sources/cargo-fuzz',cargo=True) for i in range(2)]
 assert r.same_file_bytes(outputs[0],outputs[1]) and sha(outputs[0])==EXPECTED['cargo_fuzz']['executable_sha256']
 shutil.copyfile(outputs[0],STAGE/'toolchain/bin/cargo-fuzz');(STAGE/'toolchain/bin/cargo-fuzz').chmod(0o755)
 recipe=V['fuzz_smoke']['bounded_child_process_graph']['prebuilt_libfuzzer'];steps=[]
 for source in recipe['sources']:steps.append([x.replace('SOURCE',source).replace('OBJECT',source[:-4]+'.o') for x in recipe['compile_argv_template']])
 steps.extend([recipe['archive_argv'],recipe['ranlib_argv']])
 libraries=[compile_portable('libfuzzer-'+str(i),steps) for i in range(2)]
 assert r.same_file_bytes(libraries[0],libraries[1])
 installed=STAGE/recipe['installed_component_path'];shutil.copyfile(libraries[0],installed);installed.chmod(0o444)
 phase('frozen-native-runtime',lambda:r.materialize_native_runtime(V,STAGE,WORK))
 descriptor=r.create_production_descriptor(V,STAGE)
 assert descriptor==EXPECTED and r.descriptor_transport(descriptor)==TRANSPORT,'Every descriptor field and final file hash must match the immutable existing descriptor'
 r.normalize_cache_modes(STAGE,descriptor);r.validate_cache(descriptor,root=STAGE)
 final=PARENT/descriptor['build_inputs_sha256'];r.publish_cache(STAGE,final,descriptor);r.check_build_inputs()
 result={'status':'passed','cache':str(final),'build_inputs_sha256':descriptor['build_inputs_sha256'],'complete_descriptor_byte_equal':True,'cargo_fuzz_two_clean_builds_identical':True,'libfuzzer_two_clean_builds_identical':True,'native_provisioning_or_release_acceptance':False}
except BaseException as error:
 result={'status':'failed','error':repr(error),'code':getattr(error,'code',None)};(OUT/'exception.log').write_text(traceback.format_exc())
finally:
 result.update(finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),phases=phases,commands=commands,downloads=downloads,frozen_native_resource_limits=LIMITS,portable_numeric_limits={k:v for k,v in LIMITS.items() if k!='virtual_memory_bytes_per_process'},portable_emulator_virtual_address_space_limit=None,source_inputs_unchanged=all(sha(ROOT/row['path'])==row['sha256'] for row in raw_inputs),native_guards_changed=False,descriptor_or_hash_pins_changed=False,scope='Portable reconstruction authenticates exact SDK bytes only. The preceding attempt preserved the native 16-GiB per-process virtual-address limit and the emulator failed before target build-script execution. This trusted tool-byte reconstruction does not apply that emulator-host address limit; the complete existing descriptor must match. Native host, separate noexec views, cgroup delegation and source-execution isolation acceptance remain pending. This does not replace the official native provision/gate result.')
 (OUT/'result.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items() if k not in {'phases','commands','downloads'}}),flush=True)
 if result['status']!='passed':raise SystemExit(1)
