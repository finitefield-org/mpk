"""Review full fixed-array comparison inputs, preserved failures and observations."""
from pathlib import Path
import datetime
import hashlib
import json
import re
import subprocess
import sys
import types
import urllib.parse
import io
import runpy

ROOT = Path.cwd()
OUT = Path(__file__).resolve().parent
BASE = "a49b1c39da16058e7b7c131fba1547c8efd3e0f1"
REPORT = OUT/"input-review.json"
ARCHIVE_PATH = OUT/'native-go/go1.25.0.darwin-arm64.tar.gz'
decode_archive = runpy.run_path(str(OUT/'transport-archive.py'))['decode']

def read_input(path):
    path=Path(path)
    return decode_archive()if path==ARCHIVE_PATH else path.read_bytes()


def git(*args):
    return subprocess.check_output(["/usr/bin/git", *args])


def seal(path, raw=None):
    raw = read_input(path) if raw is None else raw
    return {"path": str(path), "sha256": hashlib.sha256(raw).hexdigest(), "bytes": len(raw)}


assert git("rev-parse", "HEAD").decode().strip() == BASE
assert not git("diff", "--name-only", "HEAD", "--", "crates/mpk-core", "crates/mpk-cert", "crates/mpk-kernel", "go-tools", "release")
assert not Path(".github/workflows").exists()
scanner = Path("scripts/check-no-active-gir.sh")
code = scanner.read_text().split("<<'PY'\n",1)[1].rsplit("\nPY",1)[0].rsplit("raise SystemExit(main())",1)[0]
module = types.ModuleType("mpk_array_equality_input_review")
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
exclusions = module.active_retained_record_exclusions(manifest)
assert len(exclusions) == 51
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
    if path.suffix == ".json":
        row = json.loads(raw)
        def data_payloads(value, location=""):
            if isinstance(value,dict):
                for key,item in value.items():
                    data_payloads(item,location+"/"+key)
            elif isinstance(value,list):
                if value and all(isinstance(item,(int,bool))and 0<=item<=255 for item in value)and ("bytes"in location or "bits"in location or location.endswith(("/left","/right"))):
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

pair=json.loads((OUT/'paired-complete/result.json').read_bytes())
selected=OUT/'epochs/complete-controls/selected/result.json'
inventory=json.loads((selected.parent/'candidates/control-inventory.json').read_bytes())
total=inventory['complete_candidates'];negative=total-2
assert pair['status']=='passed' and len(pair['cases'])==pair['same_byte_inputs']==total
assert pair['zero_axiom_acceptances']==2 and pair['genuine_core_type_mismatches']==negative
assert inventory['altered_conclusions']==246 and inventory['typed_wrong_definitions']==47
assert inventory['actual_typed_witness_domains']>60
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
    else:
        assert rust['verdict']==go['verdict']=='rejected'
        assert json.loads(rust['error_detail'])['code']=='CORE_TYPE_MISMATCH' and go['error_kind']=='core_check'

# Every execution has its complete prelaunch source epoch, including failures.
archives=[json.loads(path.read_bytes())for path in sorted((OUT/'epochs').glob('*/inputs.json'))]
assert len(archives)==13
oids=sorted({row['original_git_blob']for archive in archives for row in archive['sources']if 'original_git_blob'in row})
blob=subprocess.check_output(['/usr/bin/git','cat-file','--batch'],input=('\n'.join(oids)+'\n').encode());originals={};cursor=0
for oid in oids:
    end=blob.index(b'\n',cursor);actual,kind,size=blob[cursor:end].decode().split();size=int(size)
    assert actual==oid and kind=='blob';originals[oid]=blob[end+1:end+1+size];cursor=end+size+2
assert cursor==len(blob)
preimages=0
for archive in archives:
    for row in archive['sources']:
        if 'original_git_blob'in row:
            raw=originals[row['original_git_blob']];assert row['original_commit']==BASE
        else:raw=Path(row['complete_changed_input']).read_bytes()
        assert len(raw)==row['bytes'] and hashlib.sha256(raw).hexdigest()==row['sha256'];preimages+=1
assert preimages==5454
for path in (OUT/'epochs').glob('*/inputs.json'):
    assert seal(path.parent/'run-checks.py')['sha256']==json.loads(path.read_bytes())['runner_sha256']
