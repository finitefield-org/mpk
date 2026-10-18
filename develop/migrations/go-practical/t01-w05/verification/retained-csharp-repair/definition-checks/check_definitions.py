import argparse,hashlib,json,platform,subprocess,time
from pathlib import Path
parser=argparse.ArgumentParser(description='Repeat current definition checks into a new evidence directory.')
parser.add_argument('output_directory',type=Path)
parser.add_argument('--rust-checker',type=Path,required=True)
parser.add_argument('--go-checker',type=Path,required=True)
args=parser.parse_args()
repo=Path(__file__).resolve().parents[7]
out=args.output_directory.resolve();out.mkdir(parents=True,exist_ok=False)
engines={'rust':args.rust_checker.resolve(),'go':args.go_checker.resolve()}
certificates={};members=[]
for kind in ['documents','fragments','utf8']:
 root=repo/('develop/migrations/csharp-03/ordinary-foundation/boundary-'+kind+'/boolean-refreeze')
 files=sorted(root.glob('*.hex'));assert len(files)==3
 for path in files:
  raw=bytes.fromhex(path.read_text());digest=hashlib.sha256(raw).hexdigest()
  if digest in certificates:assert certificates[digest]['bytes']==raw
  else:certificates[digest]={'bytes':raw,'representative':path}
  members.append({'path':str(path.relative_to(repo)),'raw_certificate_sha256':digest,'certificate_bytes':len(raw),'hex_file_sha256':hashlib.sha256(path.read_bytes()).hexdigest()})
rows=[]
for index,(digest,item)in enumerate(certificates.items()):
 reports={};runs=[]
 for engine,binary in engines.items():
  cmd=[str(binary),'check'if engine=='rust'else'verify',str(item['representative'])];start=time.monotonic()
  result=subprocess.run(cmd,cwd=repo,capture_output=True,timeout=60)
  (out/(f'{index:02}-{engine}.stdout.json')).write_bytes(result.stdout);(out/(f'{index:02}-{engine}.stderr.log')).write_bytes(result.stderr)
  assert result.returncode==0 and not result.stderr,(engine,result.returncode,result.stderr[-1000:])
  report=json.loads(result.stdout);assert report['verdict']=='accepted';assert report.get('axiom_count',0)==0
  if engine=='rust':assert report['axiom_report']['summary']['total_axiom_count']==0
  reports[engine]=report
  runs.append({'engine':engine,'command':cmd,'exit_code':result.returncode,'wall_seconds':time.monotonic()-start,'stdout_sha256':hashlib.sha256(result.stdout).hexdigest()})
 assert reports['rust']['hashes']==reports['go']['hashes']
 assert reports['rust']['module']==reports['go']['module']
 assert reports['rust']['declaration_count']==reports['go']['declaration_count']
 rows.append({'raw_certificate_sha256':digest,'accepted_both':True,'axioms':0,'hashes':reports['rust']['hashes'],'runs':runs})
 print('Accepted both engines',index+1,'of',len(certificates),flush=True)
record={'scope':'Actual current ordinary-definition Certificate v0 acceptance, local Darwin development binaries; not original application theorem proofs or native installed-release evidence','definition_certificates_only':True,'members':members,'distinct_certificates':len(rows),'deduplication_reason':'Source contexts share byte-identical definition certificates; every raw member is compared byte for byte before using its identical checked representative. Context binding is separately tested by normal source/import/golden tests.','observations':rows,'checkers':{k:{'path':str(p),'sha256':hashlib.sha256(p.read_bytes()).hexdigest()}for k,p in engines.items()},'platform':platform.platform(),'go_compiler':subprocess.check_output(['/usr/local/go/bin/go','version'],text=True).strip(),'rust_compiler':subprocess.check_output(['rustc','--version'],text=True).strip(),'profile_activated':False,'application_proofs_completed':False}
(out/'result.json').write_text(json.dumps(record,indent=2,sort_keys=True)+'\n');print('PASS',len(members),'members',len(rows),'distinct zero-axiom definition certificates')
