"""Review complete lexicographic-order sources, certificates and actual finite data observations."""
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
BASE = "86f29e185fa4911f5a03a68f9c75c37055215a7b"
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
module = types.ModuleType("mpk_string_bounded_order_input_review")
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
        for field in ["all256_left_bytes","all256_right_bytes","all_right_bytes","all64_left_length_bits","all64_right_length_bits"]:
            if field in row:
                payload = bytes(row[field]);assert len(payload) == (64 if field.startswith("all64_")else 256)
                scan(name+"#"+field,payload);decoded.append(seal(name+"#"+field,payload))
        if "left_byte" in row:
            payload = bytes([row["left_byte"]]);scan(name+"#left_byte",payload);decoded.append(seal(name+"#left_byte",payload))
    if path.suffix == ".md":
        for target in re.findall(r"(?<!!)\[[^\]]*\]\(([^\n)]+)\)",raw.decode()):
            target = target.strip().strip("<>")
            if re.match(r"^[a-zA-Z][a-zA-Z0-9+.-]*:",target) or target.startswith("#"):
                continue
            clean = urllib.parse.unquote(target.split("#",1)[0])
            links.append({"document":name,"target":target,"exists":(path.parent/clean).resolve().exists()})

pair=json.loads((OUT/'paired/result.json').read_bytes())
assert pair['status']=='passed' and len(pair['cases'])==62
assert pair['zero_axiom_acceptances']==2 and pair['genuine_core_type_mismatches']==60
for key in ['actual_driver_prelaunch','Rust_execution','Rust_execution_at_launch','Rust_execution_at_completion','unchanged_Go_build_receipt','go_binary']:
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
    else:assert json.loads(rust['error_detail'])['code']=='CORE_TYPE_MISMATCH' and go['error_kind']=='core_check'
archives=[json.loads(path.read_bytes())for path in sorted((OUT/'epochs').glob('*/inputs.json'))]
assert len(archives)==3
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
assert preimages==1143
for path in (OUT/'epochs').glob('*/inputs.json'):assert seal(path.parent/'run-checks.py')['sha256']==json.loads(path.read_bytes())['runner_sha256']
executions=[]
for path in sorted((OUT/'epochs').glob('*/*/result.json')):
    row=json.loads(path.read_bytes());assert row['status']in {'passed','failed'}and row['whole_source_inputs_unchanged']
    assert row['source_inputs']==json.loads((path.parent.parent/'inputs.json').read_bytes())['sources']
    executions.append({'result':seal(path),'stdout':seal(path.parent/'stdout.log'),'stderr':seal(path.parent/'stderr.log'),'status':row['status'],'command':row['command']})
assert len(executions)==7 and sum(row['status']=='failed'for row in executions)==1
failed=OUT/'epochs/semantic-controls/selected'
assert json.loads((failed/'result.json').read_bytes())['status']=='failed'
assert not list((failed/'candidates').glob('*.mpcert'))
current=json.loads((OUT/'epochs/direction-corrected/inputs.json').read_bytes())
assert all(seal(row['path'])['sha256']==row['sha256']for row in current['sources'])
prior=json.loads((OUT/'epochs/semantic-controls/inputs.json').read_bytes());old={row['path']:row for row in prior['sources']};new={row['path']:row for row in current['sources']}
changed=[name for name in old if old[name]['sha256']!=new[name]['sha256']]
assert changed==['crates/mpk-vc/src/go_practical_ordinary/string/order/bounded.rs']
before=Path(old[changed[0]]['complete_changed_input']).read_text()
old_chain='        let e = bool_right("Or", a.clone(), bool_left("Or", total.reverse(), b.clone()))\n            .reverse()\n            .then(Equality::boolean(\n                formula(a.clone(), b.clone()),\n                bit(true),\n                bounded("ChoiceTotal", vec![a.clone(), b.clone()]),\n            ))?;'
new_chain='        let e = bool_right("Or", a.clone(), bool_left("Or", total.reverse(), b.clone())).then(\n            Equality::boolean(\n                formula(a.clone(), b.clone()),\n                bit(true),\n                bounded("ChoiceTotal", vec![a.clone(), b.clone()]),\n            ),\n        )?;'
assert before.count(old_chain)==1
assert Path(changed[0]).read_text()==before.replace(old_chain,new_chain)
assert not git('diff','--name-only','HEAD','--','crates/mpk-vc/src/go_practical_ordinary/composition.rs','crates/mpk-vc/src/go_practical_ordinary/string/equality.rs','crates/mpk-vc/src/go_practical_ordinary/string/order/string_values.rs','crates/mpk-vc/tests/support','Cargo.toml','Cargo.lock','proofs')
fixtures=OUT/'epochs/direction-corrected/selected/candidates'
assert len(list(fixtures.glob('*.mpcert')))==62
assert len(list(fixtures.glob('false-*.mpcert')))==23 and len(list(fixtures.glob('premise-*.mpcert')))==37
assert len(list(fixtures.glob('bounded-*.input.json')))==75 and len(list(fixtures.glob('bounded-*.observation.json')))==150
bounded_steps=0
for path in fixtures.glob('bounded-*.input.json'):
    source=json.loads(path.read_bytes());n,m=source['lengths'];a,b=bytes(source['all256_left_bytes']),bytes(source['all256_right_bytes'])
    assert len(a)==len(b)==256 and 0<=n<=256 and 0<=m<=256 and not source['source_or_nominal_authority']
    x,y=a[:n],b[:m];same=x==y;expected=[x<y,x<=y,x>y,x>=y,same,same,same,same,True,True,True];assert source['expected_columns']==expected
    old_path=OUT.parent/'string-order/epochs/semantic-controls/selected/candidates'/path.name.replace('bounded-','string-',1)
    old_input=json.loads(old_path.read_bytes())
    for key in ['lengths','all256_left_bytes','all256_right_bytes']:assert old_input[key]==source[key]
    for mode in ['raw','normalized']:
        stem=path.name.removesuffix('.input.json')+'-'+mode
        output=json.loads((fixtures/(stem+'.observation.json')).read_bytes())
        assert output['actual']==output['expected']==expected and output['complete_carrier_observed']
        assert not output['definition_callbacks']and not output['theorem_bodies_executed'];bounded_steps+=output['actual_AST_steps']
        for count in range(1,12):
            partial=json.loads((fixtures/f'{stem}-{count:02}.partial.json').read_bytes());assert partial['available_results']==expected[:count]and partial['actual_AST_steps']<=output['actual_AST_steps']
