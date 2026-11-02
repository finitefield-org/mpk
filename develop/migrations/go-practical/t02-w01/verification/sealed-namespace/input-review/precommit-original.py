from pathlib import Path
import hashlib,json,subprocess,sys,types
repo=Path.cwd();root=Path('develop/migrations/go-practical/t02-w01/verification/sealed-namespace');base='7405cf5fd5c88f0e31ca96076618e296869ad095'
def git(*args):return subprocess.run(['/usr/bin/git',*args],check=True,stdout=subprocess.PIPE,stderr=subprocess.PIPE).stdout
def seal(p):
 p=Path(p);b=p.read_bytes();return {'path':str(p),'sha256':hashlib.sha256(b).hexdigest(),'bytes':len(b)}
assert git('rev-parse','HEAD').decode().strip()==base
assert git('branch','--show-current').decode().strip()=='codex/go-practical'
review=root/'input-review/changed-inputs-review.json';r=json.loads(review.read_text());assert r['status']=='passed' and r['selected_files']==179 and not(r['findings'] or r['whitespace_errors'] or r['git_errors'] or r['missing_links'])
expected={x['path']:x for x in r['inputs']};expected[str(review)]=seal(review);assert len(expected)==180
for n,v in expected.items():assert seal(n)==v,n
assert not git('diff','--name-only','-z')
assert not git('ls-files','--others','--exclude-standard','-z')
assert set(git('diff','--cached','--name-only','-z','HEAD').decode().strip('\0').split('\0'))==set(expected)
raw=git('diff','--cached','--raw','--abbrev=40','-z','HEAD').split(b'\0');assert raw.pop()==b'' and len(raw)==360
for i in range(0,len(raw),2):
 header=raw[i].decode().split();n=raw[i+1].decode();assert header[4] in ['A','M']
 b=Path(n).read_bytes();assert hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()==header[3],n
scanner=Path('scripts/check-no-active-gir.sh');code=scanner.read_text().split("<<'PY'\n",1)[1].rsplit('\nPY',1)[0].rsplit('raise SystemExit(main())',1)[0]
m=types.ModuleType('mpk_namespace_review_self');m.__file__=str(repo/scanner);sys.modules[m.__name__]=m;argv=sys.argv;sys.argv=[str(scanner),str(repo),'strict']
try:exec(compile(code,str(scanner),'exec'),m.__dict__);matchers=m.load_matchers()
finally:sys.argv=argv
findings=[vars(f) for f in m.detect(str(review),review.read_text(),matchers)];assert not findings,findings
whitespace=git('diff','--no-index','--check','--','/dev/null',str(review));assert not whitespace
f=json.loads((root/'frozen-design-preservation.json').read_text());assert f['frozen_inputs']==255
for x in f['inputs']:assert seal(x['path'])==x
c=json.loads((root/'checkpoint-audit.json').read_text());assert c['status']=='passed'
out={'status':'passed','base_commit':base,'branch':'codex/go-practical','staged_files':180,'reviewed_changed_inputs':179,'review_self_checked':True,'all_staged_blob_bytes_equal_reviewed_sha256_and_length':True,'unstaged_or_untracked_files':0,'review':seal(review),'checkpoint_audit':seal(root/'checkpoint-audit.json'),'frozen_inputs_unchanged':255,'all_actual_validation_handles_terminal':True,'full_T02_gate_passed':False,'CSharp_tests_executed':False,'T02_W01_complete':False,'goal_status':'active'}
Path('/private/tmp/mpk-go-sealed-namespace-precommit-audit.json').write_text(json.dumps(out,indent=2)+'\n');print(json.dumps(out))
