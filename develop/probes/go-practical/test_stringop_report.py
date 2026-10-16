"""Reject lost byte/range/output-bound conditions and overstated source evidence."""
import copy
import json
from unittest import TestCase, main

import stringop_differential as native
from check_stringop_report import check
from run_stringop_probes import RECORD

class StringOperationReportTests(TestCase):
    @classmethod
    def setUpClass(cls):
        cls.report = json.loads((RECORD / 'observations.json').read_text())
        cls.inputs = native.inputs()
        cls.outputs = json.loads((RECORD / 'stringop-output.json').read_text())

    @staticmethod
    def case(r, n=256, family='substring-read', mutation=0):
        return next(x for x in r['cases'] if (x['case']['capacity'], x['case']['family'], x['case']['mutation']) == (n, family, mutation))

    def reject(self, change, pattern):
        r = copy.deepcopy(self.report); change(r)
        with self.assertRaisesRegex(ValueError, pattern): check(r)

    def reject_native(self, predicate, change):
        outputs = copy.deepcopy(self.outputs)
        pos = next(i for i, (r, a) in enumerate(zip(self.inputs, outputs)) if predicate(r, a))
        change(outputs[pos])
        with self.assertRaisesRegex(ValueError, 'string operation mismatch|field/type mismatch'): native.compare(self.inputs, outputs)

    def test_actual_report(self): self.assertEqual(check(self.report), 248)
    def test_cannot_complete_w03(self): self.reject(lambda r: r.update(status='complete'), 'W03 completion')
    def test_cannot_hide_source_binding(self): self.reject(lambda r: r['pending'].pop(0), 'pending scope')
    def test_cannot_hide_prefix_or_application_work(self): self.reject(lambda r: r['pending'].pop(1), 'pending scope')
    def test_cannot_claim_application_proof(self): self.reject(lambda r: self.case(r)['case'].update(scope='complete Go application proof'), 'candidate scope')
    def test_cannot_admit_padding(self): self.reject(lambda r: r.update(boundary_probe_scope='all padding cells admitted'), 'source admission')
    def test_cannot_freeze_ceiling(self): self.reject(lambda r: r.update(ceiling_scope='profile budget'), 'resource budget')
    def test_cannot_hide_parallelism(self): self.reject(lambda r: r.update(measurement_parallel_groups=1), 'concurrency scope')
    def test_cannot_drop_zero_bound(self): self.reject(lambda r: r['stringop_groups'].pop(0), 'stringop coverage')
    def test_cannot_drop_above_256_boundary(self): self.reject(lambda r: r['stringop_groups'].pop(), 'stringop coverage')
    def test_cannot_drop_negative_pair(self): self.reject(lambda r: r['cases'].remove(self.case(r, mutation=1)), 'case coverage')
    def test_cannot_drop_right_coverage(self): self.reject(lambda r: r['cases'].remove(self.case(r, family='concat-right-coverage')), 'case coverage')
    def test_cannot_shadow_identity(self): self.reject(lambda r: r['cases'][1]['case'].update(id=r['cases'][0]['case']['id']), 'shadowed candidate')
    def test_cannot_use_bool_bound(self): self.reject(lambda r: r['cases'][0]['case'].update(capacity=False), 'number types')
    def test_cannot_truncate_word(self): self.reject(lambda r: self.case(r)['case'].update(source_word_bits=8), 'complete source/length/byte')
    def test_cannot_truncate_length(self): self.reject(lambda r: self.case(r)['case'].update(length_bits=8), 'complete source/length/byte')
    def test_cannot_keep_only_one_payload_bit(self): self.reject(lambda r: self.case(r)['case'].update(payload_bits=1), 'complete source/length/byte')
    def test_cannot_drop_padding_bytes(self): self.reject(lambda r: self.case(r, n=257)['case'].update(quantified_storage_bits=257*8), 'finite byte domain')
    def test_cannot_drop_second_concat_store(self): self.reject(lambda r: self.case(r, family='concat-right-read')['case'].update(quantified_storage_bits=256*8), 'finite byte domain')
    def test_cannot_assume_result_sign(self): self.reject(lambda r: self.case(r)['case'].update(goal_debug='Forall("result_nonnegative", premise)'), 'caller assumption')
    def test_cannot_assume_carry(self): self.reject(lambda r: self.case(r)['case'].update(goal_debug='Forall("no_wrap", premise)'), 'caller assumption')
    def test_cannot_assume_index_fit(self): self.reject(lambda r: self.case(r)['case'].update(goal_debug='Forall("fits", premise)'), 'caller assumption')
    def test_cannot_accept_erased_range(self): self.reject(lambda r: self.case(r, family='substring-length', mutation=1)['case'].update(expected='accepted'), 'candidate identity')
    def test_cannot_cross_theorem(self): self.reject(lambda r: self.case(r)['case'].update(theorem='Unrelated'), 'theorem identity')
    def test_cannot_cross_raw_certificate(self): self.reject(lambda r: self.case(r).update(raw_sha256='0'*64), 'byte identity')
    def test_cannot_hide_source_change(self): self.reject(lambda r: r['inputs'].pop(), 'stale source identity')
    def test_cannot_drop_checker(self): self.reject(lambda r: self.case(r)['observations'].pop('go'), 'independent checker')
    def test_cannot_introduce_axiom(self): self.reject(lambda r: self.case(r)['observations']['rust']['report'].update(axiom_count=1), 'zero-axiom')
    def test_cannot_cross_exports(self): self.reject(lambda r: self.case(r)['observations']['go']['report']['hashes'].update(export='0'*64), 'hashes disagree')
    def test_cannot_cross_both_results(self): self.reject(lambda r: self.case(r).update(observations=copy.deepcopy(self.case(r, family='concat-carry')['observations'])), 'checker/certificate identity')
    def test_cannot_call_fuel_a_false_claim(self):
        def change(r):
            row = self.case(r, mutation=1)
            row['observations']['rust']['report']['error_detail'] = json.dumps({'code':'CORE_FUEL_EXHAUSTED'})
            row['observations']['go']['report']['error_detail'] = 'core reduction fuel exhausted'
        self.reject(change, 'intended type boundary')
    def test_cannot_omit_false_witness(self): self.reject(lambda r: r['stringop_observations']['false_claim_witnesses'].pop(), 'observation/witness coverage')
    def test_cannot_call_bound_failure_go_panic(self):
        def change(r): next(w for w in r['stringop_observations']['false_claim_witnesses'] if w['domain']=='source_concat_exceeds_profile_without_panic')['domain']='source_panic'
        self.reject(change, 'observation/witness coverage')
    def test_cannot_call_invalid_range_bounded_source(self):
        def change(r): next(w for w in r['stringop_observations']['false_claim_witnesses'] if w['domain']=='source_substring_panic')['domain']='bounded_source'
        self.reject(change, 'observation/witness coverage')
    def test_cannot_promote_native_evidence(self): self.reject(lambda r: r['stringop_observations'].update(scope='complete source-bound proof'), 'scope/byte identity')
    def test_native_cannot_drop_inputs(self):
        with self.assertRaisesRegex(ValueError, 'input coverage'): native.compare(self.inputs[1:], self.outputs[1:])
    def test_native_cannot_drop_output_field(self): self.reject_native(lambda r,a: True, lambda a: a.pop('concat_bound'))
    def test_native_cannot_change_bool_to_int(self): self.reject_native(lambda r,a: True, lambda a: a.update(concat_bound=int(a['concat_bound'])))
    def test_native_row_is_an_object(self):
        outputs=copy.deepcopy(self.outputs); outputs[0]=None
        with self.assertRaisesRegex(ValueError,'field/type mismatch'): native.compare(self.inputs,outputs)
    def test_empty_substring_is_constructed(self): self.reject_native(lambda r,a: a['substring_done'] and a['substring_length']=='0', lambda a: a.update(substring_done=False))
    def test_empty_substring_has_no_byte(self): self.reject_native(lambda r,a: a['substring_done'] and a['substring_length']=='0', lambda a: a.update(substring_read=True))
    def test_substring_rejects_negative_lower(self): self.reject_native(lambda r,a: int(r['lower'])<0, lambda a: a.update(substring_panicked=False))
    def test_substring_rejects_reversed_endpoints(self): self.reject_native(lambda r,a: int(r['lower'])>int(r['upper']), lambda a: a.update(substring_guard=True))
    def test_substring_checks_length_not_backing_padding(self): self.reject_native(lambda r,a: int(r['upper'])>r['left_length'] and int(r['upper'])<=len(bytes.fromhex(r['left'])), lambda a: a.update(substring_done=True))
    def test_substring_checks_min_int(self): self.reject_native(lambda r,a: int(r['lower'])==-(1<<63), lambda a: a.update(substring_guard=True))
    def test_substring_checks_max_int(self): self.reject_native(lambda r,a: int(r['upper'])==(1<<63)-1, lambda a: a.update(substring_guard=True))
    def test_substring_failed_parent_does_not_attempt_index(self): self.reject_native(lambda r,a: a['substring_panicked'], lambda a: a.update(substring_read_panicked=True))
    def test_substring_length_is_upper_minus_lower(self): self.reject_native(lambda r,a: a['substring_done'] and int(r['lower'])>0 and a['substring_length']!='0', lambda a: a.update(substring_length=str(int(a['substring_length'])+1)))
    def test_substring_keeps_all_byte_bits(self): self.reject_native(lambda r,a: a['substring_read'] and a['substring_value']>127, lambda a: a.update(substring_value=a['substring_value']&1))
    def test_substring_nul_is_a_value(self): self.reject_native(lambda r,a: a['substring_read'] and a['substring_value']==0, lambda a: a.update(substring_read=False))
    def test_substring_invalid_utf8_is_preserved(self):
        self.reject_native(lambda r,a: a['substring_done'] and a['substring_bytes']=='ff', lambda a: a.update(substring_bytes='efbfbd'))
    def test_substring_cannot_skip_lower_offset(self): self.reject_native(lambda r,a: a['substring_read'] and int(r['lower'])>0 and r['index']=='0', lambda a: a.update(raw_substring_mapped='0'))
    def test_substring_tail_is_unobservable(self): self.reject_native(lambda r,a: a['substring_done'], lambda a: a.update(substring_tail_same=False))
    def test_concat_above_bound_still_succeeds(self): self.reject_native(lambda r,a: a['input_bound'] and not a['concat_bound'], lambda a: a.update(concat_bytes=''))
    def test_concat_bound_is_separate_from_go_success(self): self.reject_native(lambda r,a: a['input_bound'] and not a['concat_bound'], lambda a: a.update(concat_bound=True))
    def test_concat_length_addition(self): self.reject_native(lambda r,a: r['right_length']>0, lambda a: a.update(concat_length=str(int(a['concat_length'])+1)))
    def test_concat_carry_is_computed(self): self.reject_native(lambda r,a: True, lambda a: a.update(concat_carry=True))
    def test_concat_left_is_preserved(self): self.reject_native(lambda r,a: a['left_source_read'], lambda a: a.update(concat_left_value=a['concat_left_value']^255))
    def test_concat_right_uses_left_length_offset(self): self.reject_native(lambda r,a: a['right_source_read'] and r['left_length']>0 and r['index']=='0', lambda a: a.update(concat_right_mapped='0'))
    def test_concat_right_keeps_all_byte_bits(self): self.reject_native(lambda r,a: a['right_source_read'] and a['concat_right_value']>127, lambda a: a.update(concat_right_value=a['concat_right_value']&1))
    def test_concat_tail_is_unobservable(self): self.reject_native(lambda r,a: True, lambda a: a.update(concat_tail_same=False))
    def test_source_outside_bound_is_not_admitted(self): self.reject_native(lambda r,a: not a['input_bound'], lambda a: a.update(input_bound=True))
    def test_value_equality_does_not_establish_index_coverage(self): self.reject_native(lambda r,a: a['substring_read'] and not a['query_covered'], lambda a: a.update(query_covered=True))
    def test_inputs_remain_unchanged(self): self.reject_native(lambda r,a: True, lambda a: a.update(input_unchanged=False))

if __name__ == '__main__': main()
