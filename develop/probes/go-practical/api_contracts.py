"""Inactive API v4 wire contracts for every declared successor route.

These are design validators. They do not implement a kernel, endpoint, or
registered source frontend. All objects are closed and all fields required.
"""
import re
from api_json import canonical, parse, clone, MAX_BYTES, MAX_DEPTH, MAX_STRING
from identity_catalog import SHARED
from operation_signatures import exact, need
from run_proof_pilot import sha, ROOT

PROFILE = SHARED['api'][0]
REQUEST = SHARED['api_request'][0]
RESPONSE = SHARED['api_response'][0]
SESSION = SHARED['api_session'][0]
REQUEST_FIELDS = ('schema', 'api_profile', 'semantic_context', 'selection',
                  'session_ref', 'method', 'path', 'arguments')
SESSION_FIELDS = ('schema', 'api_profile', 'semantic_context', 'selection',
                  'session_id', 'incarnation', 'revision', 'module_name',
                  'proof_profile', 'lifecycle', 'source_state', 'helper_only')
RESPONSE_FIELDS = ('schema', 'api_profile', 'semantic_context', 'selection',
                   'request_ref', 'session_ref', 'helper_only', 'result')


def array(item): return {'array': item}
def choice(*values): return {'enum': list(values)}
def nullable(item): return {'nullable': item}


REF = {'schema': 'text', 'canonical_sha256': 'sha256'}
SESSION_REF = {'schema': choice(SESSION), 'session_id': 'text',
               'incarnation': 'sha256', 'revision': 'safe_uint',
               'canonical_sha256': 'sha256'}
SOURCE_REFS = {k: {'schema': choice(SHARED[f][0]), h: 'sha256'}
               for k, f, h in [('vir', 'vir', 'vir_hash'),
                               ('source_map', 'source_map', 'source_map_hash'),
                               ('frontend_manifest', 'frontend_manifest', 'source_manifest_hash')]}
VC_REF = {'schema': choice(SHARED['vc'][0]), 'vc_hash': 'sha256'}
TARGET = {'kind': choice('member', 'group'), 'id': 'text'}
CANDIDATE = {'candidate_id': 'text', 'proof_root': 'id32'}
PROFILES = choice('core-bootstrap', 'mvp-structural', 'mvp-strict')
OPERATION_ERRORS = ('AI_API_V4_PROFILE_OPERATION', 'AI_API_V4_SESSION_FROZEN',
                    'AI_API_V4_SOURCE_STATE', 'AI_API_V4_SOURCE_BINDING',
                    'AI_API_V4_VC_BINDING', 'AI_API_V4_ORDINARY_PENDING',
                    'AI_API_V4_CANDIDATE_EXECUTOR_REQUIRED',
                    'AI_API_V4_ORDINARY_EXECUTOR_REQUIRED', 'AI_API_V4_REVISION_EXHAUSTED')
TERM_OUT = {'term_id': 'id32'}
PROOF_OUT = {'proof_id': 'id32'}
DIAGNOSTIC = {'code': 'text', 'node_id': nullable('id32'),
              'expected_type_id': nullable('id32'), 'actual_type_id': nullable('id32'),
              'context_summary': array('id32'), 'repair_hints': array('text')}
CANDIDATE_RESULT = {**CANDIDATE, 'helper_status': choice('valid', 'invalid'),
                    'diagnostic': nullable(DIAGNOSTIC)}


def route(path, request, response, *, method='POST', mutates=True,
          structural=False, executor='ordinary'):
    return {'method': method, 'path': path, 'arguments': request,
            'value': response, 'mutates': mutates,
            'checker_profiles': ['mvp-structural', 'mvp-strict'] if structural else
            ['core-bootstrap', 'mvp-structural', 'mvp-strict'], 'executor': executor}


