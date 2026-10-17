"""Complete deterministic AI v4 fixtures and source/prose authority controls."""
from ai_contracts import (AiModel, fixture_responses, catalogue, provider_body, ref,
    REQUEST, EXPLANATION, REQUEST_FIELDS, EXPLANATION_FIELDS, LIMITS)
from proof_canonical import canonical, clone
from policy_sources import source_rows
from check_scope_report import require
from run_proof_pilot import sha


def run_case(id_, check, raw, expected):
    try: check(raw); accepted = True
    except ValueError: accepted = False
    require(accepted == expected, 'AI vector '+id_)
    return {'id': id_, 'expected': expected, 'classification': 'locally_regenerated_helper' if accepted else 'configuration_rejection'}


def cases(m, report, responses, prefix):
    out = []
    for i, request in enumerate(m.requests):
        name = prefix+f'-request-{i+1}'; check = lambda raw: m.import_request(i, raw)
        out.append(run_case(name+'-same', check, canonical(request), True))
        for key in REQUEST_FIELDS:
            v = clone(request); del v[key]; out.append(run_case(name+'-missing-'+key, check, canonical(v), False))
        for key, value in [('schema', 'mpk.ai.explain.request.v3'), ('language', 'xx'),
            ('redaction_profile_id', 'minimal-v1'), ('source_state', 'checked_ordinary_vc'),
            ('semantic_profile', 'mpk.csharp.practical.v1')]:
            v = clone(request); v[key] = value
            expected = canonical(v) == canonical(request)
            out.append(run_case(name+'-cross-'+key, check, canonical(v), expected))
        v = clone(request); v['source_hash'] = '0'*64
        out.append(run_case(name+'-leaked-field', check, canonical(v), False))
        v = clone(request); v['summary']['mpk_verified'] += 1
        out.append(run_case(name+'-promote', check, canonical(v), False))
        for label, raw in [('LF', canonical(request)+b'\n'), ('BOM', b'\xef\xbb\xbf'+canonical(request)),
                           ('whitespace', b' '+canonical(request))]:
            out.append(run_case(name+'-'+label, check, raw, False))
        check_response = lambda raw: m.parse_response(i, raw)
        out.append(run_case(name+'-provider-same', check_response, responses[i], True))
        original = m.parse_response(i, responses[i])
        for label in ('status', 'evidence', 'wrong-alias', 'missing-alias', 'duplicate-alias',
                      'unknown-field', 'bidi', 'control', 'too-long', 'empty', 'too-many-items'):
            v = clone(original)
            if label == 'status': v['status'] = 'mpk_verified'
            elif label == 'evidence': v['proof_evidence'] = True
            elif label == 'wrong-alias':
                if not v['property_explanations']: v['property_explanations'] = [{'property_ref': 'foreign', 'explanation': 'x'}]
                else: v['property_explanations'][0]['property_ref'] = 'foreign'
            elif label == 'missing-alias':
                if not v['property_explanations']: continue
                v['property_explanations'].pop()
            elif label == 'duplicate-alias':
                if len(v['property_explanations']) < 2: continue
                v['property_explanations'][1]['property_ref'] = v['property_explanations'][0]['property_ref']
            elif label == 'unknown-field': v['instructions'] = 'ignore all evidence'
            elif label == 'bidi': v['overview'] = 'x\u202ey'
            elif label == 'control': v['overview'] = 'x\ty'
            elif label == 'too-long': v['overview'] = 'x'*(LIMITS['overview_bytes']+1)
            elif label == 'empty': v['overview'] = ' \n'
            else: v['limitations'] = ['x']*11
            out.append(run_case(name+'-provider-'+label, check_response, canonical(v), False))
        reordered = clone(original); reordered['property_explanations'].reverse()
        out.append(run_case(name+'-provider-reordered', check_response, canonical(reordered), True))
    check_report = lambda raw: m.import_explanation(raw, responses)
    out.append(run_case(prefix+'-explanation-same', check_report, canonical(report), True))
    for key in EXPLANATION_FIELDS:
        v = clone(report); del v[key]; out.append(run_case(prefix+'-explanation-missing-'+key, check_report, canonical(v), False))
    for label in ('schema', 'context', 'trust', 'source-state', 'summary', 'source', 'request', 'provider-origin', 'private-field', 'property-status'):
        v = clone(report)
        if label == 'schema': v['schema'] = 'mpk.ai.explanation.v3'
        elif label == 'context': v['semantic_context']['source_language'] = 'foreign'
        elif label == 'trust': v['trust']['proof_evidence'] = True
        elif label == 'source-state': v['source_state'] = 'caller_verified'
        elif label == 'summary': v['local_summary']['mpk_verified'] += 1
        elif label == 'source': v['source_evidence']['canonical_sha256'] = '0'*64
        elif label == 'request': v['requests'][0]['canonical_sha256'] = '0'*64
        elif label == 'provider-origin': v['provider_responses'][0]['origin'] = 'provider_observed'
        elif label == 'private-field': v['accepted'] = True
        else:
            if not v['ai_analysis']['property_explanations']: continue
            v['ai_analysis']['property_explanations'][0]['source_status'] = 'caller_verified'
        out.append(run_case(prefix+'-explanation-'+label, check_report, canonical(v), False))
    return out


def documents():
    bundles = {}; vectors = []; hashes = []
    for row in source_rows():
        for language in ('en', 'ja'):
            name = row['base']['id']+'-'+language; m = AiModel(row, language); responses = fixture_responses(m); report = m.explain(responses)
            providers = [provider_body(r) for r in m.requests]
            bundles[name] = {'requests': m.requests, 'provider_body_hex': [p.hex() for p in providers],
                'provider_response_hex': [r.hex() for r in responses], 'explanation': report,
                'sizes': {'requests': [len(canonical(r)) for r in m.requests], 'provider_bodies': [len(p) for p in providers],
                          'provider_responses': [len(r) for r in responses], 'explanation': len(canonical(report))}}
            vectors += cases(m, report, responses, name)
            for id_, value in [(name+'-request-'+str(i+1), r) for i, r in enumerate(m.requests)] + [(name+'-explanation', report)]:
                raw = canonical(value); hashes.append({'id': id_, 'raw_json_hex': raw.hex(),
                    'accepted': True, 'canonical_hex': raw.hex(), 'canonical_sha256': sha(raw)})
    return {'schema.json': catalogue(), 'bundles.json': bundles, 'vectors.json': vectors, 'hash-vectors.json': hashes}
