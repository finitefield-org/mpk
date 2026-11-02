"""Read the exact frozen public archive in validated byte ranges; no pin changes."""
from pathlib import Path
from concurrent.futures import ThreadPoolExecutor,as_completed
import urllib.request,ssl,hashlib,json,time,os,datetime,traceback
URL='https://github.com/llvm/llvm-project/releases/download/llvmorg-18.1.8/clang+llvm-18.1.8-x86_64-linux-gnu-ubuntu-18.04.tar.xz'
SIZE=1044930068
SHA='54ec30358afcc9fb8aa74307db3046f5187f9fb89fb37064cdde906e062ebf36'
BASE=Path('/private/tmp/mpk-go-rust-scope/llvm-ranges');BASE.mkdir(exist_ok=False)
RECORD=Path('develop/migrations/go-rust-only/verification/llvm-ranges');RECORD.mkdir(exist_ok=False)
CONTEXT=ssl.create_default_context(cafile='/etc/ssl/cert.pem')
CHUNK=64*1024*1024
rows=[]
def digest(path):
 h=hashlib.sha256()
 with path.open('rb') as f:
  for block in iter(lambda:f.read(1024*1024),b''):h.update(block)
 return h.hexdigest()
def fetch(index,start,end):
 offset=start;parts=[];attempts=[]
 for attempt in range(8):
  if offset>end:break
  p=BASE/f'chunk-{index:02d}-attempt-{attempt:02d}.bytes';began=time.time();row={'start':offset,'end':end,'attempt':attempt,'path':str(p)}
  request=urllib.request.Request(URL,headers={'User-Agent':'mpk-frozen-archive-range-fetch-v1','Range':f'bytes={offset}-{end}','Accept-Encoding':'identity'})
  try:
   with urllib.request.urlopen(request,timeout=60,context=CONTEXT) as response:
    row.update(status=response.status,content_range=response.headers.get('Content-Range'),content_length=response.headers.get('Content-Length'))
    assert response.status==206 and row['content_range']==f'bytes {offset}-{end}/{SIZE}',row
    expected=end-offset+1;observed=0
    with p.open('xb') as out:
     while observed<expected:
      block=response.read(min(1024*1024,expected-observed))
      if not block:break
      out.write(block);observed+=len(block)
    assert observed==expected,('incomplete range',observed,expected)
  except BaseException as error:row['error']=repr(error)
  if p.exists():
   size=p.stat().st_size;assert size<=end-offset+1
   row.update(bytes=size,sha256=digest(p));parts.append({'path':str(p),'start':offset,'bytes':size,'sha256':row['sha256']});offset+=size
  row['seconds']=time.time()-began;attempts.append(row)
  (BASE/f'chunk-{index:02d}-state.json').write_text(json.dumps({'index':index,'start':start,'end':end,'next_offset':offset,'parts':parts,'attempts':attempts},indent=2)+'\n')
  if offset<=end:time.sleep(min(2**attempt,30))
 result={'index':index,'start':start,'end':end,'complete':offset==end+1,'parts':parts,'attempts':attempts}
 print(json.dumps({'chunk':index,'complete':result['complete'],'bytes':offset-start,'attempts':len(attempts)}),flush=True)
 return result
try:
 with ThreadPoolExecutor(max_workers=8) as pool:
  futures=[pool.submit(fetch,i,start,min(SIZE-1,start+CHUNK-1)) for i,start in enumerate(range(0,SIZE,CHUNK))]
  for f in as_completed(futures):rows.append(f.result())
 assert all(r['complete'] for r in rows)
 destination=Path('/private/tmp/mpk-go-rust-scope/download-archive-backup')/'clang+llvm-18.1.8-x86_64-linux-gnu-ubuntu-18.04.tar.xz'
 assert not destination.exists();h=hashlib.sha256();size=0
 with destination.open('xb') as output:
  for row in sorted(rows,key=lambda r:r['index']):
   for item in row['parts']:
    p=Path(item['path']);assert digest(p)==item['sha256']
    with p.open('rb') as source:
     for block in iter(lambda:source.read(1024*1024),b''):output.write(block);h.update(block);size+=len(block)
 assert size==SIZE and h.hexdigest()==SHA,(size,h.hexdigest())
 result={'status':'passed','path':str(destination),'sha256':SHA,'bytes':size,'ranges':sorted(rows,key=lambda r:r['index']),'TLS_verification':'system /etc/ssl/cert.pem; enabled','pin_unchanged':True}
except BaseException as error:
 result={'status':'failed','error':repr(error),'ranges':rows};(RECORD/'exception.log').write_text(traceback.format_exc())
finally:
 result['finished_at_utc']=datetime.datetime.now(datetime.timezone.utc).isoformat()
 (RECORD/'result.json').write_text(json.dumps(result,indent=2)+'\n');(RECORD/'runner.py').write_bytes(Path(__file__).read_bytes())
 print(json.dumps({k:v for k,v in result.items() if k not in {'ranges'}}),flush=True)
 if result['status']!='passed':raise SystemExit(1)
