"""Protect source/body identities, definedness, complete bytes and private scope."""
import copy
import json
from pathlib import Path
from unittest import TestCase,main,mock
import affix_differential as native
import check_affix_initial as initial
from affix_build_sources import SCOPE as BUILD_SCOPE,check_sources,expected_sources,inventory
from check_affix_report import check
from run_affix_probes import RECORD

class AffixReportTests(TestCase):
    @classmethod
    def setUpClass(cls):cls.report=json.loads((RECORD/'observations.json').read_text())
    @staticmethod
    def case(r,n=256,kind='prefix',family='binding',mutation=0):return next(x for x in r['cases'] if (x['case']['capacity'],x['case']['kind'],x['case']['family'],x['case']['mutation'])==(n,kind,family,mutation))
    def reject(self,change,pattern):
        r=copy.deepcopy(self.report);change(r)
        with self.assertRaisesRegex(ValueError,pattern):check(r)
    def test_actual_report(self):self.assertEqual(check(self.report),304)
    def test_cannot_complete_w03(self):self.reject(lambda r:r.update(status='complete'),'W03 completion')
    def test_cannot_hide_adapter_work(self):self.reject(lambda r:r['pending'].pop(0),'pending scope')
    def test_cannot_activate_private_evidence(self):self.reject(lambda r:self.case(r)['case'].update(scope='public Go profile'),'candidate scope')
    def test_cannot_admit_padding(self):self.reject(lambda r:r.update(boundary_probe_scope='all padding admitted'),'source admission')
    def test_cannot_freeze_timeout(self):self.reject(lambda r:r.update(ceiling_scope='profile budget'),'resource budget')
    def test_cannot_hide_parallelism(self):self.reject(lambda r:r.update(measurement_parallel_groups=1),'concurrency scope')
    def test_cannot_drop_zero_group(self):self.reject(lambda r:r['affix_groups'].pop(0),'affix coverage')
    def test_cannot_drop_plus_one_group(self):self.reject(lambda r:r['affix_groups'].pop(),'affix coverage')
    def test_cannot_drop_complete_byte_mutant(self):self.reject(lambda r:r['cases'].remove(self.case(r,n=1,mutation=1)),'case coverage')
    def test_cannot_drop_actual_suffix_read(self):self.reject(lambda r:r['cases'].remove(self.case(r,kind='suffix',family='matched-read')),'case coverage')
    def test_cannot_shadow_identity(self):self.reject(lambda r:r['cases'][1]['case'].update(id=r['cases'][0]['case']['id']),'shadowed candidate')
    def test_cannot_use_bool_capacity(self):self.reject(lambda r:r['cases'][0]['case'].update(capacity=False),'number types')
    def test_cannot_truncate_word(self):self.reject(lambda r:self.case(r)['case'].update(source_word_bits=8),'complete source/length/byte')
    def test_cannot_truncate_length(self):self.reject(lambda r:self.case(r)['case'].update(length_bits=8),'complete source/length/byte')
    def test_cannot_keep_one_payload_bit(self):self.reject(lambda r:self.case(r)['case'].update(payload_bits=1),'complete source/length/byte')
    def test_cannot_omit_pattern_store(self):self.reject(lambda r:self.case(r)['case'].update(quantified_storage_bits=8*256),'finite source/pattern')
    def test_cannot_drop_padding_bytes(self):self.reject(lambda r:self.case(r,n=257)['case'].update(quantified_storage_bits=16*257),'finite source/pattern')
    def test_cannot_add_nonnegative_oracle(self):self.reject(lambda r:self.case(r)['case'].update(goal_debug='Forall("source_nonnegative", premise)'),'caller assumption')
    def test_cannot_add_fit_oracle(self):self.reject(lambda r:self.case(r)['case'].update(goal_debug='Forall("fits", premise)'),'caller assumption')
    def test_cannot_accept_one_bit_model(self):self.reject(lambda r:self.case(r,n=1,mutation=1)['case'].update(expected='accepted'),'candidate identity')
    def test_cannot_cross_theorem(self):self.reject(lambda r:self.case(r)['case'].update(theorem='Unrelated'),'theorem identity')
    def test_cannot_cross_raw_bytes(self):self.reject(lambda r:self.case(r).update(raw_sha256='0'*64),'byte identity')
    def test_cannot_drop_source_pin(self):self.reject(lambda r:r['inputs'].pop(),'stale source identity')
    def test_cannot_drop_generated_input(self):self.reject(lambda r:r['generated_sources']['files'].pop(),'generated source identity')
    def test_cannot_change_generated_source(self):self.reject(lambda r:r['generated_sources']['files'][0].update(sha256='0'*64),'generated source identity')
    def test_cannot_omit_checker(self):self.reject(lambda r:self.case(r)['observations'].pop('go'),'independent checker')
    def test_cannot_add_axiom(self):self.reject(lambda r:self.case(r)['observations']['rust']['report'].update(axiom_count=1),'zero-axiom')
    def test_cannot_cross_export_hash(self):self.reject(lambda r:self.case(r)['observations']['go']['report']['hashes'].update(export='0'*64),'hashes disagree')
    def test_cannot_cross_both_checkers(self):self.reject(lambda r:self.case(r).update(observations=copy.deepcopy(self.case(r,family='canonical')['observations'])),'checker/certificate identity')
    def test_cannot_count_fuel_as_false(self):
        def change(r):
            c=self.case(r,mutation=1);c['observations']['rust']['report']['error_detail']=json.dumps({'code':'CORE_FUEL_EXHAUSTED'});c['observations']['go']['report']['error_detail']='core reduction fuel exhausted'
        self.reject(change,'intended type boundary')
    def test_cannot_promote_native_scope(self):self.reject(lambda r:r['affix_observations'].update(scope='registered adapter proof'),'scope/byte identity')
    def test_cannot_drop_witness(self):self.reject(lambda r:r['affix_observations']['false_claim_witnesses'].pop(),'witness/coverage')
    def test_cannot_call_library_panics(self):
        def change(r):next(w for w in r['affix_observations']['false_claim_witnesses'] if w['domain']=='optional_index_panic')['domain']='HasPrefix_panics'
        self.reject(change,'witness/coverage')
    def test_witness_bool_is_not_integer(self):
        def change(r):r['affix_observations']['false_claim_witnesses'][0]['actual']=1
        self.reject(change,'witness/coverage')

