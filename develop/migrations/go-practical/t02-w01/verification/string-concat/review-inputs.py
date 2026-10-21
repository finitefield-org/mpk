"""Review complete frozen concatenation sources, actual checker inputs and observations."""
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
BASE = "089dbb38dc2a241ac6203b455a41e34dabc1424e"
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
module = types.ModuleType("mpk_string_concat_input_review")
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
assert pair['status']=='passed'and len(pair['cases'])==117
assert pair['zero_axiom_acceptances']==2 and pair['genuine_core_type_mismatches']==115
for key in ['actual_driver_prelaunch','Rust_execution','Rust_execution_at_launch','Rust_execution_at_completion','Rust_foundation_execution','Rust_foundation_original','unchanged_Go_build_receipt','go_binary']:
    assert seal(pair[key]['path'])==pair[key]
assert all(seal(row['path'])==row for row in pair['go_production_sources'])
for row in pair['cases']:
    for key in ['input','Rust_report','stdout','stderr']:assert seal(row[key]['path'])==row[key]
    rust=json.loads(Path(row['Rust_report']['path']).read_bytes());go=json.loads(Path(row['stdout']['path']).read_bytes())
    assert rust==row['Rust']and go==row['Go']
    if row['expected_acceptance']:
        assert rust['verdict']==go['verdict']=='accepted'and rust['hashes']==go['hashes']
        assert rust['axiom_count']==0 and not rust['axiom_report']['entries']
        assert go['hashes']['axiom_report']=='0ebc281c3a8d37e2d1a9ce033773e2865f96a13186a6364cb3446204c6a990d5'
    else:assert json.loads(rust['error_detail'])['code']=='CORE_TYPE_MISMATCH'and go['error_kind']=='core_check'
archives=[json.loads(path.read_bytes())for path in sorted((OUT/'epochs').glob('*/inputs.json'))]
assert len(archives)==7
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
        assert len(raw)==row['bytes']and hashlib.sha256(raw).hexdigest()==row['sha256'];preimages+=1
for path in (OUT/'epochs').glob('*/inputs.json'):assert seal(path.parent/'run-checks.py')['sha256']==json.loads(path.read_bytes())['runner_sha256']
executions=[]
for path in sorted((OUT/'epochs').glob('*/*/result.json')):
    row=json.loads(path.read_bytes());assert row['status']in {'passed','failed'}and row['whole_source_inputs_unchanged']
    assert row['source_inputs']==json.loads((path.parent.parent/'inputs.json').read_bytes())['sources']
    executions.append({'result':seal(path),'stdout':seal(path.parent/'stdout.log'),'stderr':seal(path.parent/'stderr.log'),'status':row['status'],'command':row['command']})
assert len(executions)==10 and sum(row['status']=='failed'for row in executions)==3
for epoch in ['preservation','named-content-branches']:
    failed=OUT/'epochs'/epoch/'foundation/candidates'
    assert (failed/'concat-foundation.mpcert').is_file()
    assert json.loads(json.loads((failed/'concat-foundation.rust.json').read_bytes())['error_detail'])['code']=='CORE_TYPE_MISMATCH'
assert 'E0425'in (OUT/'epochs/operand-names/foundation/stderr.log').read_text()
assert not list((OUT/'epochs/operand-names/foundation/candidates').glob('*.mpcert'))
current=json.loads((OUT/'epochs/remaining-controls/inputs.json').read_bytes())
prior=json.loads((OUT/'epochs/harness-import/inputs.json').read_bytes())
assert [(row['path'],row['sha256'],row['bytes'])for row in current['sources']]==[(row['path'],row['sha256'],row['bytes'])for row in prior['sources']]
assert all(seal(row['path'])['sha256']==row['sha256']for row in current['sources'])
selected=json.loads((OUT/'epochs/remaining-controls/selected/result.json').read_bytes())
assert selected['status']=='passed'and selected['command'][-4:]==['--skip','concat_preserves_all2142_prior_identities','--skip','observations_cover_full_carriers_capacity_and_normalization']
for mode in ['foundation','observations','lint','format']:assert json.loads((OUT/f'epochs/harness-import/{mode}/result.json').read_bytes())['status']=='passed'
assert not git('diff','--name-only','HEAD','--','crates/mpk-vc/src/go_practical_ordinary/syntax.rs','crates/mpk-vc/src/go_practical_ordinary/composition.rs','crates/mpk-vc/src/go_practical_ordinary/string/equality.rs','crates/mpk-vc/src/go_practical_ordinary/string/order','crates/mpk-vc/src/go_practical_ordinary/string/slice/carrier.rs','crates/mpk-vc/src/go_practical_ordinary/string/slice/guards.rs','crates/mpk-vc/src/go_practical_ordinary/string/slice/preservation.rs','crates/mpk-vc/src/go_practical_ordinary/string/slice/signed.rs','crates/mpk-vc/tests/support/go_string_equality_eval.rs','Cargo.toml','Cargo.lock','proofs')
fixtures=OUT/'epochs/remaining-controls/selected/candidates'
assert len(list(fixtures.glob('*.mpcert')))==116
assert len(list(fixtures.glob('false-*.mpcert')))==46 and len(list(fixtures.glob('premise-*.mpcert')))==62 and len(list(fixtures.glob('definition-*.mpcert')))==7

