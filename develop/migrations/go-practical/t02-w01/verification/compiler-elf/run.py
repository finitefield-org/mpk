"""Complete source epochs for pinned original ELF section/decompression checks."""
from pathlib import Path
import datetime, hashlib, json, os, shutil, subprocess, sys, tarfile, time

ROOT=Path(__file__).resolve().parent
REPO=Path.cwd()
SDK=Path('/private/tmp/mpk-go125-20261025-download/go')
CHANGED=['crates/mpk-vc/src/go_practical_type_keys/compiler_inputs/build_info/formats.rs', 'crates/mpk-vc/src/go_practical_type_keys/compiler_inputs/build_info/formats/elf.rs', 'crates/mpk-vc/src/go_practical_type_keys/compiler_inputs/build_info/formats/elf/compression.rs', 'crates/mpk-vc/src/go_practical_type_keys/compiler_inputs/build_info/formats/elf/compression/hash.rs', 'crates/mpk-vc/tests/go_practical_compiler_elf.rs', 'third_party/ruzstd-go125/src/lib.rs', 'third_party/ruzstd-go125/src/mpk_go125_blocks.rs', 'third_party/ruzstd-go125/src/decoding/block_decoder.rs', 'third_party/ruzstd-go125/src/decoding/sequence_execution.rs']
GO_CHANGED=['go-tools/go2vir/practical_compiler_elf_test.go']
BORROWED=[
 'compiler-inputs/native/native-originals',
 'compiler-build-info/fixtures',
 'module-manifests/native-toolchain/native-originals',
 'graph-import/native/native-originals',
 'observation-json/owner-large/native-originals',
 'root-expansion/native-final/native-originals',
 'root-expansion/owner/native-originals',
 'module-observations/native/native-originals',
 'export-inputs/native/native-originals',
 'module-graphs/native-initial/native-originals/graph-owned',
 'graph-inputs/native/native-originals/graph-owned',
 'module-graphs/edges-final/native-originals',
 'module-graphs/replacements-final/native-originals',
 'module-graphs/toolchains-final/native-originals',
]
CACHE={}
def seal(path):
 path=Path(path);before=path.stat()
 key=(str(path.resolve()),before.st_dev,before.st_ino,before.st_size,before.st_mtime_ns,before.st_ctime_ns,before.st_mode)
 if key not in CACHE:
  raw=path.read_bytes();after=path.stat()
  assert (before.st_dev,before.st_ino,before.st_size,before.st_mtime_ns,before.st_ctime_ns,before.st_mode)==(after.st_dev,after.st_ino,after.st_size,after.st_mtime_ns,after.st_ctime_ns,after.st_mode)
  CACHE[key]={'path':str(path.resolve()),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'mode':before.st_mode&0o777}
 return CACHE[key]
