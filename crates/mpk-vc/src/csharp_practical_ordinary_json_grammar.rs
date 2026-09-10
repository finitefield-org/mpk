//! Ordinary cursor/cell/depth composition for typed compound JSON parsing.
use super::hex_codecs::{call, truth};
use super::integer_format::{circuit_with_block_bits, define};
use super::temporal::literal;
use super::*;

const NAME: &str = "Mpk.CSharp.Ordinary.JsonGrammar";
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryJsonGrammarDefinition {
    /// C5 document length, C5 start, C5 nesting depth, Bool has children -> C7.
    /// A compound value contributes one cell before any children are read.
    pub begin_definition: String,
    pub cursor_definition: String,
    pub child_depth_definition: String,
    /// C7 previous header and C6 exact syntax result -> C7; no added cells.
    pub syntax_definition: String,
    /// Two C7 headers -> C7; add child cells with the inclusive 65,536 bound.
    pub child_definition: String,
    /// C24 document, C7 header, C3 outer ending -> C7; delimiter is not consumed.
    pub finish_definition: String,
}

fn circuit(b: &mut Builder, id: &str, c: Circuit, out: Word) -> R<String> {
    circuit_with_block_bits(b, &format!("{NAME}.{id}"), c, out, 7)
}
fn bounded(c: &mut Circuit, word: &[Bit], maximum: u128) -> Bit {
    c.lt(word, &literal(maximum + 1, word.len()), false)
}
pub(super) fn header_valid(c: &mut Circuit, head: &[Bit]) -> Bit {
    let mut valid = head[0];
    let end_valid = bounded(c, &head[2..34], 1_048_576);
    valid = c.and(valid, end_valid);
    let cells_valid = bounded(c, &head[34..66], TOTAL_VALUE_CELLS_MAX.into());
    valid = c.and(valid, cells_valid);
    let empty = c.equal(&head[34..66], &literal(0, 32));
    let nonempty = c.not(empty);
    valid = c.and(valid, nonempty);
    for bit in &head[66..] {
        let zero = c.not(*bit);
        valid = c.and(valid, zero);
    }
    valid
}
fn packet(c: &mut Circuit, valid: Bit, eof: Bit, end: Word, cells: Word) -> Word {
    let mut out = vec![T, eof];
    out.extend(end);
    out.extend(cells);
    out.resize(128, F);
    c.select(valid, &out, &vec![F; 128])
}

/// Join complete child headers. `exclude_container_cell` is reserved for a
/// JSON-only wrapper (Transition events array or anonymous map entry object),
/// whose own cell is absent from MonomorphicValue's recursive cell count.
/// Ordinary source products and standalone ordered_entry keep that cell.
/// The child must already have satisfied its own shape, depth and role bound.
pub(super) fn emit_child(b: &mut Builder, exclude_container_cell: bool) -> R<String> {
    let mut c = Circuit::new(&[128, 128]);
    let head = c.inputs[0].clone();
    let child = c.inputs[1].clone();
    let mut valid = header_valid(&mut c, &head);
    let child_valid = header_valid(&mut c, &child);
    valid = c.and(valid, child_valid);
    let not_end = c.not(head[1]);
    valid = c.and(valid, not_end);
    let advances = c.lt(&head[2..34], &child[2..34], false);
    valid = c.and(valid, advances);
    // Child validity still requires its own positive, bounded cell count.
    // Subtract the implicit container before adding: an empty container has
    // zero payload, and a valid total of 65,536 must never pass through a
    // prematurely rejected intermediate total of 65,537.
    let contribution = if exclude_container_cell {
        c.sub(&child[34..66], &literal(1, 32)).0
    } else {
        child[34..66].to_vec()
    };
    let (cells, overflow) = c.add(&head[34..66], &contribution, F);
    let no_overflow = c.not(overflow);
    valid = c.and(valid, no_overflow);
    let cells_valid = bounded(&mut c, &cells, TOTAL_VALUE_CELLS_MAX.into());
    valid = c.and(valid, cells_valid);
    let out = packet(&mut c, valid, child[1], child[2..34].to_vec(), cells);
    let name = if exclude_container_cell {
        "ChildContainerPayload"
    } else {
        "Child"
    };
    circuit(b, name, c, out)
}

