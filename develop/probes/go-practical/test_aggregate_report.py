import copy
import json
from pathlib import Path
from unittest import TestCase,main
import aggregate_build_sources as build
import aggregate_differential as native
import check_aggregate_initial as initial
from check_aggregate_report import check
from run_aggregate_probes import RECORD

class AggregateReportTests(TestCase):
 @classmethod
 def setUpClass(cls):cls.report=json.loads((RECORD/'observations.json').read_text())
 def reject(self,fn,pattern):
  r=copy.deepcopy(self.report);fn(r)
  with self.assertRaisesRegex(ValueError,pattern):check(r)
 def case(self,r,f='prefix-read'):return next(c for c in r['cases'] if c['case']['byte_bound']==256 and c['case']['family']==f and c['case']['mutation']==0)
 def test_actual(self):self.assertEqual(check(self.report),98)
 def test_completion_is_not_invented(self):self.reject(lambda r:r.update(status='complete'),'scope')
 def test_production_scope_is_preserved(self):self.reject(lambda r:r['pending'].pop(),'scope')
 def test_boundaries_are_retained(self):self.reject(lambda r:r['aggregate_groups'].pop(),'workload')
 def test_padding_is_not_admission(self):self.reject(lambda r:r.update(boundary_probe_scope='all padding admitted'),'padding/admission')
 def test_private_ceiling_is_not_budget(self):self.reject(lambda r:r.update(ceiling_scope='profile budget'),'ceiling')
 def test_concurrency_is_reported(self):self.reject(lambda r:r.update(measurement_parallel_groups=1),'concurrency')
 def test_complete_cell_is_retained(self):self.reject(lambda r:self.case(r)['case'].update(cell_bits=128),'cell/padding')
 def test_complete_heap_address_is_retained(self):self.reject(lambda r:self.case(r)['case'].update(heap_address_bits=8),'heap domains')
 def test_complete_byte_is_retained(self):self.reject(lambda r:self.case(r)['case'].update(payload_bits=1),'heap domains')
 def test_bool_is_not_bound(self):self.reject(lambda r:r['cases'][0]['case'].update(byte_bound=False),'numeric types')
 def test_actual_line_guard_is_not_an_oracle(self):self.reject(lambda r:self.case(r)['case'].update(goal_debug='Forall("fits", premise)'),'caller fit/sign')
 def test_source_pins_are_closed(self):self.reject(lambda r:r['inputs'].pop(),'source identity')
 def test_generated_sources_are_closed(self):self.reject(lambda r:r['generated_sources']['files'].pop(),'generated source')
 def test_witnesses_are_complete(self):self.reject(lambda r:r['aggregate_observations']['false_claim_witnesses'].pop(),'witnesses/counts')
 def test_both_checkers_are_required(self):self.reject(lambda r:self.case(r)['observations'].pop('go'),'independent checker')
 def test_zero_axiom_policy_is_preserved(self):self.reject(lambda r:self.case(r)['observations']['rust']['report'].update(axiom_count=1),'zero-axiom')
 def test_accepted_hashes_bind_certificate(self):self.reject(lambda r:self.case(r)['observations']['go']['report']['hashes'].update(certificate='0'*64),'hashes disagree')
 def test_fuel_is_not_false_claim_evidence(self):
  def alter(r):
   c=next(c for c in r['cases'] if c['case']['mutation']==1)
   c['observations']['rust']['report']['error_detail']=json.dumps({'code':'CORE_FUEL_EXHAUSTED'})
  self.reject(alter,'resource failures')

class AggregateNativeTests(TestCase):
 @classmethod
 def setUpClass(cls):
  cls.cases=native.inputs();cls.outputs=json.loads((RECORD/native.ARTIFACTS[1]).read_text())
 def reject(self,predicate,key,value):
  outputs=copy.deepcopy(self.outputs)
  i=next(i for i,(r,a) in enumerate(zip(self.cases,outputs)) if predicate(r,a));outputs[i][key]=value
  with self.assertRaisesRegex(ValueError,'observation mismatch'):native.compare(self.cases,outputs)
 def test_original_go_observations(self):self.assertTrue(native.compare(self.cases,self.outputs)['projection_counterexamples'])
 def test_struct_amount_copy_preserves_sku(self):self.reject(lambda r,a:True,'frame_sku_preserved',False)
 def test_signed_amount_is_complete(self):self.reject(lambda r,a:int(a['amount'])<0,'amount','0')
 def test_offset_participates_in_address(self):self.reject(lambda r,a:int(r['offset'])>0 and a['line_read'],'absolute','0')
 def test_index_is_full64(self):self.reject(lambda r,a:int(r['index'])==-(1<<63),'absolute','0')
 def test_nil_view_has_no_read(self):self.reject(lambda r,a:r['nil'],'line_read',True)
 def test_source_panics_are_explicit(self):self.reject(lambda r,a:a['line_panicked'],'line_panicked',False)
 def test_profile_bound_is_explicit(self):self.reject(lambda r,a:a['line_read'] and not a['profile_input'],'profile_input',True)
 def test_prefix_high_bit_survives(self):self.reject(lambda r,a:a['prefix'] and a['pattern_read'],'prefix_byte',-1)
 def test_suffix_high_bit_survives(self):self.reject(lambda r,a:a['suffix'] and a['pattern_read'],'suffix_byte',-1)
 def test_inputs_are_preserved(self):self.reject(lambda r,a:True,'input_unchanged',False)
 def test_native_bool_is_strict(self):self.reject(lambda r,a:True,'descriptor_valid',1)
 def test_every_false_claim_has_a_source_witness(self):self.assertEqual(len(native.negative_witnesses(self.cases,self.outputs)),49)

class AggregateBuildDiagnosticTests(TestCase):
 def test_generated_sources(self):self.assertEqual(build.check_sources({'scope':build.SCOPE,'files':build.inventory(build.expected_sources())}),47)
 def test_missing_generated_module_rejects(self):
  files=build.inventory(build.expected_sources());files.pop()
  with self.assertRaisesRegex(ValueError,'generated source'):build.check_sources({'scope':build.SCOPE,'files':files})
 def test_rust_recipe_does_not_admit_go_source(self):
  with self.assertRaisesRegex(ValueError,'generated source'):build.check_sources({'scope':'Go source binding','files':build.inventory(build.expected_sources())})
 def test_initial_failures_remain_diagnostics(self):self.assertEqual(initial.check(),3)

class AggregateResourceHostTests(TestCase):
 def test_adjacent_hardware_identity(self):
  from run_proof_pilot import sha
  r=json.loads((RECORD/'local-resource-host.json').read_text());report=json.loads((RECORD/'observations.json').read_text())
  self.assertEqual(r['schema'],'mpk.go_practical.local_resource_host.v0')
  self.assertEqual(r['measurement_ref'],{'path':(RECORD/'observations.json').relative_to(RECORD.parents[4]).as_posix(),'sha256':sha((RECORD/'observations.json').read_bytes())})
  self.assertEqual(r['hardware']['physical_memory_bytes'],68719476736)
  self.assertEqual(r['hardware']['model'],'MacBookPro18,4')
  self.assertEqual(r['observed_host'],{k:report['host'][k] for k in ('system','release','machine','cpu_count')})
  self.assertIsNone(report['host']['model']);self.assertIsNone(report['host']['physical_memory_bytes'])

if __name__=='__main__':main()
