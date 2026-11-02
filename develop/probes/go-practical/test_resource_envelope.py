import copy
import json
from unittest import TestCase,main
from measure_sequential_resources import RECORD
from check_resource_envelope import check
class SequentialResourceTests(TestCase):
 @classmethod
 def setUpClass(cls):cls.report=json.loads((RECORD/'observations.json').read_text())
 def reject(self,edit,reason):
  r=copy.deepcopy(self.report);edit(r)
  with self.assertRaisesRegex(ValueError,reason):check(r)
 def test_actual_selected_resources(self):self.assertEqual(check(self.report),14)
 def test_cannot_claim_entire_program_budget(self):self.reject(lambda r:r.update(scope='all programs fit'),'schema/scope')
 def test_cannot_hide_parallel_children(self):self.reject(lambda r:r.update(direct_child_parallelism=3),'concurrency/budget')
 def test_cannot_freeze_private_ceiling(self):self.reject(lambda r:r.update(ceiling_scope='normative profile budget'),'concurrency/budget')
 def test_cannot_hide_production_work(self):self.reject(lambda r:r['pending'].pop(),'remaining freeze/production')
 def test_cannot_drop_source_identity(self):self.reject(lambda r:r['inputs'].pop(),'source/artifact')
 def test_cannot_cross_prerequisite_report(self):self.reject(lambda r:r['prerequisite_artifacts'][0].update(sha256='0'*64),'source/artifact')
 def test_cannot_drop_plus_one_aggregate(self):self.reject(lambda r:r['rows'].pop(11),'representative coverage')
 def test_cannot_cross_complete_cell(self):self.reject(lambda r:r['rows'][10]['case'].update(cell_bits=128),'complete candidate')
 def test_cannot_call_private_extension_production(self):self.reject(lambda r:r['rows'][-1].update(checker_scope='unchanged production checker implementations'),'private/production')
 def test_cannot_cross_checker_binary(self):self.reject(lambda r:r['checker_binaries'].update(base='0'*64),'binary/prerequisite')
 def test_cannot_call_group_construction_single_claim_cost(self):self.reject(lambda r:r['rows'][0].update(construction_scope='single theorem only'),'construction scope')
 def test_cannot_cross_candidate_bytes(self):self.reject(lambda r:r['rows'][0].update(raw_sha256='0'*64),'candidate bytes')
 def test_cannot_drop_go_checker(self):self.reject(lambda r:r['rows'][0]['observations'].pop('go'),'independent checkers')
 def test_cannot_accept_axioms(self):self.reject(lambda r:r['rows'][0]['observations']['rust']['report'].update(axiom_count=1),'zero-axiom')
 def test_cannot_cross_accepted_hashes(self):self.reject(lambda r:r['rows'][0]['observations']['go']['report']['hashes'].update(export='0'*64),'hashes disagree')
 def test_cannot_change_observed_maximum(self):self.reject(lambda r:r['maxima'].update(terms=1),'observed maxima')
 def test_cannot_cross_hardware_receipt(self):self.reject(lambda r:r.update(hardware_receipt='unrelated.json'),'hardware receipt')
if __name__=='__main__':main()
