#!/usr/bin/env python3
"""Retain the complete arithmetic predecessor and current frontend draft scope."""
import re
from urllib.parse import unquote
from run_frontend_wire import ROOT,RECORD,check_retained
from run_word_arithmetic import check_retained as previous
from check_scope_report import require,strict_json,sha


def main():
    w=RECORD.parent;r=check_retained();old=previous();require(len(old['inputs'])==729 and old['outcomes']=={'accepted_zero_axiom':4,'ordinary_type_mismatch':16},'preceding full word addition evidence retained')
    work=strict_json((w/'freeze-worklist.json').read_bytes());d=work['frontend_envelope_draft'];require(d['cases']==r['cases'] and d['native_queries']==r['native_queries'] and d['hash_vectors']==r['hash_vectors'] and d['source_inputs']==len(r['inputs']) and d['targeted_tests']==18,'frontend worklist/evidence closure')
    require(work['status']=='in progress; not frozen or published' and work['open_decisions'] and d['profile_activated'] is False,'no W04 freeze or registration')
    inv=strict_json((w/'surface-inventory.json').read_bytes());pins=inv['discovered_files'];require(len(pins)==len({x['path'] for x in pins})==143,'all predecessor paths')
    for x in pins+inv['baseline']+[x for o in inv['component_owners'] for x in o['existing_anchors']]:require(sha((ROOT/x['path']).read_bytes())==x['sha256'],'unchanged predecessor/baseline')
    files=sorted(p for p in w.rglob('*.json') if 'verification' not in p.relative_to(w).parts)
    for p in files:strict_json(p.read_bytes())
    docs=[w/'README.md',RECORD/'README.md',ROOT/'develop/docs/09_go_practical_subset_design-todo.md',ROOT/'develop/docs/go-practical-implementation-traceability-ledger.md']+sorted(p for p in w.glob('*/README.md') if p!=RECORD/'README.md')+[w/'foundation-limits/semantic-rules.md',w/'type-error-zero-decisions.md',w/'operation-loop-decisions.md'];count=0
    for p in docs:
        for t in re.findall(r'\[[^\]\n]*\]\(([^)\n]+)\)',p.read_text()):
            t=t.strip('<>')
            if re.match(r'^[a-z]+://',t) or t.startswith('#'):continue
            require((p.parent/unquote(t.split('#',1)[0])).is_file(),'document link: '+t);count+=1
    require(list(map(int,re.findall(r'^## (\d+)\.',docs[3].read_text(),re.M)))==list(range(1,54)),'ordered ledger1–53')
    print(f"PASS:{len(files)} strict JSONs,{len(r['inputs'])} source pins,{r['cases']} frontend cases/{r['native_queries']} actual native queries/{r['hash_vectors']} hashes,143 predecessors,{len(docs)} documents/{count} links; W04 open")


if __name__=='__main__':main()
