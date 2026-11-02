"""Inactive AI v4 redaction,batching and locally regenerated explanation roots.

No provider connection or proof decision exists in this design model.
"""
from collections import Counter
import json
import re
import unicodedata
from proof_canonical import canonical, parse, clone
from operation_signatures import exact, need
from identity_catalog import SHARED
from policy_sources import source_rows
from policy_roots import Policy
from run_policy_roots import RECORD as POLICY_RECORD, policy_from_outputs
from check_scope_report import strict_json
from run_proof_pilot import ROOT, sha

REQUEST = SHARED['ai_explain_request'][0]
EXPLANATION = SHARED['ai_explanation'][0]
REDACTION = 'mpk.evidence.redaction.v2'
PROJECTION = 'mpk.evidence.ai_projection.v2'
RESPONSE_SCHEMA = 'mpk.ai.explanation.response.v0'
STATES = ('mpk_verified', 'proof_pending', 'helper_only', 'unsupported')
KINDS = ('checked_declaration', 'checked_theory_certificate', 'helper_artifact', 'unsupported_feature')
HELPERS = ('source', 'contract', 'verification_ir', 'vc', 'ai_analysis', 'ci_status')
CATEGORIES = ('non_negative_result', 'result_bounded_by_input',
    'refund_bounded_by_available_paid_amount', 'fee_or_discount_bounded_by_cap',
    'selected_branch_result_equals_input', 'integer_runtime_safety',
    'source_binding', 'representation', 'operation', 'definedness', 'frame',
    'control', 'contract', 'call', 'totality', 'unrecognized')
GO_CATEGORIES = {'function_source_binding': 'source_binding', 'entry_representation': 'representation',
    'node_member': 'operation', 'contract_definedness': 'definedness', 'function_frame': 'frame',
    'region_control': 'control', 'partial_correctness': 'contract', 'postcondition': 'contract',
    'function_totality': 'totality', 'loop_initialization': 'control',
    'loop_preservation': 'control', 'loop_exit': 'control', 'call_precondition': 'call', 'call_relation': 'call'}
PUBLIC_PARAMETERS = {
    'go_fixed': ('pointer_width', 'target_id'),
    'rust_checked': ('overflow_mode', 'panic_mode', 'pointer_width', 'target_id'),
    'csharp_scalar': ('check_overflow_default', 'documentation_mode', 'language_version',
        'nullable_context', 'optimization', 'platform', 'pointer_width', 'source_kind',
        'target_framework', 'target_id', 'unsafe'),
    'java_scalar': ('annotation_processing', 'encoding', 'language_version', 'preview', 'release', 'target_id'),
    'go_practical': ('compiler_release', 'effective_languages', 'goamd64', 'goarch', 'goos', 'pointer_width', 'target_id')}
PROVIDER_METADATA = ('provider', 'project', 'location', 'requested_model', 'model_version',
    'response_id', 'create_time', 'finish_reason', 'attempts', 'prompt_tokens',
    'thinking_tokens', 'response_tokens', 'total_tokens')
LIMITS = {'properties_per_batch': 32, 'sanitized_request_bytes': 65536,
    'provider_request_bytes': 98304, 'provider_response_bytes': 1048576,
    'explanation_bytes': 2097152, 'json_depth': 128, 'retained_identifier_bytes': 4096,
    'overview_bytes': 2000, 'item_bytes': 500, 'list_items': 10, 'total_prose_bytes': 32768}
REQUEST_FIELDS = ('schema', 'language', 'display_language', 'projection_profile_id',
    'redaction_profile_id', 'source_language', 'semantic_profile', 'semantic_parameters',
    'policy', 'source_state', 'summary', 'trusted_evidence_summary', 'helper_artifact_summary', 'batch', 'properties')
EXPLANATION_FIELDS = ('schema', 'generator', 'trust', 'semantic_context', 'selection_reference',
    'policy_contract', 'ai_contract', 'source_evidence', 'source_receipt', 'requests',
    'source_state', 'local_summary', 'provider_responses', 'ai_analysis')
