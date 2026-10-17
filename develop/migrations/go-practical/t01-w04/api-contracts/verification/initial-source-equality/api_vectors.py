"""Reconstructible API contract/transaction vectors and actual retained listing."""
from functools import lru_cache
from api_contracts import (ROUTES, REQUEST, RESPONSE, SESSION, PROFILE, wire,
    read_wire, validate, session_ref, raw_ref)
from api_json import canonical, clone, parse
from api_protocol import ApiModel
from policy_sources import source_rows
from run_proof_pilot import ROOT, sha
from check_scope_report import strict_json, require

PREVIOUS = ROOT/'develop/migrations/go-practical/t01-w04/policy-api'


@lru_cache(None)
def _native_trials(): return (PREVIOUS/'api-trials.json').read_bytes()


def native_listing(id_):
    rows = strict_json(_native_trials()); row = next(r for r in rows if r['id'] == id_)
    return clone(row['result']['operations'][3]['value']['members'])


def model(row): return ApiModel(row, sha(('API-v4-private-model:' + row['base']['id']).encode()))


def example(shape, m):
    if type(shape) is dict:
        if 'enum' in shape: return shape['enum'][0]
        if 'nullable' in shape: return None
        if 'array' in shape: return []
        return {k: example(t, m) for k, t in shape.items()}
    return {'id32': 0, 'safe_uint': 0, 'bool': True, 'text': 'example',
            'name': 'Probe.Api', 'sha256': '0'*64, 'chunks': ['00'],
            'embedded_vir': m.b['module']}[shape]


def arguments(route, m):
    a = example(route['arguments'], m)
    if 'source_refs' in a: a['source_refs'] = clone(m.source_refs)
    if 'vc_ref' in a: a['vc_ref'] = clone(m.vc_ref)
    if route['path'] == '/module/new': a['proof_profile'] = 'mvp-strict'
    return a


def source_sequence(row):
    m = model(row); records = []
    paths = ['/module/new', '/vir/import', '/vc/generate', '/vc/list',
             '/term/sort', '/vc/start-proof', '/vc/attach-candidate',
             '/vc/check-candidate', '/module/export-certificate',
             '/module/freeze', '/vc/list', '/term/sort']
    for p in paths:
        route = next(r for r in ROUTES if r['path'] == p)
        r = m.request(p, arguments(route, m), route['method']); before = clone(m.session)
        raw = m.exchange(wire(r)); response = read_wire(raw)
        m.import_response(raw); m.import_session(wire(m.session))
        if response['result']['kind'] == 'error': require(before == m.session, 'all operation failures atomic')
        records.append({'request': r, 'response': response, 'session': clone(m.session),
                        'state_changed': before != m.session})
    return m, records


def request_cases(row, prepared):
    out = []
    for route in ROUTES:
        # /module/new starts with its own isolated empty authority.
        m = model(row) if route['path'] == '/module/new' else prepared
        r = m.request(route['path'], arguments(route, m), route['method'])
        variants = [('same', r, True)]
        for k in r:
            x = clone(r); del x[k]; variants.append(('missing-' + k, x, False))
        x = clone(r); x['accepted'] = True; variants.append(('extra-root', x, False))
        x = clone(r); x['arguments']['proof_evidence'] = True; variants.append(('extra-argument', x, False))
        x = clone(r); x['schema'] = 'mpk.ai.api.request.v3'; variants.append(('reserved-CSharp-v3', x, False))
        x = clone(r); x['api_profile'] = 'mpk.ai.api.v2'; variants.append(('released-v2', x, False))
        x = clone(r); x['semantic_context']['source_language'] = 'foreign'; variants.append(('cross-context', x, False))
        x = clone(r); x['selection']['selection_hash'] = '0'*64; variants.append(('cross-selection', x, False))
        x = clone(r); x['method'] = 'PUT'; variants.append(('wrong-method', x, False))
        if r['session_ref']:
            for k in ('revision', 'incarnation', 'canonical_sha256', 'session_id'):
                x = clone(r); x['session_ref'][k] = (r['session_ref'][k] + 1) if k == 'revision' else ('1'*64 if k != 'session_id' else 'other')
                variants.append(('cross-session-' + k, x, False))
        for k in r['arguments']:
            x = clone(r); del x['arguments'][k]; variants.append(('missing-argument-' + k, x, False))
            x = clone(r); x['arguments'][k] = False; variants.append(('wrong-argument-type-' + k, x, False))
        for id_, value, expected in variants:
            try: m.validate_request(wire(value)); accepted = True
            except ValueError: accepted = False
            require(accepted == expected, row['base']['id'] + route['path'] + ':' + id_)
            out.append({'id': row['base']['id'] + route['path'] + ':' + id_, 'expected': expected,
                        'classification': 'request_shape_valid' if accepted else 'configuration_rejection'})
    return out