executions=[]
for path in sorted((OUT/'epochs').glob('*/*/result.json')):
    row=json.loads(path.read_bytes());assert row['status']in {'passed','failed'} and row['whole_source_inputs_unchanged']
    assert row['source_inputs']==json.loads((path.parent.parent/'inputs.json').read_bytes())['sources']
    executions.append({'result':seal(path),'stdout':seal(path.parent/'stdout.log'),'stderr':seal(path.parent/'stderr.log'),'status':row['status'],'command':row['command']})
assert len(executions)==19 and sum(row['status']=='failed'for row in executions)==6
compile_failures={'complete-fold':'E0505','full-relations':'mismatched closing delimiter','typed-comparator-laws':'E0618','comparison-conditions':'E0308'}
for epoch,code in compile_failures.items():
    assert code in (OUT/f'epochs/{epoch}/foundation/stderr.log').read_text()
    assert not list((OUT/f'epochs/{epoch}/foundation/candidates').glob('*.mpcert'))
assert 'Linkage'in (OUT/'epochs/complete-inequality/foundation/stderr.log').read_text()
assert not list((OUT/'epochs/complete-inequality/foundation/candidates').glob('*.mpcert'))

# The original accepted corruption is evidence of a missing law, never a rejection.
failed_pair=json.loads((OUT/'paired/result.json').read_bytes())
assert failed_pair['status']=='failed' and len(failed_pair['cases'])==6
assert failed_pair['actual_driver_prelaunch']==seal(OUT/'paired/check-paired.py')
for row in failed_pair['cases']:
    for key in ['input','Rust_report','stdout','stderr']:assert seal(row[key]['path'])==row[key]
    assert row['Rust']==json.loads(Path(row['Rust_report']['path']).read_bytes())
    assert row['Go']==json.loads(Path(row['stdout']['path']).read_bytes())
gap=failed_pair['cases'][-1];assert gap['name']=='definition-D0.ByteNotEqual'
assert gap['Rust']['verdict']==gap['Go']['verdict']=='accepted' and not gap['expected_acceptance']
assert gap['Rust']['hashes']==gap['Go']['hashes'] and gap['Rust']['axiom_count']==0
stop=json.loads((OUT/'failed-run-stop/result.json').read_bytes())
assert stop['signal']==15 and stop['signal_sent'] and not stop['original_outputs_reconstructed']
assert stop['target_pid']==27590 and '27590 27589 'in stop['process_before_signal']
assert '27589 27584 'in stop['parent_before_signal']
old_selected=json.loads((OUT/'epochs/remaining-controls/selected/result.json').read_bytes())
assert old_selected['status']=='failed' and old_selected['exit_code']==101
assert 'definition-D0.ByteNotEqual'in (OUT/'epochs/remaining-controls/selected/stderr.log').read_text()
assert not (OUT/'epochs/remaining-controls/selected/candidates/control-inventory.json').exists()
repaired=next(row for row in pair['cases']if row['name']==gap['name'])
assert repaired['Rust']['verdict']==repaired['Go']['verdict']=='rejected'

current=json.loads((OUT/'epochs/complete-controls/inputs.json').read_bytes())
prior=json.loads((OUT/'epochs/closed-complement/inputs.json').read_bytes())
assert [(row['path'],row['sha256'],row['bytes'])for row in current['sources']]==[(row['path'],row['sha256'],row['bytes'])for row in prior['sources']]
assert all(seal(row['path'])['sha256']==row['sha256']for row in current['sources'])
assert json.loads(selected.read_bytes())['status']=='passed'
for mode in ['foundation','observations','lint','format']:
    assert json.loads((OUT/f'epochs/closed-complement/{mode}/result.json').read_bytes())['status']=='passed'
assert not git('diff','--name-only','HEAD','--','crates/mpk-vc/src/go_practical_ordinary/syntax.rs','crates/mpk-vc/src/go_practical_ordinary/composition.rs','crates/mpk-vc/src/go_practical_ordinary/string','crates/mpk-vc/src/go_practical_ordinary/array/guards.rs','crates/mpk-vc/src/go_practical_ordinary/array/normalization.rs','crates/mpk-vc/src/go_practical_ordinary/array/relation.rs','crates/mpk-vc/src/go_practical_ordinary/array/update.rs','crates/mpk-vc/tests/support/go_string_equality_eval.rs','Cargo.toml','Cargo.lock','proofs','scripts/check-no-active-gir.sh','release')
fixtures=selected.parent/'candidates'
assert len(list(fixtures.glob('*.mpcert')))==total-1
assert len(list(fixtures.glob('false-*.mpcert')))==246
assert len(list(fixtures.glob('premise-*.mpcert')))==inventory['actual_typed_witness_domains']
assert len(list(fixtures.glob('definition-*.mpcert')))==47