TRUST = {'classification': 'untrusted_helper_analysis', 'proof_evidence': False,
    'source_access': False, 'disclaimer': 'AI-generated explanation. Verification status is determined only by MPK evidence and checkers.'}


def text(s, maximum):
    need(type(s) is str and s.strip() and len(s.encode()) <= maximum and not any(
        (unicodedata.category(c) == 'Cc' and c != '\n') or ord(c) in (0x61c, 0x200e, 0x200f) or
        0x202a <= ord(c) <= 0x202e or 0x2066 <= ord(c) <= 0x2069 for c in s), 'bounded generated prose,no controls/bidi')


def bounded(raw, maximum):
    need(type(raw) is bytes and len(raw) <= maximum, 'AI document byte budget')
    value = parse(raw); stack = [(value, 0)]
    while stack:
        node, depth = stack.pop()
        if type(node) in (dict, list):
            need(depth < LIMITS['json_depth'], 'AI container depth')
            stack += [(v, depth+1) for v in (node.values() if type(node) is dict else node)]
    return value


def encode(value, maximum):
    raw = canonical(value); bounded(raw, maximum); return raw


def ref(value): return {'schema': value['schema'], 'canonical_sha256': sha(canonical(value))}


def old_prompt(name):
    source = (ROOT/'crates/mpk-cli/src/ai_explain.rs').read_text()
    block = source.split('const '+name+': &str = concat!(', 1)[1].split('\n);', 1)[0]
    return ''.join(json.loads(t) for t in re.findall(r'"(?:\\.|[^"\\])*"', block))


def provider_schema(aliases):
    item = {'type': 'string', 'minLength': 1, 'maxLength': LIMITS['item_bytes']}
    return {'type': 'object', 'properties': {
        'overview': {'type': 'string', 'minLength': 1, 'maxLength': LIMITS['overview_bytes']},
        'property_explanations': {'type': 'array', 'minItems': len(aliases), 'maxItems': len(aliases),
            'items': {'type': 'object', 'properties': {'property_ref': {'type': 'string', **({'enum': aliases} if aliases else {})},
                'explanation': item}, 'required': ['property_ref', 'explanation'], 'additionalProperties': False}},
        'limitations': {'type': 'array', 'maxItems': LIMITS['list_items'], 'items': item},
        'next_steps': {'type': 'array', 'maxItems': LIMITS['list_items'], 'items': item}},
        'required': ['overview', 'property_explanations', 'limitations', 'next_steps'], 'additionalProperties': False}


def provider_body(request):
    raw = encode(request, LIMITS['sanitized_request_bytes'])
    aliases = [p['ref'] for p in request['properties']]
    template = old_prompt('USER_TEMPLATE_V1'); need(template.count('{{SANITIZED_PAYLOAD_JSON}}') == 1, 'closed retained prompt placeholder')
    body = {'systemInstruction': {'parts': [{'text': old_prompt('SYSTEM_INSTRUCTION_V1')}]},
        'contents': [{'role': 'user', 'parts': [{'text': template.replace('{{SANITIZED_PAYLOAD_JSON}}', raw.decode())}]}],
        'generationConfig': {'candidateCount': 1, 'temperature': 0.0, 'maxOutputTokens': 8192,
            'responseFormat': [{'text': {'mimeType': 'APPLICATION_JSON', 'schema': provider_schema(aliases)}}],
            'thinkingConfig': {'thinkingLevel': 'MINIMAL', 'includeThoughts': False}}}
    raw = (json.dumps(body, ensure_ascii=False, indent=2) + '\n').encode()
    need(len(raw) <= LIMITS['provider_request_bytes'], 'full pretty provider request budget')
    return raw


