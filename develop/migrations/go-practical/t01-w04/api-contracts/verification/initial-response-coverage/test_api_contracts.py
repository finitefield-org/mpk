"""API helper authority, replay, pending-Go and failure-atomicity design checks."""
import unittest
from unittest.mock import patch
from api_json import canonical, parse, clone, MAX_BYTES, MAX_DEPTH
from api_contracts import (ROUTES, BY_ROUTE, REQUEST, SESSION, RESPONSE, PROFILE,
    assert_route_inventory, validate, wire, read_wire, session_ref, raw_ref)
from api_protocol import ApiModel
from api_vectors import model, source_sequence, documents, arguments
from policy_sources import source_rows
from run_api_contracts import RECORD, check_retained
from check_scope_report import strict_json


class ApiTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.rows = {r['base']['id']: r for r in source_rows()}
        cls.docs, cls.vectors = documents()

    def go(self): return source_sequence(self.rows['go-two-overlays'])

    def test_all33_declared_routes_have_closed_request_and_response_contracts(self):
        assert_route_inventory(); self.assertEqual(len(BY_ROUTE), 33)
        self.assertEqual(sum(r['method'] == 'GET' for r in ROUTES), 1)
        for row in self.rows.values():
            m = model(row)
            for route in ROUTES: validate(route['arguments'], arguments(route, m))

    def test_complete_five_profile_eight_bundle_wire_and_transaction_matrix(self):
        self.assertEqual(len(self.docs), 8)
        self.assertEqual(len({row['base']['module']['semantic_context']['semantic_profile'] for row in self.rows.values()}), 5)
        self.assertGreater(len(self.vectors), 5000)
        for row in self.docs.values():
            self.assertEqual(len(row), 12)
            self.assertTrue(all(r['response']['helper_only'] is True and r['session']['helper_only'] is True for r in row))

    def test_go_vc_list_keeps_all169_unelaborated_demands_and_no_ordinary_members(self):
        count = 0
        for id_ in ('go-two-overlays', 'go-default-contracts'):
            r = self.docs[id_][3]['response']['result']['value']
            self.assertEqual(r['ordinary_status'], 'unelaborated'); self.assertEqual(r['members'], [])
            self.assertEqual(r['unelaborated_request_ids'], self.rows[id_]['base']['vc']['obligations']['unelaborated_request_ids'])
            count += len(r['unelaborated_request_ids'])
        self.assertEqual(count, 169)

    def test_go_pending_blocks_all_candidate_routes_and_source_export(self):
        for id_ in ('go-two-overlays', 'go-default-contracts'):
            for i in (5, 6, 7, 8):
                x = self.docs[id_][i]
                self.assertEqual(x['response']['result']['code'], 'AI_API_V4_ORDINARY_PENDING')
                self.assertFalse(x['state_changed'])

    def test_valid_ordinary_payload_never_becomes_simulated_endpoint_success(self):
        for row in self.docs.values():
            self.assertEqual(row[4]['response']['result']['code'], 'AI_API_V4_ORDINARY_EXECUTOR_REQUIRED')
            self.assertFalse(row[4]['state_changed'])
        for id_ in ('rust64', 'rust32', 'go_fixed', 'csharp', 'java', 'go_fixed_synthetic'):
            self.assertEqual(self.docs[id_][5]['response']['result']['code'], 'AI_API_V4_CANDIDATE_EXECUTOR_REQUIRED')

    def test_every_failed_model_transaction_preserves_exact_session_bytes(self):
        failed = 0
        for rows in self.docs.values():
            previous = None
            for r in rows:
                if r['response']['result']['kind'] == 'error':
                    self.assertEqual(canonical(r['session']), canonical(previous)); self.assertFalse(r['state_changed']); failed += 1
                previous = r['session']
        self.assertEqual(failed, 48)

    def test_freeze_preserves_listing_and_rejects_later_term_allocation(self):
        for r in self.docs.values():
            self.assertEqual(r[9]['session']['lifecycle'], 'frozen')
            self.assertEqual(r[10]['response']['result']['kind'], 'ok'); self.assertFalse(r[10]['state_changed'])
            self.assertEqual(r[11]['response']['result']['code'], 'AI_API_V4_SESSION_FROZEN')

    def test_stale_replayed_mutation_and_forged_session_hash_cannot_update(self):
        m, seq = self.go(); before = wire(m.session)
        with self.assertRaises(ValueError): m.exchange(wire(seq[1]['request']))
        forged = clone(m.session); forged['revision'] += 1
        r = m.request('/module/freeze', {}); r['session_ref'] = session_ref(forged)
        with self.assertRaises(ValueError): m.exchange(wire(r))
        self.assertEqual(wire(m.session), before)

    def test_other_incarnation_cannot_reuse_even_identical_session_id(self):
        row = self.rows['go-two-overlays']; a = model(row); b = ApiModel(row, '1'*64)
        for m in (a, b): m.exchange(wire(m.request('/module/new', {'module_name': 'Probe.Api', 'proof_profile': 'mvp-strict'})))
        self.assertEqual(a.session['session_id'], b.session['session_id'])
        r = a.request('/module/freeze', {})
        with self.assertRaises(ValueError): b.exchange(wire(r))

    def test_encoding_or_invalid_backend_output_failure_is_not_semantic_rejection_or_commit(self):
        m = model(self.rows['rust64']); r = wire(m.request('/module/new', {'module_name': 'Probe.Api', 'proof_profile': 'mvp-strict'}))
        with patch('api_protocol.wire', side_effect=ValueError('encoding budget')):
            with self.assertRaises(ValueError): m.exchange(r)
        self.assertIsNone(m.session); self.assertEqual(m.responses, {})
        with patch.object(m, 'transition', return_value=(None, {'accepted': True})):
            with self.assertRaises(ValueError): m.exchange(r)
        self.assertIsNone(m.session); self.assertEqual(m.responses, {})

    def test_revision_exhaustion_is_atomic(self):
        m = model(self.rows['rust64']); m.exchange(wire(m.request('/module/new', {'module_name': 'Probe.Api', 'proof_profile': 'mvp-strict'})))
        m.session['revision'] = (1 << 53)-1; before = wire(m.session)
        response = read_wire(m.exchange(wire(m.request('/module/freeze', {}))))
        self.assertEqual(response['result']['code'], 'AI_API_V4_REVISION_EXHAUSTED'); self.assertEqual(wire(m.session), before)

    def test_response_must_match_locally_generated_outcome_beyond_valid_hashes(self):
        m, seq = self.go(); original = seq[3]['response']; forged = clone(original)
        forged['result']['value']['ordinary_status'] = 'ready'; forged['result']['value']['unelaborated_request_ids'] = []
        self.assertEqual(forged['request_ref'], original['request_ref'])
        with self.assertRaises(ValueError): m.import_response(wire(forged))
        fresh = model(self.rows['go-two-overlays'])
        with self.assertRaises(ValueError): fresh.import_response(wire(original))

    def test_complete_independent_source_authority_cannot_be_swapped_after_cache(self):
        row = clone(self.rows['rust64']); model(row)
        row['base']['module']['program']['kind'] = 'go_practical'
        with self.assertRaises(ValueError): model(row)

    def test_integer_to_bool_source_replacement_cannot_hide_behind_equal_Python_values(self):
        initial = RECORD/'verification/initial-source-equality'
        request = read_wire((initial/'candidate-request.json').read_bytes())
        m = model(self.rows['go_fixed'])
        m.exchange(wire(m.request('/module/new', {'module_name': 'Probe.Equality', 'proof_profile': 'mvp-strict'})))
        self.assertEqual(request['arguments']['vir'], m.b['module'])  # Python's confusable comparison.
        self.assertNotEqual(canonical(request['arguments']['vir']), canonical(m.b['module']))
        before = wire(m.session); r = read_wire(m.exchange(wire(request)))
        self.assertEqual(r['result']['code'], 'AI_API_V4_SOURCE_BINDING'); self.assertEqual(wire(m.session), before)

    def test_required_nulls_u32_names_and_certificate_chunks_are_unambiguous(self):
        validate('id32', (1 << 32)-1)
        for value in (-1, 1 << 32, True, 1.0, '0'):
            with self.assertRaises(ValueError): validate('id32', value)
        validate('name', "Core.Nat.rec'")
        for value in ('', ' A', 'A.', '0A', 'A.日本'):
            with self.assertRaises(ValueError): validate('name', value)
        for value in ([], [''], ['0'], ['AA'], ['00', '00']):
            with self.assertRaises(ValueError): validate('chunks', value)

    def test_canonical_transport_and_raw_hash_exclude_exactly_one_LF(self):
        m, seq = self.go(); r = seq[0]['request']; ref = raw_ref(r)
        from run_proof_pilot import sha
        self.assertEqual(ref['canonical_sha256'], sha(canonical(r)))
        self.assertNotEqual(ref['canonical_sha256'], sha(wire(r)))
        for bad in (wire(r)[:-1], wire(r) + b'\n', b' ' + wire(r), b'\xef\xbb\xbf' + wire(r)):
            with self.assertRaises(ValueError): read_wire(bad)

    def test_envelope_depth_adds_four_levels_without_widening_embedded_artifact(self):
        self.assertEqual((MAX_BYTES, MAX_DEPTH), (269484032, 772))
        value = b'['*772 + b'0' + b']'*772; self.assertEqual(canonical(parse(value)), value)
        with self.assertRaises(ValueError): parse(b'[' + value + b']')
        from proof_canonical import canonical as source_canonical
        with self.assertRaises(ValueError): source_canonical(parse(value))
        with self.assertRaises(ValueError): parse(b'{"x":0,"x":0}')
        with self.assertRaises(ValueError): parse(b'9007199254740992')

    def test_reserved_CSharp_and_released_API_profiles_are_not_widened(self):
        m = model(self.rows['rust64']); r = m.request('/module/new', {'module_name': 'Probe.Api', 'proof_profile': 'mvp-strict'})
        for profile in ('mpk.ai.api.v2', 'mpk.ai.api.v3'):
            value = clone(r); value['api_profile'] = profile
            with self.assertRaises(ValueError): m.exchange(wire(value))

    def test_retained_native_API_traces_and_independent_hashes_match_complete_record(self):
        r = check_retained()
        self.assertEqual(r['retained_native_API_operations'], 102)
        self.assertEqual(r['retained_native_atomic_failures'], 72)
        self.assertFalse(r['compiled_APIv4_executed']); self.assertFalse(r['profile_activated'])


if __name__ == '__main__': unittest.main()
