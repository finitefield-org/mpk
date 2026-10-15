from pathlib import Path
from hashlib import sha256
from datetime import datetime,timezone
import json,shutil
repo=Path('/private/tmp/mpk-w09-packed-pattern-proofs');temp=Path('/tmp/mpk-w09-context-refinement')
root=repo/'develop/migrations/csharp-03/ordinary-foundation/verification-logs/control-predicates'
prior=root/'with-pattern-refinement-candidates';out=root/'with-context-refinement-candidates';out.mkdir(exist_ok=True)
def copy(src,dst):
 dst.mkdir(parents=True,exist_ok=True)
 for p in src.iterdir():
  if not p.is_file():continue
  name=p.name
  if name.endswith('.log') or name.endswith('.stderr'):name+='.txt'
  shutil.copyfile(p,dst/name)
copy(Path('/tmp/mpk-w09-pattern-candidates/server-final'),prior/'server-linux/final')
shutil.copyfile('/tmp/mpk-w09-pattern-candidates/final-linux-fetch.py',prior/'server-linux/final/fetch.py')
copy(Path('/tmp/mpk-w09-pattern-candidates/server-linux'),prior/'server-linux/launch')
shutil.copyfile('/tmp/mpk-w09-pattern-candidates/launch-wrapper.py',prior/'server-linux/launch/launch-wrapper.py')
for name in ['attempt-1','final','bounds-final','context-certificates','candidate-certificates','generated-candidate-checks','context-checks','original-probe']:
 copy(temp/name,out/name)
for name in ['attempt-1','final','bounds-final']:
 state=json.loads((out/name/'status.json').read_text());assert state['status']=='passed'
 for row in state['stages']:assert sha256((out/name/(row['stage']+'.log.txt')).read_bytes()).hexdigest()==row['log_sha256']
a=json.loads((out/'final/source-manifest.json').read_text());b=json.loads((out/'bounds-final/source-manifest.json').read_text())
changes=[name for name in sorted(set(a)|set(b)) if a.get(name)!=b.get(name)]
assert changes==['crates/mpk-vc/src/csharp_practical_ordinary_ownership_proofs.rs']
for name,digest in b.items():assert sha256((repo/name).read_bytes()).hexdigest()==digest,name
restored=json.loads((out/'original-probe/execution.json').read_text())['test_file_restored_sha256']
assert restored==sha256((repo/'crates/mpk-vc/tests/support/csharp_practical_ordinary_control_pattern_packed_proof_tests.rs').read_bytes()).hexdigest()
assert restored==b['crates/mpk-vc/tests/support/csharp_practical_ordinary_control_pattern_packed_proof_tests.rs']
small={}
for folder in ['generated-candidate-checks','context-checks']:
 receipt=json.loads((out/folder/'verification.json').read_text());assert receipt['status']=='passed'
 for row in receipt['stages']:
  stem=row['case']+'-'+row['backend']
  assert sha256((out/folder/(stem+'.mpcert')).read_bytes()).hexdigest()==row['input_file_sha256']
  assert sha256((out/folder/(stem+'.json')).read_bytes()).hexdigest()==row['report_sha256']
  assert sha256((out/folder/(stem+'.stderr.txt')).read_bytes()).hexdigest()==row['stderr_sha256']
  source=out/('candidate-certificates' if folder=='generated-candidate-checks' else 'context-certificates')/(row['case']+'.mpcert')
  assert source.read_bytes()==(out/folder/(stem+'.mpcert')).read_bytes()
 small[folder]=len(receipt['stages'])
audit={'status':'passed','recorded_at':datetime.now(timezone.utc).isoformat(),'unique_targeted_tests_passed':8,'current_unit_tests_passed':6,'retained_consumer_tests':2,'small_fixture_dual_checker_stages_passed':sum(small.values()),'source_changed_after_consumer_replay':changes,'retention_reason':'Only context sharing-cost saturation and its new DAG unit changed after both consumers passed. Closed modes return child costs below their small sharing threshold, so their sums cannot overflow and saturation does not alter their terms or sharing. Candidate/context units, Clippy and format were rechecked at the final source.','temporary_original_probe_test_restored':True,'original_is_binding_refinements_pending':7,'application_proofs_pending':987,'full_t_gate':'deferred to T06-W12'}
(out/'verification.json').write_text(json.dumps(audit,indent=2,sort_keys=True)+'\n')
tools=out/'tool-sources';tools.mkdir(exist_ok=True)
for name in ['run.py','run-final.py','run-bounds.py','probe-original.py','check-candidates.py','check-context.py','check-generated.py','check-shared-dag.py','preserve.py']:
 shutil.copyfile(temp/name,tools/name)
