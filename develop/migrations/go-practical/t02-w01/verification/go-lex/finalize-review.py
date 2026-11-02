"""Review only changed metadata after a completed, byte-pinned full input audit."""
from pathlib import Path
import datetime,hashlib,json,re,subprocess,sys,types,urllib.parse
ROOT=Path.cwd();OUT=Path(__file__).resolve().parent;REPORT=OUT/'input-review.json';PRIOR=OUT/'full-input-review.json'
def git(*args):return subprocess.check_output(['/usr/bin/git',*args])
def seal(p,raw=None):
 raw=Path(p).read_bytes()if raw is None else raw;return {'path':str(p),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()}
assert git('rev-parse','HEAD').decode().strip()=='13edcaf13d267febe501e5b7b66135d20915fbc8'
assert not PRIOR.exists();PRIOR.write_bytes(REPORT.read_bytes());prior=json.loads(PRIOR.read_bytes());assert prior['status']=='passed'and not prior['findings']and not prior['missing_links']and prior['source_inputs_unchanged']
expected={r['path']:r for r in prior['inputs']}
scanner=Path('scripts/check-no-active-gir.sh');code=scanner.read_text().split("<<'PY'\n",1)[1].rsplit('\nPY',1)[0].rsplit('raise SystemExit(main())',1)[0];module=types.ModuleType('mpk_lex_delta_review');module.__file__=str(ROOT/scanner);sys.modules[module.__name__]=module;sys.argv=[str(scanner),str(ROOT),'strict'];exec(compile(code,str(scanner),'exec'),module.__dict__);matchers=module.load_matchers();manifest,files,fixtures,_=module.validate_manifest();assert(len(matchers),len(module.SCOPES),len(manifest['sealed_records']),len(module.active_retained_record_exclusions(manifest)))==(127,4,39,51)
selected=git('diff','--name-only','-z','HEAD','--')+git('ls-files','--others','--exclude-standard','-z');paths=sorted(set(selected.decode().split('\0'))-{'',str(REPORT.relative_to(ROOT))});rows=[];findings=[];links=[];changed=[]
for name in paths:
 path=Path(name);assert path.is_file()and not path.is_symlink();raw=path.read_bytes();row=seal(name,raw);rows.append(row)
 if expected.get(name)==row:continue
 assert name.startswith(str(OUT.relative_to(ROOT))+'/'),name
 assert path.suffix in {'.md','.py','.json'},name
 findings.extend(vars(f)for f in module.detect(name,raw.decode(),matchers));changed.append(row)
 if path.suffix=='.md':
  for target in re.findall(r'(?<!!)\[[^\]]*\]\(([^\n)]+)\)',raw.decode()):
   target=target.strip().strip('<>')
   if re.match(r'^[a-zA-Z][a-zA-Z0-9+.-]*:',target)or target.startswith('#'):continue
   file=urllib.parse.unquote(target.split('#',1)[0]);links.append({'document':name,'target':target,'exists':(path.parent/file).resolve().exists()})
assert set(expected)<=set(paths)
byte_audit=json.loads((OUT/'original-byte-audit/result.json').read_bytes());assert byte_audit['status']=='passed'and byte_audit['actual_original_files_compared_byte_for_byte']==35114 and byte_audit['actual_original_bytes_compared']==565549694
missing=[r for r in links if not r['exists']];assert not findings and not missing
record={**prior,'status':'passed','observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'inputs':rows,'prior_complete_source_and_decoded_archive_audit':seal(PRIOR),'targeted_metadata_delta':changed,'scope_reason':'Complete original source/preimage/native/Rust/decoded-literal audit already passed. Re-read every prior input and require identical full SHA/bytes before reuse; only changed/new artifact metadata/docs/audit utilities are rescanned. Production source, test inputs, archives and all previous evidence remain exact. No scanner rule or exception changed.','actual_original_byte_and_mode_audit':seal(OUT/'original-byte-audit/result.json'),'findings':findings,'missing_links':missing,'source_inputs_unchanged':True,'final_metadata_delta_only':True}
REPORT.write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({'status':'passed','unchanged_prior_inputs':len(expected)-sum(r['path']in expected for r in changed),'changed_or_new_metadata_inputs':len(changed),'current_input_files':len(rows)}))
