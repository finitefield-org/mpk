"""Protect overlap, actual computed guards, full cells and retained evidence."""
import copy
import json
from unittest import TestCase,main
import copy_differential as native
from check_copy_report import check
from run_copy_probes import RECORD

class CopyReportTests(TestCase):
 @classmethod
 def setUpClass(cls):
  cls.report=json.loads((RECORD/'observations.json').read_text());cls.inputs=native.inputs();cls.outputs=json.loads((RECORD/'copy-output.json').read_text())
 def reject(self,change,pattern):
  r=copy.deepcopy(self.report);change(r)
  with self.assertRaisesRegex(ValueError,pattern):check(r)
 @staticmethod
 def case(r,n=128,family='copied-read',mutation=0):return next(x for x in r['cases'] if (x['case']['capacity'],x['case']['family'],x['case']['mutation'])==(n,family,mutation))
 def reject_native(self,predicate,change):
  out=copy.deepcopy(self.outputs);pos=next(i for i,(r,a) in enumerate(zip(self.inputs,out)) if predicate(r,a));change(out[pos])
  with self.assertRaisesRegex(ValueError,'copy mismatch|field/type mismatch'):native.compare(self.inputs,out)
 def test_actual_report(self):self.assertEqual(check(self.report),84)
 def test_cannot_complete_w03(self):self.reject(lambda r:r.update(status='complete'),'W03 completion')
 def test_cannot_hide_ownership(self):self.reject(lambda r:r['pending'].pop(0),'pending scope')
 def test_cannot_claim_captured_source(self):self.reject(lambda r:r['cases'][0]['case'].update(scope='complete captured Go theorem'),'candidate scope')
 def test_cannot_admit_padding(self):self.reject(lambda r:r.update(boundary_probe_scope='padded storage is admitted'),'source admission')
 def test_cannot_freeze_timeout(self):self.reject(lambda r:r.update(ceiling_scope='profile budget'),'resource budget')
 def test_cannot_drop_empty_storage(self):self.reject(lambda r:r['construction_groups'].pop(0),'construction coverage')
 def test_cannot_drop_poststate_mutant(self):self.reject(lambda r:r['cases'].remove(self.case(r,family='snapshot',mutation=1)),'case coverage')
 def test_cannot_shadow_case(self):self.reject(lambda r:r['cases'][1]['case'].update(id=r['cases'][0]['case']['id']),'shadowed candidate')
 def test_cannot_truncate_word(self):self.reject(lambda r:r['cases'][0]['case'].update(source_word_bits=8),'full source/descriptor/element')
 def test_cannot_drop_carry(self):self.reject(lambda r:r['cases'][0]['case'].update(extent_intermediate_bits=64),'widened extent')
 def test_cannot_drop_nil_capacity(self):self.reject(lambda r:r['cases'][0]['case'].update(descriptor_bits=128),'full source/descriptor/element')
 def test_cannot_drop_tag_half(self):self.reject(lambda r:self.case(r)['case'].update(element_bits=64),'full source/descriptor/element')
 def test_cannot_drop_padding_cells(self):self.reject(lambda r:self.case(r,n=129)['case'].update(quantified_storage_bits=129*128),'complete backing/padding')
 def test_cannot_merge_regions(self):self.reject(lambda r:r['cases'][0]['case'].update(logical_region_count=1),'region/source pointer')
 def test_cannot_admit_pointer_bits(self):self.reject(lambda r:r['cases'][0]['case'].update(source_pointer_bits_admitted=True),'region/source pointer')
 def test_cannot_require_constructor_canonical(self):self.reject(lambda r:r['cases'][0]['case'].update(goal_debug='Forall("offset_canonical", assumption)'),'caller assumption')
 def test_cannot_accept_erased_guard(self):self.reject(lambda r:self.case(r,family='source-guard',mutation=1)['case'].update(expected='accepted'),'candidate identity')
 def test_cannot_cross_certificate(self):self.reject(lambda r:r['cases'][0].update(raw_sha256='0'*64),'byte identity')
 def test_cannot_drop_checker(self):self.reject(lambda r:r['cases'][0]['observations'].pop('go'),'independent checker')
 def test_cannot_introduce_axiom(self):self.reject(lambda r:r['cases'][0]['observations']['rust']['report'].update(axiom_count=1),'zero-axiom')
 def test_cannot_cross_export(self):self.reject(lambda r:r['cases'][0]['observations']['go']['report']['hashes'].update(export='0'*64),'hashes disagree')
 def test_cannot_cross_both_results(self):self.reject(lambda r:self.case(r,family='count').update(observations=copy.deepcopy(self.case(r,family='frame')['observations'])),'checker/certificate identity')
 def test_cannot_call_fuel_false(self):
  def change(r):
   row=self.case(r,family='snapshot',mutation=1);row['observations']['rust']['report']['error_detail']=json.dumps({'code':'CORE_FUEL_EXHAUSTED'});row['observations']['go']['report']['error_detail']='core reduction fuel exhausted'
  self.reject(change,'intended type boundary')
 def test_cannot_change_case_count(self):self.reject(lambda r:r['cases'][0]['case'].update(count=1),'candidate identity')
 def test_cannot_use_bool_capacity(self):self.reject(lambda r:r['cases'][0]['case'].update(capacity=False),'number types')
 def test_cannot_cross_theorem(self):self.reject(lambda r:r['cases'][0]['case'].update(theorem='Unrelated'),'theorem identity')
 def test_cannot_hide_source_change(self):self.reject(lambda r:r['inputs'].pop(),'stale source identity')
 def test_cannot_omit_observer_inputs(self):
  with self.assertRaisesRegex(ValueError,'input coverage'):native.compare(self.inputs[1:],self.outputs[1:])
 def test_cannot_promote_observer_ownership(self):self.reject(lambda r:r['copy_observations'].update(scope='borrowed writes admitted'),'scope/byte identity')
 def test_cannot_omit_witness(self):self.reject(lambda r:r['copy_observations']['false_claim_witnesses'].pop(),'copy witnesses')
 def test_cannot_change_witness_domain(self):self.reject(lambda r:r['copy_observations']['false_claim_witnesses'][0].update(domain='source panic'),'copy witnesses')
 def test_cannot_cross_witnesses(self):self.reject(lambda r:r['copy_observations']['false_claim_witnesses'].reverse(),'copy witnesses')
 def test_native_cannot_use_destination_length(self):self.reject_native(lambda r,a:a['copied'] and int(r['destination']['length'])>int(r['source']['length']),lambda a:a.update(count=str(int(a['count'])+1)))
 def test_native_cannot_change_return_count(self):self.reject_native(lambda r,a:a['copied'] and int(a['count'])>0,lambda a:a.update(actual_count='0'))
 def test_native_cannot_use_poststate_source(self):self.reject_native(lambda r,a:a['copied_guard'] and a['copied'] and a['source_before']!=a['source_after'],lambda a:a.update(destination_after=a['source_after']))
 def test_native_cannot_use_forward_loop(self):self.reject_native(lambda r,a:a['copied'] and a['destination_read'] and a['destination_after']!=a['naive_destination'],lambda a:a.update(destination_after=a['naive_destination']))
 def test_native_cannot_zero_frame(self):self.reject_native(lambda r,a:a['copied'] and a['destination_read'] and not a['copied_guard'],lambda a:a.update(destination_after=native.ZERO))
 def test_native_cannot_change_other_region(self):self.reject_native(lambda r,a:a['copied'] and len(a['after'][1-int(r['destination']['region'])])>0,lambda a:a['after'][1].__setitem__(0,native.ZERO))
 def test_native_cannot_index_nil_destination(self):self.reject_native(lambda r,a:r['destination']['nil'],lambda a:a.update(destination_guard=True))
 def test_native_cannot_erase_source_guard(self):self.reject_native(lambda r,a:a['copied'] and a['destination_guard'] and not a['source_guard'],lambda a:a.update(source_guard=True))
 def test_native_cannot_truncate_negative_index(self):self.reject_native(lambda r,a:r['index']==str(-(1<<63)),lambda a:a.update(relative='0'))
 def test_native_cannot_change_slice_metadata(self):self.reject_native(lambda r,a:a['copied'],lambda a:a.update(metadata_preserved=False))
 def test_native_cannot_admit_padding(self):self.reject_native(lambda r,a:r['profile']<int(r['storage'])<=native.access.padding(r['profile']),lambda a:a.update(storage_bound=True))
 def test_native_cannot_hide_uncovered_key(self):self.reject_native(lambda r,a:a['destination_read'] and not a['covered'],lambda a:a.update(covered=True))
 def test_native_cannot_admit_dead_binding(self):self.reject_native(lambda r,a:a['copied'] and not a['destination_valid'],lambda a:a.update(destination_valid=True))
 def test_native_cannot_count_construct_failure_as_copy(self):self.reject_native(lambda r,a:not a['copied'],lambda a:a.update(copied=True))
 def test_native_cannot_hide_index_panic(self):self.reject_native(lambda r,a:a['source_read_panicked'],lambda a:a.update(source_read_panicked=False))

if __name__=='__main__':main()
