"""Source-bound API v4 design model; compiled helper execution remains external."""
from api_contracts import (PROFILE, REQUEST, RESPONSE, SESSION, REQUEST_FIELDS,
    RESPONSE_FIELDS, SESSION_FIELDS, SESSION_REF, BY_ROUTE, SOURCE_REFS, VC_REF,
    PROFILES, OPERATION_ERRORS, choice, validate, raw_ref, session_ref, read_wire, wire)
from api_json import canonical, clone
from operation_signatures import exact, need
from policy_sources import source_rows
from policy_roots import source_checked
from run_proof_pilot import sha


class OperationError(ValueError): pass


def op_need(ok, code):
    if not ok:
        need(code in OPERATION_ERRORS, 'frozen operation error code')
        raise OperationError(code)


class ApiModel:
    """One fixed independent source authority and one server-owned incarnation.

    Only model session/source transitions run here. Ordinary operations require
    their actual typed executor; a valid payload is never simulated success.
    """
    def __init__(self, row, incarnation):
        expected = next((r for r in source_rows() if r['base']['id'] == row['base']['id']), None)
        need(expected is not None and canonical(row) == canonical(expected), 'immutable independently validated source authority')
        source_checked(row['base']['id']); validate('sha256', incarnation)
        self.row = clone(expected); self.b = self.row['base']; self.incarnation = incarnation
        self.context = clone(self.b['module']['semantic_context'])
        self.selection = clone(self.row['request']['semantic_request']['selection'])
        self.source_refs = {k: {'schema': self.b[f]['schema'], h: self.b[f][h]}
                            for k, f, h in [('vir', 'module', 'vir_hash'), ('source_map', 'source_map', 'source_map_hash'),
                                            ('frontend_manifest', 'manifest', 'source_manifest_hash')]}
        self.vc_ref = {'schema': self.b['vc']['schema'], 'vc_hash': self.b['vc']['vc_hash']}
        self.go = self.b['module']['program']['kind'] == 'go_practical'
        self.pending = clone(self.b['vc']['obligations']['unelaborated_request_ids']) if self.go else []
        self.session = None; self.responses = {}

    def request(self, path, arguments, method='POST'):
        return {'schema': REQUEST, 'api_profile': PROFILE, 'semantic_context': clone(self.context),
                'selection': clone(self.selection), 'session_ref': session_ref(self.session) if self.session else None,
                'method': method, 'path': path, 'arguments': clone(arguments)}

    def validate_request(self, raw):
        r = read_wire(raw); exact(r, REQUEST_FIELDS)
        need(r['schema'] == REQUEST and r['api_profile'] == PROFILE, 'exact API v4 family')
        need(type(r['method']) is str and type(r['path']) is str, 'route strings')
        route = BY_ROUTE.get((r['method'], r['path'])); need(route is not None, 'declared exact route')
        need(r['semantic_context'] == self.context and r['selection'] == self.selection, 'complete context and selection identity')
        if r['path'] == '/module/new': need(r['session_ref'] is None and self.session is None, 'new isolated session')
        else:
            validate(SESSION_REF, r['session_ref'])
            need(self.session is not None and r['session_ref'] == session_ref(self.session), 'current server-owned snapshot,not caller state or hash alone')
        validate(route['arguments'], r['arguments'])
        return r, route

    def validate_session(self, s):
        exact(s, SESSION_FIELDS)
        need(s['schema'] == SESSION and s['api_profile'] == PROFILE and s['helper_only'] is True, 'closed helper session')
        need(s['semantic_context'] == self.context and s['selection'] == self.selection and s['incarnation'] == self.incarnation, 'session context/selection/incarnation')
        validate('text', s['session_id']); validate('safe_uint', s['revision']); validate('name', s['module_name'])
        validate(PROFILES, s['proof_profile']); validate(choice('active', 'frozen'), s['lifecycle'])
        state = s['source_state']; need(type(state) is dict, 'source state')
        kind = state.get('kind'); need(type(kind) is str and kind in ('empty', 'vir_imported', 'vc_generated'), 'closed source-state discriminant')
        fields = ['kind'] + ([] if kind == 'empty' else ['source_refs']) + (['vc_ref', 'ordinary_status', 'unelaborated_request_ids'] if kind == 'vc_generated' else [])
        exact(state, fields)
        if kind != 'empty': need(state['source_refs'] == self.source_refs, 'session source graph')
        if kind == 'vc_generated':
            need(state['vc_ref'] == self.vc_ref and state['ordinary_status'] == ('unelaborated' if self.go else 'ready') and
                 state['unelaborated_request_ids'] == self.pending, 'exact source-derived ordinary state; no readiness flag')

    def transition(self, r, route):
        """Prepare against a copy; no visible mutation before encoding/validation."""
        p = r['path']; a = r['arguments']; s = clone(self.session)
        if p == '/module/new':
            s = {'schema': SESSION, 'api_profile': PROFILE, 'semantic_context': clone(self.context),
                 'selection': clone(self.selection), 'session_id': 'model-s1', 'incarnation': self.incarnation,
                 'revision': 0, 'module_name': a['module_name'], 'proof_profile': a['proof_profile'],
                 'lifecycle': 'active', 'source_state': {'kind': 'empty'}, 'helper_only': True}
            return s, {'session_id': s['session_id']}
        op_need(s['proof_profile'] in route['checker_profiles'], 'AI_API_V4_PROFILE_OPERATION')
        op_need(not route['mutates'] or s['lifecycle'] == 'active', 'AI_API_V4_SESSION_FROZEN')
        kind = s['source_state']['kind']
        if p == '/vir/import':
            op_need(kind == 'empty', 'AI_API_V4_SOURCE_STATE')
            op_need(a['vir'] == self.b['module'] and a['source_refs'] == self.source_refs, 'AI_API_V4_SOURCE_BINDING')
            s['source_state'] = {'kind': 'vir_imported', 'source_refs': clone(self.source_refs)}
            value = {'source_refs': clone(self.source_refs)}
        elif p == '/vc/generate':
            op_need(kind == 'vir_imported', 'AI_API_V4_SOURCE_STATE')
            op_need(a['source_refs'] == self.source_refs, 'AI_API_V4_SOURCE_BINDING')
            value = {'vc_ref': clone(self.vc_ref), 'ordinary_status': 'unelaborated' if self.go else 'ready',
                     'unelaborated_request_ids': clone(self.pending)}
            s['source_state'] = {'kind': 'vc_generated', 'source_refs': clone(self.source_refs), **clone(value)}
        elif p.startswith('/vc/'):
            op_need(kind == 'vc_generated', 'AI_API_V4_SOURCE_STATE')
            op_need(a['vc_ref'] == self.vc_ref, 'AI_API_V4_VC_BINDING')
            if p == '/vc/list':
                members = []
                if not self.go:
                    # Use actual compiled API listing, never synthesize ordinary targets.
                    from api_vectors import native_listing
                    members = native_listing(self.b['id'])
                value = {'vc_ref': clone(self.vc_ref), 'ordinary_status': 'unelaborated' if self.go else 'ready',
                         'members': members, 'unelaborated_request_ids': clone(self.pending)}
            else:
                op_need(not self.pending, 'AI_API_V4_ORDINARY_PENDING')
                raise OperationError('AI_API_V4_CANDIDATE_EXECUTOR_REQUIRED')
        elif p == '/module/freeze':
            s['lifecycle'] = 'frozen'; value = {'lifecycle': 'frozen'}
        else:
            if p == '/module/export-certificate': op_need(not self.pending or kind == 'empty', 'AI_API_V4_ORDINARY_PENDING')
            raise OperationError('AI_API_V4_ORDINARY_EXECUTOR_REQUIRED')
        if route['mutates']:
            op_need(s['revision'] < (1 << 53)-1, 'AI_API_V4_REVISION_EXHAUSTED')
            s['revision'] += 1
        return s, value

    def exchange(self, raw):
        r, route = self.validate_request(raw)
        # Transport/shape/context/stale-snapshot errors have no typed artifact.
        # Valid request failures use the same closed response root and old state.
        try:
            proposed, value = self.transition(r, route)
            result = {'kind': 'ok', 'value': value}
        except OperationError as e:
            proposed = clone(self.session)
            result = {'kind': 'error', 'phase': 'operation', 'code': str(e)}
        if proposed is not None: self.validate_session(proposed)
        response = {'schema': RESPONSE, 'api_profile': PROFILE, 'semantic_context': clone(self.context),
                    'selection': clone(self.selection), 'request_ref': raw_ref(r),
                    'session_ref': session_ref(proposed) if proposed else None,
                    'helper_only': True, 'result': result}
        self.validate_response(r, route, response, proposed)
        encoded = wire(response); read_wire(encoded)
        self.session = clone(proposed)  # Last step,after every fallible operation.
        self.responses[raw_ref(r)['canonical_sha256']] = encoded
        return encoded

    def validate_response(self, r, route, response, expected_session):
        exact(response, RESPONSE_FIELDS)
        need(response['schema'] == RESPONSE and response['api_profile'] == PROFILE and response['helper_only'] is True, 'closed helper response')
        need(response['semantic_context'] == self.context and response['selection'] == self.selection and response['request_ref'] == raw_ref(r), 'complete request/response binding')
        need(response['session_ref'] == (session_ref(expected_session) if expected_session else None), 'complete response/session binding')
        result = response['result']; need(type(result) is dict, 'result union')
        if result.get('kind') == 'ok':
            exact(result, ('kind', 'value')); validate(route['value'], result['value'])
        else:
            exact(result, ('kind', 'phase', 'code'))
            need(result['kind'] == 'error' and result['phase'] == 'operation', 'closed failure arm')
            validate(choice(*OPERATION_ERRORS), result['code'])

    def import_response(self, raw):
        # Shape/hash consistency alone cannot authenticate a backend outcome.
        value = read_wire(raw); exact(value, RESPONSE_FIELDS)
        validate({'schema': choice(REQUEST), 'canonical_sha256': 'sha256'}, value['request_ref'])
        expected = self.responses.get(value['request_ref']['canonical_sha256'])
        need(expected is not None and raw == expected, 'response must match the locally regenerated complete transaction')
        return value

    def import_session(self, raw):
        value = read_wire(raw); self.validate_session(value)
        need(value == self.session, 'session snapshot is server-owned,not imported caller authority')
        return value
