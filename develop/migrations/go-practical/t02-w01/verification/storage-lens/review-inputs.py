"""Review complete compositional storage views and full original proof/native inputs."""
from pathlib import Path
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
BASE = "6c347926a1722d86bf1cfe8b3062eef1a8bac993"
REPORT = OUT/"input-review.json"


def git(*args):
    return subprocess.check_output(["/usr/bin/git", *args])


def seal(path, raw=None):
    raw = Path(path).read_bytes() if raw is None else raw
    return {"path": str(path), "sha256": hashlib.sha256(raw).hexdigest(), "bytes": len(raw)}


assert git("rev-parse", "HEAD").decode().strip() == BASE
assert not git("diff", "--name-only", "HEAD", "--", "crates/mpk-core", "crates/mpk-cert", "crates/mpk-kernel", "go-tools", "release")
assert not Path(".github/workflows").exists()
scanner = Path("scripts/check-no-active-gir.sh")
code = scanner.read_text().split("<<'PY'\n",1)[1].rsplit("\nPY",1)[0].rsplit("raise SystemExit(main())",1)[0]
module = types.ModuleType("mpk_storage_lens_input_review")
module.__file__ = str(ROOT/scanner)
sys.modules[module.__name__] = module
sys.argv = [str(scanner),str(ROOT),"strict"]
exec(compile(code,str(scanner),"exec"),module.__dict__)
matchers = module.load_matchers()
manifest, scanner_files, fixtures, _ = module.validate_manifest()
module.run_fixture_self_test(manifest,matchers)
assert (len(matchers),len(module.SCOPES),len(manifest["sealed_records"])) == (127,4,39)
for name in scanner_files | fixtures:
    assert Path(name).read_bytes() == git("show",BASE+":"+name)
exclusions=module.active_retained_record_exclusions(manifest)
assert len(exclusions)==51
configured = scanner_files | fixtures | exclusions
findings, inputs, links, decoded = [], [], [], []
cache = {}
rules = [m for m in matchers if m["scope"] == "exact-path"]


def scan(name,raw):
    key = (hashlib.sha256(raw).hexdigest(),tuple(module.path_matches(name,m["path"]) for m in rules))
    if key not in cache:
        cache[key] = [vars(f) for f in module.detect(name,raw.decode(errors="ignore"),matchers)]
    findings.extend({**finding,"path":name} for finding in cache[key])


# The report is a prospective output, not an absent input dependency.
if not REPORT.exists():
    REPORT.write_text(json.dumps({"status":"running","basis_commit":BASE})+"\n")
selected = git("diff","--name-only","-z","HEAD","--") + git("ls-files","--others","--exclude-standard","-z")
names = sorted(set(selected.decode().split("\0")) - {"",str(REPORT.relative_to(ROOT))})
for name in names:
    path = ROOT/name
    assert path.is_file() and not path.is_symlink(), name
    raw = path.read_bytes()
    inputs.append(seal(name,raw))
    if name not in configured:
        scan(name,raw)
    if path.suffix in {".json", ".log"}:
        try: row = json.loads(raw)
        except (ValueError, UnicodeError): row = None
        def data_payloads(value, location=""):
            if isinstance(value,dict):
                for key,item in value.items():
                    data_payloads(item,location+"/"+key)
            elif isinstance(value,list):
                if value and all(isinstance(item,(int,bool))and 0<=item<=255 for item in value)and ("bytes"in location.lower() or "bits"in location.lower() or "replacement"in location.lower()):
                    payload=bytes(value);scan(name+"#"+location,payload);decoded.append(seal(name+"#"+location,payload))
                else:
                    for i,item in enumerate(value):data_payloads(item,location+"/"+str(i))
        data_payloads(row)
    if path.suffix == ".md":
        for target in re.findall(r"(?<!!)\[[^\]]*\]\(([^\n)]+)\)",raw.decode()):
            target = target.strip().strip("<>")
            if re.match(r"^[a-zA-Z][a-zA-Z0-9+.-]*:",target) or target.startswith("#"):
                continue
            clean = urllib.parse.unquote(target.split("#",1)[0])
            links.append({"document":name,"target":target,"exists":(path.parent/clean).resolve().exists()})

