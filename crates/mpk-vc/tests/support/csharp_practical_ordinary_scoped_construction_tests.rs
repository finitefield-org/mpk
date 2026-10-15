use super::*;

fn encode(mut c: Certificate) -> Vec<u8> {
    c.export_block = mpk_cert::build_export_block(&c).unwrap();
    c.axiom_report = mpk_cert::build_axiom_report(&c).unwrap();
    c.hashes.export_hash = mpk_cert::export_block_hash(&c.export_block);
    c.hashes.axiom_report_hash = mpk_cert::axiom_report_hash_for_report(&c.axiom_report);
    mpk_cert::encode::encode_certificate(&c)
}
fn wrong_ownership(mut c: Certificate, name: &str) -> Vec<u8> {
    let index = global(&c, name) as usize;
    let DeclarationKind::Def {
        ty,
        mut value,
        reducibility,
    } = c.declarations[index].kind
    else {
        panic!("missing failure definition")
    };
    let mut binders = vec![];
    while let TermNode::Lam { ty, body } = c.term_table[value as usize] {
        binders.push(ty);
        value = body;
    }
    value = c.term_table.len() as u32;
    let yes = global(&c, "Std.Bool.true");
    c.term_table.push(TermNode::Const {
        global: yes,
        levels: vec![],
    });
    for binder in binders.into_iter().rev() {
        let next = c.term_table.len() as u32;
        c.term_table.push(TermNode::Lam {
            ty: binder,
            body: value,
        });
        value = next;
    }
    c.declarations[index].kind = DeclarationKind::Def {
        ty,
        value,
        reducibility,
    };
    encode(c)
}