class AiModel:
    def __init__(self, row, language='ja'):
        need(type(language) is str and language in ('en', 'ja'), 'registered explanation language')
        native = policy_from_outputs(strict_json((POLICY_RECORD/'native-queries.json').read_bytes()), strict_json((POLICY_RECORD/'native.json').read_bytes()))
        self.authority = Policy(row, native); self.row = clone(self.authority.row); self.b = self.row['base']
        self.docs = self.authority.build(); self.e = self.docs['evidence']; self.receipt = self.docs['receipt']
        for stage in ('scan', 'evidence', 'reproduction', 'receipt'): self.authority.validate(stage, self.docs[stage])
        self.language = language; self.go = self.authority.go; self.rows = self.properties()
        self.summary = {'total': len(self.rows), **{s: sum(p['status'] == s for p in self.rows) for s in STATES}}
        self.ai_contract = {'profile_entry_sha256': self.e['semantic_context']['profile_entry_sha256'],
            'contract_id': 'mpk.profile.ai.'+self.b['module']['program']['kind']+'.v2',
            'value': {'display_language': {'go': 'Go', 'rust': 'Rust', 'csharp': 'C#', 'java': 'Java'}[self.e['semantic_context']['source_language']],
                'projection_profile_id': PROJECTION, 'redaction_profile_id': REDACTION,
                'proof_authority': False, 'source_access': False}}
        if self.go: self.ai_contract['value']['foundation_descriptor'] = clone(self.e['semantic_context']['foundation_descriptor'])
        self.requests, self.aliases = self.prepare()

    def properties(self):
        if self.go:
            requests = [q for f in self.b['vc']['obligations']['functions'] for q in f['requests']]
            need(sorted(q['request_hash'] for q in requests) == self.e['proof']['unelaborated_request_ids'], 'every unelaborated demand exactly once')
            need(all(q['requirement']['kind'] in GO_CATEGORIES for q in requests), 'closed registered Go requirement categories')
            return [{'local_id': q['request_hash'], 'category': GO_CATEGORIES[q['requirement']['kind']],
                'status': 'proof_pending', 'evidence_kinds': [], 'source_index': i} for i, q in enumerate(requests)]
        rows = []
        for i, p in enumerate(self.e['properties']):
            text(p['id'], LIMITS['retained_identifier_bytes']); need('\n' not in p['id'] and p['status'] in STATES, 'source-owned identifier/status')
            kinds = {x['kind'] for m in p['members'] for x in m['evidence']}; need(kinds <= set(KINDS), 'closed evidence kinds')
            match = re.fullmatch(r'Payment policy obligation classified as ([a-z][a-z0-9_]{0,63})\.', p['description'])
            category = match[1] if match and match[1] in CATEGORIES[:6] else 'unrecognized'
            rows.append({'local_id': p['id'], 'category': category, 'status': p['status'],
                         'evidence_kinds': sorted(kinds, key=KINDS.index), 'source_index': i})
        need(len({r['local_id'] for r in rows}) == len(rows), 'unique retained property identity')
        return rows

    def prepare(self):
        rows = sorted(self.rows, key=lambda p: (STATES.index(p['status']), CATEGORIES.index(p['category']),
            sum(1 << KINDS.index(k) for k in p['evidence_kinds']), p['source_index']))
        batches = [rows[i:i+32] for i in range(0, len(rows), 32)] or [[]]
        c = self.e['semantic_context']; parameters = clone(c['semantic_parameters'])
        parameters['value'] = {k: parameters['value'][k] for k in PUBLIC_PARAMETERS[self.b['module']['program']['kind']]}
        policy = {k: self.e['policy_contract']['value'][k] for k in ('strategy_profile', 'checker_profile', 'axiom_profile')}
        checked = self.e['proof']['kind'] == 'checked_retained_ordinary_vc'
        trusted = {'rust_fast_kernel': 'accepted' if checked else 'not_run',
            'reference_checker': 'accepted' if checked else 'not_run',
            'certificate_candidates': 1 if checked else 0,
            'axiom_count': self.e['proof']['certificate']['axiom_count'] if checked else None}
        helpers = Counter(x['kind'] for x in self.e['helper_artifacts']); need(set(helpers) <= set(HELPERS), 'closed helper categories')
        requests, aliases = [], []; ordinal = 0
        for index, batch in enumerate(batches):
            local = [{**p, 'ref': f'property-{ordinal+i+1:04d}'} for i, p in enumerate(batch)]; ordinal += len(batch)
            request = {'schema': REQUEST, 'language': self.language, **{k: self.ai_contract['value'][k] for k in ('display_language', 'projection_profile_id', 'redaction_profile_id')},
                'source_language': c['source_language'], 'semantic_profile': c['semantic_profile'], 'semantic_parameters': parameters,
                'policy': policy, 'source_state': self.receipt['status'], 'summary': self.summary, 'trusted_evidence_summary': trusted,
                'helper_artifact_summary': [{'artifact': h, 'count': helpers[h]} for h in HELPERS if helpers[h]],
                'batch': {'index': index+1, 'count': len(batches)},
                'properties': [{k: p[k] for k in ('ref', 'category', 'status', 'evidence_kinds')} for p in local]}
            encode(request, LIMITS['sanitized_request_bytes']); provider_body(request)
            requests.append(clone(request)); aliases.append(clone(local))
        need(sum(len(x) for x in aliases) == len(self.rows), 'complete deterministic batching,never truncation')
        return requests, aliases

    def import_request(self, index, raw):
        v = bounded(raw, LIMITS['sanitized_request_bytes']); exact(v, REQUEST_FIELDS)
        need(raw == canonical(self.requests[index]), 'exact regenerated sanitized request without LF')

    def parse_response(self, index, raw):
        v = bounded(raw, LIMITS['provider_response_bytes'])
        exact(v, ('overview', 'property_explanations', 'limitations', 'next_steps'))
        text(v['overview'], LIMITS['overview_bytes']); total = len(v['overview'].encode())
        need(type(v['property_explanations']) is list and len(v['property_explanations']) == len(self.aliases[index]), 'exact batch response cardinality')
        allowed = {p['ref'] for p in self.aliases[index]}; seen = set()
        for p in v['property_explanations']:
            exact(p, ('property_ref', 'explanation')); need(type(p['property_ref']) is str and p['property_ref'] in allowed and p['property_ref'] not in seen, 'exact unique response alias')
            seen.add(p['property_ref']); text(p['explanation'], LIMITS['item_bytes']); total += len(p['explanation'].encode())
        for key in ('limitations', 'next_steps'):
            need(type(v[key]) is list and len(v[key]) <= LIMITS['list_items'], 'bounded generated list')
            for item in v[key]: text(item, LIMITS['item_bytes']); total += len(item.encode())
        need(total <= LIMITS['total_prose_bytes'], 'total generated prose budget')
        return v

    def explain(self, responses):
        need(type(responses) is list and len(responses) == len(self.requests), 'every batch response required')
        analyses = []; observations = []; properties = []
        for index, raw in enumerate(responses):
            parsed = self.parse_response(index, raw); analyses.append({k: parsed[k] for k in ('overview', 'limitations', 'next_steps')})
            generated = {p['property_ref']: p['explanation'] for p in parsed['property_explanations']}
            for p in self.aliases[index]: properties.append({'source_index': p['source_index'], 'property_id': p['local_id'],
                'source_status': p['status'], 'explanation': generated[p['ref']]})
            observations.append({'batch_index': index+1, 'origin': 'offline_fixture',
                'provider_request_sha256': sha(provider_body(self.requests[index])),
                'provider_response_sha256': sha(raw), 'metadata': None})
        properties.sort(key=lambda p: p['source_index'])
        import tomllib
        version = tomllib.loads((ROOT/'Cargo.toml').read_text())['workspace']['package']['version']
        report = {'schema': EXPLANATION, 'generator': {'name': 'mpk', 'version': version}, 'trust': clone(TRUST),
            'semantic_context': clone(self.e['semantic_context']), 'selection_reference': ref(self.e['selection']),
            'policy_contract': clone(self.e['policy_contract']), 'ai_contract': clone(self.ai_contract),
            'source_evidence': ref(self.e), 'source_receipt': ref(self.receipt), 'requests': [ref(r) for r in self.requests],
            'source_state': self.receipt['status'], 'local_summary': clone(self.summary),
            'provider_responses': observations, 'ai_analysis': {'batches': analyses,
                'property_explanations': [{k: p[k] for k in ('property_id', 'source_status', 'explanation')} for p in properties]}}
        encode(report, LIMITS['explanation_bytes']); return report

    def import_explanation(self, raw, responses):
        value = bounded(raw, LIMITS['explanation_bytes']); exact(value, EXPLANATION_FIELDS)
        need(raw == canonical(self.explain(responses)), 'complete locally regenerated explanation,not provider status/hash authority')


