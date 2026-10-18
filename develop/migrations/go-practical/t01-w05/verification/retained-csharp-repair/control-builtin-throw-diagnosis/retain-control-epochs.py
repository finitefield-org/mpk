from pathlib import Path
import subprocess,json,hashlib,re
out=Path('develop/migrations/go-practical/t01-w05/retained-csharp/control-preservation-epochs');out.mkdir();after=out/'after-complete-source-executions';after.mkdir();base=Path('develop/migrations/csharp-03/ordinary-foundation/control-edges');checkpoint=subprocess.check_output(['/usr/bin/git','rev-parse','f7333bc9b34c9df73ad19fece69c7127b608131c^']).decode().strip();compiled='320ecd67ab420e1ea806196271eea3271f929fbe';allowed={'/source_ir_sha256','/foundation_sha256','/control_vc_sha256'}
def seal(p):b=p.read_bytes();return {'path':str(p),'sha256':hashlib.sha256(b).hexdigest(),'bytes':len(b)}
def blob(ref,p):
 data=subprocess.check_output(['/usr/bin/git','show',ref+':'+str(p)]);oid=subprocess.check_output(['/usr/bin/git','rev-parse',ref+':'+str(p)]).decode().strip();return data,oid
cases=[];members={};audit=[]
for name in sorted(p.name for p in base.glob('*.json')):
 current=base/name;data,oid=blob(checkpoint,current);target=after/name;target.write_bytes(data);old=json.loads(data);now=json.loads(current.read_bytes());diff=[]
 def walk(a,b,p):
  assert type(a)==type(b),p
  if isinstance(a,dict):
   assert set(a)==set(b),p
   for k in sorted(a):walk(a[k],b[k],p+'/'+k)
  elif isinstance(a,list):
   assert len(a)==len(b),p
   for i,(a,b) in enumerate(zip(a,b)):walk(a,b,p+'/'+str(i))
  elif a!=b:diff.append({'path':p,'retained':a,'current':b})
 walk(old,now,'');assert {r['path'] for r in diff}==allowed,(name,diff)
 for r in diff:assert re.fullmatch('[0-9a-f]{64}',r['retained']) and re.fullmatch('[0-9a-f]{64}',r['current'])
 oldhex,hexoid=blob(checkpoint,current.with_suffix('.hex'));newhex=current.with_suffix('.hex').read_bytes();assert oldhex==newhex,name;(after/Path(name).with_suffix('.hex')).write_bytes(oldhex)
 for p,o in [(target,oid),(after/Path(name).with_suffix('.hex'),hexoid)]:members[str(p)]={**seal(p),'git_source_commit':checkpoint,'git_source_path':str(base/p.name),'git_blob':o}
 for p in [current,current.with_suffix('.hex')]:
  b,o=blob(compiled,p);assert b==p.read_bytes();members[str(p)]={**seal(p),'git_source_commit':compiled,'git_source_path':str(p),'git_blob':o}
 cases.append({'id':Path(name).stem,'after_metadata':str(target),'after_certificate':str(after/Path(name).with_suffix('.hex')),'current_metadata':str(current),'current_certificate':str(current.with_suffix('.hex')),'certificate_bytes_identical':True,'metadata_changes':diff})
assert len(cases)==18
folders=['before-phi-joins','before-memory-bindings','before-public-projection','before-builtin-throw','before-node-entry','before-memory-effects','before-source-frames','before-native-operations','before-native-options','before-integrated-slots','before-native-exceptions','before-native-literals','before-source-steps'];families=[]
for folder in folders:
 directory=base.parent/'verification-logs/control-edges'/folder/'control-edges';paths=sorted(directory.glob('*.hex'));expected=17 if folders.index(folder)<6 else 18;assert len(paths)==expected,(folder,len(paths));pairs=[]
 for p in paths:
  assert (after/p.name).exists();pairs.append({'id':p.stem,'before_metadata':str(p.with_suffix('.json')),'before_certificate':str(p)})
  for q in [p,p.with_suffix('.json')]:
   raw,oid=blob(compiled,q);assert raw==q.read_bytes();members[str(q)]={**seal(q),'git_source_commit':compiled,'git_source_path':str(q),'git_blob':oid}
 families.append({'id':folder,'before_directory':str(directory),'program_count':expected,'strip_phis':folder=='before-phi-joins','cases':pairs})
assert sum(f['program_count'] for f in families)==228
manifest={'schema':'mpk.go_practical.control_preservation_epochs.v0','status':'exact_git_inputs_retained_tests_pending','after_source_commit':checkpoint,'current_source_commit':compiled,'after_directory':str(after),'after_cases':cases,'families':families,'program_comparisons':228,'input_member_count':len(members),'members':list(members.values()),'current_vs_after_differences':'Every one of18 certificate files is byte-identical; every metadata value is identical except the three enumerated root hashes changed by approved-freeze commitf7333bc9b34c9df73ad19fece69c7127b608131c. No semantic field is normalized or removed.','whole_gate_passed':False,'application_proofs_discharged':False};p=out/'manifest.json';p.write_text(json.dumps(manifest,indent=2)+'\n');print(json.dumps({'checkpoint':checkpoint,'comparisons':228,'families':13,'input_members':len(members),'archive_members':len(list(after.iterdir())),'archive_bytes':sum(p.stat().st_size for p in after.iterdir()),'manifest_sha256':hashlib.sha256(p.read_bytes()).hexdigest()}))
