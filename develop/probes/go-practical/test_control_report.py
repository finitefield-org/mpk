import copy
import json
from unittest import TestCase,main
import control_foundation_build as build
import control_differential as native
import check_control_initial as initial
from check_control_report import check
from run_control_probes import RECORD,inventory

class ControlReportTests(TestCase):
 @classmethod
 def setUpClass(cls):cls.report=json.loads((RECORD/'observations.json').read_text())
 def reject(self,edit,reason):
  r=copy.deepcopy(self.report);edit(r)
  with self.assertRaisesRegex(ValueError,reason):check(r)
 def test_actual_private_report(self):self.assertEqual(check(self.report),27)
 def test_cannot_claim_production_admission(self):self.reject(lambda r:r.update(scope='production induction admitted'),'private/production')
 def test_cannot_hide_future_bindings(self):self.reject(lambda r:r['pending'].pop(),'private/production')
 def test_cannot_complete_w03(self):self.reject(lambda r:r.update(status='complete'),'private/production')
 def test_cannot_hide_private_checker(self):self.reject(lambda r:r['checker_binaries'].update(go='bc21d279807f5994a16287a532db65cac3bbb170bfa4d0ca5c6729825e24c62c'),'production checker')
 def test_cannot_drop_shared_checker_source(self):self.reject(lambda r:r['mirrored_checker_inputs'].pop(),'mirrored checker')
 def test_cannot_drop_generated_candidate_source(self):self.reject(lambda r:r['generated_producer_inputs'].pop(),'generated producer')
 def test_cannot_cross_source_pins(self):self.reject(lambda r:r['inputs'].pop(),'stale source')
 def test_cannot_drop_stop_composition(self):self.reject(lambda r:r['cases'].pop(),'case coverage')
 def test_cannot_drop_prior_opaque_boundary(self):self.reject(lambda r:r['interface_cases'].pop(12),'interface coverage')
 def test_cannot_call_finite_unrolling_induction(self):self.reject(lambda r:r['cases'][0]['case'].update(iteration_domain='128 unrolled steps'),'iteration domain')
 def test_cannot_drop_frame_bits(self):self.reject(lambda r:r['cases'][0]['case'].update(frame_bits=1),'complete frame')
 def test_cannot_cross_theorem(self):self.reject(lambda r:r['cases'][0]['case'].update(theorem='Unrelated'),'theorem identity')
 def test_cannot_cross_certificate_bytes(self):self.reject(lambda r:r['cases'][0].update(raw_sha256='0'*64),'byte identity')
 def test_cannot_drop_second_checker(self):self.reject(lambda r:r['cases'][0]['observations'].pop('go'),'independent checker')
 def test_cannot_add_axiom(self):self.reject(lambda r:r['cases'][0]['observations']['rust']['report'].update(axiom_count=1),'zero-axiom')
 def test_cannot_cross_accepted_hash(self):self.reject(lambda r:r['cases'][0]['observations']['go']['report']['hashes'].update(export='0'*64),'hashes disagree')
 def test_cannot_count_fuel_as_false(self):
  def edit(r):r['cases'][1]['observations']['rust']['report']['error_detail']=json.dumps({'code':'CORE_FUEL_EXHAUSTED'})
  self.reject(edit,'resource failure')
 def test_cannot_call_malformed_type_a_semantic_negative(self):self.reject(lambda r:r['interface_cases'][1].update(classification='core_type_mismatch'),'rejection classification')
 def test_cannot_erase_nonvacuous_witness(self):self.reject(lambda r:r['native_observations']['false_claim_witnesses'].pop(),'witnesses')
 def test_cannot_turn_private_ceiling_into_budget(self):self.reject(lambda r:r.update(ceiling_scope='profile budget'),'ceiling scope')

