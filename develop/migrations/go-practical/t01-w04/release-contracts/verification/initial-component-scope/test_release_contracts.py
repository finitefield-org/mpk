"""Release metadata,actual legacy selection,and non-promotable build state."""
import unittest
from release_contracts import *
from release_vectors import documents
from run_release_contracts import RECORD,check_retained
from check_scope_report import strict_json


class ReleaseTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.native=strict_json((RECORD/'native-legacy.json').read_bytes())[0];cls.m=ReleaseModel(cls.native)
    def test_all_seven_roots_and_six_required_contexts(self):
        self.assertEqual(len(self.m.docs),7);self.assertEqual(len(self.m.contexts),6)
        for f,values in self.m.docs.items():
            for i,v in enumerate(values if type(values)is list else[values]):self.assertFalse(self.m.import_json(f,wire(v),i)['runtime_ready'])
    def test_actual_old_validator_resolves_all_four_profiles_and_five_targets(self):
        self.assertEqual((self.native['frontends'],self.native['toolchains'],len(self.native['tuples'])),(4,4,5))
        self.assertTrue(all(not x['accepted']for x in self.native['controls']))
        self.assertEqual({r['semantic_context']['source_language']for r in self.native['tuples']},{'go','rust','csharp','java'})
    def test_retained_bytes_modes_and_runtime_component_metadata_are_preserved(self):
        for f,field in [('frontend_bundle','frontend_bundles'),('toolchain_bundle','toolchain_bundles')]:
            for old,new in zip(self.m.old[field],self.m.docs[f]):
                self.assertNotEqual(old['bundle_id'],new['bundle_id'])
                self.assertEqual(new['inventory']['members'],[{'path':p['path'],'raw_sha256':p['sha256'],'size_bytes':p['size_bytes']}for p in old['inventory']['files']])
                self.assertEqual(new['file_modes'],[{'path':p['path'],'executable':p['executable']}for p in old['inventory']['files']])
                for k in ('main','subordinate_binaries','components','execution_host_profile_id'):
                    if k in old:self.assertEqual(old[k],new[k])
    def test_every_new_runtime_selection_remains_blocked_until_real_build(self):
        for t in self.m.docs['bundle_registry']['tuples']:
            with self.assertRaisesRegex(ValueError,'materialization'):self.m.resolve(t['semantic_context'],t['frontend_bundle_id'],t['toolchain_bundle_id'])
    def test_practical_go_has_no_relabelled_fixed_go_frontend(self):
        go=next(c for c in self.m.contexts if c['semantic_profile']=='mpk.go.practical.v1')
        self.assertFalse(any(t['semantic_context']['semantic_profile']=='mpk.go.practical.v1'for t in self.m.docs['bundle_registry']['tuples']))
        t=next(t for t in self.m.docs['bundle_registry']['tuples']if t['semantic_context']['source_language']=='go')
        with self.assertRaises(ValueError):self.m.resolve(go,t['frontend_bundle_id'],t['toolchain_bundle_id'])
        self.assertIn({'kind':'tuple','id':key(go)},self.m.docs['bundle_candidate']['missing'])
    def test_rehashed_passed_evidence_and_receipts_cannot_authorize_activation(self):
        v=clone(self.m.docs['release_evidence']);v['checks'][0]['status']='passed';v['state']='passed'
        with self.assertRaises(ValueError):self.m.validate('release_evidence',v)
        v=clone(self.m.docs['release_receipt']);v['status']='release_verified';v['receipt_hash']=digest(domain('release_receipt'),{k:x for k,x in v.items()if k!='receipt_hash'})
        with self.assertRaises(ValueError):self.m.authorize_activation(v)
        with self.assertRaisesRegex(ValueError,'T07'):self.m.authorize_activation(self.m.docs['release_receipt'])
    def test_execution_mode_and_origin_are_bound_even_when_content_hashes_stay_same(self):
        v=clone(self.m.docs['frontend_bundle'][0]);v['file_modes'][0]['executable']=not v['file_modes'][0]['executable']
        v['bundle_sha256']=digest(CONTENT_DOMAIN,{k:x for k,x in v.items()if k!='bundle_sha256'})
        with self.assertRaises(ValueError):self.m.validate('frontend_bundle',v)
        v=clone(self.m.docs['frontend_bundle'][0]);v['build_origin']['kind']='reproduced'
        with self.assertRaises(ValueError):self.m.validate('frontend_bundle',v)
    def test_historical_linkage_reference_is_not_a_complete_release_registry(self):
        with self.assertRaises(ValueError):self.m.bind_frontend_reference(release_fixture()['reference'])
        self.m.bind_frontend_reference(release_ref(self.m.docs['bundle_registry']))
    def test_new_hashes_transports_and_parent_graph_have_no_self_cycle(self):
        image=self.m.docs['release_registry'];self.assertIsNone(image['consumer']);self.assertIsNone(image['image_inventory'])
        self.assertEqual(image['bundle_registry'],release_ref(self.m.docs['bundle_registry']))
        self.assertNotIn('release_registry',self.m.docs['bundle_registry'])
        for raw in (canonical(image),wire(image)+b'\n',b' '+wire(image)):
            with self.assertRaises(ValueError):read_wire(raw)
    def test_complete_record_and_independent_rust_go_content_domains(self):
        r=check_retained();self.assertEqual(r['hash_vectors'],13);self.assertEqual(r['retained_tuples'],5)
        self.assertFalse(r['native_release_observed']);self.assertFalse(r['profile_activated'])


if __name__=='__main__':unittest.main()