ROUTES = [
    route('/module/new', {'module_name': 'name', 'proof_profile': PROFILES},
          {'session_id': 'text'}, executor='session'),
    route('/module/import', {'byte_chunks': 'chunks', 'certificate_hash': 'sha256',
                            'export_hash': 'sha256', 'axiom_report_hash': 'sha256'},
          {'export_hash': 'sha256', 'declaration_count': 'safe_uint'}),
    route('/module/freeze', {}, {'lifecycle': choice('frozen')}, executor='session'),
    route('/module/export-certificate', {},
          {'byte_chunks': 'chunks', 'certificate_hash': 'sha256',
           'source_manifest_ref': nullable({'schema': choice(SHARED['certificate_manifest'][0]),
                                            'source_manifest_hash': 'sha256'})}, mutates=False),
    route('/term/sort', {'universe': 'id32'}, TERM_OUT),
    route('/term/var', {'index': 'id32'}, TERM_OUT),
    route('/term/const', {'name': 'name', 'levels': array('id32')}, TERM_OUT),
    route('/term/app', {'function': 'id32', 'arguments': array('id32')}, TERM_OUT),
    route('/term/lam', {'ty': 'id32', 'body': 'id32'}, TERM_OUT),
    route('/term/pi', {'ty': 'id32', 'body': 'id32'}, TERM_OUT),
    route('/term/let', {'ty': 'id32', 'value': 'id32', 'body': 'id32'}, TERM_OUT),
    route('/term/check', {'term': 'id32', 'expected_type': 'id32'}, {'ok': 'bool'}),
    route('/term/infer', {'term': 'id32'}, {'type_id': 'id32'}),
    route('/term/defeq', {'left': 'id32', 'right': 'id32'}, {'equal': 'bool'}, mutates=False),
    route('/proof/exact', {'term': 'id32', 'expected_type': 'id32'}, PROOF_OUT),
    route('/proof/apply', {'function_proof': 'id32', 'argument_proofs': array('id32'),
                           'expected_type': 'id32'}, PROOF_OUT),
    route('/proof/intro', {'domain_type': 'id32', 'body_proof': 'id32',
                           'expected_type': 'id32'}, PROOF_OUT),
    route('/proof/refl', {'term': 'id32', 'expected_type': 'id32'}, PROOF_OUT),
    route('/proof/let', {'value': 'id32', 'body_proof': 'id32',
                         'expected_type': 'id32'}, PROOF_OUT, structural=True),
    route('/proof/rewrite', {'eq_proof': 'id32', 'target_proof': 'id32',
                             'expected_type': 'id32'}, PROOF_OUT, structural=True),
    route('/proof/eq-rec', {'motive': 'id32', 'eq_proof': 'id32',
                            'base_proof': 'id32', 'expected_type': 'id32'},
          PROOF_OUT, structural=True),
    route('/proof/constructor', {'constructor': 'name', 'argument_proofs': array('id32'),
                                 'expected_type': 'id32'}, PROOF_OUT, structural=True),
    route('/proof/recursor', {'recursor': 'name', 'motive': 'id32',
                              'minor_proofs': array('id32'), 'major_proof': 'id32',
                              'expected_type': 'id32'}, PROOF_OUT, structural=True),
    route('/proof/conv', {'proof': 'id32', 'expected_type': 'id32',
                          'defeq_witness': nullable('id32')}, PROOF_OUT),
    route('/proof/theory', {'theory_certificate': 'id32', 'expected_type': 'id32'}, PROOF_OUT),
    route('/proof/check-node', {'proof_id': 'id32'},
          {'proof_id': 'id32', 'ok': 'bool', 'term_id': 'id32'}),
    route('/proof/check-decl', {'name': 'name', 'type_id': 'id32', 'proof_id': 'id32'},
          {'ok': 'bool'}, mutates=False),
    route('/vir/import', {'vir': 'embedded_vir', 'source_refs': SOURCE_REFS},
          {'source_refs': SOURCE_REFS, 'unit_count': 'safe_uint', 'function_count': 'safe_uint'}, executor='source'),
    route('/vc/generate', {'source_refs': SOURCE_REFS},
          {'vc_ref': VC_REF, 'ordinary_status': choice('ready', 'unelaborated'),
           'unelaborated_request_ids': array('sha256'), 'vc': 'embedded_vc'}, executor='source'),
    route('/vc/list', {'vc_ref': VC_REF},
          {'vc_ref': VC_REF, 'ordinary_status': choice('ready', 'unelaborated'),
           'members': array({'member_id': 'text', 'function_id': 'text',
                             'kind': 'text', 'group_id': 'text'}),
           'unelaborated_request_ids': array('sha256')},
          method='GET', mutates=False, executor='source'),
    route('/vc/start-proof', {'vc_ref': VC_REF, 'target': TARGET},
          {'target': TARGET, 'target_id': 'text'}, executor='candidate'),
    route('/vc/attach-candidate', {'vc_ref': VC_REF, 'target_id': 'text', **CANDIDATE},
          {'target_id': 'text', **CANDIDATE}, executor='candidate'),
    route('/vc/check-candidate', {'vc_ref': VC_REF, 'target_id': 'text',
                                 'mode': choice('fail_fast_per_candidate'),
                                 'candidates': array(CANDIDATE)},
          {'target_id': 'text', 'mode': choice('fail_fast_per_candidate'),
           'results': array(CANDIDATE_RESULT)}, mutates=False, executor='candidate'),
]
BY_ROUTE = {(r['method'], r['path']): r for r in ROUTES}


