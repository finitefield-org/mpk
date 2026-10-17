"""Final W04 interface audit, not an ordinary elaborator or proof checker.

Historical helpers stay unchanged. Materialized statements require the future
independent source importer and both checkers; no synthetic success arm here.
"""
from collections import Counter
from pathlib import Path
import json

from go_obligations import FAMILIES, require_materialized, OrdinaryElaborationPending
from policy_sources import source_rows, profile_envelope
from proof_components import fragments
from proof_canonical import canonical
from run_proof_pilot import ROOT, sha

RECORD = ROOT / "develop/migrations/go-practical/t01-w04/program-binding"
CONTEXT_FIELDS = (
    "schema", "semantic_context", "source_ir_hash", "source_map_hash",
    "source_manifest_hash", "obligation_program_hash", "foundation_descriptor",
    "context_certificate_chunks", "context_certificate_sha256", "definitions",
    "required_theorems", "assembly_hash",
)
BINDING_FIELDS = (
    "schema", "semantic_context", "source_ir_hash", "source_map_hash",
    "source_manifest_hash", "obligation_program_hash", "assembly_hash",
    "functions", "binding_hash",
)
MEMBER_FIELDS = (
    "request_hash", "name", "type_term_id", "declaration_hash",
)
FUNCTION_FIELDS = (
    "function_decl_id", "source_scope_sha256", "signature_id", "body_hash",
    "contract_hash", "source_definition", "ir_definition", "contract_definition",
    "entry_relation_definition", "completion_relation_definition", "request_hashes",
)

# Bound variable legend and judgments are normative in program-binding/README.md.
# These are interface schemas. Strings do not stand in for kernel propositions.
SIGNATURES = {
    "function_source_binding": (
        "SourceWF(f) and IRWF(f) and forall e,s. EntryRep(f,e,s) -> Correspond(f,e,s)",
        "independent source interpreter, all prefixes and all completion kinds",
        "replace source AST by the emitted IR or assume correspondence",
    ),
    "entry_representation": (
        "forall e. AdmittedEntry(f,e) -> exists s. EntryRep(f,e,s)",
        "full argument/product/alias/initialized-tail representation",
        "assume caller fit, separation, no-wrap or copied-list slice semantics",
    ),
    "contract_definedness": (
        "forall e,s. ClausePhase(f,c,e,s) -> Defined(c,e,s)",
        "own clause excluded; ordered earlier requires and short-circuit guards only",
        "use the clause being checked, its result or an unproved invariant",
    ),
    "postcondition": (
        "forall e,s,o. Pre(f,e) and Run(f,e,s,o) and Normal(o) -> Defined(c,e,o) and Holds(c,e,o)",
        "every normal return, exact positional/named results and entry old-state",
        "drop early return, permute tuple members or read old from the final heap",
    ),
    "function_frame": (
        "forall e,s,p. Pre(f,e) and EntryRep(f,e,s) and Reach(f,s,p) -> Frame(f,e,p)",
        "every reachable prefix, borrowed inputs and exact write permissions",
        "allow an input write because the original value is restored on return",
    ),
    "partial_correctness": (
        "forall e,s. Pre(f,e) and EntryRep(f,e,s) -> SafePrefixes(f,s) and NormalPost(f,e,s)",
        "all paths, runtime safety and domain obligations; divergence allowed",
        "interpret finite proof fuel as termination, falsehood or source panic",
    ),
    "function_totality": (
        "forall e,s. Pre(f,e) and EntryRep(f,e,s) -> exists o. Run(f,e,s,o) and Normal(o)",
        "actual normal termination plus separately required partial correctness",
        "use a finite slice length or the partial contract as a termination axiom",
    ),
    "node_member": (
        "forall e,s. ReachSite(f,node,e,s) -> Member(op,category,ordinal,e,s)",
        "exact concrete member of the independently regenerated 56-operation table",
        "substitute a different node, member ordinal, operation or free predicate",
    ),
    "access_member": (
        "forall e,s. ReachSite(f,node,e,s) -> Access(access_ordinal,e,s)",
        "full address arithmetic, typed owner/view, domain and permissions",
        "truncate addresses or omit an access point/initialized tail",
    ),
    "call_precondition": (
        "forall e,s. ReachCall(f,node,e,s) -> Defined(Pre(g),Actual(node,s)) and Pre(g,Actual(node,s))",
        "evaluated receiver/arguments from one invocation; actual callee and contract",
        "assume a callee precondition or evaluate an argument twice",
    ),
    "call_relation": (
        "forall e,s,o. ReachCall(f,node,e,s) and Run(g,Actual(node,s),s,o) -> JointCall(node,s,o)",
        "one callee execution couples complete result product and output state",
        "combine results and effects from distinct calls or import unproved contracts",
    ),
    "call_totality": (
        "forall e,s. ReachCall(f,node,e,s) -> exists o. Run(g,Actual(node,s),s,o) and Normal(o)",
        "conditional on actual reached arguments; only required by total callers",
        "promote a partial callee to globally total or add recursive theorem cycles",
    ),
    "adapter_binding": (
        "RegisteredExactAdapter(node) and forall a,s,o. SourceAdapter(node,a,s,o) iff AdapterModel(node,a,s,o)",
        "registered compiler/symbol/body/target and full precondition/effect relation",
        "trust a package name, caller adapter, mismatched body or target",
    ),
    "region_control": (
        "forall e,s. ReachRegion(f,region,e,s) -> CorrectTerminator(region,e,s)",
        "exact control stack and continuation; distinct yield/return/break/continue",
        "change an abrupt exit target or discard an enclosing post step",
    ),
    "loop_initialization": (
        "forall e,s. ReachFirstHeader(f,loop,e,s) -> DefinedInv(loop,e,s) and Inv(loop,e,s)",
        "first header after initializer and range capture; candidate invariant checked",
        "assume even a generated invariant before its initialization proof",
    ),
    "loop_preservation": (
        "forall e,h,k. Inv(loop,e,h) and ContinuingIteration(loop,h,k) -> DefinedInv(loop,e,k) and Inv(loop,e,k)",
        "all continuing backedges including continue with the correct post",
        "require decrease on return/break or skip the post on continue",
    ),
    "loop_exit": (
        "forall e,h,o. Inv(loop,e,h) and ExitingIteration(loop,h,o) -> ExitRelation(loop,e,h,o)",
        "false test, break and early return keep their own guards/continuations",
        "infer a false test at break or ignore result/effect propagation",
    ),
    "loop_variant_domain": (
        "forall e,h. ReachVariantHeader(loop,e,h) -> Defined(V,e,h) and NatRank(V,e,h)",
        "well-founded Nat rank after exact signed/full-word conversion",
        "use wrapped machine subtraction or an unproved nonnegative assumption",
    ),
    "loop_variant_decrease": (
        "forall e,h,k. Inv(loop,e,h) and ContinuingIteration(loop,h,k) -> Rank(V,e,k) < Rank(V,e,h)",
        "strict decrease only on continuing edges; exact source loop binding",
        "accept equal/wrapped ranks or borrow another loop's variant",
    ),
}


