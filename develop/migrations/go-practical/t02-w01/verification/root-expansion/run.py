"""Complete original source epochs for independent dependency manifest checks."""
from pathlib import Path
import datetime, hashlib, json, os, shutil, subprocess, sys, tarfile, time

ROOT=Path(__file__).resolve().parent
REPO=Path.cwd()
SDK=Path('/private/tmp/mpk-go125-20261025-download/go')
CHANGED=[
 'crates/mpk-vc/src/go_practical_type_keys/module_observations/graph_inputs.rs',
 'crates/mpk-vc/src/go_practical_type_keys/module_observations/graph_inputs/expansion.rs',
 'crates/mpk-vc/src/go_practical_type_keys/module_observations/graph_inputs/root_sort.rs',
 'crates/mpk-vc/src/go_practical_type_keys/module_observations/graph_inputs/tests.rs',
 'crates/mpk-vc/tests/go_practical_graph_inputs.rs',
]
GO_CHANGED=['go-tools/go2vir/practical_module_graph.go','go-tools/go2vir/practical_module_graph_inputs.go','go-tools/go2vir/practical_graph_expansion_oracle_test.go']
BORROWED=[
 'module-manifests/native-toolchain/native-originals',
 'graph-import/native/native-originals',
 'module-observations/native/native-originals',
 'export-inputs/native/native-originals',
 'module-graphs/native-initial/native-originals/graph-owned',
 'graph-inputs/native/native-originals/graph-owned',
 'module-graphs/edges-final/native-originals',
 'module-graphs/replacements-final/native-originals',
 'module-graphs/toolchains-final/native-originals',
]
def seal(path):
 path=Path(path);raw=path.read_bytes()
 return {'path':str(path.resolve()),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'mode':path.stat().st_mode&0o777}
