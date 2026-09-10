use super::boundary_document_tests::document_sources;
use super::*;
use core_eval::{bit, sparse_cube};

// Construct concrete input storage only; all byte decoding and validity
// decisions execute the generated ordinary definitions.
fn document(bytes: &[u8], length: u32, inactive: &[(usize, u8)]) -> V {
    let mut cells = BTreeSet::new();
    for k in 0..32 {
        if length & (1 << k) != 0 {
            cells.insert(k << 19);
        }
    }
    for (at, byte) in bytes
        .iter()
        .copied()
        .enumerate()
        .chain(inactive.iter().copied())
    {
        assert!(at < 1_048_576);
        for k in 0..8 {
            if byte & (1 << k) != 0 {
                cells.insert(1 | (at << 1) | (k << 21));
            }
        }
    }
    sparse_cube(24, cells)
}
fn with_program(f: impl FnOnce(&mpk_cert::encode::Certificate, &OrdinaryBoundaryUtf8Definition)) {
    let bundle = b();
    let (_, row, facts) = document_sources()
        .into_iter()
        .find(|(id, _, _)| id.starts_with("document-"))
        .unwrap();
    let (context, captures) = support::replay_context(&bundle, &row);
    let source = ValidatedDataSource::import_captured_facts(
        &bundle,
        &context,
        &captures,
        &serde_json::to_vec(&facts).unwrap(),
    )
    .unwrap();
    let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
    let p = generate_csharp_practical_ordinary_boundary_utf8(emitted.vir()).unwrap();
    let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
    validate_csharp_practical_certificate_structure(&cert).unwrap();
    f(&cert, p.definition().unwrap());
}
fn check(cert: &mpk_cert::encode::Certificate, d: &OrdinaryBoundaryUtf8Definition, bytes: &[u8]) {
    let expected = std::str::from_utf8(bytes).is_ok() && !bytes.starts_with(&[0xef, 0xbb, 0xbf]);
    assert_eq!(
        bit(run(
            cert,
            &d.valid_definition,
            vec![document(bytes, bytes.len() as u32, &[])]
        )),
        expected,
        "UTF-8 length {}, prefix {:?}, suffix {:?}",
        bytes.len(),
        &bytes[..bytes.len().min(8)],
        &bytes[bytes.len().saturating_sub(8)..]
    );
}
#[test]
fn csharp_03_t06_w09_boundary_utf8_semantics() {
    with_program(|cert, d| {
        let mut cases = vec![
            vec![],
            b"ascii\0".to_vec(),
            vec![0xc2, 0x80],
            vec![0xdf, 0xbf],
            vec![0xe0, 0xa0, 0x80],
            vec![0xed, 0x9f, 0xbf],
            vec![0xee, 0x80, 0x80],
            vec![0xf0, 0x90, 0x80, 0x80],
            vec![0xf4, 0x8f, 0xbf, 0xbf],
            vec![0xef, 0xbb, 0xbf],
            vec![0x61, 0xef, 0xbb, 0xbf],
        ];
        for byte in 0..=255 {
            cases.push(vec![byte]);
        }
        for second in 0..=255 {
            for lead in [0xc2, 0xdf] {
                cases.push(vec![lead, second]);
            }
            for lead in [0xe0, 0xed, 0xef] {
                cases.push(vec![lead, second, 0x80]);
            }
            for lead in [0xf0, 0xf4] {
                cases.push(vec![lead, second, 0x80, 0x80]);
            }
        }
        for value in [
            vec![0xc2, 0x80],
            vec![0xe0, 0xa0, 0x80],
            vec![0xf0, 0x90, 0x80, 0x80],
        ] {
            for n in 1..value.len() {
                cases.push(value[..n].to_vec());
            }
            for position in 1..value.len() {
                for byte in [0, 0x7f, 0xc0, 0xff] {
                    let mut bad = value.clone();
                    bad[position] = byte;
                    cases.push(bad);
                }
            }
        }
        for (i, bytes) in cases.iter().enumerate() {
            if i % 64 == 0 {
                eprintln!("UTF-8 semantic case {i}/{}", cases.len());
            }
            check(cert, d, bytes);
        }
        for bytes in [vec![], vec![b'a'], vec![0xc2, 0x80]] {
            let input = document(
                &bytes,
                bytes.len() as u32,
                &[
                    (bytes.len(), 0xff),
                    (63, 0xf5),
                    (64, 0xff),
                    (1_048_575, 0xff),
                ],
            );
            assert!(bit(run(cert, &d.valid_definition, vec![input])));
        }
        for length in [1_048_577, 1 << 31, u32::MAX] {
            assert!(!bit(run(
                cert,
                &d.valid_definition,
                vec![document(&[], length, &[])]
            )));
        }
        eprintln!(
            "UTF-8 {} oracle cases, 3 inactive storage and 3 full-u32 overflow cases",
            cases.len()
        );
    });
}
#[test]
fn csharp_03_t06_w09_boundary_utf8_block_boundaries() {
    with_program(|cert, d| {
        for boundary in [64, 16_384, 65_536] {
            for offset in 1..=3 {
                for sequence in [
                    &[0xf4, 0x8f, 0xbf, 0xbf][..],
                    &[0xf4, 0x90, 0x80, 0x80][..],
                    &[0xe0, 0xa0][..],
                ] {
                    let mut bytes = vec![b'a'; boundary - offset];
                    bytes.extend(sequence);
                    eprintln!("UTF-8 boundary {boundary}, offset {offset}, sequence {sequence:?}");
                    check(cert, d, &bytes);
                }
            }
        }
    });
}
#[test]
fn csharp_03_t06_w09_boundary_utf8_full_document_bound() {
    with_program(|cert, d| {
        let mut bytes = vec![b'a'; 1_048_576];
        eprintln!("UTF-8 full 1 MiB ASCII");
        check(cert, d, &bytes);
        bytes[1_048_572..].copy_from_slice(&[0xf4, 0x8f, 0xbf, 0xbf]);
        eprintln!("UTF-8 full 1 MiB maximum scalar tail");
        check(cert, d, &bytes);
        bytes[1_048_575] = 0xff;
        eprintln!("UTF-8 full 1 MiB invalid final byte");
        check(cert, d, &bytes);
    });
}
#[test]
fn csharp_03_t06_w09_boundary_utf8_original_sources() {
    let bundle = b();
    let out = std::env::var_os("MPK_W09_BOUNDARY_UTF8_OUT").map(std::path::PathBuf::from);
    if let Some(dir) = &out {
        fs::create_dir_all(dir).unwrap();
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/boundary-utf8");
    let mut rows = vec![];
    let mut observed = false;
    let mut previous: Option<(Vec<u8>, Vec<u8>)> = None;
    let mut examined = 0;
    for (id, row, facts) in document_sources() {
        examined += 1;
        let (context, captures) = support::replay_context(&bundle, &row);
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(&facts).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        let vir = emitted.vir();
        let p = generate_csharp_practical_ordinary_boundary_utf8(vir)
            .unwrap_or_else(|e| panic!("{id}: {e:?}"));
        let vc = generate_csharp_practical_vc(PracticalVcSource {
            artifact_context: &context,
            captured_inputs: &captures,
            vir,
        })
        .unwrap();
        let boundary = vc.boundary_vcs();
        assert_eq!(p.definition().is_none(), boundary.contracts().is_empty());
        let meta: Value = serde_json::from_slice(&p.canonical_bytes()).unwrap();
        let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&cert).unwrap();
        assert_eq!(
            import_csharp_practical_ordinary_boundary_utf8(
                &p.canonical_bytes(),
                p.certificate_bytes(),
                vir
            )
            .unwrap(),
            p
        );
        for field in [
            "schema",
            "source_ir_sha256",
            "foundation_sha256",
            "boundary_program_sha256",
            "definition",
            "certificate_sha256",
        ] {
            let mut forged = meta.clone();
            forged[field] = json!("forged");
            assert!(import_csharp_practical_ordinary_boundary_utf8(
                &serde_json::to_vec(&forged).unwrap(),
                p.certificate_bytes(),
                vir
            )
            .is_err());
        }
        let mut bad = p.certificate_bytes().to_vec();
        *bad.last_mut().unwrap() ^= 1;
        assert!(
            import_csharp_practical_ordinary_boundary_utf8(&p.canonical_bytes(), &bad, vir)
                .is_err()
        );
        if let Some((m, c)) = previous.as_ref() {
            assert!(import_csharp_practical_ordinary_boundary_utf8(m, c, vir).is_err());
        }
        previous = Some((p.canonical_bytes(), p.certificate_bytes().to_vec()));
        let Some(d) = p.definition() else { continue };
        assert_eq!(d.document.depth, 24);
        assert_eq!(d.block_bytes, 64);
        assert_eq!(d.blocks_per_step, 2);
        assert_eq!(d.scan_steps, 8_192);
        assert!(d.static_transformers > 8_194 && d.static_transformers <= 16_384);
        let hex = p
            .certificate_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
            + "\n";
        if let Some(dir) = &out {
            fs::write(dir.join(format!("{id}.hex")), &hex).unwrap();
        } else {
            assert_eq!(
                fs::read_to_string(root.join(format!("{id}.hex"))).unwrap(),
                hex
            );
        }
        rows.push(json!({"id":id,"metadata":meta,"terms":cert.term_table.len(),"declarations":cert.declarations.len()}));
        if !observed {
            assert!(bit(run(
                &cert,
                &d.valid_definition,
                vec![document(&[], 0, &[])]
            )));
            observed = true;
        }
    }
    assert_eq!(examined, 68);
    assert_eq!(rows.len(), 3);
    assert!(observed);
    eprintln!(
        "boundary UTF-8 contexts {}, nonempty {}",
        examined,
        rows.len()
    );
    let manifest = json!({"contexts_examined":examined,"sources":rows});
    if let Some(dir) = &out {
        fs::write(
            dir.join("certificates.json"),
            serde_json::to_vec_pretty(&manifest).unwrap(),
        )
        .unwrap();
    } else {
        assert_eq!(
            read("ordinary-foundation/boundary-utf8/certificates.json"),
            manifest
        );
    }
}

