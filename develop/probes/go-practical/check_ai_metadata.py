#!/usr/bin/env python3
"""Check current AI design,immediate API predecessor,and all historical owners."""
import re
from urllib.parse import unquote
from run_ai_contracts import ROOT, RECORD, check_retained
from run_api_contracts import check_retained as previous
from check_scope_report import strict_json, require, sha


def main():
    w = RECORD.parent; r = check_retained(); old = previous()
    require(old['cases'] == 6800 and len(old['inputs']) == 847, 'complete preceding API design retained')
    work = strict_json((w/'freeze-worklist.json').read_bytes()); d = work['ai_contract_draft']
    require(d['cases'] == r['cases'] and d['source_inputs'] == len(r['inputs']) and d['targeted_tests'] == 16, 'AI worklist/test/input closure')
    require(work['status'] == 'in progress; not frozen or published' and work['open_decisions'], 'remaining W04/T01/T02–T11 objective active')
    inv = strict_json((w/'surface-inventory.json').read_bytes()); require(len(inv['discovered_files']) == 143, 'all predecessor consumer paths')
    for x in inv['discovered_files'] + inv['baseline'] + [x for o in inv['component_owners'] for x in o['existing_anchors']]: require(sha((ROOT/x['path']).read_bytes()) == x['sha256'], 'unchanged inventoried source')
    files = sorted(p for p in w.rglob('*.json') if 'verification' not in p.relative_to(w).parts)
    for p in files: strict_json(p.read_bytes())
    docs = [w/'README.md', RECORD/'README.md', ROOT/'develop/docs/09_go_practical_subset_design-todo.md', ROOT/'develop/docs/go-practical-implementation-traceability-ledger.md'] + sorted(p for p in w.glob('*/README.md') if p != RECORD/'README.md') + [w/'foundation-limits/semantic-rules.md', w/'type-error-zero-decisions.md', w/'operation-loop-decisions.md']; links = 0
    for p in docs:
        for target in re.findall(r'\[[^\]\n]*\]\(([^)\n]+)\)', p.read_text()):
            target = target.strip('<>')
            if re.match(r'^[a-z]+://', target) or target.startswith('#'): continue
            require((p.parent/unquote(target.split('#', 1)[0])).is_file(), 'document link: '+target); links += 1
    require(list(map(int, re.findall(r'^## (\d+)\.', docs[3].read_text(), re.M))) == list(range(1, 57)), 'ordered ledger1–56')
    print(f"PASS:{len(files)} strict JSONs,{len(r['inputs'])} source pins,{r['cases']} AI cases/40 hashes,143 predecessors,{len(docs)} documents/{links} links; W04 open")


if __name__ == '__main__': main()