def fixture_responses(model):
    return [canonical({'overview': 'MPKが提供した検証状態の説明です。',
        'property_explanations': [{'property_ref': p['ref'], 'explanation': 'この文章は説明用です。検証状態はMPKの証跡から決まります。'} for p in aliases],
        'limitations': ['AIの説明は証明にはなりません。'], 'next_steps': ['未完了の証明義務を確認してください。']}) for aliases in model.aliases]


def catalogue():
    return {'schema': 'mpk.go_practical.ai_contract_draft.v0', 'roots': {REQUEST: list(REQUEST_FIELDS), EXPLANATION: list(EXPLANATION_FIELDS)},
        'hash_rule': 'raw SHA256 of whole canonical root without LF; no own hash or invented AI domain',
        'new_recipe_ids': [PROJECTION, REDACTION], 'retained_provider_response_schema': RESPONSE_SCHEMA,
        'limits': LIMITS, 'states': list(STATES), 'evidence_kinds': list(KINDS), 'helpers': list(HELPERS),
        'categories': list(CATEGORIES), 'Go_category_map': GO_CATEGORIES, 'public_parameter_allowlists': {k: list(v) for k, v in PUBLIC_PARAMETERS.items()},
        'batching': 'stable status/category/evidence-kind/source-index ordering,at most32 properties per request; one zero-property diagnostic batch when empty; complete coverage or explicit size rejection,never truncation',
        'provider_response': {'fields': ['overview', 'property_explanations', 'limitations', 'next_steps'],
            'property': ['property_ref', 'explanation'], 'unknown_fields': 'reject; no provider-supplied status/context/hash/proof fields'},
        'nested_fields': {'generator': ['name', 'version'], 'trust': list(TRUST),
            'summary': ['total', *STATES], 'property': ['ref', 'category', 'status', 'evidence_kinds'],
            'trusted_evidence_summary': ['rust_fast_kernel', 'reference_checker', 'certificate_candidates', 'axiom_count'],
            'helper_summary': ['artifact', 'count'], 'batch': ['index', 'count'],
            'reference': ['schema', 'canonical_sha256'], 'analysis': ['batches', 'property_explanations'],
            'batch_analysis': ['overview', 'limitations', 'next_steps'],
            'local_property_explanation': ['property_id', 'source_status', 'explanation']},
        'local_provider_observation': {'required_fields': ['batch_index', 'origin', 'provider_request_sha256', 'provider_response_sha256', 'metadata'],
            'arms': {'offline_fixture': 'metadata=null; this model only', 'provider_observed': 'metadata=closed provider metadata object,from the independently observed local transport adapter,never provider prose'},
            'metadata_fields': list(PROVIDER_METADATA),
            'metadata_contract': 'exact existing SuccessorAiReportRequest plus SuccessorAiProviderProvenance fields,provider=vertex-ai; reuse existing model/project/location/token68/create_time/STOP/attempt1..3/nullable-token<=10000000 validators; request metadata is independently bound to the actual local call'},
        'authority': 'full local context/selection/evidence/receipt and locally retained alias map; source-private identities/hashes/text never enter provider payload; all AI prose remains untrusted',
        'materialized_Go_rule': 'after independently checked ordinary target binding exists,use exact validated policy properties/status/evidence kinds and the same complete batching algorithm as retained profiles; this model has only unelaborated Go authorities and cannot manufacture that state; remaining Go policy materialization closes with the ordinary-program/migration design batch',
        'owners': {'production': 'crates/mpk-cli/src/go_practical_ai_explain.rs (T07-W01)',
            'test': 'crates/mpk-cli/tests/go_practical_ai_explain.rs (T07-W01)',
            'retained': ['crates/mpk-cli/tests/csharp_ai_explain.rs', 'crates/mpk-cli/tests/java_policy_verify.rs']},
        'remaining': ['release roots,ordinary-program signatures and exhaustive reviewed consumer migration closure for W04',
            'compiled v4 batching/registered profile/real-provider adapter integration in T07; full T02–T11 proof and release work']}
