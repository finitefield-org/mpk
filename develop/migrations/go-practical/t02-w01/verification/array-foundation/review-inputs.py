"""Review complete frozen affixes sources, actual checker inputs and observations."""
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
BASE = "29aef5e2504cf4a753ceae571bbe30e26db6be6d"
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
module = types.ModuleType("mpk_array_foundation_input_review")
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
configured = scanner_files | fixtures | module.active_retained_record_exclusions(manifest)
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
    if path.suffix == ".json":
        row = json.loads(raw)
        def data_payloads(value, location=""):
            if isinstance(value,dict):
                for key,item in value.items():
                    data_payloads(item,location+"/"+key)
            elif isinstance(value,list):
                if value and all(isinstance(item,(int,bool))and 0<=item<=255 for item in value)and ("bytes"in location or "bits"in location):
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

pair=json.loads((OUT/'paired/result.json').read_bytes())
inventory=json.loads((OUT/'epochs/remaining-controls/selected/candidates/control-inventory.json').read_bytes())
total=inventory['complete_candidates'];negative=total-2
assert pair['status']=='passed' and len(pair['cases'])==total
assert pair['zero_axiom_acceptances']==2 and pair['genuine_core_type_mismatches']==negative
assert inventory['altered_conclusions']==303 and inventory['typed_wrong_definitions']==37 and inventory['actual_typed_witness_domains']>60
for key in ['actual_driver_prelaunch','Rust_execution','Rust_execution_at_launch','Rust_execution_at_completion','Rust_foundation_execution','Rust_foundation_original','unchanged_Go_build_receipt','go_binary']:
    assert seal(pair[key]['path'])==pair[key]
assert all(seal(row['path'])==row for row in pair['go_production_sources'])
for row in pair['cases']:
    for key in ['input','Rust_report','stdout','stderr']:assert seal(row[key]['path'])==row[key]
    rust=json.loads(Path(row['Rust_report']['path']).read_bytes());go=json.loads(Path(row['stdout']['path']).read_bytes())
    assert rust==row['Rust'] and go==row['Go']
    if row['expected_acceptance']:
        assert rust['verdict']==go['verdict']=='accepted' and rust['hashes']==go['hashes']
        assert rust['axiom_count']==0 and not rust['axiom_report']['entries']
        assert go['hashes']['axiom_report']=='0ebc281c3a8d37e2d1a9ce033773e2865f96a13186a6364cb3446204c6a990d5'
    else:assert json.loads(rust['error_detail'])['code']=='CORE_TYPE_MISMATCH' and go['error_kind']=='core_check'
archives=[json.loads(path.read_bytes())for path in sorted((OUT/'epochs').glob('*/inputs.json'))]
assert len(archives)==9
oids=sorted({row['original_git_blob']for archive in archives for row in archive['sources']if 'original_git_blob'in row})
blob=subprocess.check_output(['/usr/bin/git','cat-file','--batch'],input=('\n'.join(oids)+'\n').encode());originals={};cursor=0
for oid in oids:
    end=blob.index(b'\n',cursor);actual,kind,size=blob[cursor:end].decode().split();size=int(size);assert actual==oid and kind=='blob'
    originals[oid]=blob[end+1:end+1+size];cursor=end+size+2
assert cursor==len(blob)
preimages=0
for archive in archives:
    for row in archive['sources']:
        if 'original_git_blob'in row:raw=originals[row['original_git_blob']];assert row['original_commit']==BASE
        else:raw=Path(row['complete_changed_input']).read_bytes()
        assert len(raw)==row['bytes'] and hashlib.sha256(raw).hexdigest()==row['sha256'];preimages+=1
for path in (OUT/'epochs').glob('*/inputs.json'):assert seal(path.parent/'run-checks.py')['sha256']==json.loads(path.read_bytes())['runner_sha256']
executions=[]
for path in sorted((OUT/'epochs').glob('*/*/result.json')):
    row=json.loads(path.read_bytes());assert row['status']in {'passed','failed'} and row['whole_source_inputs_unchanged']
    assert row['source_inputs']==json.loads((path.parent.parent/'inputs.json').read_bytes())['sources']
    executions.append({'result':seal(path),'stdout':seal(path.parent/'stdout.log'),'stderr':seal(path.parent/'stderr.log'),'status':row['status'],'command':row['command']})