def read(path):return json.loads(Path(path).read_bytes())
epoch=OUT/'epochs/complete-harness';pair=read(OUT/'paired/result.json');inventory=read(epoch/'laws/candidates/control-inventory.json')
total=inventory['complete_candidates'];negative=total-2
assert inventory['altered_conclusions']==181 and inventory['typed_wrong_definitions']==16 and inventory['actual_typed_witness_domains']>100
assert pair['status']=='passed'and pair['same_byte_inputs']==len(pair['cases'])==total
assert pair['zero_axiom_acceptances']==2 and pair['genuine_core_type_mismatches']==negative
for key in ['actual_driver_prelaunch','go_binary','unchanged_Go_build_receipt']:assert seal(pair[key]['path'])==pair[key]
assert all(seal(r['path'])==r for r in pair['go_production_sources'])
for row in pair['Rust_executions_at_launch'].values():assert seal(row['path'])==row
for row in pair['Rust_executions_at_completion'].values():
    for v in row.values():assert seal(v['path'])==v
for case in pair['cases']:
    for key in ['input','Rust_report','stdout','stderr']:assert seal(case[key]['path'])==case[key]
    rust=read(case['Rust_report']['path']);go=read(case['stdout']['path']);assert rust==case['Rust']and go==case['Go']
    if case['expected_acceptance']:
        assert rust['verdict']==go['verdict']=='accepted'and rust['hashes']==go['hashes']and rust['axiom_count']==0 and not rust['axiom_report']['entries']
    else:assert rust['verdict']==go['verdict']=='rejected'and json.loads(rust['error_detail'])['code']=='CORE_TYPE_MISMATCH'and go['error_kind']=='core_check'
archives=[read(p)for p in sorted((OUT/'epochs').glob('*/inputs.json'))];assert len(archives)==11
oids=sorted({r['original_git_blob']for a in archives for r in a['sources']if 'original_git_blob'in r})
raw=subprocess.check_output(['/usr/bin/git','cat-file','--batch'],input=('\n'.join(oids)+'\n').encode());originals={};cursor=0
for oid in oids:
    end=raw.index(b'\n',cursor);actual,kind,size=raw[cursor:end].decode().split();size=int(size);assert actual==oid and kind=='blob';originals[oid]=raw[end+1:end+1+size];cursor=end+size+2
assert cursor==len(raw)
preimages=0
for archive in archives:
    for row in archive['sources']:
        if 'original_git_blob'in row:
            buf=originals[row['original_git_blob']];assert row['original_commit']==BASE
        else:buf=Path(row['complete_changed_input']).read_bytes()
        assert len(buf)==row['bytes']and hashlib.sha256(buf).hexdigest()==row['sha256'];preimages+=1
for path in (OUT/'epochs').glob('*/inputs.json'):assert seal(path.parent/'run-checks.py')['sha256']==read(path)['runner_sha256']
executions=[]
for path in sorted((OUT/'epochs').glob('*/*/result.json')):
    row=read(path);assert row['status']in {'passed','failed'}and row['source_inputs']==read(path.parent.parent/'inputs.json')['sources']
    if row['formatter_mutation_expected']:
        assert row['status']=='passed'and row['command'][0]=='rustfmt'and '--check'not in row['command']
    else:assert row['whole_source_inputs_unchanged']
    executions.append({'result':seal(path),'stdout':seal(path.parent/'stdout.log'),'stderr':seal(path.parent/'stderr.log'),'status':row['status'],'command':row['command'],'formatter_mutation_expected':row['formatter_mutation_expected']})
assert len(executions)==17 and sum(r['status']=='failed'for r in executions)==4
for name in ['initial-foundation','construction-diagnostic']:
    path=OUT/'epochs'/name/'foundation';assert read(path/'result.json')['status']=='failed'and not list((path/'candidates').glob('*.mpcert'))
failed=read(OUT/'original-failures/result.json');assert failed['status']=='passed'and failed['original_inputs_rejected']==2 and not failed['original_Rust_outputs_reconstructed']and not failed['current_resource_limits_changed']
for key in ['actual_driver_prelaunch','go_binary']:assert seal(failed[key]['path'])==failed[key]
for row in failed['cases']:
    for key in ['input','original_Rust_report','original_source_inputs','original_Rust_execution','stdout','stderr']:assert seal(row[key]['path'])==row[key]
    assert row['Rust']==read(row['original_Rust_report']['path'])and row['Go']==read(row['stdout']['path'])
    assert row['Rust']['verdict']==row['Go']['verdict']=='rejected'and row['Go']['error_kind']=='core_check'
current=read(epoch/'inputs.json')['sources'];assert all(seal(r['path'])['sha256']==r['sha256']for r in current)
for mode in ['foundation','observations','definitions','alphabet','laws','lint','format']:
    row=read(epoch/mode/'result.json');assert row['status']=='passed'and row['source_inputs']==current and row['whole_source_inputs_unchanged']