observations=OUT/'epochs/harness-import/observations/candidates'
assert len(list(observations.glob('carrier-*.input.json')))==32
assert len(list(observations.glob('carrier-*.observation.json')))==100 and len(list(observations.glob('carrier-*.domain.json')))==28
whole_steps=carry_steps=0
for path in observations.glob('carrier-*.input.json'):
    source=json.loads(path.read_bytes());n,m=source['lengths'];a,b=bytes(source['all256_left_bytes']),bytes(source['all256_right_bytes'])
    assert len(a)==len(b)==256 and not source['source_or_nominal_authority']and not source['admitted_source_counterexample']
    for side,value in [('left',n),('right',m)]:assert source['all64_'+side+'_length_bits']==[bool(value&(1<<i))for i in range(64)]
    expected_fit=n+m<=256
    for mode in range(4):
        stem=path.name.removesuffix('.input.json')+f'-mode{mode}'
        flag=json.loads((observations/(stem+'.fit.partial.json')).read_bytes());length=json.loads((observations/(stem+'.length.partial.json')).read_bytes())
        assert flag['fits']==flag['expected_fits']==expected_fit and not flag['Go_panic_claimed']
        assert length['fits']==expected_fit and not length['carry']and length['length']==n+m
        assert length['all64_result_length_bits']==[bool((n+m)&(1<<i))for i in range(64)]
        if expected_fit:
            expected=list(a[:n]+b[:m])+[0]*(256-n-m)
            output=json.loads((observations/(stem+'.observation.json')).read_bytes())
            assert output['fits']and not output['carry']and output['length']==n+m
            assert output['all256_result_bytes']==output['expected_all256_result_bytes']==expected
            assert output['all64_result_length_bits']==[bool((n+m)&(1<<i))for i in range(64)]
            assert output['complete_carrier_observed']and not output['definition_callbacks']and not output['theorem_bodies_executed']
            assert output['output_replays']==(4 if n+m>64 else 1)
            if n+m:assert json.loads((observations/(stem+'.first-byte.partial.json')).read_bytes())['first_byte_equal']
            if n+m>64:
                for chunk in range(4):
                    partial=json.loads((observations/(stem+f'.chunk-{chunk}.partial.json')).read_bytes())
                    assert partial['fresh_interpreter_complete_inputs']and partial['observed_prefix_bytes']==expected[:(chunk+1)*64]
                    assert partial['actual_AST_steps']<=output['actual_AST_steps']
        else:
            output=json.loads((observations/(stem+'.domain.json')).read_bytes())
            assert not output['fits']and not output['expected_fits']and not output['carry']and output['length']==n+m
            assert output['full_input_preserved']and not output['Go_panic_claimed']and not output['definition_callbacks']and not output['theorem_bodies_executed']
            assert not (observations/(stem+'.observation.json')).exists()
        whole_steps+=output['actual_AST_steps']
assert len(list(observations.glob('carry-*.input.json')))==len(list(observations.glob('carry-*.observation.json')))==8
for path in observations.glob('carry-*.input.json'):
    source=json.loads(path.read_bytes());n,m=source['lengths'];output=json.loads(path.with_name(path.name.replace('.input.json','.observation.json')).read_bytes())
    assert len(source['all256_left_bytes'])==len(source['all256_right_bytes'])==256
    for side,value in [('left',n),('right',m)]:assert source['all64_'+side+'_length_bits']==[bool(value&(1<<i))for i in range(64)]
    assert not source['source_or_nominal_authority']and not source['admitted_source_counterexample']
    assert output['carry']==output['expected_carry']==(n+m>2**64-1)
    assert output['length']==output['expected_length']==(n+m)%(2**64)
    assert output['natural_sum_decimal']==str(n+m)and output['all64_result_length_bits']==[bool((n+m)&(1<<i))for i in range(64)]
    assert not output['fits_evaluated']and not output['Go_panic_claimed']and not output['definition_callbacks']and not output['theorem_bodies_executed'];carry_steps+=output['actual_AST_steps']