class AffixNativeTests(TestCase):
    @classmethod
    def setUpClass(cls):
        cls.inputs=native.inputs();cls.outputs=json.loads((RECORD/'affix-output.json').read_text());cls.binding=json.loads((RECORD/'affix-source-binding.json').read_text())
    def reject_native(self,predicate,key,value):
        a=copy.deepcopy(self.outputs);i=next(i for i,(r,x) in enumerate(zip(self.inputs,a)) if predicate(r,x));a[i][key]=value
        with self.assertRaisesRegex(ValueError,'affix mismatch|field/type mismatch'):native.compare(self.inputs,a)
    def reject_binding(self,change,pattern='body/identity|scope/compiler'):
        b=copy.deepcopy(self.binding);change(b)
        with self.assertRaisesRegex(ValueError,pattern):native.check_binding(b)
    def test_four_original_functions(self):self.assertEqual(native.check_binding(self.binding),4)
    def test_cannot_register_candidate(self):self.reject_binding(lambda b:b.update(registered=True))
    def test_cannot_cross_go_release(self):self.reject_binding(lambda b:b.update(go_version='go1.26.0'))
    def test_cannot_admit_package_by_name(self):self.reject_binding(lambda b:b.update(scope='strings.* supported'))
    def test_cannot_cross_function_body(self):self.reject_binding(lambda b:b['functions'][2].update(declaration=b['functions'][3]['declaration']))
    def test_cannot_cross_file_hash(self):self.reject_binding(lambda b:b['functions'][2].update(file_sha256='0'*64))
    def test_cannot_reorder_symbols(self):self.reject_binding(lambda b:b['functions'].reverse())
    def test_cannot_drop_internal_body(self):self.reject_binding(lambda b:b['functions'].pop())
    def test_cannot_cross_qualified_symbol(self):self.reject_binding(lambda b:b['functions'][2].update(symbol='other.HasPrefix'))
    def test_cannot_change_signature(self):self.reject_binding(lambda b:b['functions'][0].update(declaration='func HasPrefix(s []byte) bool { return true }'))
    def test_cannot_drop_native_input(self):
        with self.assertRaisesRegex(ValueError,'input coverage'):native.compare(self.inputs[1:],self.outputs[1:])
    def test_cannot_change_bool_type(self):self.reject_native(lambda r,a:True,'prefix',1)
    def test_empty_prefix_matches(self):self.reject_native(lambda r,a:r['pattern_length']==0,'prefix',False)
    def test_empty_suffix_matches(self):self.reject_native(lambda r,a:r['pattern_length']==0,'suffix',False)
    def test_longer_prefix_is_false(self):self.reject_native(lambda r,a:r['pattern_length']>r['source_length'],'prefix',True)
    def test_longer_suffix_is_false(self):self.reject_native(lambda r,a:r['pattern_length']>r['source_length'],'suffix',True)
    def test_short_circuit_prefix_never_panics(self):self.reject_native(lambda r,a:r['pattern_length']>r['source_length'],'prefix_panicked',True)
    def test_short_circuit_suffix_never_panics(self):self.reject_native(lambda r,a:r['pattern_length']>r['source_length'],'suffix_panicked',True)
    def test_unguarded_prefix_slice_panics(self):self.reject_native(lambda r,a:a['unconditional_prefix_panic'],'unconditional_prefix_panic',False)
    def test_unguarded_suffix_slice_panics(self):self.reject_native(lambda r,a:a['unconditional_suffix_panic'],'unconditional_suffix_panic',False)
    def test_prefix_body_matches_library(self):self.reject_native(lambda r,a:a['prefix'],'prefix_body',False)
    def test_suffix_body_matches_library(self):self.reject_native(lambda r,a:a['suffix'],'suffix_body',False)
    def test_source_tail_preserves_prefix(self):self.reject_native(lambda r,a:a['prefix'],'masked_prefix',False)
    def test_source_tail_preserves_suffix(self):self.reject_native(lambda r,a:a['suffix'],'masked_suffix',False)
    def test_high_bit_is_not_discarded(self):self.reject_native(lambda r,a:a['input_bound'] and not a['prefix'] and native.finite_relation(r,'prefix',one_bit=True),'prefix',True)
    def test_prefix_and_suffix_are_distinct(self):self.reject_native(lambda r,a:a['prefix'] and not a['suffix'],'suffix',True)
    def test_suffix_start_uses_both_lengths(self):self.reject_native(lambda r,a:r['source_length']>r['pattern_length']>0,'suffix_start','0')
    def test_suffix_query_is_full64(self):self.reject_native(lambda r,a:int(r['index'])==-(1<<63),'suffix_mapped','0')
    def test_pattern_index_at_length_is_undefined(self):self.reject_native(lambda r,a:int(r['index'])==r['pattern_length'],'pattern_read',True)
    def test_matched_prefix_byte(self):self.reject_native(lambda r,a:a['prefix'] and a['pattern_read'],'prefix_value',-1)
    def test_matched_suffix_byte(self):self.reject_native(lambda r,a:a['suffix'] and a['pattern_read'],'suffix_value',-1)
    def test_source_outside_bound_is_not_admitted(self):self.reject_native(lambda r,a:not a['input_bound'],'input_bound',True)
    def test_equal_byte_value_does_not_establish_coverage(self):self.reject_native(lambda r,a:not a['query_covered'] and a['prefix_read'],'query_covered',True)
    def test_inputs_are_preserved(self):self.reject_native(lambda r,a:True,'input_unchanged',False)
    def test_native_row_is_an_object(self):
        a=copy.deepcopy(self.outputs);a[0]=None
        with self.assertRaisesRegex(ValueError,'field/type mismatch'):native.compare(self.inputs,a)

