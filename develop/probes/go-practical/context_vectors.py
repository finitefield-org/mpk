"""Closed protocol corpus; positive data is not a copied legacy validator."""
import copy
from context_protocol import GO_PROFILE, registration_fixture, raw_context, raw_request, legacy_projection, active_registry
from identity_catalog import SHARED, GO
from identity_wire import canonical, digest

def contexts():
    values = {'go_fixed': ('mpk.go.fixed.v0', {'target_id': 'linux/amd64', 'pointer_width': 64}), 'rust64': ('mpk.rust.checked.v0', {'target_id': 'x86_64-unknown-linux-gnu', 'pointer_width': 64, 'overflow_mode': 'checked', 'panic_mode': 'abort'}), 'rust32': ('mpk.rust.checked.v0', {'target_id': 'i686-unknown-linux-gnu', 'pointer_width': 32, 'overflow_mode': 'checked', 'panic_mode': 'abort'}), 'go_practical': (GO_PROFILE, registration_fixture()['go_parameters'])}
    return {name: raw_context(profile, value) for name, (profile, value) in values.items()}

def go_root(function='Quote', receiver=None):
    return {'package': 'example.test/quote', 'function': function, 'receiver': receiver}

def go_selection(roots=None):
    return {'schema': GO['selection'][0], 'value': {'roots': [go_root()] if roots is None else roots}}

def selections():
    return {'go_fixed': {'schema': 'mpk.selection.go_function.v0', 'value': {'package': 'quote', 'function': 'quote.Quote'}}, 'rust64': {'schema': 'mpk.selection.rust_function.v0', 'value': {'package': 'quote', 'crate': 'quote', 'kind': 'lib', 'function': 'quote::quote'}}, 'rust32': {'schema': 'mpk.selection.rust_function.v0', 'value': {'package': 'quote', 'crate': 'quote', 'kind': 'bin', 'function': 'quote::quote'}}, 'go_practical': go_selection()}

def assign(value, path, replacement):
    out = copy.deepcopy(value)
    node = out
    for key in path[:-1]:
        node = node[key]
    node[path[-1]] = copy.deepcopy(replacement)
    return out

def remove(value, path):
    out = copy.deepcopy(value)
    node = out
    for key in path[:-1]:
        node = node[key]
    del node[path[-1]]
    return out

def rehash_registry(registry):
    out = copy.deepcopy(registry)
    for entry in out['profiles']:
        entry['entry_sha256'] = digest(SHARED['registry_entry'][1], {k: v for k, v in entry.items() if k != 'entry_sha256'})
    out['registry_sha256'] = digest(SHARED['registry'][1], {k: v for k, v in out.items() if k != 'registry_sha256'})
    return out

