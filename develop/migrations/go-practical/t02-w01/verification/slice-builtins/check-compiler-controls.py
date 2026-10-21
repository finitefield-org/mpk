"""Compile selected same original builtin sources without running applications."""
from pathlib import Path
import datetime,hashlib,json,os,subprocess,time
OUT=Path(__file__).resolve().parent;RUN=OUT/'compiler-controls';RUN.mkdir(exist_ok=False);(RUN/'driver.py').write_bytes(Path(__file__).read_bytes())
GOROOT=Path('/private/tmp/mpk-go-practical-original-packages-sdk-20261021/go');env=os.environ.copy();env.update(GOROOT=str(GOROOT),GOTOOLCHAIN='local',GOENV='off',GOWORK='off',GOPROXY='off',GOSUMDB='off',CGO_ENABLED='0')
def seal(p):
 raw=Path(p).read_bytes();return {'path':str(p),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()}
corpus=OUT/'corpus.json';cases=json.loads(corpus.read_bytes());labels=['edge-append(PairBytesString()...)','copy-nil-bytes','copy-nil-str','append-spread-nil-str'];results=[]
for label in labels:
 case=next(row for row in cases if row['input']['label']==label);n=case['input']['index'];directory=RUN/f'case-{n:05}';directory.mkdir();(directory/'input.json').write_text(json.dumps(case,indent=2)+'\n')
 source=next(s for s in case['input']['sources']if s['object']['path']=='policy.go');raw=bytes.fromhex(source['raw_hex']);p=directory/'original.go.raw';p.write_bytes(raw);work=Path('/private/tmp')/f'mpk-go-slice-builtins-compiler-{n}';work.mkdir(exist_ok=False);original=work/'original.go';original.write_bytes(raw)
 command=[str(GOROOT/'bin/go'),'tool','compile','-p','example.com/scalar','-o',str(work/'original.o'),str(original)];record={'status':'running','label':label,'command':command,'actual_driver_prelaunch':seal(RUN/'driver.py'),'original_input':seal(directory/'input.json'),'original_source':seal(p),'compiler_source':seal(original),'applications_executed':False};result=directory/'result.json';result.write_text(json.dumps(record,indent=2)+'\n');start=time.monotonic()
 with(directory/'stdout.log').open('wb')as stdout,(directory/'stderr.log').open('wb')as stderr:r=subprocess.run(command,env=env,stdout=stdout,stderr=stderr,check=False)
 record.update(status='accepted'if r.returncode==0 else'not_accepted',exit_code=r.returncode,seconds=time.monotonic()-start,stdout=seal(directory/'stdout.log'),stderr=seal(directory/'stderr.log'),source_bytes_unchanged=original.read_bytes()==p.read_bytes()==raw);result.write_text(json.dumps(record,indent=2)+'\n');results.append(record)
assert all(r['source_bytes_unchanged']for r in results)
(RUN/'result.json').write_text(json.dumps({'status':'observed','corpus':seal(corpus),'actual_driver_prelaunch':seal(RUN/'driver.py'),'original_controls':results,'applications_executed':False},indent=2)+'\n');print(json.dumps([{'label':r['label'],'status':r['status'],'exit':r['exit_code']}for r in results]))
