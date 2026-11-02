"""Reject lost allocation, append, borrow, range and provenance conditions."""
import copy
import json
from unittest import TestCase,main
import construction_differential as native
from check_construction_report import check
from run_construction_probes import RECORD

class ConstructionReportTests(TestCase):
 @classmethod
 def setUpClass(cls):
  cls.report=json.loads((RECORD/'observations.json').read_text());cls.inputs=native.inputs();cls.outputs=json.loads((RECORD/'construction-output.json').read_text())
 def reject(self,change,pattern):
  r=copy.deepcopy(self.report);change(r)
  with self.assertRaisesRegex(ValueError,pattern):check(r)
 @staticmethod
 def case(r,n=128,kind='make',family='validity',mutation=0):return next(x for x in r['cases'] if (x['case']['capacity'],x['case']['kind'],x['case']['family'],x['case']['mutation'])==(n,kind,family,mutation))
 def reject_native(self,predicate,change):
  out=copy.deepcopy(self.outputs);pos=next(i for i,(r,a) in enumerate(zip(self.inputs,out)) if predicate(r,a));change(out[pos])
  with self.assertRaisesRegex(ValueError,'construction mismatch|field/type mismatch|capacity violates|capacity type'):native.compare(self.inputs,out)
 def test_actual_report(self):self.assertEqual(check(self.report),276)
 def test_cannot_complete_w03(self):self.reject(lambda r:r.update(status='complete'),'W03 completion')
 def test_cannot_hide_ownership(self):self.reject(lambda r:r['pending'].pop(0),'pending scope')
 def test_cannot_claim_captured_source(self):self.reject(lambda r:r['cases'][0]['case'].update(scope='complete captured Go function'),'candidate scope')
 def test_cannot_admit_padding_or_growth(self):self.reject(lambda r:r.update(boundary_probe_scope='growth and padding are supported'),'source admission')
 def test_cannot_freeze_timeout(self):self.reject(lambda r:r.update(ceiling_scope='profile budget'),'resource budget')
 def test_cannot_hide_measurement_overlap(self):self.reject(lambda r:r.update(measurement_parallel_groups=1),'concurrency scope')
 def test_cannot_drop_zero_storage(self):self.reject(lambda r:r['construction_groups'].pop(0),'construction coverage')
 def test_cannot_drop_snapshot_mutant(self):self.reject(lambda r:r['cases'].remove(self.case(r,kind='append',family='snapshot',mutation=1)),'case coverage')
 def test_cannot_drop_borrowed_composition(self):self.reject(lambda r:r['cases'].remove(self.case(r,kind='composed',family='borrowed-frame')),'case coverage')
 def test_cannot_shadow_case(self):self.reject(lambda r:r['cases'][1]['case'].update(id=r['cases'][0]['case']['id']),'shadowed candidate')
 def test_cannot_truncate_word(self):self.reject(lambda r:r['cases'][0]['case'].update(source_word_bits=8),'full source/descriptor/element')
 def test_cannot_drop_extent_carry(self):self.reject(lambda r:r['cases'][0]['case'].update(extent_intermediate_bits=64),'widened extent')
 def test_cannot_drop_descriptor_fields(self):self.reject(lambda r:r['cases'][0]['case'].update(descriptor_bits=128),'full source/descriptor/element')
 def test_cannot_drop_tag_payload(self):self.reject(lambda r:self.case(r,kind='append',family='read')['case'].update(element_bits=64),'full source/descriptor/element')
 def test_cannot_drop_padding_cells(self):self.reject(lambda r:self.case(r,n=129)['case'].update(quantified_storage_bits=129*128),'complete backing/padding')
 def test_cannot_merge_regions(self):self.reject(lambda r:r['cases'][0]['case'].update(logical_region_count=1),'region/source pointer')
 def test_cannot_admit_pointer_bits(self):self.reject(lambda r:r['cases'][0]['case'].update(source_pointer_bits_admitted=True),'region/source pointer')
 def test_cannot_add_internal_constructor_premise(self):self.reject(lambda r:r['cases'][0]['case'].update(goal_debug='Forall("offset_canonical", premise)'),'caller assumption')
 def test_cannot_accept_erased_make_guard(self):self.reject(lambda r:self.case(r,mutation=1)['case'].update(expected='accepted'),'candidate identity')
 def test_cannot_cross_certificate(self):self.reject(lambda r:r['cases'][0].update(raw_sha256='0'*64),'byte identity')
 def test_cannot_omit_checker(self):self.reject(lambda r:r['cases'][0]['observations'].pop('go'),'independent checker')
 def test_cannot_introduce_axiom(self):self.reject(lambda r:r['cases'][0]['observations']['rust']['report'].update(axiom_count=1),'zero-axiom')
 def test_cannot_cross_export(self):self.reject(lambda r:r['cases'][0]['observations']['go']['report']['hashes'].update(export='0'*64),'hashes disagree')
 def test_cannot_cross_both_results_within_module(self):self.reject(lambda r:self.case(r).update(observations=copy.deepcopy(self.case(r,kind='append',family='length')['observations'])),'checker/certificate identity')
 def test_cannot_call_fuel_false(self):
  def change(r):
   row=self.case(r,kind='append',family='snapshot',mutation=1);row['observations']['rust']['report']['error_detail']=json.dumps({'code':'CORE_FUEL_EXHAUSTED'});row['observations']['go']['report']['error_detail']='core reduction fuel exhausted'
  self.reject(change,'intended type boundary')
 def test_cannot_change_case_count(self):self.reject(lambda r:r['cases'][0]['case'].update(count=1),'candidate identity')
 def test_cannot_use_bool_capacity(self):self.reject(lambda r:r['cases'][0]['case'].update(capacity=False),'number types')
 def test_cannot_cross_theorem(self):self.reject(lambda r:r['cases'][0]['case'].update(theorem='Unrelated'),'theorem identity')
 def test_cannot_hide_source_change(self):self.reject(lambda r:r['inputs'].pop(),'stale source identity')
 def test_cannot_omit_native_inputs(self):
  with self.assertRaisesRegex(ValueError,'input coverage'):native.compare(self.inputs[1:],self.outputs[1:])
 def test_cannot_promote_growth_or_ownership(self):self.reject(lambda r:r['construction_observations'].update(scope='complete reallocating append with borrowed writes'),'scope/byte identity')
 def test_cannot_omit_witness(self):self.reject(lambda r:r['construction_observations']['false_claim_witnesses'].pop(),'construction witnesses')
 def test_cannot_relabel_growth_as_source_panic(self):
  def change(r):next(w for w in r['construction_observations']['false_claim_witnesses'] if w['domain']=='source_reallocates_not_panics')['domain']='source_panic'
  self.reject(change,'construction witnesses')
 def test_cannot_relabel_forged_fresh_binding_as_source(self):
  def change(r):next(w for w in r['construction_observations']['false_claim_witnesses'] if w['domain']=='raw_invalid_fresh_binding')['domain']='bounded_source'
  self.reject(change,'construction witnesses')
 def test_cannot_cross_witnesses(self):self.reject(lambda r:r['construction_observations']['false_claim_witnesses'].reverse(),'construction witnesses')
 def test_native_make_empty_is_nonnil(self):self.reject_native(lambda r,a:r['kind']=='make' and a['constructed'] and a['actual_capacity']=='0',lambda a:a.update(actual_nil=True))
 def test_native_make_checks_negative_length(self):self.reject_native(lambda r,a:r['kind']=='make' and int(r['length'])<0,lambda a:a.update(make_panicked=False))
 def test_native_make_checks_length_capacity_order(self):self.reject_native(lambda r,a:r['kind']=='make' and int(r['length'])>int(r['capacity'])>=0,lambda a:a.update(guard=True))
 def test_native_make_initializes_tail(self):self.reject_native(lambda r,a:r['kind']=='make' and a['constructed'] and len(a['full_values'])>len(a['actual_values']),lambda a:a['full_values'].__setitem__(-1,native.ONES))
 def test_native_freshness_is_computed(self):self.reject_native(lambda r,a:r['kind']=='make' and not a['fresh'],lambda a:a.update(fresh=True))
 def test_native_old_live_mask_is_preserved(self):self.reject_native(lambda r,a:r['kind']=='make' and a['query_live'],lambda a:a.update(new_query_live=False))
 def test_native_make_preserves_borrowed_values(self):self.reject_native(lambda r,a:r['kind']=='make' and a['fresh'] and a['query_live'] and bool(a['borrowed_after']),lambda a:a.update(borrowed_after=native.ZERO))
 def test_native_make_padding_is_not_admitted(self):self.reject_native(lambda r,a:r['kind']=='make' and a['constructed'] and not a['storage_bound'],lambda a:a.update(storage_bound=True))
 def test_native_zero_equality_does_not_prove_coverage(self):self.reject_native(lambda r,a:r['kind']=='make' and a['full_read_defined'] and not a['query_covered'],lambda a:a.update(query_covered=True))
 def test_native_cannot_hide_growth(self):self.reject_native(lambda r,a:r['kind']=='append' and a['appended'] and not a['no_reallocation'],lambda a:a.update(no_reallocation=True))
 def test_native_cannot_panic_for_profile_capacity(self):self.reject_native(lambda r,a:r['kind']=='append' and a['appended'] and not a['no_reallocation'],lambda a:a.update(appended=False))
 def test_native_growth_requires_enough_capacity(self):self.reject_native(lambda r,a:r['kind']=='append' and a['appended'] and not a['no_reallocation'],lambda a:a.update(actual_capacity='0'))
 def test_native_no_growth_preserves_exact_capacity(self):self.reject_native(lambda r,a:r['kind']=='append' and a['appended'] and a['no_reallocation'],lambda a:a.update(actual_capacity=str(int(a['actual_capacity'])+1)))
 def test_native_growth_policy_remains_unspecified(self):
  out=copy.deepcopy(self.outputs);pos=next(i for i,(r,a) in enumerate(zip(self.inputs,out)) if r['kind']=='append' and a['appended'] and not a['no_reallocation']);a=out[pos];a['actual_capacity']=str(int(a['actual_capacity'])+1);a['full_values'].append(native.ZERO)
  self.assertEqual(native.compare(self.inputs,out),self.report['construction_observations']['outcomes'])
 def test_native_append_preserves_old_header(self):self.reject_native(lambda r,a:r['kind']=='append' and a['appended'],lambda a:a.update(old_metadata_preserved=False))
 def test_native_append_reuses_storage_when_observable(self):self.reject_native(lambda r,a:r['kind']=='append' and a['probe_observed'] and a['no_reallocation'],lambda a:a.update(shares_old=False))
 def test_native_append_nil_zero_preserves_nil(self):self.reject_native(lambda r,a:r['kind']=='append' and a['actual_nil'],lambda a:a.update(actual_nil=False))
 def test_native_append_uses_prestate_source(self):self.reject_native(lambda r,a:r['kind']=='append' and a['no_reallocation'] and a['source_read'] and a['appended_at']!=a['source_after'],lambda a:a.update(appended_at=a['source_after']))
 def test_native_append_is_not_forward_loop(self):self.reject_native(lambda r,a:r['kind']=='append' and a['no_reallocation'] and a['source_read'] and a['appended_at']!=a['naive_at'],lambda a:a.update(appended_at=a['naive_at']))
 def test_native_append_updates_saved_full_alias(self):self.reject_native(lambda r,a:r['kind']=='append' and a['no_reallocation'] and a['source_read'] and a['alias_at']!=a['full_before'] and bool(a['full_before']),lambda a:a.update(alias_at=a['full_before']))
 def test_native_append_keeps_original_length(self):self.reject_native(lambda r,a:r['kind']=='append' and a['appended'] and a['source_read'] and not a['destination_read'],lambda a:a.update(destination_read=True))
 def test_native_append_preserves_prefix(self):self.reject_native(lambda r,a:r['kind']=='append' and a['no_reallocation'] and a['destination_read'],lambda a:a.update(destination_after=native.ZERO))
 def test_native_append_preserves_unused_capacity(self):self.reject_native(lambda r,a:r['kind']=='append' and a['no_reallocation'] and a['full_read'] and int(r['index'])>=int(a['total']),lambda a:a.update(full_after=native.ZERO))
 def test_native_append_preserves_before_offset(self):self.reject_native(lambda r,a:r['kind']=='append' and a['appended'] and a['no_reallocation'] and 0<=int(r['index'])<int(r['destination']['offset']),lambda a:a['after'][0].__setitem__(0,native.ZERO))
 def test_native_failed_parent_never_attempts_full_read(self):
  row=next(a for r,a in zip(self.inputs,self.outputs) if r['kind']=='append' and not a['destination_constructed'] and r['destination']['nil'] and r['destination']['length']=='1')
  self.assertFalse(row['full_read']);self.assertFalse(row['full_read_panicked'])
 def test_native_cannot_reintroduce_failed_parent_full_attempt(self):self.reject_native(lambda r,a:r['kind']=='append' and not a['destination_constructed'],lambda a:a.update(full_read_panicked=True))
 def test_native_cannot_truncate_word(self):self.reject_native(lambda r,a:r['kind']=='append' and r['index']==str(-(1<<63)),lambda a:a.update(mapped='0'))
 def test_native_composed_empty_stays_nonnil(self):self.reject_native(lambda r,a:r['kind']=='composed' and a['appended'] and a['actual_length']=='0',lambda a:a.update(actual_nil=True))
 def test_native_composed_source_values(self):self.reject_native(lambda r,a:r['kind']=='composed' and a['appended'] and a['source_read'] and a['result_read'],lambda a:a.update(result_value=native.ZERO))
 def test_native_composed_preserves_borrowed_input(self):self.reject_native(lambda r,a:r['kind']=='composed' and a['appended'] and a['query_live'] and bool(a['borrowed_after']),lambda a:a.update(borrowed_after=native.ZERO))
 def test_native_composed_requires_fresh_binding(self):self.reject_native(lambda r,a:r['kind']=='composed' and not a['fresh'],lambda a:a.update(fresh=True))
 def test_native_composed_requires_source_binding(self):self.reject_native(lambda r,a:r['kind']=='composed' and not a['source_valid'] and a['appended'],lambda a:a.update(source_valid=True))
 def test_native_composed_reuses_preallocation(self):self.reject_native(lambda r,a:r['kind']=='composed' and a['probe_observed'] and a['no_reallocation'],lambda a:a.update(shares_preallocation=False))
 def test_native_row_must_be_an_object(self):
  out=copy.deepcopy(self.outputs);out[-1]=None
  with self.assertRaisesRegex(ValueError,'field/type mismatch'):native.compare(self.inputs,out)

if __name__=='__main__':main()