assert whole_steps==535952368 and carry_steps==184041
for short in ['CapacityWord','Fits','Carry','LengthWord','RightWord','Data','Value']:
    source=json.loads((fixtures/f'definition-{short}.input.json').read_bytes());output=json.loads((fixtures/f'definition-{short}.countermodel.json').read_bytes())
    assert len(source['all256_left_bytes'])==len(source['all256_right_bytes'])==256
    assert not source['source_or_nominal_authority']and not source['admitted_source_counterexample']
    assert output['actual_wrong_definition_countermodel']and not output['definition_callbacks']and not output['theorem_bodies_executed']
    if short=='CapacityWord':assert output['capacity_word']==257 and output['expected_capacity_word']==256
    elif short in ['Fits','Carry']:assert output['actual']!=output['expected']and not output['Go_panic_claimed']
    else:assert output['length']!=output['expected_length']or output['all256_result_bytes']!=output['expected_all256_result_bytes']
frozen=json.loads(Path('develop/migrations/go-practical/t02-w01/verification/native-observation/frozen-design-preservation.json').read_bytes())['inputs']
assert len(frozen)==255 and all(seal(row['path'])==row for row in frozen)
go_inputs=json.loads(Path('develop/migrations/go-practical/t02-w01/verification/string-constants/alias-display/analysis-inputs/captured-inputs.json').read_bytes())['build_inputs'];assert len(go_inputs)==186
for row in go_inputs:
    raw=(Path('go-tools/go2vir')/row['path']).read_bytes();assert len(raw)==row['bytes']and hashlib.sha256(raw).hexdigest()==row['raw_sha256']
for name in ['develop/docs/go-practical-implementation-traceability-ledger.md','develop/migrations/go-practical/t02-w01/README.md']:assert Path(name).read_bytes().startswith(git('show',BASE+':'+name))
original=json.loads(git('show',BASE+':develop/migrations/go-practical/t02-w01/requirements.json'));latest=json.loads(Path('develop/migrations/go-practical/t02-w01/requirements.json').read_bytes());row=next(row for row in latest['requirements']if row['id']=='T02-W01-R01')
assert row.pop('preparatory_string_concat_checkpoint')['status']=='scoped_checks_passed'and latest==original
if (OUT/'git-whitespace-review.json').exists():
    white=json.loads((OUT/'git-whitespace-review.json').read_bytes());assert white['status']=='passed_with_original_output_retained'and white['all_other_staged_files_pass_whitespace_check']
    assert not white['original_Cargo_output_bytes_modified']and not white['obsolete_IR_matcher_scope_seal_or_exclusion_changes']
    for warning in white['exact_raw_output_warnings']:assert seal(warning['path'])=={key:warning[key]for key in ['path','sha256','bytes']}
missing=[row for row in links if not row['exists']];unchanged=all(seal(row['path'])==row for row in inputs)
record={'status':'passed'if not findings and not missing and unchanged else'failed','basis_commit':BASE,'observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'inputs':inputs,'source_inputs_unchanged':unchanged,'findings':findings,'missing_links':missing,'links_checked':len(links),'complete_same_byte_inputs':117,'zero_axiom_acceptances':2,'genuine_core_type_mismatches':115,'full_original_source_epochs':7,'verified_source_preimages':preimages,'actual_executions':executions,'affected_Rust_roots':5,'passed_roots_repeated_without_source_change':False,'whole_input_pairs':32,'whole_carrier_observations':100,'profile_capacity_rejections':28,'whole_and_domain_AST_steps':whole_steps,'full64_carry_cases':8,'full64_carry_AST_steps':carry_steps,'typed_wrong_definition_countermodels':7,'source_or_native_Go_proof_from_model_claimed':False,'complete_decoded_data_payloads':decoded,'frozen_inputs_unchanged':255,'Go_frontend_build_inputs_unchanged':186,'matcher_scope_seal_counts':[127,4,39],'new_source_JSON_directory_or_blanket_exclusions':False,'core_and_Go_ref_implementations_unchanged':True,'native_release_acceptance':False,'full_T_gate_passed':False}
scan(str(REPORT.relative_to(ROOT)),json.dumps(record,indent=2).encode());record['findings']=findings;record['status']='passed'if not findings and not missing and unchanged else'failed'
REPORT.write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({key:value for key,value in record.items()if key not in {'inputs','actual_executions','complete_decoded_data_payloads'}}));raise SystemExit(record['status']!='passed')