# Native commands preserve the complete SDK archive and every regular SDK file.
import tarfile
from pathlib import PurePosixPath
native=json.loads((OUT/'native-go/result.json').read_bytes())
typed=json.loads((OUT/'native-comparability/result.json').read_bytes())
assert native['status']==typed['status']=='passed'
for owner,keys in [(native,['actual_driver_prelaunch','toolchain_archive','whole_original_sdk_inputs','compiler_binary','source','cases_input','oracle_binary']),(typed,['actual_driver_prelaunch','pinned_sdk_receipt','compiler_binary','whole_original_sdk_inputs','toolchain_archive'])]:
    for key in keys:assert seal(owner[key]['path'])==owner[key]
for row in native['negative_sources']+typed['sources']:assert seal(row['path'])==row
sdk=json.loads(Path(native['whole_original_sdk_inputs']['path']).read_bytes())
assert len(sdk)==14496 and sum(row['bytes']for row in sdk)==202722007
assert all(seal(row['path'])==row for row in sdk)
sdk_root=Path(native['environment']['GOROOT']);expected={row['path']:row for row in sdk}
assert {str(path)for path in sdk_root.rglob('*')if path.is_file()}==set(expected)
members=set();archive_bytes=decode_archive()
archive_transport=json.loads((OUT/'toolchain-archive/manifest.json').read_bytes())
assert archive_transport['original_archive']==native['toolchain_archive']
preservation=json.loads((OUT/'archive-preservation/result.json').read_bytes())
assert preservation['status']=='passed'and preservation['fully_decoded_bytes_equal_to_actual_original_input']
for key in ['actual_driver_prelaunch','original_native_execution','lossless_transport','retained_original_external_copy']:
    assert seal(preservation[key]['path'])==preservation[key]
assert not ARCHIVE_PATH.exists()and Path(preservation['retained_original_external_copy']['path']).read_bytes()==archive_bytes
original_binary_findings=[vars(f)for f in module.detect(str(ARCHIVE_PATH.relative_to(ROOT))+'#decoded-original-gzip',archive_bytes.decode(errors='ignore'),matchers)]
assert len(original_binary_findings)==3 and all(row['matcher_id']=='token-gir-prose'for row in original_binary_findings)
old_review=json.loads((OUT/'original-binary-review/input-review.json').read_bytes())
assert old_review['status']=='failed'and len(old_review['findings'])==3 and old_review['source_inputs_unchanged']and not old_review['missing_links']
assert all(row['matcher_id']=='token-gir-prose'and row['path']==str(ARCHIVE_PATH.relative_to(ROOT))for row in old_review['findings'])
assert {row['line']for row in original_binary_findings}=={row['line']for row in old_review['findings']}
assert next(row for row in old_review['inputs']if row['path']==str(ARCHIVE_PATH.relative_to(ROOT)))['sha256']==native['toolchain_archive']['sha256']
for path in (OUT/'original-binary-review/inputs').rglob('*'):
    if not path.is_file():continue
    name=str(path.relative_to(OUT/'original-binary-review/inputs')).removesuffix('.original')
    row=next(row for row in old_review['inputs']if row['path']==name)
    assert seal(name,path.read_bytes())==row
reviewer_preimages=json.loads((OUT/'reviewer-preimages/manifest.json').read_bytes())
assert not reviewer_preimages['original_source_bytes_trimmed']
assert len(reviewer_preimages['snapshots'])==3
for row in reviewer_preimages['snapshots']:
    encoded=(OUT/row['relative_transport']).read_bytes();raw=bytes.fromhex(encoded.decode())
    assert b''.join(raw[i:i+32].hex().encode()+b'\n'for i in range(0,len(raw),32))==encoded
    assert len(raw)==row['bytes']and hashlib.sha256(raw).hexdigest()==row['sha256']
    scan(row['original_path']+'#decoded-original-reviewer',raw)
    if 'original_review_input_path'in row:
        assert seal(row['original_review_input_path'],raw)==next(item for item in old_review['inputs']if item['path']==row['original_review_input_path'])
with tarfile.open(fileobj=io.BytesIO(archive_bytes),mode='r:gz')as tf:
    for item in tf:
        path=PurePosixPath(item.name);assert path.parts[0]=='go'and '..'not in path.parts
        if item.isdir():continue
        assert item.isfile() and item.name not in members;members.add(item.name)
        name=str(sdk_root/Path(*path.parts[1:]));raw=tf.extractfile(item).read()
        assert seal(name,raw)==expected[name]
        scan(str(Path(native['toolchain_archive']['path']).relative_to(ROOT))+'#'+item.name,raw)
