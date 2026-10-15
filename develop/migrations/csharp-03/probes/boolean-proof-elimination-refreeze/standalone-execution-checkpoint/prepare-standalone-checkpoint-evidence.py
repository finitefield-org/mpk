import hashlib,json,pathlib,shutil
b=pathlib.Path(__file__).parent;out=b/'standalone-checkpoint-evidence';out.mkdir(exist_ok=False);h=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
for folder,count in [('program-alias-name-graphs',93),('program-alias-name-graphs-2',93),('program-alias-dual-checkers',186)]:
 state=json.loads((b/folder/'status.json').read_bytes());assert state['status'].startswith('passed_all_93') and len(state['stages'])==count and all(x['exit_code']==0 for x in state['stages'])
 for p in sorted((b/folder).rglob('*')):
  if p.is_file() and p.suffix!='.mpcert':
   dst=out/folder/p.relative_to(b/folder);dst.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,dst)
for folder in ['program-alias-regeneration']:
 state=json.loads((b/folder/'status.json').read_bytes());assert state['status']=='passed' and state['exit_code']==0
 for p in sorted((b/folder).glob('*')):
  if p.is_file():
   dst=out/folder/p.name;dst.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,dst)
 shutil.copyfile(b/folder/'generated/generation.json',out/folder/'generation.json')
for n in ['program-alias-plan.json','program-alias-comparison-2.json','program-alias-comparison-4.json','program-alias-compile-command.json','program-alias-compile.log.txt','context-name-rebindings-3.json','actual-contract-attachment-rebindings.json','contract-attachment-rebinding-audit.json','name-graph-cache-equivalence-audit.json','prepare-program-alias-plan.py','build-program-alias-driver.py','regenerate-program-aliases.rs','run-program-alias-regeneration.py','compare-program-alias-regeneration-2.py','compare-program-alias-regeneration-4.py','run-program-alias-name-graphs.py','run-program-alias-name-graphs-2.py','run-program-alias-dual-checkers.py','compare-certificate-context-names.rs','compare-certificate-context-names-2.rs','name-graph-compile-command-2.json','prepare-standalone-program-promotion.py','prepare-standalone-checkpoint-evidence.py']:
 shutil.copyfile(b/n,out/n)
for folder in ['standalone-program-promotion-calendar-merge']:
 shutil.copytree(b/folder,out/folder)
manifest={str(p.relative_to(out)):h(p) for p in sorted(out.rglob('*')) if p.is_file()};(out/'file-manifest.json').write_text(json.dumps({'schema':'mpk.evidence_file_manifest.v1','files':manifest},sort_keys=True,indent=2)+'\n');print('verified evidence files',len(manifest))