pub(super) fn emit(
    b: &mut Builder,
    document: &OrdinaryBoundaryDocumentDefinition,
) -> R<OrdinaryJsonGrammarDefinition> {
    let mut c = Circuit::new(&[32, 32, 32, 1]);
    let length = c.inputs[0].clone();
    let start = c.inputs[1].clone();
    let depth = c.inputs[2].clone();
    let children = c.inputs[3][0];
    let mut valid = bounded(&mut c, &length, 1_048_576);
    let past = c.lt(&length, &start, false);
    let fits = c.not(past);
    valid = c.and(valid, fits);
    let depth_valid = bounded(&mut c, &depth, 32);
    valid = c.and(valid, depth_valid);
    let last_depth = c.equal(&depth, &literal(32, 32));
    let too_deep = c.and(last_depth, children);
    let child_depth_valid = c.not(too_deep);
    valid = c.and(valid, child_depth_valid);
    let eof = c.equal(&start, &length);
    let out = packet(&mut c, valid, eof, start, literal(1, 32));
    let begin_definition = circuit(b, "Begin", c, out)?;

    let c = Circuit::new(&[128]);
    let out = c.inputs[0][2..34].to_vec();
    let cursor_definition = circuit(b, "Cursor", c, out)?;
    let mut c = Circuit::new(&[32]);
    let depth = c.inputs[0].clone();
    let out = c.add(&depth, &literal(1, 32), F).0;
    let child_depth_definition = circuit(b, "ChildDepth", c, out)?;

    let mut c = Circuit::new(&[128, 64]);
    let head = c.inputs[0].clone();
    let syntax = c.inputs[1].clone();
    let mut valid = header_valid(&mut c, &head);
    let not_end = c.not(head[1]);
    valid = c.and(valid, not_end);
    valid = c.and(valid, syntax[0]);
    let advances = c.lt(&head[2..34], &syntax[2..34], false);
    valid = c.and(valid, advances);
    let end_valid = bounded(&mut c, &syntax[2..34], 1_048_576);
    valid = c.and(valid, end_valid);
    for bit in &syntax[34..] {
        let zero = c.not(*bit);
        valid = c.and(valid, zero);
    }
    let out = packet(
        &mut c,
        valid,
        syntax[1],
        syntax[2..34].to_vec(),
        head[34..66].to_vec(),
    );
    let syntax_definition = circuit(b, "Syntax", c, out)?;

    let child_definition = emit_child(b, false)?;

    let mut c = Circuit::new(&[128, 32, 8, 8]);
    let head = c.inputs[0].clone();
    let length = c.inputs[1].clone();
    let ending = c.inputs[2].clone();
    let next = c.inputs[3].clone();
    let mut valid = header_valid(&mut c, &head);
    let length_valid = bounded(&mut c, &length, 1_048_576);
    valid = c.and(valid, length_valid);
    let eof = c.equal(&head[2..34], &length);
    let eof_matches = c.equal(&[head[1]], &[eof]);
    valid = c.and(valid, eof_matches);
    let exists = c.lt(&head[2..34], &length, false);
    let at_eof = c.equal(&ending, &literal(0, 8));
    let mut accepts = c.and(at_eof, eof);
    for (kind, byte) in [(1, b','), (2, b']'), (3, b'}')] {
        let selected = c.equal(&ending, &literal(kind, 8));
        let equal = c.equal(&next, &literal(byte as u128, 8));
        let matched = c.and(selected, equal);
        let matched = c.and(matched, exists);
        accepts = c.or(accepts, matched);
    }
    valid = c.and(valid, accepts);
    let out = packet(
        &mut c,
        valid,
        eof,
        head[2..34].to_vec(),
        head[34..66].to_vec(),
    );
    let finish = circuit(b, "FinishPacket", c, out)?;
    let doc = b.var(2)?;
    let head = b.var(1)?;
    let ending = b.var(0)?;
    let cursor = call(b, &cursor_definition, vec![head])?;
    let length = call(b, &document.length_definition, vec![doc])?;
    let next = call(b, &document.read_byte_definition, vec![doc, cursor])?;
    let body = call(b, &finish, vec![head, length, ending, next])?;
    let finish_definition = format!("{NAME}.Finish");
    define(b, &finish_definition, &[24, 7, 3], 7, body)?;
    Ok(OrdinaryJsonGrammarDefinition {
        begin_definition,
        cursor_definition,
        child_depth_definition,
        syntax_definition,
        child_definition,
        finish_definition,
    })
}