def host_contract():
    return {
        "schema": "mpk.go_practical.host_resource_design.v0",
        "host_profile_id": "mpk.host.linux-x86_64-gnu.go_practical.v1",
        "probe_profile_id": "mpk.release.probe.go_practical.v1",
        "os": "linux", "architecture": "x86_64", "abi": "gnu",
        "minimum_kernel_abi": "6.4.0", "native_execution_required": True,
        "frontend": {
            "wall_ns": 120_000_000_000, "cgroup_memory_max_bytes": 2_147_483_648,
            "cgroup_memory_swap_max_bytes": 0, "cgroup_pids_max": 128,
            "rlimit_as_bytes": 17_179_869_184, "rlimit_nofile": 1024,
            "rlimit_core_bytes": 0, "tmpfs_bytes": 536_870_912,
            "tmpfs_inodes": 131_072, "direct_child_parallelism": 1,
            "GOMAXPROCS": "1", "GOMEMLIMIT": "1610612736",
        },
        "proof_child": {
            "wall_ns": 60_000_000_000, "peak_rss_bytes": 1_073_741_824,
            "cgroup_memory_max_bytes": 1_073_741_824,
            "cgroup_memory_swap_max_bytes": 0, "cgroup_pids_max": 128,
            "rlimit_as_bytes": 17_179_869_184, "rlimit_nofile": 1024,
            "rlimit_core_bytes": 0, "tmpfs_bytes": 67_108_864,
            "tmpfs_inodes": 16_384, "direct_child_parallelism": 1,
        },
        "certificate": {"bytes": 2_097_152, "terms": 131_072, "declarations": 4096},
        "required_primitives": [
            "atomic cgroup placement before untrusted execution", "cgroup v2 memory/swap/pids accounting",
            "descendant task enumeration and kill; empty cgroup before receipt",
            "mount,user,network,pid namespaces", "no network; closed environment",
            "read-only no-follow immutable input handles", "nonroot; no capabilities; no_new_privileges",
            "tmpfs noswap,nodev,nosuid,noexec with block and inode accounting",
            "read-only private proc", "Go thread/process syscall allowlist",
            "monotonic parent wall watchdog; bounded stdout/stderr and publication",
            "atomic no-replace output publication after successful complete validation",
        ],
        "frontend_executor": "crates/mpk-cli/src/go_practical_frontend_runner.rs",
        "capture_executor": "go-tools/go2vir/practical_capture.go",
        "proof_executor": "crates/mpk-cli/src/go_practical_release.rs",
        "tests": ["crates/mpk-cli/tests/go_practical_frontend_runner.rs",
                  "crates/mpk-cli/tests/go_practical_release.rs"],
        "work_items": ["GO-PRAC-T02-W01", "GO-PRAC-T07-W01", "GO-PRAC-T07-W02"],
        "failure_rule": "missing primitive -> environment_error; killed/over-budget -> resource_pending; no source rejection, false theorem, fallback or widened budget",
        "measurement_scope": "design budgets with headroom over Darwin workload points; native Linux process-tree enforcement/corpus fit remain mandatory T07 gates",
    }


