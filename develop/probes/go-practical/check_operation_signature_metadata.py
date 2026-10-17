#!/usr/bin/env python3
"""Check the complete56 operation-signature draft and retained contract evidence."""
import re
from urllib.parse import unquote
from run_operation_signatures import ROOT, HERE, RECORD, check_retained
from run_contract_wire import check_retained as check_contracts
from check_scope_report import strict_json, require, sha
from identity_catalog import generate as identities
from contract_scope import SCOPE_SCHEMA, SCOPE_DOMAIN, BINDING_SCHEMA, BINDING_DOMAIN
from closed_carriers import TYPE_SCHEMA, TYPE_DOMAIN, INSTANCE_SCHEMA, INSTANCE_DOMAIN
from operation_signatures import SCHEMA, DOMAIN


def main():
    w=RECORD.parent;report=check_retained();prior=check_contracts()
    require(len(report["inputs"])==587 and report["outcomes"]=={"typed_unproved":56,"configuration_rejection":168},"complete new operation closure")
    require(len(prior["inputs"])==577 and prior["hash_vectors"]==97 and prior["outcomes"]=={"normalized_unproved":5,"normalized_pending_variant":1,"configuration_rejection":18},"unchanged full preceding contract evidence")
    old=identities();ids={r["identity"] for r in old["identities"]}|set(old["reserved_csharp"]["identities"])|{SCOPE_SCHEMA,BINDING_SCHEMA,TYPE_SCHEMA,INSTANCE_SCHEMA}
    domains={r["domain"] for r in old["identities"]}|set(old["reserved_csharp"]["domains"])|{SCOPE_DOMAIN,BINDING_DOMAIN,TYPE_DOMAIN,INSTANCE_DOMAIN}
    require(SCHEMA not in ids and DOMAIN not in domains,"unreserved distinct concrete operation identity/domain")
    work=strict_json((w/"freeze-worklist.json").read_bytes());d=work["operation_signature_draft"]
    require((d["operation_families"],d["concrete_signatures"],d["configuration_vectors"],d["hash_vectors"],d["targeted_tests"])==(56,56,224,56,24),"operation/worklist coverage")
    require(d["actual_vir_emitted"] is False and d["operation_proofs_checked"] is False and work["status"]=="in progress; not frozen or published" and work["open_decisions"],"W04 and complete source/proof goal remain open")
    inventory=strict_json((w/"surface-inventory.json").read_bytes());pins=inventory["discovered_files"]
    require(len(pins)==len({p["path"] for p in pins})==143,"predecessor coverage")
    for p in pins+inventory["baseline"]+[p for o in inventory["component_owners"] for p in o["existing_anchors"]]:require(sha((ROOT/p["path"]).read_bytes())==p["sha256"],"unchanged predecessor/baseline")
    files=sorted(p for p in w.rglob("*.json") if "verification" not in p.relative_to(w).parts)
    for p in files:strict_json(p.read_bytes())
    readmes=[w/"README.md",RECORD/"README.md"]+[w/name/"README.md" for name in ("contract-wire","closed-carriers","foundation-limits","context-protocol","identity-dag","structural-limits","source-admission","storage-lifetimes","storage-views","overlay-contracts","variant-termination","nat-termination")]
    docs=readmes+[w/"foundation-limits/semantic-rules.md",w/"type-error-zero-decisions.md",w/"operation-loop-decisions.md",ROOT/"develop/docs/09_go_practical_subset_design-todo.md",ROOT/"develop/docs/go-practical-implementation-traceability-ledger.md"]
    links=0
    for p in docs:
        for target in re.findall(r"\[[^\]\n]*\]\(([^)\n]+)\)",p.read_text()):
            target=target.strip("<>")
            if re.match(r"^[a-z]+://",target) or target.startswith("#"):continue
            require((p.parent/unquote(target.split("#",1)[0])).is_file(),"local link: "+target);links+=1
    require(list(map(int,re.findall(r"^## (\d+)\.",docs[-1].read_text(),re.M)))==list(range(1,45)),"ordered ledger1–44")
    print(f"PASS:{len(files)} strict JSONs,56 operation families/224 vectors/56 hashes,retained contract97 hashes/577 pins,143 predecessors,{len(docs)} documents/{links} links; W04 open")


if __name__=="__main__":main()