assert not git('diff','--name-only','HEAD','--','crates/mpk-vc/src/go_practical_ordinary/syntax.rs','crates/mpk-vc/src/go_practical_ordinary/product.rs','crates/mpk-vc/src/go_practical_ordinary/product_relation.rs','crates/mpk-vc/src/go_practical_ordinary/array.rs','crates/mpk-vc/src/go_practical_ordinary/array','crates/mpk-vc/src/go_practical_ordinary/string','crates/mpk-vc/tests/support/go_string_equality_eval.rs','Cargo.toml','Cargo.lock','proofs','release')
foundation=epoch/'foundation/candidates/lens-foundation.mpcert';assert foundation.stat().st_size==1115975
assert (epoch/'observations/candidates/observed-lens-foundation.mpcert').read_bytes()==foundation.read_bytes()
for name,status in [('native-go','failed'),('native-go-current','passed')]:
    native=read(OUT/name/'result.json');assert native['status']==status
    for key in ['actual_driver_prelaunch','pinned_original_sdk_receipt','whole_original_sdk_inputs','lossless_original_sdk_transport','compiler_binary','cases_input']:assert seal(native[key]['path'])==native[key]
    assert all(seal(r['path'])==r for r in native['sources'])
    for command in native['commands']:
        assert command['exit_code']==0
        for key in ['stdout','stderr']:assert seal(command[key]['path'])==command[key]
        assert not Path(command['stderr']['path']).read_bytes()
    assert Path(native['commands'][0]['stdout']['path']).read_text().strip()=='go version go1.25.0 darwin/arm64'
    assert not native['source_or_registered_context_proof']and not native['native_Linux_release_acceptance']and not native['closed_profile_source_admission_claimed']
# The first Go run built the same output binary path; its exact original bytes
# differ because its missing JSON tag is retained in its saved source. Only
# the current binary is asserted above; the original binary hash stays evidence.
assert seal(native['oracle_binary']['path'])==native['oracle_binary']
binary_archive=read(OUT/'native-binary-preservation.json');assert binary_archive['status']=='current_original_binary_retained'and not binary_archive['previous_failed_binary_bytes_retained']and not binary_archive['previous_failed_binary_original_bytes_reconstructed']
encoded=binary_archive['whole_exact_hex_transport'];assert seal(encoded['path'])==encoded
decoded_binary=bytes.fromhex(Path(encoded['path']).read_text());assert len(decoded_binary)==native['oracle_binary']['bytes']and hashlib.sha256(decoded_binary).hexdigest()==native['oracle_binary']['sha256']
scan(encoded['path']+'#complete-decoded-current-binary',decoded_binary)
assert native['whole_input_cases']==203 and native['full_root_snapshots']==1015 and native['array_lengths']==22 and native['nested_array_splits']==9 and native['path_lengths']==16
sdk=read(native['whole_original_sdk_inputs']['path']);assert len(sdk)==14496 and sum(r['bytes']for r in sdk)==202722007 and all(seal(r['path'])==r for r in sdk)
root=Path(native['environment']['GOROOT']);assert {str(p)for p in root.rglob('*')if p.is_file()}=={r['path']for r in sdk}
transport=read(native['lossless_original_sdk_transport']['path']);parts=[]
for row in transport['chunks']:
    path=Path(native['lossless_original_sdk_transport']['path']).parent.parent/row['relative_path'];encoded=path.read_bytes();assert seal(path)==row['encoded'];buf=bytes.fromhex(encoded.decode());assert len(buf)==row['raw_bytes']and hashlib.sha256(buf).hexdigest()==row['raw_sha256'];parts.append(buf)
whole=b''.join(parts);assert len(whole)==transport['original_archive']['bytes']==57906112 and hashlib.sha256(whole).hexdigest()==transport['original_archive']['sha256']
cases=read(OUT/'native-go-current/cases.json');oracle=read(OUT/'native-go-current/views.stdout.log');assert len(cases)==len(oracle)==203
observations=epoch/'observations/candidates';steps=outputs=0
for case,go in zip(cases,oracle):
    name=case['id'];source=read(observations/(name+'.input.json'))
    assert source['whole_original_case']==case and len(case['bytes'])==len(case['replacement'])==256 and not source['source_type_path_live_store_or_permission_proof']
    assert source['all64_index_bits']==[bool(case['index']&(1<<i))for i in range(64)]
    out=read(observations/(name+'.observation.json'));assert out['actual_ordinary']==out['original_native_Go']==go and not out['definition_callbacks']and not out['theorem_bodies']and 0<out['actual_AST_steps']<=50000000
    outputs+=5;used=out['actual_AST_steps']
    if case['kind']=='nested':
        out=read(observations/(name+'-associative.observation.json'));assert out['actual_ordinary']==out['original_native_Go']==go['snapshots'][3]and 0<out['actual_AST_steps']<=50000000;used=out['actual_AST_steps'];outputs+=1
    steps+=used