pub(super) fn assemble(b: &mut Builder, value_depth: u32) -> R<String> {
    let padded = value_depth.max(7);
    let depth = padded + 1;
    let name = format!("{NAME}.Assemble.D{value_depth}");
    if b.globals.contains_key(&name) {
        return Ok(name);
    }
    let head = b.var(depth + 1)?;
    let value = b.var(depth)?;
    let valid = core_read(b, head, 0, 7)?;
    let leaf = |b: &mut Builder, value, width| -> R<u32> {
        let selectors = (0..width)
            .map(|i| b.var(padded - 1 - i))
            .collect::<R<Vec<_>>>()?;
        let mut leaf = b.app(value, selectors)?;
        for i in width..padded {
            let selector = b.var(padded - 1 - i)?;
            let zero = truth(b, false)?;
            leaf = core_mux(b, selector, zero, leaf)?;
        }
        Ok(leaf)
    };
    let h = leaf(b, head, 7)?;
    let v = leaf(b, value, value_depth)?;
    let role = b.var(padded)?;
    let body = core_mux(b, role, v, h)?;
    let zero = truth(b, false)?;
    let body = core_mux(b, valid, body, zero)?;
    let body = b.wrap_selectors(depth, body)?;
    define(b, &name, &[7, value_depth], depth, body)?;
    Ok(name)
}

#[cfg(test)]
mod tests {
    use super::super::super::test_eval::{apply, bit, run, sparse_cube, V};
    use super::*;
    fn word(n: u32, width: usize) -> V {
        V::Cube((0..width).map(|i| n & (1 << i) != 0).collect())
    }
    fn header(eof: bool, end: u32, cells: u32) -> Vec<bool> {
        let mut v = vec![false; 128];
        v[0] = true;
        v[1] = eof;
        for i in 0..32 {
            v[2 + i] = end & (1 << i) != 0;
            v[34 + i] = cells & (1 << i) != 0;
        }
        v
    }
    fn syntax(eof: bool, end: u32) -> V {
        V::Cube(header(eof, end, 0)[..64].to_vec())
    }
    fn at(cert: &Certificate, value: &V, depth: u32, index: usize) -> bool {
        let mut v = value.clone();
        for i in 0..depth {
            v = apply(cert, v, V::Bit(index & (1 << i) != 0));
        }
        bit(v)
    }
    fn check(cert: &Certificate, value: V, expected: Option<(bool, u32, u32)>) {
        let expected = expected.map_or_else(
            || vec![false; 128],
            |(eof, end, cells)| header(eof, end, cells),
        );
        for (i, want) in expected.into_iter().enumerate() {
            assert_eq!(at(cert, &value, 7, i), want, "header bit {i}");
        }
    }
    fn document(length: u32, at: usize, byte: u8) -> V {
        let mut ones = (0..32)
            .filter(|i| length & (1 << i) != 0)
            .map(|i| i << 19)
            .collect::<BTreeSet<_>>();
        for i in 0..8 {
            if byte & (1 << i) != 0 {
                ones.insert(1 | (at << 1) | (i << 21));
            }
        }
        sparse_cube(24, ones)
    }
    #[test]
    fn json_grammar_container_payload_counts_before_parent_bound() {
        let mut b = Builder::new().unwrap();
        let whole = emit_child(&mut b, false).unwrap();
        let payload = emit_child(&mut b, true).unwrap();
        let bytes = b.finish().unwrap();
        if let Some(path) = std::env::var_os("MPK_W09_JSON_CONTAINER_CERT_OUT") {
            std::fs::write(path, &bytes).unwrap();
        }
        let cert = decode_canonical_certificate(&bytes).unwrap();
        crate::csharp_practical_vc_model::validate_csharp_practical_certificate_structure(&cert)
            .unwrap();
        let mut cases = 0;
        // Semantic oracle: outer cells + all elements' cells. The separately
        // parsed JSON child carries one extra cell for its implicit container.
        for parent in [0_u32, 1, 2, 65_535, 65_536, 65_537, u32::MAX] {
            for elements in [0_u32, 1, 2, 65_534, 65_535, 65_536] {
                let child = elements + 1;
                for (name, excludes) in [(&whole, false), (&payload, true)] {
                    let total = u64::from(parent) + u64::from(elements) + u64::from(!excludes);
                    let good = (1..=65_536).contains(&parent) && child <= 65_536 && total <= 65_536;
                    check(
                        &cert,
                        run(
                            &cert,
                            name,
                            vec![
                                V::Cube(header(false, 1, parent)),
                                V::Cube(header(true, 2, child)),
                            ],
                        ),
                        good.then_some((true, 2, total as u32)),
                    );
                    cases += 1;
                }
            }
        }
        // Zero, oversized and overflowing child counts cannot be made valid
        // by subtraction. Both cursor and reserved-bit checks remain active.
        for cells in [0, 65_537, u32::MAX] {
            check(
                &cert,
                run(
                    &cert,
                    &payload,
                    vec![
                        V::Cube(header(false, 1, 1)),
                        V::Cube(header(true, 2, cells)),
                    ],
                ),
                None,
            );
            cases += 1;
        }
        for end in [0, 1, 1_048_576, 1_048_577, u32::MAX] {
            check(
                &cert,
                run(
                    &cert,
                    &payload,
                    vec![
                        V::Cube(header(false, 1, 65_536)),
                        V::Cube(header(false, end, 1)),
                    ],
                ),
                (end > 1 && end <= 1_048_576).then_some((false, end, 65_536)),
            );
            cases += 1;
        }
        for (which, positions) in [(0, vec![0, 1, 66, 127]), (1, vec![0, 66, 127])] {
            for position in positions {
                let mut heads = [header(false, 1, 65_536), header(true, 2, 1)];
                heads[which][position] = !heads[which][position];
                check(
                    &cert,
                    run(&cert, &payload, heads.into_iter().map(V::Cube).collect()),
                    None,
                );
                cases += 1;
            }
        }
        eprintln!("JSON container payload: {cases} complete C7 packets, including empty containers and exact cumulative limits");
    }