runner=(prior/'tool-sources/run-targeted.py').read_text()
start=runner.index('"""');end=runner.index('"""',start+3)+3
runner='''"""Verify complete candidate generation, contextual Boolean proofs and affected consumers.

The contextual units use actual equality proof terms under free branch binders;
missing facts fail and wrong proofs reject. A small shared DAG checks generator
bounds without declaring kernel acceptance. Candidate units require every named
original refinement and an unchanged certificate prefix. The free-selector unit
and both original ownership/default consumers use the changed normalizer; their
original pins must remain identical. These are eight distinct targeted tests.
The prior fixed-source 18-context packed replay is already terminal and its
production generators are unchanged here, so it is retained. Clippy includes
library and test targets. Source/application proofs remain pending; the original
is_binding probe still cannot produce its seven required refinements. The full
T06 gate is deferred to T06-W12.
"""'''+runner[end:]
a=runner.index('        jobs = [');b=runner.index('        binaries = {}',a)
runner=runner[:a]+'''        jobs = [
            ('context-units', ['test', '-p', 'mpk-vc', '--lib', 'context_boolean_normalization_'] + tail, 2),
            ('candidate-units', ['test', '-p', 'mpk-vc', '--lib', 'pattern_refinement_candidate_'] + tail, 3),
            ('ownership-unit', ['test', '-p', 'mpk-vc', '--lib', 'ownership_normalization_preserves_free_selector_under_binder'] + tail, 1),
            ('ownership-consumer', test + ['csharp_03_t06_w09_ownership_proof_candidates'] + tail, 1),
            ('closed-default-consumer', test + ['csharp_03_t06_w09_binding_defaults_closed_boolean_proof_candidates'] + tail, 1),
            ('clippy', ['clippy', '-p', 'mpk-vc', '--lib', '--tests', '--', '-D', 'warnings'], 0),
            ('format', ['fmt', '-p', 'mpk-vc', '--', '--check'], 0),
        ]
'''+runner[b:]
runner=runner.replace("env.pop('MPK_W09_PATTERN_CANDIDATE_UNIT_OUTPUT', None)","env.pop('MPK_W09_PATTERN_CANDIDATE_UNIT_OUTPUT', None)\n        env.pop('MPK_W09_CONTEXT_BOOLEAN_UNIT_OUTPUT', None)\n        env.pop('MPK_W09_OWNERSHIP_PROOFS_OUT', None)\n        env.pop('MPK_W09_DEFAULT_CLOSED_PROOFS_OUT', None)")
compile(runner,'run-targeted.py','exec');(tools/'run-targeted.py').write_text(runner)
launch=(prior/'tool-sources/launch-linux.py').read_text().replace('mpk-w09-pattern-refinement-candidates-','mpk-w09-context-refinement-candidates-').replace('with-pattern-refinement-candidates','with-context-refinement-candidates')
compile(launch,'launch-linux.py','exec');(tools/'launch-linux.py').write_text(launch)
old=json.loads((prior/'source-manifest.json').read_text())
keys=set(old['source_hashes'])|{str(p.relative_to(repo)) for p in tools.iterdir()}
keys.add(str((out/'original-probe/probe.rs').relative_to(repo)))
manifest={'source_hashes':{name:sha256((repo/name).read_bytes()).hexdigest() for name in sorted(keys)},'fixture_hashes':old['fixture_hashes']}
for name,digest in manifest['fixture_hashes'].items():assert sha256((repo/name).read_bytes()).hexdigest()==digest,name
(out/'source-manifest.json').write_text(json.dumps(manifest,indent=2,sort_keys=True)+'\n')
print('Saved source/test/checker evidence; original source and application proofs remain pending.')