assert outputs==1060 and steps==67909080
assert read(observations/'observation-inventory.json')=={'actual_AST_steps':67909080,'definition_callbacks':False,'full_root_outputs':1060,'observer_ceilings_changed':False,'shared_alias_reads':406,'theorem_bodies':False,'whole_original_cases':203}
countermodels=list((epoch/'definitions/candidates').glob('definition-*.countermodel.json'));assert len(countermodels)==16
for path in countermodels:
    out=read(path);source=read(path.with_name(path.name.replace('.countermodel.json','.input.json')));typing=read(path.with_name(path.name.replace('.countermodel.json','.typing.json')))
    assert len(source['whole_bytes'])==256 and source['whole64_index_bits']==[False]*64 and source['raw_model_only']and not source['source_or_live_store_authority']
    assert typing['actual_definition_type_checked']and typing['complete_candidate_saved_before_check']
    assert out['different']and out['actual']!=out['expected']and not out['definition_callbacks']and not out['theorem_bodies']and 0<out['actual_AST_steps']<=50000000
frozen=read('develop/migrations/go-practical/t02-w01/verification/native-observation/frozen-design-preservation.json')['inputs'];assert len(frozen)==255 and all(seal(r['path'])==r for r in frozen)
go_inputs=read('develop/migrations/go-practical/t02-w01/verification/string-constants/alias-display/analysis-inputs/captured-inputs.json')['build_inputs'];assert len(go_inputs)==186
for row in go_inputs:
    buf=(Path('go-tools/go2vir')/row['path']).read_bytes();assert len(buf)==row['bytes']and hashlib.sha256(buf).hexdigest()==row['raw_sha256']
for name in ['develop/docs/go-practical-implementation-traceability-ledger.md','develop/migrations/go-practical/t02-w01/README.md']:assert Path(name).read_bytes().startswith(git('show',BASE+':'+name))
original=json.loads(git('show',BASE+':develop/migrations/go-practical/t02-w01/requirements.json'));latest=read('develop/migrations/go-practical/t02-w01/requirements.json');row=next(r for r in latest['requirements']if r['id']=='T02-W01-R01')
assert row.pop('preparatory_storage_lens_checkpoint')['status']=='scoped_checks_passed'and latest==original
if (OUT/'git-whitespace-review.json').exists():
    white=read(OUT/'git-whitespace-review.json');assert white['status']=='passed_with_original_output_retained'and white['all_other_staged_files_pass_whitespace_check']and not white['original_Cargo_output_bytes_modified']
    for row in white['exact_raw_output_warnings']:assert seal(row['path'])=={k:row[k]for k in ['path','sha256','bytes']}
missing=[r for r in links if not r['exists']];unchanged=all(seal(r['path'])==r for r in inputs)
record={'status':'passed'if not findings and not missing and unchanged else'failed','basis_commit':BASE,'observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'inputs':inputs,'source_inputs_unchanged':unchanged,'findings':findings,'missing_links':missing,'links_checked':len(links),'complete_same_byte_inputs':total,'zero_axiom_acceptances':2,'genuine_core_type_mismatches':negative,'full_original_source_epochs':len(archives),'verified_source_preimages':preimages,'actual_executions':executions,'affected_Rust_roots':5,'native_source_cases':203,'native_root_snapshots':1015,'complete_ordinary_root_outputs':1060,'actual_AST_steps':steps,'typed_wrong_definition_countermodels':16,'original_failed_complete_inputs_independently_rejected':2,'original_construction_failures_before_candidate':2,'original_native_JSON_tag_failure_retained':True,'complete_decoded_data_payloads':decoded,'frozen_inputs_unchanged':255,'Go_frontend_build_inputs_unchanged':186,'whole_original_sdk_files_unchanged':14496,'matcher_scope_seal_exclusion_counts':[127,4,39,51],'new_source_JSON_directory_or_blanket_exclusions':False,'core_and_Go_ref_implementations_unchanged':True,'source_or_storage_permission_proof_claimed':False,'native_release_acceptance':False,'full_T_gate_passed':False}
scan(str(REPORT.relative_to(ROOT)),json.dumps(record,indent=2).encode());record['findings']=findings;record['status']='passed'if not findings and not missing and unchanged else'failed'
REPORT.write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({k:v for k,v in record.items()if k not in ['inputs','actual_executions','complete_decoded_data_payloads']}));raise SystemExit(record['status']!='passed')
