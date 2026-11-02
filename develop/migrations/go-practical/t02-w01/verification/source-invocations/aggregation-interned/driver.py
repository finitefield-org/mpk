"""Rebundle the entire original declaration union; no mathematical statement is removed."""
from pathlib import Path
import datetime,hashlib,json,os,subprocess,sys
ROOT=Path(__file__).resolve().parent;OUT=ROOT/(sys.argv[1] if len(sys.argv)>1 else 'aggregation');OUT.mkdir(exist_ok=False);REPO=Path.cwd()
def seal(p):
 b=Path(p).read_bytes();return {'path':str(p),'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()}
test_fingerprint=Path('target/debug/.fingerprint/mpk-vc-7848ed9502a8e03d/test-integration-test-go_practical_source_invocation.json');fingerprint=json.loads(test_fingerprint.read_bytes());libs={};fingerprints=[]
for _,name,_,expected in fingerprint['deps']:
 matching=[]
 for p in Path('target/debug/.fingerprint').glob('*/lib-'+name):
  if int.from_bytes(bytes.fromhex(p.read_text().strip()),'little')==expected:
   candidate=Path('target/debug/deps')/f'lib{name}-{p.parent.name.rsplit("-",1)[1]}.rlib'
   if candidate.exists():matching.append((p,candidate))
 assert len(matching)==1,(name,expected,matching)
 fp,lib=matching[0];libs[name]=lib;fingerprints.append(fp)
source=ROOT/'aggregate-all.rs';binary=Path('/private/tmp/mpk-go-source-invocations-aggregate-all-174')
command=['rustc','--edition=2021','-C','opt-level=1','-C','debug-assertions=yes','-C','overflow-checks=yes','-L','dependency=target/debug/deps',str(source),'-o',str(binary)]
for name,p in libs.items():command.extend(['--extern',f'{name}={p}'])
inputs=[Path(__file__),source,test_fingerprint,*fingerprints,ROOT/'generation-linked/result.json',ROOT/'generation-linked/prelaunch-inputs.json',*sorted((ROOT/'generation-linked/certificates').iterdir()),ROOT/'corpus.json',ROOT/'rust-entries/inputs.json',Path('crates/mpk-vc/tests/go_practical_source_invocation.rs'),Path('crates/mpk-vc/tests/support/go_word_division_dependencies.rs')];rows=[]
for i,p in enumerate(inputs):
 target=OUT/'inputs'/f'{i:03d}.original';target.parent.mkdir(exist_ok=True);target.write_bytes(p.read_bytes());target.chmod(p.stat().st_mode&0o777);rows.append({**seal(p),'mode':p.stat().st_mode&0o777,'complete_prelaunch_snapshot':seal(target)})
receipt=OUT/'prelaunch-inputs.json';receipt.write_text(json.dumps(rows,indent=2)+'\n');(OUT/'driver.py').write_bytes(Path(__file__).read_bytes())
record={'status':'running','actual_driver_prelaunch':seal(OUT/'driver.py'),'complete_prelaunch_inputs':seal(receipt),'exact_original_test_fingerprint':seal(test_fingerprint),'exact_compiled_library_identities':{name:seal(p) for name,p in libs.items()},'compile_command':command,'kernel_proofs_rechecked':False,'complete_final_matrix':False,'started_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat()}
def save():(OUT/'result.json').write_text(json.dumps(record,indent=2)+'\n')
save();env=os.environ.copy();env['CARGO_MANIFEST_DIR']=str(REPO/'crates/mpk-vc');env['MPK_GO_SOURCE_INVOCATION_OUTPUT']=str(OUT/'certificates')
try:
 with (OUT/'compile.stdout.log').open('wb') as out,(OUT/'compile.stderr.log').open('wb') as err:code=subprocess.run(command,env=env,stdout=out,stderr=err,check=False).returncode
 record['compile_exit_code']=code;record['compile_stdout']=seal(OUT/'compile.stdout.log');record['compile_stderr']=seal(OUT/'compile.stderr.log');save();assert code==0
 record['generated_binary']=seal(binary);record['command']=[str(binary)];save()
 with (OUT/'stdout.log').open('wb') as out,(OUT/'stderr.log').open('wb') as err:code=subprocess.run([str(binary)],env=env,stdout=out,stderr=err,check=False).returncode
 record.update(exit_code=code,stdout=seal(OUT/'stdout.log'),stderr=seal(OUT/'stderr.log'));save();assert code==0
 for row in rows:assert seal(row['path'])=={k:row[k] for k in ['path','bytes','sha256']} and Path(row['path']).stat().st_mode&0o777==row['mode']
 for name,p in libs.items():assert seal(p)==record['exact_compiled_library_identities'][name]
 record.update(status='passed',whole_inputs_unchanged=True,complete_generated_candidates=len(list((OUT/'certificates').glob('*.mpcert'))),finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat());save();print(json.dumps({'status':'passed','complete_generated_candidates':record['complete_generated_candidates'],'kernel_proofs_rechecked':False}))
except BaseException:
 record['status']='failed';save();raise