#[test]
fn csharp_03_t06_w09_boundary_utf8_packed_state_releases_prior_storage() {
    with_program(|cert, _| {
        let pack = "Mpk.CSharp.Ordinary.BoundaryUtf8.PackState";
        // Basis states cover every cursor/decoder/failure/padding bit, plus
        // both constant states. The returned cube must preserve every leaf.
        for state in std::iter::once(0u64)
            .chain(std::iter::once(u64::MAX))
            .chain((0..64).map(|k| 1u64 << k))
        {
            let output = run(
                cert,
                pack,
                (0..64).map(|k| V::Bit(state & (1u64 << k) != 0)).collect(),
            );
            for k in 0..64 {
                let mut v = output.clone();
                for i in 0..6 {
                    v = core_eval::apply(cert, v, V::Bit(k & (1 << i) != 0));
                }
                assert_eq!(bit(v), state & (1u64 << k) != 0);
            }
        }
        // Suspensions are genuine ordinary Var(0) terms holding a large prior
        // environment. This weak handle observes release without changing the
        // evaluator or replacing the ordinary packer's demand behavior.
        let var = cert
            .term_table
            .iter()
            .position(|t| matches!(t, mpk_cert::encode::TermNode::Var(0)))
            .unwrap() as u32;
        let payload = core_eval::EnvRef::new(core_eval::Env::Bind(
            V::Cube(vec![false; 4096]),
            core_eval::EnvRef::new(core_eval::Env::Empty),
        ));
        let captured = std::rc::Rc::new(core_eval::Env::Bind(V::Bit(true), payload));
        let weak = std::rc::Rc::downgrade(&captured);
        let args = (0..64)
            .map(|_| core_eval::thunk(var, captured.clone()))
            .collect();
        drop(captured);
        let output = run(cert, pack, args);
        assert!(weak.upgrade().is_some());
        let mut leaf = output.clone();
        for _ in 0..6 {
            leaf = core_eval::apply(cert, leaf, V::Bit(false));
        }
        assert!(bit(leaf));
        assert!(
            weak.upgrade().is_none(),
            "one demanded packed leaf must release all prior bit environments"
        );
        // The result stays alive while the prior storage is already released.
        let mut leaf = output;
        for _ in 0..6 {
            leaf = core_eval::apply(cert, leaf, V::Bit(true));
        }
        assert!(bit(leaf));
        // A pointwise guard can retain its unused old-state argument. Seal
        // must release that branch after observation without evaluating it.
        let captured = std::rc::Rc::new(core_eval::Env::Bind(
            V::Cube(vec![false; 4096]),
            core_eval::EnvRef::new(core_eval::Env::Empty),
        ));
        let unused = std::rc::Rc::downgrade(&captured);
        let poison = core_eval::thunk(u32::MAX, captured.clone());
        drop(captured);
        let good = sparse_cube(6, BTreeSet::from([0usize, 50]));
        let guarded = run(
            cert,
            "Mpk.CSharp.Ordinary.Cube.D6.Mux",
            vec![V::Bit(true), good, poison],
        );
        let sealed = run(
            cert,
            "Mpk.CSharp.Ordinary.BoundaryUtf8.SealState",
            vec![guarded],
        );
        assert!(unused.upgrade().is_some());
        let mut first = sealed.clone();
        for _ in 0..6 {
            first = core_eval::apply(cert, first, V::Bit(false));
        }
        assert!(bit(first));
        assert!(
            unused.upgrade().is_none(),
            "sealing must release the unselected prior-state branch"
        );
        let mut bad_bit = sealed;
        for k in 0..6 {
            bad_bit = core_eval::apply(cert, bad_bit, V::Bit(50 & (1 << k) != 0));
        }
        assert!(bit(bad_bit));
    });
}