def catalogue():
    assert set(SIGNATURES) == set(FAMILIES)
    return {
        "schema": "mpk.go_practical.program_interface_design.v0",
        "status": "inactive design; not generated ordinary propositions or checker acceptance",
        "assembly": {
            "schema": "mpk.program_certificate.ordinary_context.v3",
            "domain": "MPK-PROGRAM-ASSEMBLY-3.0", "fields": list(CONTEXT_FIELDS),
            "definition_fields": ["function_decl_id", "role", "name", "declaration_id"],
            "definition_roles": ["source", "ir", "contract", "entry_relation", "completion_relation"],
            "theorem_fields": list(MEMBER_FIELDS),
            "context_codec": "canonical Certificate v0 bytes in lowercase hex chunks <=65536 characters; full decoded certificate <=2097152 bytes; imports bind exact checked foundation exports; source_manifest absent; no axiom/theory primitive/new trusted rule",
            "table_rule": "all context tables/declarations are an exact prefix of the final proof certificate; proof terms/theorems append, never replace or renumber the context",
            "statement_rule": "whole assembly_hash plus request_hash selects exact name and type_term_id in the bound term table; declaration_hash uses the existing Certificate v0 declaration_interface_hash, which alone does not bind the term table",
            "hash_preimage": "all fields except assembly_hash, including full context bytes and exact source/IR/contract/definition/member tables; no VC/skeleton/certificate-manifest/receipt ref",
        },
        "source_binding": {
            "schema": "mpk.go.semantic_binding.v1", "domain": "MPK-GO-SEMANTIC-BINDING-1.0",
            "fields": list(BINDING_FIELDS), "function_fields": list(FUNCTION_FIELDS),
            "hash_preimage": "all fields except binding_hash; assembly precedes binding; neither root establishes that its theorem is proved",
        },
        "materialized_expansion": {
            "fields": ["kind", "assembly", "source_binding"], "kind": "materialized",
            "condition": "independently regenerate complete context/definitions/member list and source correspondence types; all requests elaborated; no unresolved candidate; source and IR well-formedness computed, never supplied as hypotheses",
            "unavailable_arm": {"kind": "unavailable"},
            "skeleton": "existing common root, Go kind, theorem_declarations are exact ordered required_theorems projected to request_hash,name,declaration_hash; source_vc_hash binds the full assembly and binding",
        },
        "families": [{"kind": k, "signature": SIGNATURES[k][0], "coverage": SIGNATURES[k][1],
                      "reject": SIGNATURES[k][2]} for k in FAMILIES],
        "dag": ["captured source/typed source scope+VIR+contracts", "obligation program",
                "ordinary assembly", "source binding", "materialized VC", "VC skeleton",
                "certificate manifest", "paired checked certificate", "policy/API/AI evidence"],
        "policy": {
            "property_fields": ["function_decl_id", "request_hash", "name", "declaration_hash", "assembly_hash", "source_vc_hash", "status"],
            "required_set": "every request exactly once; grouping is presentation only; no omitted source/frame/definedness/call/loop member",
            "checked_rule": "identical final certificate bytes accepted by both checkers; same checked foundation imports and exact context prefix/statement/manifest; Go zero axioms; exact fully discharged dependency DAG",
            "pending_rule": "unelaborated != materialized pending != candidate rejected != resource pending; no admitted source/profile claim from metadata",
            "retained_policy_rule": "dispatch compiled exact context policy: zero-axiom Go remains zero; retained mvp-theory allowlists, formats and five axiom categories remain unchanged; fixture-specific paired_certificate zero check is not the generic retained policy",
            "AI_rule": "complete policy-owned property list, all batches and global aliases; projected prose cannot supply status, target, declaration, proof, context or source authority",
        },
        "production_owners": [
            {"path": "crates/mpk-vc/src/go_practical_foundation.rs", "work_item": "GO-PRAC-T02-W03"},
            {"path": "crates/mpk-vc/src/go_practical_artifacts.rs", "work_item": "GO-PRAC-T02-W03"},
            {"path": "crates/mpk-vc/src/go_practical_vc.rs", "work_item": "GO-PRAC-T03-W03"},
            {"path": "crates/mpk-cli/src/go_practical_program_certificate.rs", "work_item": "GO-PRAC-T03-W03"},
            {"path": "crates/mpk-api/src/go_practical_policy.rs", "work_item": "GO-PRAC-T07-W01"},
        ],
    }