assert len(executions)==14 and sum(row['status']=='failed'for row in executions)==5
for epoch,code in [('complete-carriers','CORE_NOT_A_FUNCTION'),('explicit-equivalence','CORE_FUEL_EXHAUSTED')]:
    candidate=OUT/f'epochs/{epoch}/foundation/candidates/array-foundation.mpcert'
    rust=json.loads(candidate.with_suffix('.rust.json').read_bytes())
    assert rust['verdict']=='rejected' and json.loads(rust['error_detail'])['code']==code
assert 'E0505'in (OUT/'epochs/full-index-relations/foundation/stderr.log').read_text()
assert not list((OUT/'epochs/full-index-relations/foundation/candidates').glob('*.mpcert'))
for mode in ['foundation','observations']:
    assert 'E0425'in (OUT/f'epochs/native-controls/{mode}/stderr.log').read_text()
    assert not list((OUT/f'epochs/native-controls/{mode}/candidates').glob('*.mpcert'))
original_failures=json.loads((OUT/'original-failures/result.json').read_bytes())
assert original_failures['status']=='passed' and original_failures['original_inputs_rejected']==2
assert not original_failures['original_Rust_outputs_reconstructed'] and not original_failures['current_resource_limits_changed']
for key in ['actual_driver_prelaunch','go_binary']:assert seal(original_failures[key]['path'])==original_failures[key]
assert all(seal(row['path'])==row for row in original_failures['go_production_sources'])
for row in original_failures['cases']:
    for key in ['input','original_Rust_report','original_source_inputs','original_Rust_execution','stdout','stderr']:assert seal(row[key]['path'])==row[key]
    assert row['Rust']==json.loads(Path(row['original_Rust_report']['path']).read_bytes())
    assert row['Go']==json.loads(Path(row['stdout']['path']).read_bytes())
    assert row['Rust']['verdict']==row['Go']['verdict']=='rejected' and row['Go']['error_kind']=='core_check'
current=json.loads((OUT/'epochs/remaining-controls/inputs.json').read_bytes())
prior=json.loads((OUT/'epochs/harness-context/inputs.json').read_bytes())
assert [(row['path'],row['sha256'],row['bytes'])for row in current['sources']]==[(row['path'],row['sha256'],row['bytes'])for row in prior['sources']]
assert all(seal(row['path'])['sha256']==row['sha256']for row in current['sources'])
selected=json.loads((OUT/'epochs/remaining-controls/selected/result.json').read_bytes())
assert selected['status']=='passed' and selected['command'][-4:]==['--skip','arrays_preserve_all2254_prior_identities','--skip','observations_cover_every_frozen_shape_and_full_index_boundaries']
for mode in ['foundation','observations','lint','format']:assert json.loads((OUT/f'epochs/harness-context/{mode}/result.json').read_bytes())['status']=='passed'
assert not git('diff','--name-only','HEAD','--','crates/mpk-vc/src/go_practical_ordinary/syntax.rs','crates/mpk-vc/src/go_practical_ordinary/composition.rs','crates/mpk-vc/src/go_practical_ordinary/string/equality.rs','crates/mpk-vc/src/go_practical_ordinary/string/order','crates/mpk-vc/src/go_practical_ordinary/string/slice/carrier.rs','crates/mpk-vc/src/go_practical_ordinary/string/slice/guards.rs','crates/mpk-vc/src/go_practical_ordinary/string/slice/preservation.rs','crates/mpk-vc/src/go_practical_ordinary/string/slice/signed.rs','crates/mpk-vc/tests/support/go_string_equality_eval.rs','Cargo.toml','Cargo.lock','proofs')
fixtures=OUT/'epochs/remaining-controls/selected/candidates'
assert len(list(fixtures.glob('*.mpcert')))==total-1
assert len(list(fixtures.glob('false-*.mpcert')))==303 and len(list(fixtures.glob('premise-*.mpcert')))==inventory['actual_typed_witness_domains'] and len(list(fixtures.glob('definition-*.mpcert')))==37

