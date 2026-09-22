from pathlib import Path
import hashlib,json,os,re,shutil,subprocess,time
repo=Path('/Users/kazuyoshitoshiya/mpk');root=repo/'develop/migrations/csharp-03/ordinary-foundation';e=root/'verification-logs/source-invariants/equation-input-fix';base=json.loads((e.parent/'build.json').read_text());env=os.environ.copy();env.update(base['profile']);p=e/'verification.json';d=dict(status='running',supervisor_pid=os.getpid(),selection_reason='The source-equation semantic test needs a Bool leaf rather than a zero-depth function for empty source products. Rerun the full original test without reducing coverage; production and generated certificate bytes remain identical.',profile=base['profile'],source_sha256={name:hashlib.sha256((repo/name).read_bytes()).hexdigest()for name in base['source_sha256']},runs=[],full_gate='deferred_to_T06_W12')
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def save():p.write_text(json.dumps(d,indent=2)+'\n')
def run(label,cmd,expected=()):
 log=e/(label+'.log');row=dict(label=label,status='running',command=cmd,log=log.name);d['runs'].append(row);save();start=time.monotonic()
 with log.open('wb')as f:
  child=subprocess.Popen(cmd,cwd=repo,env=env,stdin=subprocess.DEVNULL,stdout=f,stderr=subprocess.STDOUT);row['pid']=child.pid;save();code=child.wait()
 text=log.read_text();passed=code==0 and all(s in text for s in expected);row.update(status='passed'if passed else'failed',exit_code=code,elapsed_seconds=round(time.monotonic()-start,3),log_sha256=sha(log));save()
 if not passed:raise RuntimeError(label+' failed')
 return log
save()
try:
 log=run('build',['cargo','test','-p','mpk-vc','--test','csharp_practical_vc','--no-run']);paths=re.findall(r'Executable .* \(([^\n]+)\)',log.read_text());assert len(paths)==1;binary=Path(paths[0]);digest=sha(binary);snapshot=Path('/tmp/mpk-w09-source-invariants-equation-fix-'+digest[:16]);shutil.copy2(binary,snapshot);d.update(binary=str(snapshot),binary_sha256=digest,sources_unchanged_during_build=all(sha(repo/name)==h for name,h in d['source_sha256'].items()));assert d['sources_unchanged_during_build'];save()
 run('original-equations',[str(snapshot),'csharp_03_t06_w09_source_invariants_original_equations','--nocapture','--test-threads=1'],['test result: ok. 1 passed; 0 failed;'])
 run('clippy',['cargo','clippy','-p','mpk-vc','--test','csharp_practical_vc','--','-D','warnings']);run('format',['cargo','fmt','--all','--','--check']);d['status']='passed'
except Exception as ex:d.update(status='failed',failure=str(ex))
save()