assert len(members)==len(sdk)and native['toolchain_archive']['bytes']==57906112
assert native['environment']==typed['environment']
for command in native['commands']+typed['cases']:
    for key in ['stdout','stderr']:assert seal(command[key]['path'])==command[key]
    assert not Path(command['stderr']['path']).read_bytes()
assert native['commands'][0]['command'][-1]=='version'
assert Path(native['commands'][0]['stdout']['path']).read_text().strip()=='go version go1.25.0 darwin/arm64'
assert [row['exit_code']for row in native['commands'][:3]]==[0,0,0]
assert native['typed_original_source_rejections']==typed['typed_original_source_rejections']==4
negative_commands=native['commands'][3:]+typed['cases'];assert len(negative_commands)==8
for row in negative_commands:
    assert row['exit_code']!=0
    raw=Path(row['stdout']['path']).read_bytes()
    assert b'cannot be compared'in raw or b'mismatched types'in raw
assert not native['source_or_registered_context_proof'] and not native['native_Linux_release_acceptance']
assert not typed['ordinary_source_or_registered_profile_proof']

cases=json.loads((OUT/'native-go/cases.json').read_bytes())
oracle=json.loads((OUT/'native-go/comparisons.stdout.log').read_bytes())
assert len(cases)==len(oracle)==native['whole_byte_comparisons']==1028
assert {row['length']for row in cases}==set(range(257))
observations=OUT/'epochs/closed-complement/observations/candidates'
foundation=OUT/'epochs/closed-complement/foundation/candidates/array-equality-foundation.mpcert'
assert (observations/'observed-array-equality-foundation.mpcert').read_bytes()==foundation.read_bytes()
assert len(foundation.read_bytes())==1132748
boundary={1,2,3,4,5,7,8,9,15,16,17,31,32,33,63,64,65,127,128,129,255,256}
steps=observed_pairs=0
for case,go in zip(cases,oracle):
    n=case['length'];assert go['id']==case['id']and go['length']==n
    left=case['left'];right=case['right'];assert len(left)==len(right)==256
    same=left[:n]==right[:n];assert go['equal']==same and go['not_equal']==(not same)
    name='case-'+case['id'];source=json.loads((observations/(name+'.input.json')).read_bytes())
    assert source['logical_length']==n and source['all256_left_bytes']==left and source['all256_right_bytes']==right
    assert source['all64_length_bits']==[bool(n&(1<<i))for i in range(64)]
    assert not source['source_or_nominal_authority']and not source['registered_profile_or_program_proof']
    for mode in range(4 if n in boundary else 1):
        stem=name+f'-mode{mode}'
        out=json.loads((observations/(stem+'.observation.json')).read_bytes())
        partial=json.loads((observations/(stem+'.equal.partial.json')).read_bytes())
        assert out['equal']==out['native_Go_equal']==partial['equal']==partial['native_Go_equal']==same
        assert out['not_equal']==out['native_Go_not_equal']==(not same)
        assert 0<partial['actual_AST_steps']<=out['actual_AST_steps']<=50000000
        assert out['complete_input_carriers_observed']and not out['definition_callbacks']and not out['theorem_bodies_executed']and not out['source_or_profile_proof']
        steps+=out['actual_AST_steps'];observed_pairs+=1
obs_inventory=json.loads((observations/'observation-inventory.json').read_bytes())
assert obs_inventory=={'actual_AST_steps':536538673,'all_lengths_0_through_256':True,'definition_callbacks':False,'observer_ceilings_changed':False,'operator_results':2584,'raw_and_normalized_case_pairs':1292,'theorem_bodies_executed':False,'whole_native_input_pairs':1028}
assert steps==536538673 and observed_pairs==1292
assert len(list(observations.glob('case-*.input.json')))==1028
assert len(list(observations.glob('case-*.observation.json')))==1292
assert len(list(observations.glob('case-*.equal.partial.json')))==1292
countermodels=list(fixtures.glob('definition-*.countermodel.json'));assert len(countermodels)==47
for path in countermodels:
    out=json.loads(path.read_bytes());source=json.loads(path.with_name(path.name.replace('.countermodel.json','.input.json')).read_bytes());n=source['logical_length']
    assert len(source['all256_left_bytes'])==len(source['all256_right_bytes'])==256
    assert source['all64_length_bits']==[bool(n&(1<<i))for i in range(64)]
    assert not source['source_or_nominal_authority']and not source['admitted_source_counterexample']
    assert out['actual_wrong_definition_countermodel']and not out['definition_callbacks']and not out['theorem_bodies_executed']and out['actual']!=out['expected']