def audit():
    rows = source_rows()
    go = [r for r in rows if "project_spec" in r["base"]]
    families = Counter()
    models = []
    for r in go:
        b = r["base"]; p = b["vc"]["obligations"]
        requests = [q for f in p["functions"] for q in f["requests"]]
        ids = [q["request_hash"] for q in requests]
        assert sorted(ids) == p["unelaborated_request_ids"] and len(ids) == len(set(ids))
        assert b["vc"]["ordinary_expansion"] == {"kind": "unavailable"}
        for q in requests:
            kind = q["requirement"]["kind"]; assert kind in SIGNATURES
            families[kind] += 1
            assert set(q["dependencies"]) <= set(ids)
        try:
            require_materialized(p)
        except OrdinaryElaborationPending:
            pass
        else:
            raise AssertionError("unelaborated source cannot have a certificate skeleton")
        models.append({"id": b["id"], "requests": len(ids), "skeleton_available": False})
    ops = fragments()
    assert len(ops) == 56 and len({r["op"] for r in ops}) == 56
    retained = []
    for r in rows:
        if "project_spec" in r["base"]: continue
        env = profile_envelope(r["base"], "policy")
        retained.append({"id": r["base"]["id"], "policy": env["value"]})
    assert len(go) == 2 and sum(r["requests"] for r in models) == 169
    structural = ROOT / "develop/migrations/go-practical/t01-w04/structural-limits/limits.json"
    source = json.loads(structural.read_text()); proof = {r["id"]: r["maximum"] for r in source["counters"] if r["id"].startswith("proof.")}
    h = host_contract()
    assert h["proof_child"]["wall_ns"] == proof["proof.checker_wall_ns_each"] == proof["proof.construction_wall_ns"]
    assert h["proof_child"]["peak_rss_bytes"] == proof["proof.peak_rss_bytes_each_child"]
    for k in ("bytes", "terms", "declarations"):
        assert h["certificate"][k] == proof["proof.certificate_" + k]
    return {"schema": "mpk.go_practical.program_interface_audit.v0", "models": models,
            "request_family_counts": dict(sorted(families.items())), "declared_families": len(SIGNATURES),
            "typed_operation_fragments": len(ops), "retained_policy_envelopes": retained,
            "structural_limits_ref": {"path": str(structural.relative_to(ROOT)), "sha256": sha(structural.read_bytes())},
            "ordinary_propositions_generated": False, "source_proofs_checked": False,
            "native_host_enforcement_observed": False, "profile_activated": False}


def write():
    RECORD.mkdir(parents=True, exist_ok=True)
    for n, value in [("interfaces.json", catalogue()), ("host-resources.json", host_contract()), ("audit.json", audit())]:
        (RECORD / n).write_bytes(canonical(value) + b"\n")


if __name__ == "__main__": write()