def artifact_cases(row, m, records):
    result = []; response = records[3]['response']; session = records[-1]['session']
    for family, value in [('response', response), ('session', session)]:
        candidates = [('same', value, True)]
        for k in value:
            x = clone(value); del x[k]; candidates.append(('missing-' + k, x, False))
        for key in ('accepted', 'mpk_verified', 'proof_evidence'):
            x = clone(value); x[key] = True; candidates.append(('promote-' + key, x, False))
        x = clone(value); x['helper_only'] = False; candidates.append(('trusted', x, False))
        x = clone(value); x['semantic_context']['source_language'] = 'foreign'; candidates.append(('context', x, False))
        x = clone(value); x['schema'] = 'mpk.ai.api.' + family + '.v3'; candidates.append(('reserved-v3', x, False))
        if family == 'response':
            x = clone(value); x['request_ref']['canonical_sha256'] = '0'*64; candidates.append(('request-replay', x, False))
            x = clone(value); x['result']['value']['ordinary_status'] = 'ready' if m.go else 'unelaborated'; candidates.append(('ordinary-promotion', x, False))
        else:
            x = clone(value); x['revision'] += 1; candidates.append(('forged-current-snapshot', x, False))
            x = clone(value); x['source_state']['unelaborated_request_ids'] = []; candidates.append(('erase-pending', x, False if m.pending else True))
        for id_, candidate, expected in candidates:
            check = m.import_response if family == 'response' else m.import_session
            try: check(wire(candidate)); accepted = True
            except ValueError: accepted = False
            require(accepted == expected, family + ':' + row['base']['id'] + ':' + id_)
            result.append({'id': row['base']['id'] + '-' + family + '-' + id_, 'expected': expected,
                           'classification': 'locally_regenerated_helper' if accepted else 'configuration_rejection'})
    r = records[3]['request']; raw = wire(r)
    for id_, bad in [('missing-LF', raw[:-1]), ('two-LFs', raw + b'\n'), ('CRLF', raw[:-1] + b'\r\n'),
                     ('BOM', b'\xef\xbb\xbf' + raw), ('whitespace', b' ' + raw),
                     ('duplicate', raw[:-2] + b',"schema":"' + REQUEST.encode() + b'"}\n')]:
        try: read_wire(bad); accepted = True
        except ValueError: accepted = False
        require(not accepted, 'API canonical transport ' + id_)
        result.append({'id': row['base']['id'] + '-transport-' + id_, 'expected': False, 'classification': 'transport_rejection'})
    return result


def documents():
    bundles = {}; vectors = []
    for row in source_rows():
        m, sequence = source_sequence(row); bundles[row['base']['id']] = sequence
        vectors += request_cases(row, m) + artifact_cases(row, m, sequence)
    return bundles, vectors


def hash_cases(bundles):
    out = []
    for id_, sequence in bundles.items():
        for family, value in [('request', sequence[2]['request']), ('response', sequence[2]['response']),
                              ('session', sequence[2]['session'])]:
            raw = canonical(value)
            out.append({'id': id_ + '-' + family, 'raw_json_hex': raw.hex(), 'domain': None,
                        'accepted': True, 'canonical_hex': raw.hex(), 'canonical_sha256': sha(raw)})
    return out