def write(path,value):path.write_text(json.dumps(value,indent=2)+'\n')
def main():
 epoch,mode=sys.argv[1:];assert epoch.replace('-','').isalnum()
 out=ROOT/epoch;out.mkdir(exist_ok=False)
 formatting=mode in ['format-write','format','go-format-write','go-format']
 files={Path(__file__).resolve()};files.update(ROOT.glob('*.py'));preserved=set()
 if formatting:
  files.update(REPO/name for name in (GO_CHANGED if mode.startswith('go-') else CHANGED))
  files.update(REPO/name for name in ['rustfmt.toml','.rustfmt.toml'] if (REPO/name).is_file())
 else:
  for name in ['crates','proofs/std','go-tools/go2vir','go-tools/mpk-checker-ref','develop/specs/vectors','fixtures','release/bundles','.cargo']:
   files.update(p for p in (REPO/name).rglob('*') if p.is_file())
  files.update(REPO/name for name in ['Cargo.toml','Cargo.lock'])
  for name in BORROWED:
   originals={p for p in (ROOT.parent/name).rglob('*') if p.is_file()}
   files.update(originals);preserved.update(originals)
  if mode in ['rust','graph','links','checksum','clippy','clippy-links','unit','fresh']:
   files.update(p for p in (ROOT/'native-final/native-originals').rglob('*') if p.is_file())
  if mode=='fresh':files.update(p for p in (ROOT/'owner/native-originals').rglob('*') if p.is_file())
 sdk_receipt=ROOT.parent/'module-graphs/original-sdk/inputs.json'
 sdk=json.loads(sdk_receipt.read_bytes())
 assert sdk['sources']==[seal(row['path']) for row in sdk['sources']]
 sources=[seal(p) for p in sorted(files)]
 with tarfile.open(out/'inputs.tar.gz','x:gz',dereference=True) as archive:
  for p in sorted(files-preserved):archive.add(p,arcname=str(p.relative_to(REPO)),recursive=False)
 receipt=out/'inputs.json'
 write(receipt,{'basis_commit':'a65e6c529cf03b3efc4d9beb92a50084690f7618','sources':sources,'complete_archive':seal(out/'inputs.tar.gz'),'preserved_original_paths':[str(p) for p in sorted(preserved)],'whole_original_sdk':seal(sdk_receipt),'sdk_archive':sdk['complete_archive'],'scope':'Keep all original source/package/fixture/proof objects; exact original earlier committed paths are reused without replacing or dropping bytes.'})
 rust=['cargo','test','--offline','--locked','-p','mpk-vc']
 targets=['--test','go_practical_graph_inputs']
 commands={
  'format-write':['rustfmt','--edition=2021','--config','skip_children=true',*CHANGED],
  'format':['rustfmt','--edition=2021','--check','--config','skip_children=true',*CHANGED],
  'go-format-write':[str(SDK/'bin/gofmt'),'-w',*GO_CHANGED],
  'go-format':[str(SDK/'bin/gofmt'),'-l',*GO_CHANGED],
  'native':[str(SDK/'bin/go'),'test','-json','-count=1','-timeout=40m','-run','^TestPractical(OriginalGraph|ModuleGraph|GraphInputs)'],
  'owner':[str(SDK/'bin/go'),'test','-json','-count=1','-timeout=40m','-run','^TestPracticalGraphImportKeepsWholeOriginalVersionedUniverse$'],
  'unit':[*rust,'--lib','go_practical_type_keys::module_observations::graph_inputs::tests','--','--nocapture','--test-threads=1'],
  'vet':[str(SDK/'bin/go'),'vet','.'],
  'rust':[*rust,*targets,'--','--nocapture','--test-threads=1'],
  'graph':[*rust,'--test','go_practical_graph_inputs','--','--nocapture','--test-threads=1'],
  'fresh':[*rust,'--test','go_practical_graph_inputs','freshly_captured_whole_owner_independently_derives_final_roots','--','--nocapture','--test-threads=1'],
  'checksum':[*rust,'--test','go_practical_graph_inputs','coherently_changed_unselected_mod_bytes_still_require_original_go_sum','--','--nocapture','--test-threads=1'],
  'links':[*rust,'--test','go_practical_graph_inputs','--','--nocapture','--test-threads=1','--skip','complete_original_dependency_parser_matches_native_lax_facts'],
  'clippy':['cargo','clippy','--offline','--locked','-p','mpk-vc','--lib',*targets,'--','-D','warnings'],
  'clippy-links':['cargo','clippy','--offline','--locked','-p','mpk-vc','--test','go_practical_graph_inputs','--','-D','warnings'],
 }
 changes={'CARGO_NET_OFFLINE':'true','CARGO_PROFILE_TEST_OPT_LEVEL':'1','CARGO_PROFILE_TEST_DEBUG_ASSERTIONS':'true','CARGO_PROFILE_TEST_OVERFLOW_CHECKS':'true','CARGO_INCREMENTAL':'0','RUST_TEST_THREADS':'1'}
 go=mode in ['native','owner','vet','go-format','go-format-write']
 if go:changes.update({'GOROOT':str(SDK),'GOTOOLCHAIN':'local','GOENV':'off','GOWORK':'off','GOFLAGS':'-mod=vendor','GOPROXY':'off','GOSUMDB':'off','CGO_ENABLED':'0','GOMAXPROCS':'1','GOCACHE':'/private/tmp/mpk-go-export-inputs-190-cache','GOMODCACHE':'/private/tmp/mpk-go-module-observations-191-mod'})
 if mode in ['native','owner']:
  native=out/'native-originals';native.mkdir();changes['MPK_GO_PRACTICAL_SYNTAX_EVIDENCE']=str(native)
 if mode in ['rust','graph','links','checksum','unit','fresh']:
  changes.update({'MPK_GO_ROOT_EXPANSION_OUTPUT':str(out/'observations/expansion'),'MPK_GO_GRAPH_INPUTS_OUTPUT':str(out/'observations/graphs'),'MPK_GO_GRAPH_MANIFEST_OUTPUT':str(out/'observations/graph-manifests'),'MPK_GO_MODULE_MANIFEST_OUTPUT':str(out/'observations/manifests'),'MPK_GO_EXPORT_OUTPUT':str(out/'observations/exports')})
 env=os.environ.copy();env.update(changes)
 for k in ['RUST_MIN_STACK','MPK_CORE_MAX_STEPS','MPK_CORE_MAX_SECONDS']:env.pop(k,None)
 command=commands[mode]
 report={'status':'running','command':command,'environment':changes,'inputs':seal(receipt),'driver':seal(__file__),'tools':{n:seal(shutil.which(n)) for n in ['cargo','rustc','rustfmt']},'started_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'selection_reason':'Source-derived Go1.25 pruned root expansion and complete transient manifest capture affect only graph preparation/import. Run native expansion/ordering comparisons and affected original graph roots, the full original versioned HTTP owner, Rust expansion unit comparisons and all graph input ownership/checksum/coherent claim controls, plus strict changed lib/test Clippy, Go vet and explicit changed formats. Original source/package/proof/error universes and all kernel caps remain unchanged. No whole gate or removed-language tests.','full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
 if go:report['go_tool']=seal(command[0])
 write(out/'result.json',report);start=time.monotonic()
 with (out/'stdout.log').open('xb') as stdout,(out/'stderr.log').open('xb') as stderr:
  code=subprocess.run(command,cwd=REPO/'go-tools/go2vir' if mode in ['native','owner','vet'] else REPO,env=env,stdout=stdout,stderr=stderr,check=False).returncode
 unchanged=sources==[seal(p) for p in sorted(files)]
 sdk_unchanged=sdk['sources']==[seal(row['path']) for row in sdk['sources']]
 expected=mode in ['format-write','go-format-write']
 report.update(status='passed' if code==0 and (unchanged or expected) and sdk_unchanged and (mode!='go-format' or not (out/'stdout.log').read_bytes()) else 'failed',exit_code=code,seconds=time.monotonic()-start,consumed_inputs_unchanged=unchanged,whole_sdk_unchanged=sdk_unchanged,expected_formatter_mutation=expected,stdout=seal(out/'stdout.log'),stderr=seal(out/'stderr.log'),finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat())
 write(out/'result.json',report)
 print(json.dumps({k:report[k] for k in ['status','exit_code','seconds','consumed_inputs_unchanged','whole_sdk_unchanged']}));raise SystemExit(report['status']!='passed')
if __name__=='__main__':main()