class AffixBuildAndInitialTests(TestCase):
    def test_generated_closure(self):self.assertEqual(check_sources({'scope':BUILD_SCOPE,'files':inventory(expected_sources())}),46)
    def test_cannot_hide_missing_generated_module(self):
        r={'scope':BUILD_SCOPE,'files':inventory(expected_sources())};r['files'].pop()
        with self.assertRaisesRegex(ValueError,'generated source identity'):check_sources(r)
    def test_cannot_add_generated_module(self):
        r={'scope':BUILD_SCOPE,'files':inventory(expected_sources())};r['files'].append({'path':'extra.rs','sha256':'0'*64})
        with self.assertRaisesRegex(ValueError,'generated source identity'):check_sources(r)
    def test_cannot_call_generated_rust_go_capture(self):
        with self.assertRaisesRegex(ValueError,'generated source identity'):check_sources({'scope':'rewritten Go package','files':inventory(expected_sources())})
    def test_initial_evidence(self):self.assertEqual(initial.check(),8)
    def reject_initial(self,filename,change,pattern):
        original=initial.read_json
        def read(root,path):
            value=original(root,path)
            if str(path).endswith(filename):change(value)
            return value
        with mock.patch.object(initial,'read_json',side_effect=read):
            with self.assertRaisesRegex(ValueError,pattern):initial.check()
    def test_cannot_promote_initial_type_failure(self):self.reject_initial('affix-initial/scope.json',lambda r:r.update(scope='false semantic claims'),'initial scope')
    def test_cannot_promote_initial_fuel(self):self.reject_initial('affix-initial/fuel/scope.json',lambda r:r.update(scope='proof of a false claim'),'initial scope')
    def test_cannot_cross_archived_source(self):self.reject_initial('affix-initial/sources.json',lambda r:r[0].update(sha256='0'*64),'initial source identity')
    def test_cannot_drop_initial_checker(self):self.reject_initial('affix-initial/observations.json',lambda r:r[0]['observations'].pop('go'),'independent checker')
    def test_cannot_relabel_initial_fuel(self):
        def change(r):r[1]['observations']['rust']['report']['error_detail']=json.dumps({'code':'CORE_TYPE_MISMATCH'})
        self.reject_initial('affix-initial/fuel/observations.json',change,'failure class')