assert bounded_steps==152401067
countermodels=[(256,257),(257,256),(257,258),(511,512),(512,511)];domain_steps=0
assert len(list(fixtures.glob('domain-*.input.json')))==len(list(fixtures.glob('domain-*.observation.json')))==5
for n,m in countermodels:
    name=f'domain-{n}-{m}';source=json.loads((fixtures/(name+'.input.json')).read_bytes());output=json.loads((fixtures/(name+'.observation.json')).read_bytes())
    assert source['lengths']==[n,m]and source['all256_left_bytes']==source['all256_right_bytes']==[0]*256
    assert source['all64_left_length_bits']==[bool(n&(1<<i))for i in range(64)]and source['all64_right_length_bits']==[bool(m&(1<<i))for i in range(64)]
    assert not source['source_or_nominal_authority']and not source['admitted_source_counterexample']
    assert output['rank_equal']and not output['string_equal']and output['actual_missing_bound_countermodel']
    assert output['upper_bounds']==output['expected_upper_bounds']==[n<=256,m<=256]
    assert output['NUL_byte_value_is_zero']and not output['present_NUL_rank_is_zero']
    assert not output['admitted_source_counterexample']and not output['definition_callbacks']and not output['theorem_bodies_executed'];domain_steps+=output['actual_AST_steps']
    for suffix in ['-01','-02','-bound1','-bound2']:assert (fixtures/(name+suffix+'.partial.json')).is_file()
assert domain_steps==5157035
frozen=json.loads(Path('develop/migrations/go-practical/t02-w01/verification/native-observation/frozen-design-preservation.json').read_bytes())['inputs']
assert len(frozen)==255 and all(seal(row['path'])==row for row in frozen)
go_inputs=json.loads(Path('develop/migrations/go-practical/t02-w01/verification/string-constants/alias-display/analysis-inputs/captured-inputs.json').read_bytes())['build_inputs'];assert len(go_inputs)==186
for row in go_inputs:
    raw=(Path('go-tools/go2vir')/row['path']).read_bytes();assert len(raw)==row['bytes']and hashlib.sha256(raw).hexdigest()==row['raw_sha256']
for name in ['develop/docs/go-practical-implementation-traceability-ledger.md','develop/migrations/go-practical/t02-w01/README.md']:assert Path(name).read_bytes().startswith(git('show',BASE+':'+name))
original=json.loads(git('show',BASE+':develop/migrations/go-practical/t02-w01/requirements.json'));latest=json.loads(Path('develop/migrations/go-practical/t02-w01/requirements.json').read_bytes());row=next(row for row in latest['requirements']if row['id']=='T02-W01-R01')
assert row.pop('preparatory_string_bounded_order_checkpoint')['status']=='scoped_checks_passed'and latest==original
if (OUT/'git-whitespace-review.json').exists():
    white=json.loads((OUT/'git-whitespace-review.json').read_bytes());assert white['status']=='passed_with_original_output_retained'and white['all_other_staged_files_pass_whitespace_check']
    assert not white['original_Cargo_output_bytes_modified']and not white['obsolete_IR_matcher_scope_seal_or_exclusion_changes']
    for warning in white['exact_raw_output_warnings']:assert seal(warning['path'])=={key:warning[key]for key in ['path','sha256','bytes']}
missing=[row for row in links if not row['exists']];unchanged=all(seal(row['path'])==row for row in inputs)
record={'status':'passed'if not findings and not missing and unchanged else'failed','basis_commit':BASE,'observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'inputs':inputs,'source_inputs_unchanged':unchanged,'findings':findings,'missing_links':missing,'links_checked':len(links),'complete_same_byte_inputs':62,'zero_axiom_acceptances':2,'genuine_core_type_mismatches':60,'full_original_source_epochs':3,'verified_source_preimages':1143,'actual_executions':executions,'bounded_whole_cases':75,'bounded_observations':150,'bounded_AST_steps':bounded_steps,'actual_missing_bound_raw_countermodels':5,'domain_AST_steps':domain_steps,'source_or_native_Go_proof_from_model_claimed':False,'complete_decoded_data_payloads':decoded,'frozen_inputs_unchanged':255,'Go_frontend_build_inputs_unchanged':186,'matcher_scope_seal_counts':[127,4,39],'new_source_JSON_directory_or_blanket_exclusions':False,'core_and_Go_ref_implementations_unchanged':True,'native_release_acceptance':False,'full_T_gate_passed':False}
scan(str(REPORT.relative_to(ROOT)),json.dumps(record,indent=2).encode());record['findings']=findings;record['status']='passed'if not findings and not missing and unchanged else'failed'
REPORT.write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({key:value for key,value in record.items()if key not in {'inputs','actual_executions','complete_decoded_data_payloads'}}));raise SystemExit(record['status']!='passed')