def validate(shape, value):
    if type(shape) is dict:
        if set(shape) == {'enum'}:
            need(type(value) is str and value in shape['enum'], 'closed enum'); return
        if set(shape) == {'nullable'}:
            if value is not None: validate(shape['nullable'], value)
            return
        if set(shape) == {'array'}:
            need(type(value) is list, 'array')
            for v in value: validate(shape['array'], v)
            return
        exact(value, shape)
        for key, child in shape.items(): validate(child, value[key])
        return
    if shape in ('id32', 'safe_uint'):
        need(type(value) is int and 0 <= value <= ((1 << 32)-1 if shape == 'id32' else (1 << 53)-1), 'unsigned integral ID,not bool')
    elif shape == 'bool': need(type(value) is bool, 'Boolean')
    elif shape == 'text': need(type(value) is str and 0 < len(value.encode()) <= MAX_STRING, 'bounded nonempty text')
    elif shape == 'name':
        validate('text', value)
        need(re.fullmatch(r"[A-Za-z_][A-Za-z_0-9']*(?:\.[A-Za-z_][A-Za-z_0-9']*)*", value), 'existing core Name grammar')
    elif shape == 'sha256': need(type(value) is str and re.fullmatch('[0-9a-f]{64}', value), 'lowercase SHA256')
    elif shape == 'chunks':
        need(type(value) is list and len(value) > 0, 'certificate chunks')
        for s in value: need(type(s) is str and 0 < len(s) <= 65536 and len(s) % 2 == 0 and re.fullmatch('[0-9a-f]+', s), 'nonempty <=32KiB byte chunk')
        need(all(len(s) == 65536 for s in value[:-1]), 'unique chunk segmentation')
    elif shape in ('embedded_vir', 'embedded_vc'):
        family = 'vir' if shape == 'embedded_vir' else 'vc'
        need(type(value) is dict and value.get('schema') == SHARED[family][0], 'common embedded source artifact root')
        from proof_canonical import canonical as embedded_canonical
        embedded_canonical(value)  # Keeps the embedded artifact's original limits.
    else: raise AssertionError('unclosed API shape: ' + str(shape))


def raw_ref(value): return {'schema': value['schema'], 'canonical_sha256': sha(canonical(value))}
def session_ref(value):
    return {**raw_ref(value), **{k: value[k] for k in ('session_id', 'incarnation', 'revision')}}
def wire(value): return canonical(value) + b'\n'
def read_wire(raw):
    need(type(raw) is bytes and len(raw) <= MAX_BYTES + 1 and raw.endswith(b'\n'), 'API transport length and LF')
    v = parse(raw[:-1]); need(raw == wire(v), 'exact canonical API transport plus one LF'); return v


def assert_route_inventory():
    text = (ROOT/'crates/mpk-api/src/successor_api.rs').read_text()
    active = text.split('pub const SUCCESSOR_ROUTES:', 1)[1].split('const fn route', 1)[0]
    pairs = re.findall(r'route\(\s*"(GET|POST)",\s*"([^"]+)"', active)
    need(len(pairs) == len(BY_ROUTE) == len(ROUTES) == 33 and set(pairs) == set(BY_ROUTE), 'all exact declared routes retained')


def catalogue():
    assert_route_inventory()
    return {'schema': 'mpk.go_practical.api_contract_draft.v0',
            'roots': {REQUEST: list(REQUEST_FIELDS), RESPONSE: list(RESPONSE_FIELDS), SESSION: list(SESSION_FIELDS)},
            'api_profile': PROFILE, 'routes': ROUTES,
            'transport': {'canonical_bytes_max': MAX_BYTES, 'wire_bytes_max': MAX_BYTES + 1,
                          'container_depth_max': MAX_DEPTH, 'decoded_string_bytes_max': MAX_STRING,
                          'terminator': 'exactly one LF', 'embedded_artifact_limits': 'unchanged; validate separately'},
            'hash_rule': 'raw SHA256 of entire canonical root without LF; no own hash field and no invented API hash domain',
            'request_ref': REF, 'session_ref': SESSION_REF,
            'operation_error_codes': list(OPERATION_ERRORS),
            'session': {'fields': list(SESSION_FIELDS), 'source_state_arms': {
                'empty': ['kind'], 'vir_imported': ['kind', 'source_refs'],
                'vc_generated': ['kind', 'source_refs', 'vc_ref', 'ordinary_status', 'unelaborated_request_ids']},
                'source_refs': SOURCE_REFS, 'vc_ref': VC_REF, 'proof_profile': PROFILES,
                'lifecycle': ['active', 'frozen'], 'ordinary_status': ['ready', 'unelaborated']},
            'response_result': {'ok': ['kind', 'value'], 'error': ['kind', 'phase', 'code']},
            'authority': 'helper_only=true for every response and session; no API proof/release acceptance state',
            'stage': 'inactive design; full declared route catalogue is not a claim of 33 implemented native endpoints'}