#[test]
fn csharp_03_t06_w09_scoped_construction_operation_proofs_original_source() {
    std::thread::Builder::new().stack_size(64*1024*1024).spawn(|| {
        let bundle=b();
        let output=std::env::var_os("MPK_W09_SCOPED_CONSTRUCTION_PROOFS_OUT").map(std::path::PathBuf::from);
        if let Some(dir)=&output { fs::create_dir_all(dir).unwrap(); }
        let filter=std::env::var("MPK_W09_SCOPED_SOURCE_FILTER").ok();
        let (mut contexts,mut candidates,mut application_ids)=(0,0,0);
        let mut kinds=BTreeSet::new();
        let mut mutated=false;
        for (id,row,facts) in sources() {
            if filter.as_ref().is_some_and(|f| f != &id) { continue; }
            let (context,captures)=support::replay_context(&bundle,&row);
            let source=ValidatedDataSource::import_captured_facts(&bundle,&context,&captures,&serde_json::to_vec(&facts).unwrap()).unwrap();
            let emitted=emit_data_phase(&bundle,&context,&captures,&source).unwrap();
            let vir=emitted.vir();
            eprintln!("{id}: generating original scoped construction operations");
            let base=generate_csharp_practical_ordinary_concrete_operations_with_allocations(vir).unwrap();
            let result=generate_csharp_practical_ordinary_scoped_construction_operation_proofs(vir).unwrap_or_else(|error| panic!("{id}: {error:?}"));
            assert_eq!(result.generic_pending_operations(),base.pending_operations());
            assert_eq!(result.pending_proof_ids(),base.pending_proof_ids());
            let full=generate_csharp_practical_vc(PracticalVcSource {artifact_context:&context,captured_inputs:&captures,vir}).unwrap();
            let eligible=full.data_vcs().operations().iter().filter(|op| {
                let definition=full.data_vcs().definitions().iter().find(|d| d.id==op.definition_id).unwrap();
                base.pending_operations().iter().any(|p| p.component.operation_id==definition.signature.id && p.reasons==[OrdinaryConcreteOperationPendingReason::InternalConstructionState,OrdinaryConcreteOperationPendingReason::SourceOwnership])
            }).collect::<Vec<_>>();
            assert_eq!(result.candidates().iter().map(|c| c.source.id.as_str()).chain(result.pending_source_operation_ids().iter().map(|s| s.as_str())).collect::<BTreeSet<_>>(),eligible.iter().map(|o| o.id.as_str()).collect::<BTreeSet<_>>());
            for candidate in result.candidates() {assert_eq!(Some(&candidate.source),eligible.iter().copied().find(|o| o.id==candidate.source.id));}
            let before=mpk_cert::decode_canonical_certificate(base.certificate_bytes()).unwrap();
            let mut certificates=vec![];
            let mut source_points=BTreeSet::new();
            for (ordinal,candidate) in result.candidates().iter().enumerate() {
                assert!(source_points.insert((&candidate.source.function_id,&candidate.source.node_id)));
                let p=&candidate.program;
                let definitions=candidate.definition_program();
                assert_eq!(definitions.public_domains(),base.public_domains());
                assert_eq!(definitions.source_clauses(),base.source_clauses());
                assert_eq!(definitions.source_observations(),base.source_observations());
                assert_eq!(definitions.pending_proof_ids(),base.pending_proof_ids());
                let added=definitions.definitions().iter().filter(|d| !base.definitions().iter().any(|old| old.component.operation_id==d.component.operation_id)).collect::<Vec<_>>();
                assert_eq!(added.len(),1);
                let d=added[0];
                let pending=base.pending_operations().iter().find(|v| v.component.operation_id==d.component.operation_id).unwrap();
                assert_eq!((&d.recipe,&d.instance_id),(&pending.recipe,&pending.instance_id));
                assert_eq!((&d.component.argument_type_ids,&d.component.result_type_id,&d.component.normal_definition),(&pending.component.argument_type_ids,&pending.component.result_type_id,&pending.component.normal_definition));
                assert_eq!(&d.component.failures[1..],&pending.component.failures[1..]);
                assert_eq!(d.component.failures[0].label,"ownership");
                assert_eq!(d.component.failures[0].argument_indices,vec![0]);
                assert_eq!(d.component.failures[0].definition.as_ref(),Some(&candidate.ownership.failure_definition));
                assert_eq!(candidate.ownership.receiver_id,candidate.source.subjects[0].id);
                assert_eq!(candidate.ownership.function_id,candidate.source.function_id);
                assert_eq!(candidate.ownership.node_id,candidate.source.node_id);
                let original=full.binding_vcs().sequents().iter().find(|s| s.kind=="concrete_definition_equivalence" && s.owner_id==d.component.operation_id).unwrap();
                assert_eq!(p.proofs().iter().find(|s| s.sequent.owner_id==d.component.operation_id).unwrap().sequent,*original);
                assert_eq!(p.proofs().iter().map(|p| &p.sequent).collect::<Vec<_>>(),definitions.conditions().iter().map(|c| &c.sequent).collect::<Vec<_>>());
                for old in base.definitions() { assert_eq!(Some(old),definitions.definitions().iter().find(|d| d.component.operation_id==old.component.operation_id)); }
                for old in base.conditions() { assert_eq!(Some(old),definitions.conditions().iter().find(|d| d.sequent.id==old.sequent.id)); }
                assert!(p.construction_storage_domains().iter().all(|d| d.private_storage_only && d.ownership_pending));
                assert!(!p.construction_storage_domains().is_empty());
                assert_eq!(p.pending_operations().len()+1,base.pending_operations().len());
                let after=mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
                assert!(after.term_table.starts_with(&before.term_table));
                for (a,z) in before.declarations.iter().zip(&after.declarations) { assert_eq!(before.name_table[a.name as usize],after.name_table[z.name as usize]); assert_eq!(a.kind,z.kind); }
                let failure=global(&after,&candidate.ownership.failure_definition) as usize;
                let DeclarationKind::Def {mut value,..}=after.declarations[failure].kind else {panic!("missing scoped failure")};
                let TermNode::Lam {body,..}=after.term_table[value as usize] else {panic!("missing exact receiver argument")}; value=body;
                for name in [&candidate.ownership.flow_theorem,&candidate.ownership.receiver_theorem] {
                    let TermNode::Let {value:proof,body,..}=after.term_table[value as usize] else {panic!("missing checked source dependency")};
                    let TermNode::Const {global,ref levels}=after.term_table[proof as usize] else {panic!("missing source theorem")};
                    assert!(levels.is_empty()); assert_eq!(&after.name_table[after.declarations[global as usize].name as usize],name); value=body;
                }
                eprintln!("{id}/{ordinal}: check {} at {}",d.component.operation_id,candidate.source.node_id);
                let report=mpk_kernel::verify_certificate_bytes(p.certificate_bytes()).unwrap();
                assert_eq!(report.axiom_count,0);
                assert!(after.proof_node_table.is_empty() && after.theory_certificates.is_empty());
                if !mutated {
                    let failure=&d.failures[0].concrete_failure_definition;
                    let definitions_only=mpk_cert::decode_canonical_certificate(definitions.certificate_bytes()).unwrap();
                    assert_eq!(mpk_kernel::verify_certificate_bytes(&wrong_ownership(definitions_only,failure)).unwrap().axiom_count,0);
                    let wrong=wrong_ownership(after.clone(),failure);
                    assert_eq!(mpk_kernel::verify_certificate_bytes(&wrong).unwrap_err().kind(),mpk_kernel::VerificationErrorKind::CoreCheck);
                    if let Some(dir)=&output {fs::write(dir.join(format!("{id}-{ordinal}-wrong.hex")),wrong.iter().map(|b| format!("{b:02x}")).collect::<String>()+"\n").unwrap();}
                    mutated=true;
                }
                certificates.push(p.certificate_bytes().to_vec());
                if let Some(dir)=&output {
                    fs::write(dir.join(format!("{id}-{ordinal}.hex")),p.certificate_bytes().iter().map(|b| format!("{b:02x}")).collect::<String>()+"\n").unwrap();
                    fs::write(dir.join(format!("{id}-{ordinal}.json")),serde_json::to_vec(candidate).unwrap()).unwrap();
                }
                kinds.insert(d.component.operation_id.rsplit('.').next().unwrap().to_owned());
                candidates+=1;
            }
            assert_eq!(import_csharp_practical_ordinary_scoped_construction_operation_proofs(&result.canonical_bytes(),&certificates,vir).unwrap(),result);
            if !result.candidates().is_empty() {
                let mut changed:Value=serde_json::from_slice(&result.canonical_bytes()).unwrap();
                changed["candidates"][0]["ownership"]["receiver_id"]=json!("wrong.receiver");
                assert!(import_csharp_practical_ordinary_scoped_construction_operation_proofs(&serde_json::to_vec(&changed).unwrap(),&certificates,vir).is_err());
                assert!(import_csharp_practical_ordinary_scoped_construction_operation_proofs(&result.canonical_bytes(),&certificates[..certificates.len()-1],vir).is_err());
            }
            if let Some(dir)=&output {fs::write(dir.join(format!("{id}-program.json")),result.canonical_bytes()).unwrap();}
            application_ids+=result.pending_proof_ids().len();contexts+=1;
            eprintln!("{id}: {} source candidates, {} generic operations and {} application IDs pending",result.candidates().len(),result.generic_pending_operations().len(),result.pending_proof_ids().len());
        }
        assert!(candidates>0 && mutated);
        if filter.is_none() { assert_eq!((contexts,application_ids),(45,987)); }
        eprintln!("scoped construction: {contexts} contexts, {candidates} candidates, operation kinds {kinds:?}");
    }).unwrap().join().unwrap();
}