def write(path,value):path.write_text(json.dumps(value,indent=2)+'\n')
def main():
 epoch,mode=sys.argv[1:];assert epoch.replace('-','').isalnum()
 out=ROOT/epoch;out.mkdir(exist_ok=False)
 formatting=mode in ['format-write','format','go-format-write','go-format']
 files={Path(__file__).resolve(),ROOT.parent/'compiler-build-info/fixtures.py'};files.update(p for p in (ROOT/'fixtures').rglob('*') if p.is_file());files.update(ROOT.glob('*.py'));preserved=set()
 if not formatting and mode not in ['native','native-zstd','vet']:files.update(p for p in (ROOT/'fixtures').rglob('*') if p.is_file());files.update(p for p in (ROOT/'zstd/observations').rglob('*') if p.is_file())
 if not formatting and mode not in ['native','native-zstd','vet']:
  pass
 if formatting:
  files.update(REPO/name for name in (GO_CHANGED if mode.startswith('go-') else CHANGED))
  files.update(REPO/name for name in ['rustfmt.toml','.rustfmt.toml'] if (REPO/name).is_file())
 else:
  selected=['go-tools/go2vir'] if mode in ['native','native-zstd','vet'] else ['crates','third_party','proofs/std','go-tools/go2vir','go-tools/mpk-checker-ref','develop/specs/vectors','fixtures','release/bundles','.cargo']
  for name in selected:
   files.update(p for p in (REPO/name).rglob('*') if p.is_file())
  if mode not in ['native','native-zstd','vet']:files.update(REPO/name for name in ['Cargo.toml','Cargo.lock'])
  for name in ([] if mode in ['native','native-zstd','vet'] else BORROWED):
   originals={p for p in (ROOT.parent/name).rglob('*') if p.is_file()}
   files.update(originals);preserved.update(originals)
  if mode in ['rust','roles','graph','links','checksum','clippy','clippy-links','unit','fresh']:
   files.update(p for p in (ROOT/'native/native-originals').rglob('*') if p.is_file())
  if mode in ['fresh','rust','graph','clippy','clippy-links']:files.update(p for p in (ROOT/'native/native-originals').rglob('*') if p.is_file())
 sdk_receipt=ROOT.parent/'module-graphs/original-sdk/inputs.json'
 sdk=json.loads(sdk_receipt.read_bytes())
 assert sdk['sources']==[seal(row['path']) for row in sdk['sources']]
 sources=[seal(p) for p in sorted(files)]
 with tarfile.open(out/'inputs.tar.gz','x:gz',dereference=True) as archive:
  for p in sorted(files-preserved):archive.add(p,arcname=str(p.relative_to(REPO)),recursive=False)
 receipt=out/'inputs.json'
 write(receipt,{'basis_commit':'133d465e86f455e0bf5c5436ba43a87b2bb02c68','sources':sources,'complete_archive':seal(out/'inputs.tar.gz'),'preserved_original_paths':[str(p) for p in sorted(preserved)],'whole_original_sdk':seal(sdk_receipt),'sdk_archive':sdk['complete_archive'],'hash_cache_policy':'Read/hash every full original before launch with stable before/after stat; reuse only identical device/inode/size/mtime_ns/ctime_ns/mode, otherwise reread/hash all bytes. No source membership or raw preimage is omitted.', 'scope':'Keep all original source/package/fixture/proof objects; exact original earlier committed paths are reused without replacing or dropping bytes.'})
 rust=['cargo','test','--offline','--locked','-p','mpk-vc']
 targets=['--test','go_practical_compiler_elf','--test','go_practical_compiler_inputs']
 commands={
  'fixtures':[sys.executable,str(ROOT/'fixtures.py')],
  'format-write':['rustfmt','--edition=2021','--config','skip_children=true',*CHANGED],
  'format':['rustfmt','--edition=2021','--check','--config','skip_children=true',*CHANGED],
  'go-format-write':[str(SDK/'bin/gofmt'),'-w',*GO_CHANGED],
  'go-format':[str(SDK/'bin/gofmt'),'-l',*GO_CHANGED],
  'native':[str(SDK/'bin/go'),'test','-json','-count=1','-timeout=40m','-run','^TestPracticalCompilerBuildInfoOriginalBytesAndGrammar$'],
  'native-zstd':[str(SDK/'bin/go'),'test','-json','-count=1','-timeout=40m','-run','^TestPracticalElfCompleteOriginalZstandardStreams$'],
  'owner':[str(SDK/'bin/go'),'test','-json','-count=1','-timeout=40m','-run','^TestPracticalCompilerInputsKeepWholeOriginalPreparedSDKAndPackages$'],
  'unit':[*rust,'--lib','go_practical_type_keys::observation_json::tests','--','--nocapture','--test-threads=1'],
  'vet':[str(SDK/'bin/go'),'vet','.'],
  'roles':[*rust,'--test','go_practical_compiler_elf','--','--nocapture','--test-threads=1'],
  'rust':[*rust,*targets,'--','--nocapture','--test-threads=1'],
  'graph':[*rust,'--test','go_practical_compiler_inputs','--','--nocapture','--test-threads=1'],
  'fresh':[*rust,'--test','go_practical_compiler_inputs','large_original_native_graph_and_export_streams_survive_full_handoff','--','--nocapture','--test-threads=1'],
  'checksum':[*rust,'--test','go_practical_compiler_inputs','coherently_changed_unselected_mod_bytes_still_require_original_go_sum','--','--nocapture','--test-threads=1'],
  'links':[*rust,'--test','go_practical_compiler_inputs','--','--nocapture','--test-threads=1','--skip','complete_original_dependency_parser_matches_native_lax_facts'],
  'clippy':['cargo','clippy','--offline','--locked','-p','mpk-vc','--lib',*targets,'--','-D','warnings'],
  'clippy-links':['cargo','clippy','--offline','--locked','-p','mpk-vc','--test','go_practical_compiler_inputs','--','-D','warnings'],
 }
 changes={'CARGO_NET_OFFLINE':'true','CARGO_PROFILE_TEST_OPT_LEVEL':'1','CARGO_PROFILE_TEST_DEBUG_ASSERTIONS':'true','CARGO_PROFILE_TEST_OVERFLOW_CHECKS':'true','CARGO_INCREMENTAL':'0','RUST_TEST_THREADS':'1'}
 go=mode in ['native','native-zstd','owner','vet','go-format','go-format-write']
 if go:changes.update({'GOROOT':str(SDK),'GOTOOLCHAIN':'local','GOENV':'off','GOWORK':'off','GOFLAGS':'-mod=vendor','GOPROXY':'off','GOSUMDB':'off','CGO_ENABLED':'0','GOMAXPROCS':'1','GOCACHE':'/private/tmp/mpk-go-export-inputs-190-cache','GOMODCACHE':'/private/tmp/mpk-go-module-observations-191-mod'})
 if mode == 'native-zstd':changes['MPK_GO_ELF_ZSTD_OUTPUT']=str(out/'observations')
 if mode == 'native':changes['MPK_GO_BUILD_INFO_FIXTURES']=str(ROOT/'fixtures')
 if mode in ['rust','roles','graph','links','checksum','unit','fresh']:
  changes.update({'MPK_GO_ELF_OUTPUT':str(out/'observations/elf'),'MPK_GO_COMPILER_INPUTS_OUTPUT':str(out/'observations/compilers'),'MPK_GO_GRAPH_INPUTS_OUTPUT':str(out/'observations/graphs'),'MPK_GO_GRAPH_MANIFEST_OUTPUT':str(out/'observations/graph-manifests'),'MPK_GO_MODULE_MANIFEST_OUTPUT':str(out/'observations/manifests'),'MPK_GO_EXPORT_OUTPUT':str(out/'observations/exports')})
 env=os.environ.copy();env.update(changes)
 for k in ['RUST_MIN_STACK','MPK_CORE_MAX_STEPS','MPK_CORE_MAX_SECONDS']:env.pop(k,None)
 command=commands[mode]
 report={'status':'running','command':command,'environment':changes,'inputs':seal(receipt),'driver':seal(__file__),'tools':{n:seal(shutil.which(n)) for n in ['cargo','rustc','rustfmt']},'started_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'selection_reason':'Preserve all187 original executable/layout/text cases and9 actual original SDK tools. Compare full ELF class/endian/section/compression/name observations with pinned Go1.25, including entire3 actual SDK Zstandard streams, complete ZLIB/Zstandard prefixes and malformed/extended/unused metadata. No SDK/source/body/Root/type/error/proof scope, native compiler/release condition, source/bundle/kernel/certificate cap or public profile changes. Run the independent ELF reader and full original5577-file/182-package/5697-object compiler owner with all prior coherent claim/content/role/manifest controls. Select only the reader and its affected compiler-input consumers, scoped Clippy/vet/formats. No whole gate or removed-language tests.','full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
 if go:report['go_tool']=seal(command[0])
 write(out/'result.json',report);start=time.monotonic()
 with (out/'stdout.log').open('xb') as stdout,(out/'stderr.log').open('xb') as stderr:
  code=subprocess.run(command,cwd=REPO/'go-tools/go2vir' if mode in ['native','native-zstd','owner','vet'] else REPO,env=env,stdout=stdout,stderr=stderr,check=False).returncode
 unchanged=sources==[seal(p) for p in sorted(files)]
 sdk_unchanged=sdk['sources']==[seal(row['path']) for row in sdk['sources']]
 expected=mode in ['format-write','go-format-write']
 report.update(status='passed' if code==0 and (unchanged or expected) and sdk_unchanged and (mode!='go-format' or not (out/'stdout.log').read_bytes()) else 'failed',exit_code=code,seconds=time.monotonic()-start,consumed_inputs_unchanged=unchanged,whole_sdk_unchanged=sdk_unchanged,expected_formatter_mutation=expected,stdout=seal(out/'stdout.log'),stderr=seal(out/'stderr.log'),finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat())
 write(out/'result.json',report)
 print(json.dumps({k:report[k] for k in ['status','exit_code','seconds','consumed_inputs_unchanged','whole_sdk_unchanged']}));raise SystemExit(report['status']!='passed')
if __name__=='__main__':main()