observations=OUT/'epochs/harness-context/observations/candidates'
assert (observations/'observed-array-foundation.mpcert').read_bytes()==(OUT/'epochs/harness-context/foundation/candidates/array-foundation.mpcert').read_bytes()
assert len(list(observations.glob('array-*.input.json')))==22 and len(list(observations.glob('array-*.observation.json')))==132
steps=read_count=0
for path in observations.glob('array-*.input.json'):
    source=json.loads(path.read_bytes());n=source['logical_length'];d=source['depth'];capacity=source['capacity'];raw=bytes(source['all256_source_bytes'])
    assert len(raw)==256 and 1<=n<=256 and capacity==1<<(n-1).bit_length() and d==capacity.bit_length()-1
    assert source['all64_length_bits']==[bool(n&(1<<i))for i in range(64)] and not source['ordinary_source_or_nominal_proof'] and not source['source_or_storage_authority']
    for op in ['raw','copy','zero','normalize','set-first','set-last']:
        stem=path.name.removesuffix('.input.json')+'-'+op
        output=json.loads((observations/(stem+'.observation.json')).read_bytes());expected=list(raw[:capacity])
        if op=='zero':expected=[0]*capacity
        if op=='normalize':expected[n:]=[0]*(capacity-n)
        if op=='set-first':expected[0]=7
        if op=='set-last':expected[n-1]=7
        assert output['all_capacity_bytes']==output['expected_all_capacity_bytes']==expected
        assert output['fresh_output_probes_keep_complete_inputs'] and not output['definition_callbacks'] and not output['theorem_bodies_executed'] and not output['source_or_Go_storage_proof']
        used=0
        for start in range(0,capacity,32):
            partial=json.loads((observations/(stem+f'.chunk-{start}.partial.json')).read_bytes())
            assert partial['observed_prefix_bytes']==expected[:min(capacity,start+32)] and partial['complete_original_input_retained'];used=partial['actual_AST_steps']
        reads=[json.loads(p.read_bytes())for p in observations.glob(stem+'.read-*.partial.json')]
        assert sorted(row['full64_index']for row in reads)==sorted([0,n//2,n-1])
        for row in reads:
            i=row['full64_index'];assert row['actual']==row['expected']==expected[i] and row['original_unchanged_byte']==row['expected_original_byte']==raw[i]
            assert row['all64_index_bits']==[bool(i&(1<<j))for j in range(64)];used+=row['actual_AST_steps'];read_count+=1
        assert used==output['actual_AST_steps'];steps+=used
assert len(list(observations.glob('guard-*.input.json')))==len(list(observations.glob('guard-*.observation.json')))==9
for path in observations.glob('guard-*.input.json'):
    source=json.loads(path.read_bytes());n,i=source['length'],source['index'];outcome=json.loads(path.with_name(path.name.replace('.input.json','.observation.json')).read_bytes())
    assert source['all64_length_bits']==[bool(n&(1<<j))for j in range(64)] and source['all64_index_bits']==[bool(i&(1<<j))for j in range(64)]
    assert source['raw_outside_profile']==(n>256) and not source['admitted_source_counterexample']
    assert outcome['unsigned']==outcome['expected_unsigned']==(i<n) and outcome['signed']==outcome['expected_signed']==(i<n and i<2**63)
    assert not outcome['out_of_bounds_read_executed'] and not outcome['definition_callbacks'] and not outcome['theorem_bodies_executed'];steps+=outcome['actual_AST_steps']
empty=json.loads((observations/'empty.observation.json').read_bytes());assert empty['canonical_dummy']==empty['expected_dummy']==False and not empty['source_or_storage_authority'];steps+=empty['actual_AST_steps']
obs_inventory=json.loads((observations/'observation-inventory.json').read_bytes())
assert obs_inventory=={'actual_AST_steps':34700456,'all_depths_0_through_8':True,'complete_array_outputs':132,'definition_callbacks':False,'empty_dummy_cases':1,'full64_guard_cases':9,'full64_reads':396,'observer_ceilings_changed':False,'positive_lengths':22,'theorem_bodies_executed':False}
assert steps==34700456 and read_count==396
countermodels=list(fixtures.glob('definition-*.countermodel.json'));assert len(countermodels)==37
for path in countermodels:
    output=json.loads(path.read_bytes());source=json.loads(path.with_name(path.name.replace('.countermodel.json','.input.json')).read_bytes());n=source['logical_length']
    assert len(source['all256_source_bytes'])==256 and source['all64_length_bits']==[bool(n&(1<<j))for j in range(64)]
    assert not source['source_or_nominal_authority'] and not source['admitted_source_counterexample']
    assert output['actual_wrong_definition_countermodel'] and not output['definition_callbacks'] and not output['theorem_bodies_executed'] and output['actual']!=output['expected']
frozen=json.loads(Path('develop/migrations/go-practical/t02-w01/verification/native-observation/frozen-design-preservation.json').read_bytes())['inputs'];assert len(frozen)==255 and all(seal(row['path'])==row for row in frozen)
go_inputs=json.loads(Path('develop/migrations/go-practical/t02-w01/verification/string-constants/alias-display/analysis-inputs/captured-inputs.json').read_bytes())['build_inputs'];assert len(go_inputs)==186
for row in go_inputs:
    raw=(Path('go-tools/go2vir')/row['path']).read_bytes();assert len(raw)==row['bytes'] and hashlib.sha256(raw).hexdigest()==row['raw_sha256']
for name in ['develop/docs/go-practical-implementation-traceability-ledger.md','develop/migrations/go-practical/t02-w01/README.md']:assert Path(name).read_bytes().startswith(git('show',BASE+':'+name))
original=json.loads(git('show',BASE+':develop/migrations/go-practical/t02-w01/requirements.json'));latest=json.loads(Path('develop/migrations/go-practical/t02-w01/requirements.json').read_bytes());row=next(row for row in latest['requirements']if row['id']=='T02-W01-R01')
assert row.pop('preparatory_array_checkpoint')['status']=='scoped_checks_passed' and latest==original
if (OUT/'git-whitespace-review.json').exists():
    white=json.loads((OUT/'git-whitespace-review.json').read_bytes());assert white['status']=='passed_with_original_output_retained' and white['all_other_staged_files_pass_whitespace_check']
    assert not white['original_Cargo_output_bytes_modified'] and not white['obsolete_IR_matcher_scope_seal_or_exclusion_changes']
    for warning in white['exact_raw_output_warnings']:assert seal(warning['path'])=={key:warning[key]for key in ['path','sha256','bytes']}
missing=[row for row in links if not row['exists']];unchanged=all(seal(row['path'])==row for row in inputs)
record={'status':'passed'if not findings and not missing and unchanged else'failed','basis_commit':BASE,'observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'inputs':inputs,'source_inputs_unchanged':unchanged,'findings':findings,'missing_links':missing,'links_checked':len(links),'complete_same_byte_inputs':total,'zero_axiom_acceptances':2,'genuine_core_type_mismatches':negative,'full_original_source_epochs':len(archives),'verified_source_preimages':preimages,'actual_executions':executions,'affected_Rust_roots':5,'passed_roots_repeated_without_source_change':False,'positive_array_lengths':22,'complete_array_outputs':132,'full64_reads':396,'full64_guard_results':9,'empty_dummy_cases':1,'actual_AST_steps':steps,'typed_wrong_definition_countermodels':37,'original_failed_inputs_independently_rejected':2,'source_or_native_Go_proof_from_model_claimed':False,'complete_decoded_data_payloads':decoded,'frozen_inputs_unchanged':255,'Go_frontend_build_inputs_unchanged':186,'matcher_scope_seal_counts':[127,4,39],'new_source_JSON_directory_or_blanket_exclusions':False,'core_and_Go_ref_implementations_unchanged':True,'native_release_acceptance':False,'full_T_gate_passed':False}
scan(str(REPORT.relative_to(ROOT)),json.dumps(record,indent=2).encode());record['findings']=findings;record['status']='passed'if not findings and not missing and unchanged else'failed'
REPORT.write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({key:value for key,value in record.items()if key not in {'inputs','actual_executions','complete_decoded_data_payloads'}}));raise SystemExit(record['status']!='passed')