    #[test]
    fn json_grammar_cursor_cell_depth_and_delimiter_boundaries() {
        let mut b = Builder::new().unwrap();
        let doc = super::super::boundary_document::emit(&mut b).unwrap();
        let grammar = emit(&mut b, &doc).unwrap();
        let bytes = b.finish().unwrap();
        let cert = decode_canonical_certificate(&bytes).unwrap();
        crate::csharp_practical_vc_model::validate_csharp_practical_certificate_structure(&cert)
            .unwrap();
        let mut cases = 0;
        for length in [0, 2, 1_048_576, 1_048_577, u32::MAX] {
            for start in BTreeSet::from([0, length, length.saturating_add(1)]) {
                for depth in [0, 31, 32, 33, u32::MAX] {
                    for children in [false, true] {
                        let good = length <= 1_048_576
                            && start <= length
                            && depth <= 32
                            && (!children || depth < 32);
                        let value = run(
                            &cert,
                            &grammar.begin_definition,
                            vec![
                                word(length, 32),
                                word(start, 32),
                                word(depth, 32),
                                V::Bit(children),
                            ],
                        );
                        check(&cert, value, good.then_some((start == length, start, 1)));
                        cases += 1;
                    }
                }
            }
        }
        for (old_end, old_cells, new_end, new_cells, valid) in [
            (1, 1, 2, 1, true),
            (1, 1, 2, 65_535, true),
            (1, 65_535, 2, 1, true),
            (1, 1, 2, 65_536, false),
            (1, 0, 2, 1, false),
            (1, 1, 2, 0, false),
            (1, u32::MAX, 2, 1, false),
            (1, 1, 2, u32::MAX, false),
            (1, 1, 1, 1, false),
            (2, 1, 1, 1, false),
            (1_048_575, 1, 1_048_576, 1, true),
            (1, 1, 1_048_577, 1, false),
        ] {
            let value = run(
                &cert,
                &grammar.child_definition,
                vec![
                    V::Cube(header(false, old_end, old_cells)),
                    V::Cube(header(true, new_end, new_cells)),
                ],
            );
            check(
                &cert,
                value,
                valid.then_some((true, new_end, old_cells.wrapping_add(new_cells))),
            );
            cases += 1;
        }
        for position in [0, 1, 66, 127] {
            let mut old = header(false, 1, 1);
            old[position] = !old[position];
            check(
                &cert,
                run(
                    &cert,
                    &grammar.child_definition,
                    vec![V::Cube(old), V::Cube(header(true, 2, 1))],
                ),
                None,
            );
            cases += 1;
        }
        for position in [0, 66, 127] {
            let mut child = header(false, 2, 1);
            child[position] = !child[position];
            check(
                &cert,
                run(
                    &cert,
                    &grammar.child_definition,
                    vec![V::Cube(header(false, 1, 1)), V::Cube(child)],
                ),
                None,
            );
            cases += 1;
        }
        for end in [0, 1, 2, 1_048_576, 1_048_577, u32::MAX] {
            let value = run(
                &cert,
                &grammar.syntax_definition,
                vec![V::Cube(header(false, 1, 65_536)), syntax(true, end)],
            );
            check(
                &cert,
                value,
                (end > 1 && end <= 1_048_576).then_some((true, end, 65_536)),
            );
            cases += 1;
        }
        for position in [0, 34, 63] {
            let mut raw = header(false, 2, 0)[..64].to_vec();
            raw[position] = !raw[position];
            check(
                &cert,
                run(
                    &cert,
                    &grammar.syntax_definition,
                    vec![V::Cube(header(false, 1, 1)), V::Cube(raw)],
                ),
                None,
            );
            cases += 1;
        }
        check(
            &cert,
            run(
                &cert,
                &grammar.syntax_definition,
                vec![V::Cube(header(true, 1, 1)), syntax(true, 2)],
            ),
            None,
        );
        cases += 1;
        for ending in [0, 1, 2, 3, 4, 8, 16, 32, 64, 128, 255] {
            for byte in [b',', b']', b'}', b':', b' ', 0xac] {
                let valid = matches!((ending, byte), (1, b',') | (2, b']') | (3, b'}'));
                let value = run(
                    &cert,
                    &grammar.finish_definition,
                    vec![
                        document(2, 1, byte),
                        V::Cube(header(false, 1, 2)),
                        word(ending, 8),
                    ],
                );
                check(&cert, value, valid.then_some((false, 1, 2)));
                cases += 1;
            }
            let value = run(
                &cert,
                &grammar.finish_definition,
                vec![
                    document(1, 0, b','),
                    V::Cube(header(true, 1, 65_536)),
                    word(ending, 8),
                ],
            );
            check(&cert, value, (ending == 0).then_some((true, 1, 65_536)));
            cases += 1;
        }
        for (length, eof, end, cells) in [
            (2, true, 1, 1),
            (1, false, 1, 1),
            (0, false, 1, 1),
            (1_048_577, false, 1, 1),
            (2, false, 1, 0),
            (2, false, 1, 65_537),
        ] {
            let value = run(
                &cert,
                &grammar.finish_definition,
                vec![
                    document(length, 1, b','),
                    V::Cube(header(eof, end, cells)),
                    word(1, 8),
                ],
            );
            check(&cert, value, None);
            cases += 1;
        }
        eprintln!("JSON grammar: {cases} complete headers across cursor, EOF, cells, depth, padding and outer delimiter boundaries");
    }