frozen=json.loads(Path('develop/migrations/go-practical/t02-w01/verification/native-observation/frozen-design-preservation.json').read_bytes())['inputs']
assert len(frozen)==255 and all(seal(row['path'])==row for row in frozen)
go_inputs=json.loads(Path('develop/migrations/go-practical/t02-w01/verification/string-constants/alias-display/analysis-inputs/captured-inputs.json').read_bytes())['build_inputs']
assert len(go_inputs)==186
for row in go_inputs:
    raw=(Path('go-tools/go2vir')/row['path']).read_bytes();assert len(raw)==row['bytes'] and hashlib.sha256(raw).hexdigest()==row['raw_sha256']
for name in ['develop/docs/go-practical-implementation-traceability-ledger.md','develop/migrations/go-practical/t02-w01/README.md']:
    assert Path(name).read_bytes().startswith(git('show',BASE+':'+name))
original=json.loads(git('show',BASE+':develop/migrations/go-practical/t02-w01/requirements.json'))
latest=json.loads(Path('develop/migrations/go-practical/t02-w01/requirements.json').read_bytes())
requirement=next(row for row in latest['requirements']if row['id']=='T02-W01-R01')
assert requirement.pop('preparatory_array_equality_checkpoint')['status']=='scoped_checks_passed' and latest==original
if (OUT/'git-whitespace-review.json').exists():
    white=json.loads((OUT/'git-whitespace-review.json').read_bytes())
    assert white['status']=='passed_with_original_output_retained' and white['all_other_staged_files_pass_whitespace_check']
    assert not white['original_Cargo_output_bytes_modified']and not white['obsolete_IR_matcher_scope_seal_or_exclusion_changes']
    for warning in white['exact_raw_output_warnings']:
        assert seal(warning['path'])=={key:warning[key]for key in ['path','sha256','bytes']}
missing=[row for row in links if not row['exists']];unchanged=all(seal(row['path'])==row for row in inputs)
record={'status':'passed'if not findings and not missing and unchanged else'failed','basis_commit':BASE,'observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'inputs':inputs,'source_inputs_unchanged':unchanged,'findings':findings,'missing_links':missing,'links_checked':len(links),'complete_same_byte_inputs':total,'zero_axiom_acceptances':2,'genuine_core_type_mismatches':negative,'full_original_source_epochs':len(archives),'verified_source_preimages':preimages,'actual_executions':executions,'affected_Rust_roots':5,'passed_roots_repeated_without_source_change':False,'whole_native_input_pairs':1028,'raw_and_normalized_case_pairs':1292,'operator_results':2584,'actual_AST_steps':steps,'typed_wrong_definition_countermodels':47,'original_accepted_corruption_gap_preserved':True,'original_failed_suite_terminated_after_failure':True,'source_or_native_Go_proof_from_model_claimed':False,'complete_decoded_data_payloads':decoded,'decoded_SDK_members_scanned':len(members),'lossless_archive_transport':seal(OUT/'toolchain-archive/manifest.json'),'original_binary_review':seal(OUT/'original-binary-review/input-review.json'),'original_compressed_tool_binary_findings':original_binary_findings,'zero_findings_claimed_for_all_original_binary_bytes':False,'frozen_inputs_unchanged':255,'Go_frontend_build_inputs_unchanged':186,'whole_original_sdk_files':14496,'whole_original_sdk_bytes':202722007,'native_type_rejections':8,'matcher_scope_seal_exclusion_counts':[127,4,39,51],'new_source_JSON_directory_or_blanket_exclusions':False,'core_and_Go_ref_implementations_unchanged':True,'native_release_acceptance':False,'full_T_gate_passed':False}
scan(str(REPORT.relative_to(ROOT)),json.dumps(record,indent=2).encode())
record['findings']=findings;record['status']='passed'if not findings and not missing and unchanged else'failed'
REPORT.write_text(json.dumps(record,indent=2)+'\n')
print(json.dumps({key:value for key,value in record.items()if key not in {'inputs','actual_executions','complete_decoded_data_payloads'}}))
raise SystemExit(record['status']!='passed')
