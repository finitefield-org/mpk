"""Real policy/API/checker evidence and non-promotable Go source obligations."""
import unittest
from collections import Counter
from pathlib import Path
from proof_canonical import canonical,clone,parse
from policy_sources import source_rows,native_query
from policy_roots import Policy,paired_certificate,bytes_of,source_checked
from policy_cases import specs,evaluate,reseal
from run_policy_roots import RECORD,policy_from_outputs,hashes,api_observations,native_imports,kernel_witnesses
from check_scope_report import strict_json
from run_proof_pilot import sha


class PolicyTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.report=strict_json((RECORD/'observations.json').read_bytes());cls.qs=strict_json((RECORD/'native-queries.json').read_bytes());cls.obs=strict_json((RECORD/'native.json').read_bytes());cls.native=staticmethod(policy_from_outputs(cls.qs,cls.obs))
        cls.docs=strict_json((RECORD/'bundles.json').read_bytes());cls.rows={r['base']['id']:r for r in source_rows()};cls.out=strict_json((RECORD/'outcomes.json').read_bytes())
    def test_complete_policy_source_proof_and_wire_matrix(self):
        docs,out=evaluate(self.native);self.assertEqual(docs,self.docs);self.assertEqual(out,self.out);self.assertEqual(len(out),len(specs()));self.assertEqual(Counter(x['classification'] for x in out),self.report['outcomes'])
    def test_real_rebound_certificates_have_two_zero_axiom_reports_and_exact_source(self):
        for name in ('rust64','rust32'):
            p=self.docs[name]['evidence']['proof'];self.assertEqual(p['kind'],'checked_retained_ordinary_vc');self.assertEqual(p['certificate']['axiom_count'],0)
            row=self.rows[name];old=self.native(native_query(row,'verify'));q={'mode':'certificate','byte_chunks':old['outcome']['byte_chunks'],'source_manifest':row['base']['certificate_manifest']};r=self.native(q)
            self.assertEqual(paired_certificate(r,row['base']['certificate_manifest']),p['certificate']);self.assertEqual(sha(bytes_of(r['byte_chunks'])),p['certificate']['canonical_sha256'])
            self.assertEqual(p['certificate']['export_hash'],old['outcome']['rust']['export_hash']);self.assertEqual(p['certificate']['axiom_report_hash'],old['outcome']['rust']['axiom_report_hash'])
    def test_four_actual_assembly_errors_are_not_source_or_false_theorem_verdicts(self):
        for name in ('go_fixed','csharp','java','go_fixed_synthetic'):
            d=self.docs[name];self.assertEqual(d['receipt']['status'],'assembly_error');self.assertEqual(d['evidence']['proof'],{'kind':'retained_assembly_error','code':'SUCCESSOR_POLICY_CERTIFICATE_ASSEMBLY','phase':'certificate_assembly'})
            actual=self.native(native_query(self.rows[name],'verify'));self.assertFalse(actual['accepted']);self.assertIn('non-Std.Bool carrier',actual['detail']);self.assertNotIn('certificate',d['evidence']['proof'])
    def test_actual_policy_scan_and_evidence_imports_reject_tampered_claims(self):
        self.assertEqual(native_imports(self.native),strict_json((RECORD/'native-imports.json').read_bytes()))
    def test_every_existing_context_api_operation_has_actual_observations(self):
        data=api_observations(self.native);self.assertEqual(data,strict_json((RECORD/'api-trials.json').read_bytes()));ops=[x for r in data for x in r['result']['operations']]
        self.assertEqual(len(ops),102);self.assertEqual({x['operation'] for x in ops},{'session/start','vir/import','vc/generate','vc/list','vc/start-proof','vc/attach-candidate','vc/check-candidate'})
        self.assertEqual(sum(x['accepted'] for x in ops),30)
    def test_all72_actual_api_failures_leave_mutation_count_unchanged(self):
        data=strict_json((RECORD/'api-trials.json').read_bytes());failed=[x for r in data for x in r['result']['operations'] if not x['accepted']];self.assertEqual(len(failed),72)
        self.assertTrue(all(x['mutation_before']==x['mutation_after']==4 for x in failed))
    def test_generated_and_started_api_targets_are_helpers_not_source_proofs(self):
        for r in strict_json((RECORD/'api-trials.json').read_bytes()):
            self.assertEqual(r['result']['mutations'],4)
            for x in r['result']['operations'][:5]:self.assertIs(x['value']['helper_only'],True)
    def test_two_valid_kernel_certificates_cannot_prove_unelaborated_go(self):
        rows=kernel_witnesses(self.native,self.docs);self.assertEqual(rows,strict_json((RECORD/'kernel-witnesses.json').read_bytes()))
        for r in rows[:2]:
            self.assertEqual(r['classification'],'kernel_accepted_go_claim_rejected');self.assertEqual(r['kernel_certificate']['axiom_count'],0)
            with self.assertRaises(ValueError):Policy(self.rows[r['id']],self.native).validate('evidence',r['forged_evidence'])
    def test_corrupted_certificate_is_a_format_control_not_an_ordinary_false_claim(self):
        row=strict_json((RECORD/'kernel-witnesses.json').read_bytes())[-1];self.assertEqual(row['classification'],'format_rejection_not_ordinary_falsehood')
        self.assertFalse(row['result']['rust']['accepted']);self.assertEqual(row['result']['reference']['exit_code'],1)
    def test_entire_common_manifest_bytes_must_match_beyond_kernel_acceptance(self):
        r=next(x for x in strict_json((RECORD/'kernel-witnesses.json').read_bytes()) if x['id']=='go-two-overlays');native=self.native(r['query']);other=self.rows['go-default-contracts']['base']['certificate_manifest']
        with self.assertRaises(ValueError):paired_certificate(native,other)
        mixed=clone(native);mixed['byte_chunks'][-1]+='00'
        with self.assertRaises(ValueError):paired_certificate(mixed,self.rows['go-two-overlays']['base']['certificate_manifest'])
    def test_go_all169_requests_and_no_skeleton_remain_unelaborated(self):
        count=0
        for name in ('go-two-overlays','go-default-contracts'):
            row=self.rows[name];b=row['base'];p=self.docs[name]['evidence']['proof'];self.assertEqual(p['kind'],'go_ordinary_unelaborated');self.assertEqual(p['unelaborated_request_ids'],b['vc']['obligations']['unelaborated_request_ids']);count+=len(p['unelaborated_request_ids']);self.assertIsNone(b['skeleton'])
        self.assertEqual(count,169)
    def test_source_identity_cache_cannot_authorize_an_altered_source_row(self):
        row=clone(self.rows['go-two-overlays']);source_checked(row['base']['id']);del row['base']['project_spec'];row['native']=clone(self.rows['rust64']['native'])
        with self.assertRaises(ValueError):Policy(row,self.native)
    def test_source_rows_return_fresh_copies_of_immutable_authority(self):
        rows=source_rows();rows[0]['base']['module']['semantic_context']['source_language']='caller'
        self.assertNotEqual(source_rows()[0]['base']['module']['semantic_context']['source_language'],'caller')
    def test_unknown_native_queries_are_fatal_probe_errors(self):
        with self.assertRaises(AssertionError):self.native({'mode':'unobserved'})
    def test_go_has_no_installed_retained_policy_or_api_fallback(self):
        for name in ('go-two-overlays','go-default-contracts'):
            with self.assertRaises(ValueError):native_query(self.rows[name],'verify')
    def test_raw_scan_identity_and_three_domain_roots_are_independently_hashed(self):
        rows=hashes(self.docs);rust=strict_json((RECORD/'rust-hashes.json').read_bytes());go=strict_json((RECORD/'go-hashes.json').read_bytes());raw=strict_json((RECORD/'go-raw-hashes.json').read_bytes())
        self.assertEqual([{k:v for k,v in x.items() if k!='canonical_sha256'} for x in rust],go);self.assertEqual([{k:x[k] for k in ('id','canonical_sha256')} for x in rust if x['accepted']],raw)
        self.assertTrue(all(x['public_scan_domain'] is None for x in rows if x['id'].endswith('-scan')))
    def test_768_depth_and_safe_json_boundaries_match_native_implementations(self):
        r={x['id']:x for x in strict_json((RECORD/'rust-hashes.json').read_bytes())}
        for name in ('depth-768','empty-depth-768','unicode-order'):self.assertTrue(r[name]['accepted'])
        for name in ('depth-769','empty-depth-769','duplicate','unsafe-integer','unpaired','bad-domain'):self.assertFalse(r[name]['accepted'])
    def test_go_and_assembly_receipts_cannot_be_promoted_by_rehashing(self):
        for name in ('go-two-overlays','go-default-contracts','go_fixed','csharp','java'):
            value=clone(self.docs[name]['receipt']);value['status']='checked_ordinary_vc';reseal('receipt',value)
            with self.assertRaises(ValueError):Policy(self.rows[name],self.native).validate('receipt',value)
    def test_native_absent_zero_count_protocol_is_validated_without_bool_coercion(self):
        row=self.rows['rust64'];old=self.native(native_query(row,'verify'));q={'mode':'certificate','byte_chunks':old['outcome']['byte_chunks'],'source_manifest':row['base']['certificate_manifest']};r=self.native(q)
        self.assertNotIn('axiom_count',r['reference']['stdout']);paired_certificate(r,row['base']['certificate_manifest']);r['reference']['stdout']['axiom_count']=False
        with self.assertRaises(ValueError):paired_certificate(r,row['base']['certificate_manifest'])
    def test_no_registered_frontend_source_semantics_activation_or_freeze_promotion(self):
        for field in ('Go_ordinary_elaborated','registered_frontend_executed','source_semantics_proved','profile_activated','W04_frozen'):self.assertIs(self.report[field],False)
        for x in self.out:
            if x['result']:
                for f in ('registered_frontend_execution','source_semantics_proved','profile_activated'):self.assertIs(x['result'][f],False)


if __name__=='__main__':unittest.main()
