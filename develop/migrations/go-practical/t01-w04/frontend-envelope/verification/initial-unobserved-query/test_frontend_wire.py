"""Closed protocol,actual public validation and explicit provenance boundaries."""
import unittest
from collections import Counter
from frontend_wire import *
from frontend_vectors import source_bases,make,artifacts,specs,evaluate,altered,facts
from run_frontend_wire import RECORD,policy_from_outputs,preservation,hashes,catalogue
from check_scope_report import strict_json


class FrontendTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.report=strict_json((RECORD/'observations.json').read_bytes());cls.qs=strict_json((RECORD/'native-queries.json').read_bytes());cls.native=strict_json((RECORD/'retained-native.json').read_bytes())
        cls.policy=staticmethod(policy_from_outputs(cls.qs,cls.native));cls.out=evaluate(cls.policy);cls.by_id={r['id']:r for r in cls.out};cls.bases={b['id']:b for b in source_bases()}
    def test_complete_closed_matrix_replays_from_actual_native_queries(self):
        self.assertEqual(self.out,strict_json((RECORD/'outcomes.json').read_bytes()));self.assertEqual(Counter(r['classification'] for r in self.out),self.report['outcomes']);self.assertEqual(len(self.out),len(specs()))
    def test_five_independent_old_context_target_artifact_baselines(self):
        self.assertEqual(preservation(self.policy),strict_json((RECORD/'preservation.json').read_bytes()));self.assertEqual(len(preservation(self.policy)),5)
    def test_six_retained_successes_include_authorized_synthetic_origin(self):
        rows=[self.by_id[b+'-same'] for b in self.bases if 'project_spec' not in self.bases[b]];self.assertEqual(len(rows),6)
        self.assertTrue(all(r['classification']=='retained_process_protocol_valid' for r in rows));self.assertEqual(self.by_id['go_fixed_synthetic-same']['classification'],'retained_process_protocol_valid')
    def test_requests_can_be_reconstructed_before_any_lowered_artifact(self):
        for b in self.bases.values():
            a,r=make(b,self.policy);self.assertEqual(set(r),set(REQUEST_FIELDS));self.assertNotIn('ir',r);self.assertNotIn('capture_context_id',r['source_snapshot']);self.assertEqual(a.validate_request(clone(r)),r)
    def test_each_independent_request_component_and_hash_are_bound(self):
        for row in specs():
            if row['mutation'].startswith('request-') or row['mutation']=='bad-request-hash':self.assertEqual(self.by_id[row['id']]['classification'],'configuration_rejection')
    def test_staged_unused_bytes_stay_in_request_but_not_selected_closure(self):
        b=self.bases['go_fixed'];a,r=make(b,self.policy,extra_available=True);_,small=make(b,self.policy)
        self.assertNotEqual(r['request_hash'],small['request_hash']);self.assertEqual(len(r['source_snapshot']['inputs']),len(small['source_snapshot']['inputs'])+1)
        self.assertEqual(self.by_id['go_fixed-unused-staged']['classification'],'retained_process_protocol_valid')
    def test_actual_csharp_generic_behavior_and_java_closed_codebook(self):
        self.assertEqual(self.by_id['csharp-diag-generic-code']['classification'],'retained_diagnostic_protocol_valid')
        for b in ('csharp','java'):
            for k in ('unknown-code','foreign-code','custom-message','code-phase'):self.assertEqual(self.by_id[b+'-diag-'+k]['classification'],'configuration_rejection')
    def test_every_new_go_code_and_allowed_phase_has_an_unregistered_diagnostic(self):
        rows=[r for r in specs() if r['base']=='go-two-overlays' and r['mutation']=='go-code'];self.assertEqual(len(rows),sum(len(c['phases']) for c in GO_CODES.values()))
        self.assertEqual({r['code'] for r in rows},set(GO_CODES));self.assertTrue(all(self.by_id[r['id']]['classification']=='go_diagnostic_model_unregistered' for r in rows))
    def test_go_spans_allow_exact_eof_and_resolve_captured_function(self):
        for b in ('go-two-overlays','go-default-contracts'):
            for k in ('go-eof','go-function-span'):self.assertEqual(self.by_id[b+'-'+k]['classification'],'go_diagnostic_model_unregistered')
            for k in ('go-span-object','go-span-negative','go-span-past-end','go-span-bool','go-span-path','go-span-function','go-function-unknown','go-function-list'):self.assertEqual(self.by_id[b+'-'+k]['classification'],'configuration_rejection')
    def test_diagnostics_cannot_smuggle_artifacts_or_raw_host_paths(self):
        for b in self.bases:
            if 'project_spec' in self.bases[b]:continue
            for k in ('diag-artifact','diag-host-posix','diag-host-uri','diag-host-drive','diag-host-unc'):self.assertEqual(self.by_id[b+'-'+k]['classification'],'configuration_rejection')
    def test_success_keeps_complete_ir_map_manifest_and_request_links(self):
        for b in self.bases:
            for k in ('wrapper-schema','wrapper-hash','wrapper-extra','missing-map','missing-manifest','map-link','manifest-input','manifest-release','wrong-request'):self.assertEqual(self.by_id[b+'-'+k]['classification'],'configuration_rejection')
    def test_canonical_transport_rejects_extra_values_and_alternate_encodings(self):
        for b in self.bases:
            for k in ('extra-lf','noncanonical','duplicate','invalid-utf8','bom','two-values'):self.assertEqual(self.by_id[b+'-'+k]['classification'],'configuration_rejection')
    def test_process_failure_precedence_matches_actual_compiled_api(self):
        for n,f,code in ((0,facts(None,True,MAX_STDERR+1),'FRONTEND_PROTOCOL_LIMIT'),(0,facts(2,True),'FRONTEND_PROCESS_KILLED'),(0,facts(2),'FRONTEND_PROTOCOL_UNEXPECTED_USAGE')):
            with self.assertRaises(ProtocolFailure) as e:transport_guard(n,f)
            self.assertEqual(e.exception.code,code)
        for r in strict_json((RECORD/'limits.json').read_bytes()):self.assertEqual(self.policy(r['query']),r['expected'])
    def test_actual_stdout_and_stderr_boundaries_remain_inclusive(self):
        limits={r['id']:r for r in strict_json((RECORD/'limits.json').read_bytes())};self.assertEqual(self.policy(limits['stdout-'+str(MAX_STDOUT)]['query'])['error_code'],'FRONTEND_PROTOCOL_TRUNCATED')
        self.assertEqual(self.policy(limits['stdout-'+str(MAX_STDOUT+1)]['query'])['error_code'],'FRONTEND_PROTOCOL_LIMIT');self.assertEqual(self.policy(limits['stderr-'+str(MAX_STDERR)]['query'])['error_code'],'FRONTEND_PROTOCOL_MISSING')
    def test_unknown_native_queries_raise_instead_of_becoming_negative_evidence(self):
        with self.assertRaises(AssertionError):self.policy({'mode':'unobserved'})
    def test_complete_preimages_agree_in_python_rust_and_go_including_768_depth(self):
        hs=hashes(self.policy);expected=[{k:v for k,v in h.items() if k not in {'domain','raw_json_hex'}} for h in hs]
        self.assertEqual(strict_json((RECORD/'rust-hashes.json').read_bytes()),expected);self.assertEqual(strict_json((RECORD/'go-hashes.json').read_bytes()),expected)
        by_id={r['id']:r for r in expected};self.assertIs(by_id['depth-768']['accepted'],True);self.assertIs(by_id['depth-769']['accepted'],False);self.assertIs(by_id['bad-domain']['accepted'],False)
    def test_no_protocol_acceptance_grants_run_registration_or_source_proof(self):
        for r in self.out:
            if r['result']:
                self.assertIs(r['result']['registered_execution'],False);self.assertIs(r['result']['source_semantics_proved'],False)
        for b in self.bases:
            for k in ('ready-flag','proof-flag','registered-flag'):self.assertEqual(self.by_id[b+'-'+k]['classification'],'configuration_rejection')
        for f in ('frontend_invoked','registered_execution','source_semantics_proved','shared_family_frozen','profile_activated'):self.assertIs(self.report[f],False)
    def test_all169_complete_go_obligations_still_require_ordinary_elaboration(self):
        plans=[b['vc']['obligations'] for b in self.bases.values() if 'project_spec' in b];self.assertEqual(sum(len(f['requests']) for p in plans for f in p['functions']),169)
        for b in self.bases.values():
            if 'project_spec' in b:self.assertIsNone(b['skeleton']);self.assertEqual(b['vc']['ordinary_expansion'],{'kind':'unavailable'})


if __name__=='__main__':unittest.main()