def cases():
    rows = []
    cs = contexts()
    ss = selections()
    fixture = registration_fixture()
    registry = fixture['registry']
    go = cs['go_practical']

    def add(kind, name, value, accepted=False):
        rows.append({'id': name, 'kind': kind, 'value': value, 'expected_accepted': accepted})
    add('registry', 'registry-compiled', registry, True)
    for name, c in cs.items():
        add('context', 'context-' + name, c, True)
        add('request', 'request-' + name, raw_request(c, ss[name]), True)
        add('linkage', 'linkage-' + name, [c, copy.deepcopy(c)], True)
    receiver = {'package': 'example.test/quote', 'type_name': 'Ledger'}
    for name, roots in (('method', [go_root(receiver=receiver)]), ('receiver-alias-syntax', [go_root(receiver={**receiver, 'type_name': 'LedgerAlias'})]), ('method-init', [go_root('init', receiver)]), ('unicode-name', [go_root('見積')]), ('ordered-multiple', sorted([go_root('A'), go_root('B', receiver)], key=canonical))):
        add('request', 'request-go-' + name, raw_request(go, go_selection(roots)), True)
    context_changes = [('old-schema', ['schema'], 'mpk.semantic_context.v1'), ('reserved-csharp-schema', ['schema'], 'mpk.semantic_context.v2'), ('future-schema', ['schema'], 'mpk.semantic_context.v4'), ('registry-family', ['profile_registry', 'schema'], 'mpk.semantic_profile.registry.v1'), ('registry-id', ['profile_registry', 'id'], 'mpk.semantic_profile.registry.v2'), ('registry-revision', ['profile_registry', 'revision'], 2), ('registry-bool', ['profile_registry', 'revision'], True), ('registry-hash', ['profile_registry', 'registry_sha256'], '0' * 64), ('entry-fixed', ['profile_entry_sha256'], cs['go_fixed']['profile_entry_sha256']), ('entry-unknown', ['profile_entry_sha256'], '0' * 64), ('language-cross', ['source_language'], 'rust'), ('profile-fixed', ['semantic_profile'], 'mpk.go.fixed.v0'), ('profile-unknown', ['semantic_profile'], 'mpk.go.practical.v2'), ('params-fixed-schema', ['semantic_parameters', 'schema'], 'mpk.semantic_parameters.go_fixed.v0'), ('foundation-null', ['foundation_descriptor'], None), ('foundation-csharp', ['foundation_descriptor', 'id'], 'mpk.csharp.practical.foundation.v1'), ('foundation-schema', ['foundation_descriptor', 'schema'], 'mpk.csharp.foundation_descriptor.v1'), ('foundation-hash', ['foundation_descriptor', 'content_sha256'], '0' * 64), ('target', ['semantic_parameters', 'value', 'target_id'], 'darwin/arm64'), ('width', ['semantic_parameters', 'value', 'pointer_width'], 32), ('width-bool', ['semantic_parameters', 'value', 'pointer_width'], True), ('goos', ['semantic_parameters', 'value', 'goos'], 'darwin'), ('goarch', ['semantic_parameters', 'value', 'goarch'], 'arm64'), ('goamd64', ['semantic_parameters', 'value', 'goamd64'], 'v3'), ('compiler', ['semantic_parameters', 'value', 'compiler_release'], 'go1.25.1'), ('language-order', ['semantic_parameters', 'value', 'effective_languages'], ['go1.25', 'go1.24', 'go1.23', 'go1.22']), ('language-future', ['semantic_parameters', 'value', 'effective_languages'], ['go1.22', 'go1.23', 'go1.24', 'go1.25', 'go1.26']), ('limits-id', ['semantic_parameters', 'value', 'limits_id'], 'mpk.go.practical.limits.v2'), ('limits-hash', ['semantic_parameters', 'value', 'limits_sha256'], '0' * 64)]
    for name, path, value in context_changes:
        add('context', 'context-reject-' + name, assign(go, path, value))
    for field in go:
        add('context', 'context-missing-' + field, remove(go, [field]))
    for path in (['profile_registry', 'revision'], ['foundation_descriptor', 'schema'], ['semantic_parameters', 'value', 'limits_sha256']):
        add('context', 'context-nested-missing-' + path[-1], remove(go, path))
    for name, path, value in (('root', ['feature_flags'], []), ('registry', ['profile_registry', 'caller_authorized'], True), ('foundation', ['foundation_descriptor', 'definitions'], []), ('envelope', ['semantic_parameters', 'feature_flags'], []), ('numeric-override', ['semantic_parameters', 'value', 'limits'], {'syntax_nodes': 999999999})):
        add('context', 'context-extra-' + name, assign(go, path, value))
    for name, c in cs.items():
        if name == 'go_practical':
            continue
        add('context', 'legacy-foundation-' + name, assign(c, ['foundation_descriptor'], go['foundation_descriptor']))
        add('context', 'legacy-extra-params-' + name, assign(c, ['semantic_parameters', 'value', 'features'], []))
    for name, path, value in (('go_fixed', ['pointer_width'], 32), ('rust64', ['target_id'], 'i686-unknown-linux-gnu'), ('rust32', ['overflow_mode'], 'wrapping')):
        add('context', 'legacy-params-' + name + '-' + path[-1], assign(cs[name], ['semantic_parameters', 'value'] + path, value))
    bad_roots = [('empty', []), ('duplicate', [go_root(), go_root()]), ('reverse', [go_root('B'), go_root('A')]), ('keyword', [go_root('return')]), ('blank', [go_root('_')]), ('package-init', [go_root('init')]), ('pointer-receiver-name', [go_root(receiver={**receiver, 'type_name': '*Ledger'})]), ('foreign-receiver', [go_root(receiver={**receiver, 'package': 'example.test/other'})]), ('receiver-extra', [go_root(receiver={**receiver, 'pointer': True})]), ('receiver-missing', [go_root(receiver={'type_name': 'Ledger'})]), ('root-extra', [{**go_root(), 'call_body': True}]), ('root-missing', [{k: v for k, v in go_root().items() if k != 'receiver'}]), ('root-null', [None]), ('function-bool', [go_root(True)]), ('package-empty', [{**go_root(), 'package': ''}]), ('package-host-path', [{**go_root(), 'package': '/tmp/quote'}]), ('package-backslash', [{**go_root(), 'package': 'quote\\sub'}]), ('package-nul', [{**go_root(), 'package': 'quote\x00sub'}])]
    for name, roots in bad_roots:
        add('request', 'selection-reject-' + name, raw_request(go, go_selection(roots)))
    for name, selection in (('fixed-schema', ss['go_fixed']), ('future-schema', {**go_selection(), 'schema': 'mpk.selection.go_functions.v2'}), ('roots-null', {'schema': GO['selection'][0], 'value': {'roots': None}}), ('value-extra', {'schema': GO['selection'][0], 'value': {'roots': [go_root()], 'allow_alias_duplicates': True}}), ('envelope-extra', {**go_selection(), 'validated': True})):
        add('request', 'selection-reject-' + name, raw_request(go, selection))
    for name, path, value in (('go_fixed', ['function'], 'foreign.Quote'), ('rust64', ['kind'], 'test'), ('rust32', ['function'], 'other::quote')):
        add('request', 'legacy-selection-' + name + '-' + path[-1] + '-' + str(len(rows)), raw_request(cs[name], assign(ss[name], ['value'] + path, value)))
    good = raw_request(go, ss['go_practical'])
    add('request', 'request-stale-hash', assign(good, ['selection', 'value', 'roots', 0, 'function'], 'Changed'))
    add('request', 'request-context-stale-hash', assign(good, ['semantic_context', 'semantic_parameters', 'value', 'goamd64'], 'v2'))
    add('request', 'request-incomplete-preimage', {**good, 'request_sha256': digest(SHARED['validated_request'][1], {'schema': good['schema'], 'selection': good['selection']})})
    add('request', 'request-extra', {**good, 'caller_validated': True})
    for field in good:
        add('request', 'request-missing-' + field, remove(good, [field]))
    add('request', 'request-old-schema', assign(good, ['schema'], 'mpk.validated_semantic_request.v1'))
    for name, reg in (('old', active_registry()), ('future', assign(registry, ['schema'], 'mpk.semantic_profile.registry.v4')), ('bool-revision', assign(registry, ['revision'], True)), ('extra', {**registry, 'caller_authorized': True}), ('entry-extra', assign(registry, ['profiles', 0, 'new_contract'], True)), ('stale-entry', assign(registry, ['profiles', 0, 'contracts', 'vir'], 'mpk.profile.vir.unknown.v2')), ('rehash-contract', rehash_registry(assign(registry, ['profiles', 0, 'contracts', 'vir'], 'mpk.profile.vir.unknown.v2'))), ('rehash-profile', rehash_registry(assign(registry, ['profiles', 0, 'semantic_profile'], 'mpk.csharp.practical.v1'))), ('rehash-foundation', rehash_registry(assign(registry, ['profiles', 0, 'foundation_descriptor'], go['foundation_descriptor']))), ('rehash-order', rehash_registry({**registry, 'profiles': list(reversed(registry['profiles']))})), ('rehash-missing-profile', rehash_registry({**registry, 'profiles': registry['profiles'][:-1]})), ('rehash-duplicate-profile', rehash_registry({**registry, 'profiles': registry['profiles'] + [registry['profiles'][0]]}))):
        add('registry', 'registry-reject-' + name, reg)
    add('linkage', 'linkage-distinct-valid-rust-targets', [cs['rust32'], cs['rust64']])
    add('linkage', 'linkage-fixed-practical', [cs['go_fixed'], go])
    for field in go:
        replacement = cs['go_fixed'].get(field)
        if replacement == go[field]:
            replacement = None
        add('linkage', 'linkage-cross-' + field, [go, assign(go, [field], replacement)])
    for name, raw in (('duplicate', b'{"schema":1,"schema":2}'), ('nested-duplicate', b'{"semantic_parameters":{"value":1,"value":2}}'), ('float', b'{"revision":1.0}'), ('unsafe-integer', b'{"revision":9007199254740992}'), ('surrogate', b'{"schema":"\\ud800"}'), ('invalid-utf8', b'{"schema":"\xff"}')):
        add('wire', 'wire-reject-' + name, raw.hex())
    assert len({r['id'] for r in rows}) == len(rows)
    return rows

def legacy_queries(rows=None):
    """Collect projections before validation so a missing oracle call is fatal."""
    by_key = {}
    for row in cases() if rows is None else rows:
        value = row['value']
        kind = row['kind']
        pairs = []
        if kind == 'context':
            pairs = [(value, None)]
        elif kind == 'request' and type(value) is dict and ('semantic_context' in value):
            pairs = [(value['semantic_context'], None), (value['semantic_context'], value.get('selection'))]
        elif kind == 'linkage':
            pairs = [(v, None) for v in value]
        for context, selection in pairs:
            if type(context) is not dict or context.get('semantic_profile') not in {e['semantic_profile'] for e in active_registry()['profiles']}:
                continue
            query = legacy_projection(context, selection)
            by_key[canonical(query)] = query
    return [by_key[k] for k in sorted(by_key)]