class ControlNativeTests(TestCase):
 @classmethod
 def setUpClass(cls):cls.cases=native.inputs();cls.outputs=json.loads((RECORD/native.ARTIFACTS[1]).read_text())
 def reject(self,predicate,key,value):
  outputs=copy.deepcopy(self.outputs);i=next(i for i,(r,a) in enumerate(zip(self.cases,outputs)) if predicate(r,a));outputs[i][key]=value
  with self.assertRaisesRegex(ValueError,'mismatch'):native.compare(self.cases,outputs)
 def test_original_go_observations(self):self.assertEqual(native.compare(self.cases,self.outputs)['continue_posts'],72)
 def test_bad_entry_has_actual_counterexample(self):self.assertTrue(any(w['case_id']=='control-invariant-entry-altered' and w['actual'] is False for w in native.witnesses(self.cases,self.outputs)))
 def test_continue_executes_post(self):self.reject(lambda r,a:r['continued'] and not r['returned'] and not r['broken'],'post_calls',0)
 def test_continue_skips_rest(self):self.reject(lambda r,a:r['continued'] and not r['returned'] and not r['broken'],'rest_calls',1)
 def test_break_skips_post(self):self.reject(lambda r,a:r['broken'] and not r['returned'],'post_calls',1)
 def test_return_precedes_break(self):self.reject(lambda r,a:r['returned'] and r['broken'],'edge',native.encode((2,1<<63)))
 def test_halted_frame_preserves_every_word(self):self.reject(lambda r,a:int(r['state']['low'])&1,'absorbed',native.encode((0,0)))
 def test_bad_step_does_not_prove_invariant(self):self.reject(lambda r,a:r['n']==1 and r['bad_step'] and native.invariant(native.decode(r['state'])),'invariant',True)
 def test_complete_source_frame_is_preserved(self):self.reject(lambda r,a:True,'input_unchanged',False)
 def test_boolean_result_has_strict_type(self):self.reject(lambda r,a:True,'input_unchanged',1)
 def test_each_false_claim_has_a_distinct_observation(self):self.assertEqual(len(native.witnesses(self.cases,self.outputs)),6)

class ControlBuildAndInitialTests(TestCase):
 def test_mirrored_checker_input_closure(self):self.assertEqual(len(build.files()),75)
 def test_generated_producer_closure(self):self.assertEqual(len(build.producer_files()),45)
 def test_both_original_checker_diagnostics_are_preserved(self):self.assertEqual(initial.check(),2)

class ControlVerificationReceiptTests(TestCase):
 def test_targeted_verification_logs(self):
  from run_proof_pilot import sha
  r=json.loads((RECORD/'verification/checks.json').read_text())
  self.assertEqual((r['rust_new_tests'],r['rust_bool_regressions'],len(r['go_passed_tests']),r['python_tests']),(7,7,9,35))
  for p in r['log_inputs']:
   raw=(RECORD/p['path']).read_bytes();self.assertEqual(sha(raw),p['sha256'])
   if 'raw_sha256' in p:self.assertEqual(sha(raw+b'\n'*p['eof_newlines_removed']),p['raw_sha256'])
  self.assertEqual(sum('TestNatInduct' in t for t in r['go_passed_tests']),5)
 def test_adjacent_host_receipt(self):
  from run_proof_pilot import ROOT,sha
  r=json.loads((RECORD/'local-resource-host.json').read_text());report=json.loads((RECORD/'observations.json').read_text())
  self.assertEqual(ROOT/r['measurement_ref']['path'],RECORD/'observations.json');self.assertEqual(r['measurement_ref']['sha256'],sha((RECORD/'observations.json').read_bytes()))
  self.assertEqual(r['observed_host'],report['host']);self.assertEqual(r['hardware']['physical_memory_bytes'],68719476736)
  self.assertEqual(r['hardware']['model'],'MacBookPro18,4');self.assertEqual(r['hardware']['logical_cpu_count'],report['host']['cpu_count'])

if __name__=='__main__':main()