class AffixResourceHostTests(TestCase):
    def test_actual_adjacent_host_receipt(self):
        from run_proof_pilot import ROOT,sha
        r=json.loads((RECORD/'local-resource-host.json').read_text());report=json.loads((RECORD/'observations.json').read_text())
        self.assertEqual(r['schema'],'mpk.go_practical.local_resource_host.v0')
        self.assertEqual(r['provider'],'read-only /usr/sbin/sysctl outside workspace sandbox')
        self.assertEqual(r['scope'],'adjacent identity of the local Darwin development host for the referenced measurement; original report model/RAM remain null; no isolated-machine guarantee, profile budget or native Linux release receipt')
        self.assertEqual(ROOT/r['measurement_ref']['path'],RECORD/'observations.json');self.assertEqual(r['measurement_ref']['sha256'],sha((RECORD/'observations.json').read_bytes()))
        self.assertEqual(r['observed_host'],{k:report['host'][k] for k in ('system','release','machine','cpu_count')});self.assertIsNone(report['host']['model']);self.assertIsNone(report['host']['physical_memory_bytes'])
        raw=dict(line.split(': ',1) for line in r['raw_stdout'].splitlines());self.assertEqual(r['command'],['/usr/sbin/sysctl',*raw.keys()]);h=r['hardware'];self.assertEqual(h['model'],raw['hw.model']);self.assertEqual(h['processor'],raw['machdep.cpu.brand_string'])
        for field,key in [('physical_memory_bytes','hw.memsize'),('physical_cpu_count','hw.physicalcpu'),('logical_cpu_count','hw.logicalcpu')]:self.assertIs(type(h[field]),int);self.assertEqual(h[field],int(raw[key]))
        self.assertEqual(h['logical_cpu_count'],report['host']['cpu_count'])

if __name__=='__main__':main()
