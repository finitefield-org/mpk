"""Review changed inputs and actual complete source epochs, outside Git sandbox."""
from pathlib import Path
import base64
import datetime
import hashlib
import json
import re
import subprocess
import sys
import types
import urllib.parse

ROOT = Path.cwd()
OUT = Path(__file__).resolve().parent
BASE = "db1eb8375120c2c4dade880d85df1d41f2164663"
REPORT = OUT / "input-review.json"


def git(*args):
    return subprocess.check_output(["/usr/bin/git", *args])


def seal(path, raw=None):
    raw = Path(path).read_bytes() if raw is None else raw
    return {"path": str(path), "sha256": hashlib.sha256(raw).hexdigest(), "bytes": len(raw)}


assert git("rev-parse", "HEAD").decode().strip() == BASE
assert not git("diff", "--name-only", "HEAD", "--", "crates", "go-tools/mpk-checker-ref", "test-vectors", "release")
assert not Path(".github/workflows").exists()
scanner = Path("scripts/check-no-active-gir.sh")
code = scanner.read_text().split("<<'PY'\n", 1)[1].rsplit("\nPY", 1)[0].rsplit("raise SystemExit(main())", 1)[0]
module = types.ModuleType("mpk_storage_source_input_review")
module.__file__ = str(ROOT / scanner)
sys.modules[module.__name__] = module
sys.argv = [str(scanner), str(ROOT), "strict"]
exec(compile(code, str(scanner), "exec"), module.__dict__)
matchers = module.load_matchers()
manifest, scanner_files, fixtures, _ = module.validate_manifest()
module.run_fixture_self_test(manifest, matchers)
assert (len(matchers), len(module.SCOPES), len(manifest["sealed_records"])) == (127, 4, 39)
for name in scanner_files | fixtures:
    assert Path(name).read_bytes() == git("show", BASE + ":" + name)
configured = scanner_files | fixtures | module.active_retained_record_exclusions(manifest)
findings, inputs, links, decoded = [], [], [], []
cache = {}
path_rules = [m for m in matchers if m["scope"] == "exact-path"]


def scan(name, raw):
    key = (hashlib.sha256(raw).hexdigest(), tuple(module.path_matches(name, m["path"]) for m in path_rules))
    if key not in cache:
        cache[key] = [vars(f) for f in module.detect(name, raw.decode(errors="ignore"), matchers)]
    findings.extend({**finding, "path": name} for finding in cache[key])


def payload(name, data):
    assert data is None or isinstance(data, str)
    raw = b"" if data is None else base64.b64decode(data, validate=True)
    scan(name, raw)
    decoded.append(seal(name, raw))
    return raw


