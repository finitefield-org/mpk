"""Reject lost source guards, wrong capacities, copied views and proof crossing."""
import copy
import json
from unittest import TestCase, main
import reslice_differential as native
from check_reslice_report import check, check_initial
from run_reslice_probes import RECORD

class ResliceReportTests(TestCase):

    @classmethod
    def setUpClass(cls):
        cls.report = json.loads((RECORD / 'observations.json').read_text())
        cls.inputs = native.inputs()
        cls.outputs = json.loads((RECORD / 'reslice-output.json').read_text())

    def reject(self, change, pattern):
        r = copy.deepcopy(self.report)
        change(r)
        with self.assertRaisesRegex(ValueError, pattern):
            check(r)

    @staticmethod
    def case(r, n, kind, family, mutation=0):
        return next((x for x in r['cases'] if (x['case']['capacity'], x['case']['kind'], x['case']['family'], x['case']['mutation']) == (n, kind, family, mutation)))

    def output(self, wanted):
        return self.outputs[self.inputs.index(wanted)]

    def reject_native(self, wanted, field, value, pattern='reslice mismatch'):
        outputs = copy.deepcopy(self.outputs)
        outputs[self.inputs.index(wanted)][field] = value
        with self.assertRaisesRegex(ValueError, pattern):
            native.compare(self.inputs, outputs)

    def test_actual_report(self):
        self.assertEqual(check(self.report), 100)

    def test_original_failed_positive_diagnostics(self):
        self.assertEqual(check_initial(), 3)

    def test_cannot_call_failed_positive_a_false_claim(self):
        f = json.loads((RECORD.parent / 'reslice-initial/finding.json').read_text())
        f['status'] = 'three false theorems'
        with self.assertRaisesRegex(ValueError, 'initial positive diagnostic scope'):
            check_initial(f)

    def test_cannot_omit_localization_attempt(self):
        f = json.loads((RECORD.parent / 'reslice-initial/finding.json').read_text())
        f['failed_positive_certificates'] = 2
        with self.assertRaisesRegex(ValueError, 'initial finding coverage'):
            check_initial(f)

    def test_cannot_close_w03(self):
        self.reject(lambda r: r.update(status='complete'), 'W03 completion')

    def test_cannot_hide_remaining_ownership(self):
        self.reject(lambda r: r['pending'].pop(0), 'pending scope')

    def test_cannot_promote_to_captured_source_theorem(self):
        self.reject(lambda r: r['cases'][0]['case'].update(scope='complete captured Go source proof'), 'candidate scope')

    def test_cannot_admit_padding(self):
        self.reject(lambda r: r.update(boundary_probe_scope='Go source accepts all padded cells'), 'source admission')

    def test_cannot_freeze_probe_timeout(self):
        self.reject(lambda r: r.update(ceiling_scope='Go profile budget'), 'resource budget')

    def test_cannot_omit_zero_storage(self):
        self.reject(lambda r: r['construction_groups'].pop(0), 'construction coverage')

    def test_cannot_omit_three_index_guard(self):
        self.reject(lambda r: r['cases'].remove(self.case(r, 128, 'three', 'validity', 1)), 'case coverage')

    def test_cannot_shadow_candidates(self):
        self.reject(lambda r: r['cases'][1]['case'].update(id=r['cases'][0]['case']['id']), 'shadowed candidate')

    def test_cannot_truncate_endpoints(self):
        self.reject(lambda r: r['cases'][0]['case'].update(source_word_bits=8), 'full source/descriptor/element')

    def test_cannot_keep_wrapped_extent_only(self):
        self.reject(lambda r: r['cases'][0]['case'].update(extent_intermediate_bits=64), 'widened extent')

    def test_cannot_drop_descriptor_fields(self):
        self.reject(lambda r: r['cases'][0]['case'].update(descriptor_bits=128), 'full source/descriptor/element')

    def test_cannot_drop_tag_payload(self):
        self.reject(lambda r: self.case(r, 128, 'two', 'read')['case'].update(element_bits=64), 'full source/descriptor/element')

    def test_cannot_drop_other_region_or_padding(self):
        self.reject(lambda r: self.case(r, 129, 'three', 'read')['case'].update(quantified_storage_bits=129 * 128), 'complete backing/padding')

    def test_cannot_merge_regions(self):
        self.reject(lambda r: r['cases'][0]['case'].update(logical_region_count=1), 'region/source pointer')

    def test_cannot_admit_source_pointers(self):
        self.reject(lambda r: r['cases'][0]['case'].update(source_pointer_bits_admitted=True), 'region/source pointer')

    def test_cannot_strengthen_caller_with_internal_constructor_fact(self):
        self.reject(lambda r: r['cases'][0]['case'].update(goal_debug='Forall("upper_canonical", extra)'), 'caller assumption')

    def test_cannot_accept_erased_range_guard(self):
        self.reject(lambda r: self.case(r, 128, 'two', 'validity', 1)['case'].update(expected='accepted'), 'candidate identity')

    def test_cannot_cross_raw_certificate(self):
        self.reject(lambda r: r['cases'][0].update(raw_sha256='0' * 64), 'byte identity')

    def test_cannot_omit_checker(self):
        self.reject(lambda r: r['cases'][0]['observations'].pop('go'), 'independent checker')

    def test_cannot_use_axiom(self):
        self.reject(lambda r: r['cases'][0]['observations']['rust']['report'].update(axiom_count=1), 'zero-axiom')

    def test_cannot_cross_export(self):
        self.reject(lambda r: r['cases'][0]['observations']['go']['report']['hashes'].update(export='0' * 64), 'hashes disagree')

    def test_cannot_cross_both_accepted_results(self):
        self.reject(lambda r: r['cases'][0].update(observations=copy.deepcopy(r['cases'][2]['observations'])), 'checker/certificate identity')

    def test_cannot_count_fuel_as_false_claim(self):

        def change(r):
            x = self.case(r, 128, 'three', 'capacity', 1)
            x['observations']['rust']['report']['error_detail'] = '{"code":"CORE_FUEL_EXHAUSTED"}'
            x['observations']['go']['report']['error_detail'] = 'core reduction fuel exhausted'
        self.reject(change, 'intended type boundary')

    def test_cannot_cross_source_pins(self):
        self.reject(lambda r: r['inputs'][0].update(sha256='0' * 64), 'source identity')

    def test_cannot_replace_host_compiler(self):
        self.reject(lambda r: r['host_toolchain_inventory'].update(files=14495), 'toolchain inventory')

    def test_cannot_hide_native_failure(self):
        self.reject(lambda r: r['reslice_observations']['go_measurement'].update(exit_code=1), 'did not execute')

    def test_cannot_promote_observer_writes_to_input_permission(self):
        self.reject(lambda r: r['reslice_observations'].update(scope='Go borrowed input writes are admitted'), 'observation scope/byte')

    def test_cannot_omit_false_claim_witness(self):
        self.reject(lambda r: r['reslice_observations']['false_claim_witnesses'].pop(), 'reslice witnesses')

    def test_cannot_reclassify_source_panic_as_legal_result(self):

        def change(r):
            w = next((w for w in r['reslice_observations']['false_claim_witnesses'] if w['domain'] == 'source_operation_panic'))
            w['domain'] = 'bounded_source'
        self.reject(change, 'reslice witnesses')

    def test_boolean_witness_is_not_numeric_zero(self):
        self.reject(lambda r: r['reslice_observations']['false_claim_witnesses'][0].update(left=0), 'reslice witnesses')

    def test_two_index_upper_uses_capacity_beyond_length(self):
        for kind in native.KINDS:
            wanted = native.row(1, kind, 1, 0, 0, 1, 0, 1, 1, 0)
            a = self.output(wanted)
            self.assertTrue(a['valid_old'] and a['guard'] and a['resliced'] and a['read_defined'] and a['full_defined'])
            self.assertFalse(a['original_defined'])
            self.assertEqual(a['actual_value'], a['full_value'])
            self.reject_native(wanted, 'original_defined', True)

    def test_two_capacity_is_not_new_length(self):
        wanted = native.row(1, 'two', 1, 0, 0, 1, 0, 0, 0, 0)
        a = self.output(wanted)
        self.assertTrue(a['resliced'])
        self.assertEqual(a['actual_length'], '0')
        self.assertEqual(a['actual_capacity'], '1')
        self.reject_native(wanted, 'actual_capacity', '0')

    def test_three_capacity_uses_maximum(self):
        wanted = native.row(1, 'three', 1, 0, 0, 1, 0, 0, 0, 0)
        a = self.output(wanted)
        self.assertTrue(a['resliced'])
        self.assertEqual(a['actual_capacity'], '0')
        self.reject_native(wanted, 'actual_capacity', '1')

    def test_nil_and_nonnil_empty_remain_distinct(self):
        for kind in native.KINDS:
            for tag in (False, True):
                a = self.output(native.row(0, kind, 0, 0, 0, 0, 0, 0, 0, 0, nil=tag))
                self.assertTrue(a['guard'] and a['resliced'] and a['valid_new'])
                self.assertEqual(a['actual_nil'], tag)
                self.assertFalse(a['read_defined'])

    def test_valid_total_descriptor_does_not_establish_source_definedness(self):
        wanted = native.row(0, 'three', 0, 0, 0, 0, -1, -1, -1, 0, nil=True)
        a = self.output(wanted)
        self.assertTrue(a['valid_old'] and a['valid_new'] and a['constructed'] and a['reslice_panicked'])
        self.assertFalse(a['guard'] or a['resliced'])
        self.reject_native(wanted, 'resliced', True)

    def test_source_operation_panic_is_not_construction_failure(self):
        for kind in native.KINDS:
            a = self.output(native.row(0, kind, 0, 0, 0, 0, -1, 0, 0, 0))
            self.assertTrue(a['constructed'] and a['reslice_panicked'])
            self.assertFalse(a['construction_panicked'] or a['read_panicked'] or a['valid_new'])

    def test_raw_invalid_metadata_is_separate(self):
        a = self.output(native.row(1, 'two', 1, 0, 2, 1, 0, 1, 1, 0))
        self.assertFalse(a['valid_old'] or a['constructed'] or a['reslice_panicked'])
        self.assertTrue(a['construction_panicked'])

    def test_forged_dead_binding_does_not_make_go_allocation_dead(self):
        a = self.output(native.row(1, 'three', 1, 0, 0, 1, 0, 1, 1, 0, dead=True))
        self.assertFalse(a['valid_old'])
        self.assertTrue(a['resliced'] and a['read_defined'])

    def test_resliced_view_observes_source_update(self):
        wanted = native.row(1, 'two', 1, 0, 0, 1, 0, 1, 1, 0)
        a = self.output(wanted)
        self.assertEqual(a['alias_after'], a['replacement'])
        self.assertNotEqual(a['alias_after'], a['actual_value'])
        self.reject_native(wanted, 'alias_after', a['actual_value'])

    def test_complete_cell_read_keeps_tag(self):
        wanted = native.row(1, 'three', 1, 0, 0, 1, 0, 1, 1, 0)
        a = self.output(wanted)
        amount, tag = a['actual_value'].split(':')
        self.assertNotEqual(tag, '0' * 16)
        self.reject_native(wanted, 'actual_value', amount + ':' + '0' * 16)

    def test_padding_does_not_raise_admission(self):
        for b in (127, 129):
            a = self.output(native.row(b, 'three', b + 1, 0, 0, b + 1, 0, b + 1, b + 1, b))
            self.assertTrue(a['read_defined'] and a['covered'])
            self.assertFalse(a['storage_bound'])

    def test_valid_go_can_exceed_entire_finite_carrier(self):
        b = 129
        p = native.access.padding(b)
        a = self.output(native.row(b, 'two', p + 1, 0, 0, p + 1, 0, p + 1, p + 1, p))
        self.assertTrue(a['read_defined'])
        self.assertFalse(a['covered'] or a['storage_bound'])
        self.assertNotEqual(a['actual_value'], a['modeled_value'])

    def test_native_flags_are_booleans(self):
        self.reject_native(self.inputs[0], 'guard', 1, 'field/type mismatch')

    def test_native_observations_are_nonvacuous(self):
        counts = native.compare(self.inputs, self.outputs)
        self.assertEqual(len(self.inputs), 4261)
        self.assertEqual(counts, {'source_operations': 4103, 'operation_panics': 130, 'construction_rejections': 28, 'source_reads': 692, 'index_panics': 3411, 'proof_domain_reads': 658, 'tail_exposures': 388, 'restricted_three_capacities': 602, 'nil_results': 14, 'nonnil_empty_results': 1765, 'alias_updates': 692, 'out_of_profile_reads': 22, 'padding_not_admission': 8, 'uncovered_reads': 14, 'valid_total_result_despite_source_panic': 7})
if __name__ == '__main__':
    main()
