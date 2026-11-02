"""Rebind Go/Rust fixture metadata while preserving intentionally invalid hashes.

The immutable basis is read with /usr/bin/git outside the sandbox beforehand.
This helper does not execute Git or claim frontend/program-proof acceptance.
"""
from pathlib import Path
import base64, hashlib, json

ROOT = Path.cwd()
ORIGINALS = Path('/private/tmp/mpk-go-rust-scope/fixture-originals')
RECORD = ROOT / 'develop/migrations/go-rust-only/verification/fixture-rebinding'
RECORD.mkdir(parents=True, exist_ok=True)
canonical = lambda value: json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(',', ':')).encode()
DOMAINS = {
    'mpk.vir.v1': ('vir_hash', 'MPK-VIR-1.0'),
    'mpk.source_map.v1': ('source_map_hash', 'MPK-SOURCE-MAP-1.0'),
    'mpk.source_manifest.v1': ('source_manifest_hash', 'MPK-SOURCE-MANIFEST-1.0'),
    'mpk.vc.v2': ('vc_hash', 'MPK-VC-2.0'),
}


def hash_kind(node):
    if 'contract_hash' in node and 'semantic_context' in node:
        return 'contract_hash', 'MPK-CONTRACT-1.0'
    return DOMAINS.get(node.get('schema'))


def derived(node, field, domain):
    return hashlib.sha256(domain.encode() + b'\0' + canonical({k: v for k, v in node.items() if k != field})).hexdigest()


def locate(node, path):
    for key in path:
        node = node[key]
    return node


def hash_nodes(node, path=()):
    if isinstance(node, dict):
        kind = hash_kind(node)
        if kind:
            yield path, kind, node
        for key, value in node.items():
            yield from hash_nodes(value, (*path, key))
    elif isinstance(node, list):
        for index, value in enumerate(node):
            yield from hash_nodes(value, (*path, index))


def replace(node, changes):
    if isinstance(node, dict):
        return {k: replace(v, changes) for k, v in node.items()}
    if isinstance(node, list):
        return [replace(v, changes) for v in node]
    if isinstance(node, str):
        seen = set()
        while node in changes and node not in seen:
            seen.add(node)
            node = changes[node]
    return node


def main():
    documents = {}
    masks = {}
    invalid = []
    preimages = []
    for p in sorted(ORIGINALS.rglob('*.json')):
        relative = p.relative_to(ORIGINALS)
        current_path = ROOT / relative
        if not current_path.is_file():
            continue
        try:
            original = json.loads(p.read_bytes())
            current = json.loads(current_path.read_bytes())
        except (ValueError, UnicodeError):
            continue
        nodes = list(hash_nodes(original))
        if not nodes:
            continue
        documents[str(relative)] = current
        masks[str(relative)] = []
        for path, (field, domain), node in nodes:
            if node.get(field) == derived(node, field, domain):
                masks[str(relative)].append((path, field, domain))
            else:
                invalid.append({'path': str(relative), 'node': path, 'field': field, 'original_value': node.get(field)})
        raw = current_path.read_bytes()
        preimages.append({'path': str(relative), 'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest(), 'base64': base64.b64encode(raw).decode()})
    before = RECORD / 'before.bytes.json'
    assert not before.exists(), 'retain original rebinding epoch'
    before.write_text(json.dumps({'schema': 'mpk.original_bytes_set.v1', 'files': preimages}, indent=2) + '\n')
    changes = {'62dda4ba6589347eea5e4d59b035b41988fc6cab65088c1039e79ecbc92e8bff': 'ccc4a984d41b5da7b3a6838cbfaeee3eb23f675a2e4d7fb5d4f9fcd7e597d538'}
    rounds = []
    for step in range(20):
        count = 0
        for path in documents:
            documents[path] = replace(documents[path], changes)
            for location, field, domain in reversed(masks[path]):
                node = locate(documents[path], location)
                new_hash = derived(node, field, domain)
                old_hash = node[field]
                if new_hash != old_hash:
                    changes[old_hash] = new_hash
                    node[field] = new_hash
                    count += 1
        rounds.append(count)
        if count == 0:
            break
    else:
        raise AssertionError('artifact hash DAG failed to converge')
    for item in invalid:
        actual = locate(documents[item['path']], item['node'])
        # A deliberately wrong link may be transported to the same wrong link
        # in the new hash epoch. It must remain wrong after transport.
        kind = hash_kind(actual)
        assert actual.get(item['field']) != derived(actual, *kind), item
    rows = []
    for path, document in documents.items():
        for location, field, domain in masks[path]:
            node = locate(document, location)
            assert node[field] == derived(node, field, domain)
        p = ROOT / path
        data = canonical(document) + b'\n'
        if p.read_bytes() != data:
            p.write_bytes(data)
            rows.append({'path': path, 'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest()})
    (RECORD / 'result.json').write_text(json.dumps({'status': 'passed', 'basis': '528b599aacfe439d4abd8f50e046a754c9a24435', 'changed': rows, 'valid_hash_nodes': sum(map(len, masks.values())), 'intentionally_invalid_hash_nodes_preserved': invalid, 'hash_DAG_rounds': rounds, 'frontend_execution_claimed': False, 'program_proof_claimed': False}, indent=2) + '\n')
    print('Rebound fixtures', len(rows), 'valid hash nodes', sum(map(len, masks.values())), 'preserved invalid nodes', len(invalid))


if __name__ == '__main__':
    main()