def source_object(obj, raw):
    assert obj["schema"] == "mpk.go.source_object.v1"
    assert len(raw) == obj["bytes"] and hashlib.sha256(raw).hexdigest() == obj["raw_sha256"]
    data = {key: value for key, value in obj.items() if key != "id"}
    canonical = json.dumps(data, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode()
    assert hashlib.sha256(b"MPK-GO-SOURCE-OBJECT-1.0\0" + canonical).hexdigest() == obj["id"]


def native(path, data):
    raw = payload(str(path) + "#original_source", data["original_source"])
    expression = payload(str(path) + "#original_expression", data["original_expression"])
    value = payload(str(path) + "#native_bytes", data["native_bytes"])
    source_object(data["source_object"], raw)
    span = data["span"]
    assert span["source_id"] == data["source_object"]["id"]
    assert 0 <= span["start"] < span["end"] <= len(raw) and raw[span["start"]:span["end"]] == expression
    assert data["package_path"] and data["native_type_display"] and data["image_digest"] and data["compiler_digest"]
    assert data["ordinary_source_evaluation_or_nominal_identity_proof"] is False
    assert data["registered_context_or_program_proof"] is False
    return value


def carrier(data, value, reject=False):
    length, slots = data["length_word"], data["byte_slots"]
    assert len(length) == 64 and all(type(bit) is bool for bit in length)
    assert len(slots) == 256 and all(len(octet) == 8 and all(type(bit) is bool for bit in octet) for octet in slots)
    number = lambda bits: sum((1 << i) for i, bit in enumerate(bits) if bit)
    if reject:
        assert len(value) == 257 and "257 bytes" in data["carrier_error"] and number(length) == 0
        assert not any(number(octet) for octet in slots)
    else:
        assert len(value) <= 256 and number(length) == len(value)
        assert bytes(number(octet) for octet in slots) == value + bytes(256-len(value))


# The self-linked report is this command's prospective output, not an absent
# input dependency. Its current body is excluded below and scanned at the end.
if not REPORT.exists():
    REPORT.write_text(json.dumps({"status":"running","basis_commit":BASE})+"\n")
selected = git("diff", "--name-only", "-z", "HEAD", "--") + git("ls-files", "--others", "--exclude-standard", "-z")
paths = sorted(set(selected.decode().split("\0")) - {"", str(REPORT.relative_to(ROOT))})
for name in paths:
    path = ROOT / name
    assert path.is_file() and not path.is_symlink(), name
    raw = path.read_bytes()
    inputs.append(seal(name, raw))
    if name not in configured:
        scan(name, raw)
    if path.suffix == ".md":
        for target in re.findall(r"(?<!!)\[[^\]]*\]\(([^\n)]+)\)", raw.decode()):
            target = target.strip().strip("<>")
            if re.match(r"^[a-zA-Z][a-zA-Z0-9+.-]*:", target) or target.startswith("#"):
                continue
            clean = urllib.parse.unquote(target.split("#", 1)[0])
            links.append({"document": name, "target": target, "exists": (path.parent / clean).resolve().exists()})


import runpy
current=OUT/'exact-locators';modes=['selected','analysis-inputs','public-boundary','vet','format','legacy']
checks={m:json.loads((current/m/'result.json').read_bytes())for m in modes}
assert all(r['status']=='passed'and r['whole_source_inputs_unchanged']for r in checks.values())and all(r['sources']==checks['selected']['sources']for r in checks.values())
assert all(seal(r['path'])['sha256']==r['sha256']for r in checks['selected']['sources'])
roots=sum('/'not in t for m in ['selected','analysis-inputs','public-boundary']for t in checks[m]['passed_tests']);subs=sum('/'in t for m in ['selected','analysis-inputs','public-boundary']for t in checks[m]['passed_tests']);assert (roots,subs)==(7,62)
archives=[json.loads(p.read_bytes())for p in sorted(OUT.glob('*/inputs.json'))];assert len(archives)==9
oids=sorted({r['original_git_blob']for a in archives for r in a['sources']if 'original_git_blob'in r})
raw=git('cat-file','--batch')if not oids else subprocess.check_output(['/usr/bin/git','cat-file','--batch'],input=('\n'.join(oids)+'\n').encode());originals={};cursor=0
for oid in oids:
 end=raw.index(b'\n',cursor);actual,kind,size=raw[cursor:end].decode().split();size=int(size);assert actual==oid and kind=='blob';originals[oid]=raw[end+1:end+1+size];cursor=end+size+2
assert cursor==len(raw)
preimages=0
for path,archive in zip(sorted(OUT.glob('*/inputs.json')),archives):
 assert seal(path.parent/'run-checks.py')==archive['actual_driver_prelaunch']
 for row in archive['sources']:
  if 'original_git_blob'in row:buf=originals[row['original_git_blob']];assert row['original_commit']==BASE
  else:buf=Path(row['complete_changed_input']).read_bytes()
  assert len(buf)==row['bytes']and hashlib.sha256(buf).hexdigest()==row['sha256'];preimages+=1
executions=[]
for path in sorted(OUT.glob('*/*/result.json')):
 row=json.loads(path.read_bytes());assert row['status']in {'passed','failed'}and row['sources']==json.loads((path.parent.parent/'inputs.json').read_bytes())['sources']
 if row['formatter_mutation_expected']:assert row['command'][0].endswith('/gofmt')and '-w'in row['command']
 else:assert row['whole_source_inputs_unchanged']
 for key in ['stdout','stderr']:assert seal(row[key]['path'])==row[key]
 executions.append({'result':seal(path),'command':row['command'],'status':row['status'],'formatter_mutation_expected':row['formatter_mutation_expected']})
assert len(executions)==24
read_native=runpy.run_path(str(OUT/'read-native.py'))['read'];native_files=0;native_bytes=0;source_objects=0;route_counts={}
def nested(data,name):
 if isinstance(data,dict):
  for key,value in data.items():
   if key in {'stdout','stderr','original_expression','whole_original_source','original_source'}and (isinstance(value,str)or value is None):payload(name+'#'+key,value)
   elif key=='whole_original_sources':
    for file,buf in value.items():payload(name+'#'+file,buf)
   else:nested(value,name+'/'+key)
 elif isinstance(data,list):
  for i,value in enumerate(data):nested(value,name+f'/{i}')
for epoch in ['original-owned','complete-shapes','source-budget','alias-observations','exact-locators']:
 manifest,files=read_native(epoch);native_files+=len(files);native_bytes+=sum(map(len,files.values()))
 for name,buf in files.items():
  proxy=str(OUT/f'native/{epoch}/{name}');scan(proxy,buf)
  if name.endswith('.json'):nested(json.loads(buf),proxy)
  if name.startswith('original-source-objects/')and name.endswith('.json'):
   obj=json.loads(buf);original=files[name[:-5]+'.raw'];source_object(obj,original);assert name.rsplit('/',1)[1][:-5]==obj['id'];source_objects+=1
 routes=[(name,json.loads(buf))for name,buf in files.items()if name.startswith('storage-route-')and name.endswith('.json')];route_counts[epoch]=len(routes)
 assert len(routes)==(14 if epoch=='original-owned'else 28 if epoch in {'complete-shapes','source-budget'}else 29 if epoch=='alias-observations'else 30)
 for name,route in routes:
  proxy=str(OUT/f'native/{epoch}/{name}');whole=payload(proxy+'#whole',route['whole_original_source']);expression=payload(proxy+'#expression',route['original_expression']);source_object(route['source_object'],whole)
  span=route['span'];assert span['source_id']==route['source_object']['id']and whole[span['start']:span['end']]==expression
  assert not route['source_type_ID_or_ordinary_semantics_proof']and not route['registered_context_or_live_owner_permission_proof']
  types=route['native_types'];assert route['result_type_node']in range(len(types))and route['root']['type_node']in range(len(types))and len(route['steps'])<=16
  for n,node in enumerate(types):
   assert node['node']==n and node['kind']in {'bool','word','string','error','array','slice','struct','named','alias','builtin_alias'}and 0<=node['depth']<=16
   if node['kind']=='word':assert node['width']in {8,16,32,64}and type(node['signed'])is bool
   if node['kind']in {'named','alias','builtin_alias','array','slice'}:assert 0<=node['child']<n
   if node['kind']=='array':assert 0<=node['length']<=256
   if node['kind']=='struct':assert len(node['fields'])<=64 and all(0<=f['type_node']<n for f in node['fields'])
  for decl in route['original_declarations']:
   buf=payload(proxy+'#declaration',decl['whole_original_source']);source_object(decl['source_object'],buf);s=decl['span'];assert s['source_id']==decl['source_object']['id']and 0<=s['start']<s['end']<=len(buf)
  at=route['root']['type_node']
  for step in route['steps']:
   assert step['parent_type_node']==at and step['result_type_node']in range(len(types));parent=at
   while types[parent]['kind']in {'named','alias','builtin_alias'}:parent=types[parent]['child']
   if step['kind']=='field':assert types[parent]['kind']=='struct'and 0<=step['field']<len(types[parent]['fields'])and types[parent]['fields'][step['field']]['type_node']==step['result_type_node']
   else:
    assert step['kind']in {'array_index','slice_index'}and types[parent]['kind']==('array'if step['kind']=='array_index'else'slice')and types[parent]['child']==step['result_type_node']
    index=step['index'];s=index['span'];assert s['source_id']==span['source_id']and 0<=s['start']<s['end']<=len(whole)and index['width']in {8,16,32,64}and type(index['signed'])is bool
   at=step['result_type_node']
  assert at==route['result_type_node']
  if route['root']['kind']=='slice_descriptor':assert route['steps'][0]['kind']=='slice_index'and route['root']['binding']['source_id']==''
  else:assert route['root']['kind']=='value_binding'and route['root']['binding']['source_id']
 assert all(json.loads(buf)['error']and not json.loads(buf)['result_has_source']for name,buf in files.items()if name.startswith('storage-rejection-'))
# Full current frontend input capture includes both new production files.
build=json.loads((current/'analysis-inputs/captured-inputs.json').read_bytes());assert len(build['build_inputs'])==188
for row in build['build_inputs']:source_object(row,(Path('go-tools/go2vir')/row['path']).read_bytes())
reproduced=json.loads((current/'analysis-inputs/reproduced-builds.json').read_bytes());assert reproduced['byte_exact_equal']and len(reproduced['builds'])==2 and not reproduced['native_Linux_acceptance_claimed']and not reproduced['registered_practical_context']
frozen=json.loads(Path('develop/migrations/go-practical/t02-w01/verification/native-observation/frozen-design-preservation.json').read_bytes())['inputs'];assert len(frozen)==255 and all(seal(r['path'])==r for r in frozen)
old=json.loads(Path('develop/migrations/go-practical/t02-w01/verification/string-constants/alias-display/analysis-inputs/captured-inputs.json').read_bytes())['build_inputs'];assert len(old)==186
for row in old:
 buf=(Path('go-tools/go2vir')/row['path']).read_bytes();assert len(buf)==row['bytes']and hashlib.sha256(buf).hexdigest()==row['raw_sha256']
for name in ['develop/docs/go-practical-implementation-traceability-ledger.md','develop/migrations/go-practical/t02-w01/README.md']:assert Path(name).read_bytes().startswith(git('show',BASE+':'+name))
original=json.loads(git('show',BASE+':develop/migrations/go-practical/t02-w01/requirements.json'));latest=json.loads(Path('develop/migrations/go-practical/t02-w01/requirements.json').read_bytes())
for row in latest['requirements']:
 if row['id']in {'T02-W01-R01','T02-W01-R06'}:assert row.pop('preparatory_original_storage_route_checkpoint')['status']=='scoped_checks_passed'
assert latest==original
missing=[r for r in links if not r['exists']];unchanged=all(seal(r['path'])==r for r in inputs)
record={'status':'passed'if not findings and not missing and unchanged else'failed','basis_commit':BASE,'observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'inputs':inputs,'source_inputs_unchanged':unchanged,'findings':findings,'missing_links':missing,'links_checked':len(links),'source_epochs':len(archives),'verified_source_preimages':preimages,'actual_executions':executions,'affected_Go_roots':roots,'subtests':subs,'native_original_files':native_files,'native_original_bytes':native_bytes,'verified_source_objects':source_objects,'native_route_counts':route_counts,'complete_decoded_data_payloads':decoded,'frontend_build_inputs':188,'previous_frontend_inputs_unchanged':186,'frozen_inputs_unchanged':255,'matcher_scope_seal_exclusion_counts':[127,4,39,51],'new_source_JSON_directory_or_blanket_exclusions':False,'Rust_core_and_view_sources_unchanged':True,'source_type_ID_or_ordinary_semantics_proof':False,'registered_context_or_live_owner_permission_proof':False,'native_release_acceptance':False,'full_T_gate_passed':False}
scan(str(REPORT.relative_to(ROOT)),json.dumps(record,indent=2).encode());record['findings']=findings;record['status']='passed'if not findings and not missing and unchanged else'failed'
REPORT.write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({k:v for k,v in record.items()if k not in {'inputs','actual_executions','complete_decoded_data_payloads'}}));raise SystemExit(record['status']!='passed')