    #[test]
    fn json_grammar_assembly_masks_all_roles_and_preserves_value_depth() {
        let mut b = Builder::new().unwrap();
        let mut defs = vec![];
        for depth in [0, 3, 7, 9, 19, 23] {
            defs.push((depth, assemble(&mut b, depth).unwrap()));
        }
        let bytes = b.finish().unwrap();
        let cert = decode_canonical_certificate(&bytes).unwrap();
        crate::csharp_practical_vc_model::validate_csharp_practical_certificate_structure(&cert)
            .unwrap();
        for (depth, name) in defs {
            let last = (1 << depth) - 1;
            let ones = BTreeSet::from([0, last]);
            for valid in [false, true] {
                let mut head = header(true, 0x12345, 7);
                head[0] = valid;
                let value = run(
                    &cert,
                    &name,
                    vec![V::Cube(head.clone()), sparse_cube(depth, ones.clone())],
                );
                let packet_depth = depth.max(7) + 1;
                for (i, bit) in head.into_iter().enumerate() {
                    assert_eq!(at(&cert, &value, packet_depth, i << 1), valid && bit);
                }
                for i in BTreeSet::from([0, last / 2, last, (1 << depth.max(7)) - 1]) {
                    assert_eq!(
                        at(&cert, &value, packet_depth, 1 | (i << 1)),
                        valid && ones.contains(&i)
                    );
                }
                if depth > 7 {
                    assert!(!at(&cert, &value, packet_depth, last << 1));
                }
            }
        }
    }
}
