"""Derive pinned Go clone requirements from complete preserved original sources.

This is a source inspection/control derivation, not kernel execution acceptance
or a source/program semantic proof.
"""
from pathlib import Path
import base64, hashlib, json, re
root=Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
originals={}
for p in (root/'pinned-go').glob('*.bytes.json'):
 row=json.loads(p.read_text());raw=base64.b64decode(row['base64'],validate=True)
 assert len(raw)==row['bytes'] and hashlib.sha256(raw).hexdigest()==row['raw_sha256']
 originals[Path(row['original_path']).name]=(p,row,raw.decode())
compiler=json.loads(Path('develop/migrations/go-practical/t02-w01/verification/native-observation/owned-current/linux-owned/replay-image.json').read_text())['compiler_inventory']
for name,(p,row,text) in originals.items():
 matched=[entry for entry in compiler['files'] if entry['path'].endswith('/'+name)]
 # Runtime and syscall source basenames are unique within the compiler src tree.
 matched=[entry for entry in matched if entry['sha256']==row['raw_sha256'] and entry['size_bytes']==row['bytes']]
 assert len(matched)==1,(name,matched)
exec_source=originals['exec_linux.go'][2];runtime_source=originals['os_linux.go'][2];pidfd_source=originals['pidfd_linux.go'][2]
constants={name:int(value,0) for name,value in re.findall(r'^\s*(CLONE_[A-Z_]+)\s*=\s*(0x[0-9a-fA-F]+|[0-9]+)',exec_source,re.M)}
assert constants['CLONE_VM']==0x100 and constants['CLONE_VFORK']==0x4000 and constants['CLONE_PIDFD']==0x1000
probe_start=exec_source.index('func doCheckClonePidfd(')
probe=exec_source[probe_start:]
assert 'flags := uintptr(CLONE_VFORK | CLONE_VM | CLONE_PIDFD)' in probe
assert 'if err := checkClonePidfd(); err != nil {' in pidfd_source
assert 'return NewSyscallError("pidfd_check_clone", err)' in pidfd_source or 'return err' in pidfd_source
thread_expression=runtime_source.split('cloneFlags = ',1)[1].split('\n)',1)[0]
thread_expression=re.sub(r'/\*.*?\*/','',thread_expression,flags=re.S)
names=re.findall(r'_CLONE_[A-Z_]+',thread_expression)
assert names==['_CLONE_VM','_CLONE_FS','_CLONE_FILES','_CLONE_SIGHAND','_CLONE_SYSVSEM','_CLONE_THREAD']
thread=0
for name in names:thread|=constants[name[1:]]
process=constants['CLONE_VM']|constants['CLONE_VFORK']|17
assert 'flags |= uintptr(SIGCHLD)' in exec_source and 'flags |= CLONE_PIDFD' in exec_source
flags={'runtime_thread':thread,'child_without_pidfd':process,'child_with_pidfd':process|constants['CLONE_PIDFD'],
       'initial_pidfd_probe_without_exit_signal':constants['CLONE_VM']|constants['CLONE_VFORK']|constants['CLONE_PIDFD']}
assert set(flags.values())=={0x50f00,0x4111,0x5111,0x5100}
record={'status':'requirements_derived_from_complete_original_pinned_sources','exact_required_clone_flags':flags,
 'originals':[{'retained_path':str(p),'original_path':row['original_path'],'sha256':row['raw_sha256'],'bytes':row['bytes']} for p,row,_ in originals.values()],
 'original_source_members_equal_actual_Linux_compiler_inventory':True,'original_Go_release':'go1.25.0',
 'kernel_native_acceptance_or_source_program_proof_claimed':False,
 'reason':'Go1.25 checks clone(CLONE_PIDFD) with no exit signal before enabling its pidfd process path. Denial would force an unintended no-pidfd fallback even on the required supported native host. Preserve that exact fourth combination, not arbitrary clone flags.'}
(root/'derived-clone-controls.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps(flags))
