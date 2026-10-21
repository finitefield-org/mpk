//! Untrusted closed-term composition with explicit proved theorem substitution.
//! Only compiled-in roots/names enter here; both cores still check every proof.
use super::*;
use serde_json::{json, Value};
use std::collections::BTreeSet;

const SCOPE: &str = "Mpk.Go.Practical.Word.SignedDivision.Prerequisite";
fn replacement(name: &str) -> Option<String> {
    for w in [8, 16, 32, 64] {
        for (old, new) in [
            ("Word.Order", "IsZeroNatural"),
            ("Word.Order", "DefinedDivisorPositive"),
            ("Word.SignedMagnitude", "ZeroSign"),
        ] {
            if name == format!("Mpk.Go.Practical.{old}.W{w}.{new}") {
                return Some(format!("Mpk.Go.Practical.Word.Zero.W{w}.{new}"));
            }
        }
    }
    None
}
pub(super) fn qualified(name: &str) -> String {
    let n = replacement(name).unwrap_or_else(|| name.into());
    for (old, new) in [
        ("Word.Order", "Order"),
        ("Word.Division", "Unsigned"),
        ("Word.SignedMagnitude", "Magnitude"),
        ("Word.SignedBounds", "Bounds"),
        ("Word.SignedProduct", "Arithmetic"),
        ("Word.Zero", "Zero"),
    ] {
        if let Some(suffix) = n.strip_prefix(&format!("Mpk.Go.Practical.{old}.")) {
            return format!("{SCOPE}.{new}.{suffix}");
        }
    }
    n
}
fn digest(v: Value) -> String {
    format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&v).expect("ordinary structural fingerprint"))
    )
}
fn fingerprints(c: &Certificate) -> BTreeMap<String, (String, String)> {
    let name = |i: u32| c.name_table[c.declarations[i as usize].name as usize].clone();
    let mut levels: Vec<String> = vec![];
    for n in &c.level_table {
        let v = match n {
            LevelNode::Zero => json!(["zero"]),
            LevelNode::Succ(i) => json!(["succ", levels[*i as usize]]),
            LevelNode::Max(a, b) => json!(["max", levels[*a as usize], levels[*b as usize]]),
            LevelNode::Param(i) => json!(["param", c.name_table[*i as usize]]),
        };
        levels.push(digest(v));
    }
    let mut terms: Vec<String> = vec![];
    for n in &c.term_table {
        let v = match n {
            TermNode::Sort(i) => json!(["sort", levels[*i as usize]]),
            TermNode::Var(i) => json!(["var", i]),
            TermNode::Const { global, levels: ls } => json!([
                "const",
                name(*global),
                ls.iter().map(|i| &levels[*i as usize]).collect::<Vec<_>>()
            ]),
            TermNode::App {
                function,
                arguments,
            } => json!([
                "app",
                terms[*function as usize],
                arguments
                    .iter()
                    .map(|i| &terms[*i as usize])
                    .collect::<Vec<_>>()
            ]),
            TermNode::Lam { ty, body } => {
                json!(["lam", terms[*ty as usize], terms[*body as usize]])
            }
            TermNode::Pi { ty, body } => json!(["pi", terms[*ty as usize], terms[*body as usize]]),
            TermNode::Let { ty, value, body } => json!([
                "let",
                terms[*ty as usize],
                terms[*value as usize],
                terms[*body as usize]
            ]),
        };
        terms.push(digest(v));
    }
    c.declarations
        .iter()
        .map(|d| {
            let (ty, v) = match &d.kind {
                DeclarationKind::Def {
                    ty,
                    value,
                    reducibility,
                } => (
                    *ty,
                    json!([
                        "definition",
                        terms[*ty as usize],
                        terms[*value as usize],
                        format!("{reducibility:?}")
                    ]),
                ),
                DeclarationKind::Theorem { ty, proof } => (
                    *ty,
                    json!(["theorem", terms[*ty as usize], terms[*proof as usize]]),
                ),
                DeclarationKind::Inductive { ty } => {
                    (*ty, json!(["inductive", terms[*ty as usize]]))
                }
                DeclarationKind::Constructor {
                    ty,
                    inductive,
                    generated,
                } => (
                    *ty,
                    json!([
                        "constructor",
                        terms[*ty as usize],
                        name(*inductive),
                        generated
                    ]),
                ),
                DeclarationKind::Recursor {
                    ty,
                    inductive,
                    generated,
                } => (
                    *ty,
                    json!(["recursor", terms[*ty as usize], name(*inductive), generated]),
                ),
                _ => unreachable!("closed ordinary inputs checked before structural comparison"),
            };
            (
                c.name_table[d.name as usize].clone(),
                (terms[ty as usize].clone(), digest(v)),
            )
        })
        .collect()
}
struct Composer<'a> {
    inputs: &'a [Certificate],
    signed_division_names: bool,
    builder: Builder,
    index: BTreeMap<String, (usize, usize)>,
    cloned: BTreeMap<String, u32>,
    visiting: BTreeSet<String>,
    terms: BTreeMap<(usize, u32), u32>,
    levels: BTreeMap<(usize, u32), u32>,
}
impl Composer<'_> {
    fn source_name(&self, name: &str) -> String {
        if self.signed_division_names {
            replacement(name).unwrap_or_else(|| name.into())
        } else {
            name.into()
        }
    }
    fn output_name(&self, name: &str) -> String {
        if self.signed_division_names {
            qualified(name)
        } else {
            name.into()
        }
    }
    fn dependency_order(&self, roots: &[String]) -> Result<Vec<String>> {
        let mut state: BTreeMap<String, u8> = BTreeMap::new();
        let mut output = vec![];
        for root in roots {
            let mut pending = vec![(self.source_name(root), false)];
            while let Some((name, exit)) = pending.pop() {
                if exit {
                    state.insert(name.clone(), 2);
                    output.push(name);
                    continue;
                }
                match state.get(&name) {
                    Some(2) => continue,
                    Some(1) => return Err(GoOrdinaryError::Linkage),
                    _ => {}
                }
                state.insert(name.clone(), 1);
                pending.push((name.clone(), true));
                let (s, i) = *self.index.get(&name).ok_or(GoOrdinaryError::Linkage)?;
                let mut nodes = match &self.inputs[s].declarations[i].kind {
                    DeclarationKind::Def { ty, value, .. } => vec![*ty, *value],
                    DeclarationKind::Theorem { ty, proof } => vec![*ty, *proof],
                    DeclarationKind::Inductive { ty }
                    | DeclarationKind::Constructor { ty, .. }
                    | DeclarationKind::Recursor { ty, .. } => vec![*ty],
                    _ => return Err(GoOrdinaryError::Foundation),
                };
                let mut dependencies = BTreeSet::new();
                let mut visited = BTreeSet::new();
                if let DeclarationKind::Constructor { inductive, .. }
                | DeclarationKind::Recursor { inductive, .. } =
                    self.inputs[s].declarations[i].kind
                {
                    dependencies.insert(self.global_name(s, inductive));
                }
                while let Some(n) = nodes.pop() {
                    if !visited.insert(n) {
                        continue;
                    }
                    match &self.inputs[s].term_table[n as usize] {
                        TermNode::Const { global, .. } => {
                            dependencies.insert(self.global_name(s, *global));
                        }
                        TermNode::App {
                            function,
                            arguments,
                        } => {
                            nodes.push(*function);
                            nodes.extend(arguments);
                        }
                        TermNode::Lam { ty, body } | TermNode::Pi { ty, body } => {
                            nodes.extend([*ty, *body])
                        }
                        TermNode::Let { ty, value, body } => nodes.extend([*ty, *value, *body]),
                        _ => {}
                    }
                }
                for dependency in dependencies.into_iter().rev() {
                    let dependency = self.source_name(&dependency);
                    if state.get(&dependency) != Some(&2) {
                        pending.push((dependency, false));
                    }
                }
            }
        }
        Ok(output)
    }
    fn global_name(&self, s: usize, i: u32) -> String {
        let c = &self.inputs[s];
        c.name_table[c.declarations[i as usize].name as usize].clone()
    }
    fn level(&mut self, s: usize, i: u32) -> Result<u32> {
        if let Some(id) = self.levels.get(&(s, i)) {
            return Ok(*id);
        }
        let node = match self.inputs[s].level_table[i as usize].clone() {
            LevelNode::Zero => LevelNode::Zero,
            LevelNode::Succ(n) => LevelNode::Succ(self.level(s, n)?),
            LevelNode::Max(a, b) => LevelNode::Max(self.level(s, a)?, self.level(s, b)?),
            LevelNode::Param(n) => {
                let name = self.inputs[s].name_table[n as usize].clone();
                let id = if let Some(id) = self
                    .builder
                    .certificate
                    .name_table
                    .iter()
                    .position(|v| *v == name)
                {
                    id as u32
                } else {
                    let id = self.builder.certificate.name_table.len() as u32;
                    self.builder.certificate.name_table.push(name);
                    id
                };
                LevelNode::Param(id)
            }
        };
        let id = if let Some(id) = self
            .builder
            .certificate
            .level_table
            .iter()
            .position(|n| *n == node)
        {
            id as u32
        } else {
            let id = self.builder.certificate.level_table.len() as u32;
            self.builder.certificate.level_table.push(node);
            id
        };
        self.levels.insert((s, i), id);
        Ok(id)
    }
    fn term(&mut self, s: usize, i: u32) -> Result<u32> {
        if let Some(id) = self.terms.get(&(s, i)) {
            return Ok(*id);
        }
        let node = match self.inputs[s].term_table[i as usize].clone() {
            TermNode::Sort(l) => TermNode::Sort(self.level(s, l)?),
            TermNode::Var(i) => TermNode::Var(i),
            TermNode::Const { global, levels } => {
                let global = self.declaration(&self.global_name(s, global))?;
                let levels = levels
                    .into_iter()
                    .map(|l| self.level(s, l))
                    .collect::<Result<Vec<_>>>()?;
                TermNode::Const { global, levels }
            }
            TermNode::App {
                function,
                arguments,
            } => {
                let function = self.term(s, function)?;
                let arguments = arguments
                    .into_iter()
                    .map(|a| self.term(s, a))
                    .collect::<Result<Vec<_>>>()?;
                TermNode::App {
                    function,
                    arguments,
                }
            }
            TermNode::Lam { ty, body } => TermNode::Lam {
                ty: self.term(s, ty)?,
                body: self.term(s, body)?,
            },
            TermNode::Pi { ty, body } => TermNode::Pi {
                ty: self.term(s, ty)?,
                body: self.term(s, body)?,
            },
            TermNode::Let { ty, value, body } => TermNode::Let {
                ty: self.term(s, ty)?,
                value: self.term(s, value)?,
                body: self.term(s, body)?,
            },
        };
        let id = self.builder.term(node)?;
        self.terms.insert((s, i), id);
        Ok(id)
    }
    fn declaration(&mut self, name: &str) -> Result<u32> {
        let name = self.source_name(name);
        if let Some(id) = self.cloned.get(&name) {
            return Ok(*id);
        }
        let out = self.output_name(&name);
        if let Some(id) = self.builder.globals.get(&out) {
            self.cloned.insert(name, *id);
            return Ok(*id);
        }
        if !self.visiting.insert(name.clone()) {
            return Err(GoOrdinaryError::Linkage);
        }
        let (s, i) = *self.index.get(&name).ok_or(GoOrdinaryError::Linkage)?;
        let kind = match self.inputs[s].declarations[i].kind.clone() {
            DeclarationKind::Def {
                ty,
                value,
                reducibility,
            } => DeclarationKind::Def {
                ty: self.term(s, ty)?,
                value: self.term(s, value)?,
                reducibility,
            },
            DeclarationKind::Theorem { ty, proof } => DeclarationKind::Theorem {
                ty: self.term(s, ty)?,
                proof: self.term(s, proof)?,
            },
            DeclarationKind::Inductive { ty } => DeclarationKind::Inductive {
                ty: self.term(s, ty)?,
            },
            DeclarationKind::Constructor {
                ty,
                inductive,
                generated,
            } => {
                let ty = self.term(s, ty)?;
                let inductive = self.declaration(&self.global_name(s, inductive))?;
                DeclarationKind::Constructor {
                    ty,
                    inductive,
                    generated,
                }
            }
            DeclarationKind::Recursor {
                ty,
                inductive,
                generated,
            } => {
                let ty = self.term(s, ty)?;
                let inductive = self.declaration(&self.global_name(s, inductive))?;
                DeclarationKind::Recursor {
                    ty,
                    inductive,
                    generated,
                }
            }
            _ => return Err(GoOrdinaryError::Foundation),
        };
        self.builder.declare(out.clone(), kind)?;
        let id = self.builder.globals[&out];
        self.visiting.remove(&name);
        self.cloned.insert(name, id);
        Ok(id)
    }
}
pub(super) fn signed_division_prerequisites(inputs: &[Certificate]) -> Result<Builder> {
    signed_division_prerequisites_with_roots(inputs, &[])
}
pub(super) fn signed_division_prerequisites_with_roots(
    inputs: &[Certificate],
    extra_roots: &[String],
) -> Result<Builder> {
    let mut index = BTreeMap::new();
    let mut seen: BTreeMap<String, (String, String)> = BTreeMap::new();
    for (s, c) in inputs.iter().enumerate() {
        if !c.imports.is_empty()
            || !c.proof_node_table.is_empty()
            || !c.theory_certificates.is_empty()
            || c.source_manifest.is_some()
            || c.axiom_report.summary.total_axiom_count != 0
            || c.declarations.iter().any(|d| {
                !matches!(
                    d.kind,
                    DeclarationKind::Def { .. }
                        | DeclarationKind::Theorem { .. }
                        | DeclarationKind::Inductive { .. }
                        | DeclarationKind::Constructor { .. }
                        | DeclarationKind::Recursor { .. }
                )
            })
        {
            return Err(GoOrdinaryError::Foundation);
        }
        let hashes = fingerprints(c);
        for (i, d) in c.declarations.iter().enumerate() {
            let name = c.name_table[d.name as usize].clone();
            if let Some(old) = seen.get(&name) {
                if *old != hashes[&name] {
                    return Err(GoOrdinaryError::Linkage);
                }
            } else {
                seen.insert(name.clone(), hashes[&name].clone());
                index.insert(name, (s, i));
            }
        }
    }
    for name in seen.keys() {
        if let Some(target) = replacement(name) {
            let a = seen.get(name).ok_or(GoOrdinaryError::Linkage)?;
            let b = seen.get(&target).ok_or(GoOrdinaryError::Linkage)?;
            if a.0 != b.0 {
                return Err(GoOrdinaryError::Linkage);
            }
            let (s, i) = index[name];
            let (t, j) = index[&target];
            if !matches!(
                inputs[s].declarations[i].kind,
                DeclarationKind::Theorem { .. }
            ) || !matches!(
                inputs[t].declarations[j].kind,
                DeclarationKind::Theorem { .. }
            ) {
                return Err(GoOrdinaryError::Foundation);
            }
        }
    }
    let builder = Builder::new()?;
    for (name, hash) in fingerprints(&builder.certificate) {
        if seen.get(&name) != Some(&hash) {
            return Err(GoOrdinaryError::Linkage);
        }
    }
    let mut composer = Composer {
        inputs,
        signed_division_names: true,
        builder,
        index,
        cloned: BTreeMap::new(),
        visiting: BTreeSet::new(),
        terms: BTreeMap::new(),
        levels: BTreeMap::new(),
    };
    let mut roots = vec![];
    for c in inputs {
        for d in &c.declarations {
            if matches!(
                d.kind,
                DeclarationKind::Inductive { .. }
                    | DeclarationKind::Constructor { .. }
                    | DeclarationKind::Recursor { .. }
            ) {
                roots.push(c.name_table[d.name as usize].clone());
            }
        }
    }
    for w in [8, 16, 32, 64] {
        roots.extend(
            ["UnsignedDivisionConservation", "UnsignedRemainderBound"]
                .map(|n| format!("Mpk.Go.Practical.Word.Division.W{w}.{n}")),
        );
        roots.extend(
            [
                "SignedMagnitudeEquivalent",
                "MagnitudeDivideDefined",
                "NegationValueSum",
            ]
            .map(|n| format!("Mpk.Go.Practical.Word.SignedMagnitude.W{w}.{n}")),
        );
        roots.extend(
            [
                "MagnitudeHalfBound",
                "MagnitudeHalfIsMinimum",
                "MagnitudeSignIsMinimum",
                "UnsignedHalfLess",
            ]
            .map(|n| format!("Mpk.Go.Practical.Word.SignedBounds.W{w}.{n}")),
        );
        roots.extend(
            ["SignedPositive", "SignedNegative"]
                .map(|n| format!("Mpk.Go.Practical.Word.Numeric.W{w}.{n}")),
        );
        roots.extend(["SignedDiv", "SignedRem"].map(|n| format!("Mpk.Go.Practical.Word.W{w}.{n}")));
    }
    for n in [
        "Make",
        "Negate",
        "FromNat",
        "Mul",
        "Add",
        "Equivalent",
        "PositiveOfMake",
        "NegativeOfMake",
        "PositiveOfFromNat",
        "NegativeOfFromNat",
        "PositiveOfNegate",
        "NegativeOfNegate",
        "PositiveOfAdd",
        "NegativeOfAdd",
        "PositiveOfMul",
        "NegativeOfMul",
    ] {
        roots.push(format!("Mpk.Go.Practical.Int.{n}"));
    }
    roots.extend_from_slice(extra_roots);
    for name in composer.dependency_order(&roots)? {
        composer.declaration(&name)?;
    }
    Ok(composer.builder)
}

/// Fixed private shift inputs retain original names and every conversion proof.
/// This untrusted assembly supplies no proof, source or import authority.
pub(super) fn word_shift_prerequisites(inputs: &[Certificate]) -> Result<Builder> {
    if inputs.len() != 3 {
        return Err(GoOrdinaryError::Foundation);
    }
    let mut index = BTreeMap::new();
    let mut seen = BTreeMap::new();
    for (s, c) in inputs.iter().enumerate() {
        if !c.imports.is_empty()
            || !c.proof_node_table.is_empty()
            || !c.theory_certificates.is_empty()
            || c.source_manifest.is_some()
            || c.axiom_report.summary.total_axiom_count != 0
            || c.declarations.iter().any(|d| {
                !matches!(
                    d.kind,
                    DeclarationKind::Def { .. }
                        | DeclarationKind::Theorem { .. }
                        | DeclarationKind::Inductive { .. }
                        | DeclarationKind::Constructor { .. }
                        | DeclarationKind::Recursor { .. }
                )
            })
        {
            return Err(GoOrdinaryError::Foundation);
        }
        let hashes = fingerprints(c);
        for (i, d) in c.declarations.iter().enumerate() {
            let name = c.name_table[d.name as usize].clone();
            if let Some(old) = seen.get(&name) {
                if old != &hashes[&name] {
                    return Err(GoOrdinaryError::Linkage);
                }
            } else {
                seen.insert(name.clone(), hashes[&name].clone());
                index.insert(name, (s, i));
            }
        }
    }
    let builder = Builder::new()?;
    for (name, hash) in fingerprints(&builder.certificate) {
        if seen.get(&name) != Some(&hash) {
            return Err(GoOrdinaryError::Linkage);
        }
    }
    let mut roots = inputs[0]
        .declarations
        .iter()
        .map(|d| inputs[0].name_table[d.name as usize].clone())
        .collect::<Vec<_>>();
    for w in [8_usize, 16, 32, 64] {
        for n in [
            "ShiftLeft",
            "ShiftRight",
            "SignedShiftRight",
            "Zero",
            "Ones",
            "Select",
            "SignedCountDefined",
        ] {
            roots.push(format!("Mpk.Go.Practical.Word.W{w}.{n}"));
        }
        for i in [0]
            .into_iter()
            .chain((0..w.trailing_zeros()).map(|j| 1_usize << j))
        {
            roots.push(format!(
                "Mpk.Go.Practical.Word.Numeric.W{w}.ShiftConservation{i}"
            ));
        }
        roots.push(format!("Mpk.Go.Practical.Word.Numeric.W{w}.ZeroValue"));
    }
    for n in [
        "PowerTwoAdd",
        "PowerTwoZero",
        "PowerTwoSucc",
        "W64.DigitsValue",
    ] {
        roots.push(format!("Mpk.Go.Practical.Word.Numeric.{n}"));
    }
    for d in 0..=6 {
        roots.push(format!("Mpk.Go.Practical.Word.Numeric.PowerTwoWeight{d}"));
    }
    let mut composer = Composer {
        inputs,
        signed_division_names: false,
        builder,
        index,
        cloned: BTreeMap::new(),
        visiting: BTreeSet::new(),
        terms: BTreeMap::new(),
        levels: BTreeMap::new(),
    };
    for name in composer.dependency_order(&roots)? {
        composer.declaration(&name)?;
    }
    Ok(composer.builder)
}

/// Fixed original dependency closure; fingerprints grant no checker authority.
pub(super) fn word_signed_order_prerequisites(inputs: &[Certificate]) -> Result<Builder> {
    if inputs.len() != 2
        || inputs[0].module != "Mpk.Go.Practical.WordOrderFoundation"
        || inputs[1].module != "Mpk.Go.Practical.WordShiftFoundation"
    {
        return Err(GoOrdinaryError::Foundation);
    }
    let mut index = BTreeMap::new();
    let mut seen = BTreeMap::new();
    for (source, certificate) in inputs.iter().enumerate() {
        if !certificate.imports.is_empty()
            || !certificate.proof_node_table.is_empty()
            || !certificate.theory_certificates.is_empty()
            || certificate.source_manifest.is_some()
            || certificate.axiom_report.summary.total_axiom_count != 0
            || certificate.declarations.iter().any(|d| {
                !matches!(
                    d.kind,
                    DeclarationKind::Def { .. }
                        | DeclarationKind::Theorem { .. }
                        | DeclarationKind::Inductive { .. }
                        | DeclarationKind::Constructor { .. }
                        | DeclarationKind::Recursor { .. }
                )
            })
        {
            return Err(GoOrdinaryError::Foundation);
        }
        let hashes = fingerprints(certificate);
        for (declaration, d) in certificate.declarations.iter().enumerate() {
            let name = certificate.name_table[d.name as usize].clone();
            if let Some(prior) = seen.get(&name) {
                if prior != &hashes[&name] {
                    return Err(GoOrdinaryError::Linkage);
                }
            } else {
                seen.insert(name.clone(), hashes[&name].clone());
                index.insert(name, (source, declaration));
            }
        }
    }
    let builder = Builder::new()?;
    for (name, hash) in fingerprints(&builder.certificate) {
        if seen.get(&name) != Some(&hash) {
            return Err(GoOrdinaryError::Linkage);
        }
    }
    let mut roots = inputs[0]
        .declarations
        .iter()
        .map(|d| inputs[0].name_table[d.name as usize].clone())
        .collect::<Vec<_>>();
    roots.push("Mpk.Go.Practical.Word.Shifts.RightStrictGap".into());
    let mut composer = Composer {
        inputs,
        signed_division_names: false,
        builder,
        index,
        cloned: BTreeMap::new(),
        visiting: BTreeSet::new(),
        terms: BTreeMap::new(),
        levels: BTreeMap::new(),
    };
    for name in composer.dependency_order(&roots)? {
        composer.declaration(&name)?;
    }
    Ok(composer.builder)
}

/// Exact finite-data and full64 count dependencies; roots are compiled in.
/// This assembly supplies no source/program, import or checked authority.
pub(super) fn word_index_prerequisites(inputs: &[Certificate]) -> Result<Builder> {
    if inputs.len() != 2
        || inputs[0].module != "Mpk.Go.Practical.WordShiftFoundation"
        || inputs[1].module != "Mpk.Go.Practical.FiniteVectorFoundation"
    {
        return Err(GoOrdinaryError::Foundation);
    }
    let mut index = BTreeMap::new();
    let mut seen = BTreeMap::new();
    for (s, c) in inputs.iter().enumerate() {
        if !c.imports.is_empty()
            || !c.proof_node_table.is_empty()
            || !c.theory_certificates.is_empty()
            || c.source_manifest.is_some()
            || c.axiom_report.summary.total_axiom_count != 0
            || c.declarations.iter().any(|d| {
                !matches!(
                    d.kind,
                    DeclarationKind::Def { .. }
                        | DeclarationKind::Theorem { .. }
                        | DeclarationKind::Inductive { .. }
                        | DeclarationKind::Constructor { .. }
                        | DeclarationKind::Recursor { .. }
                )
            })
        {
            return Err(GoOrdinaryError::Foundation);
        }
        let hashes = fingerprints(c);
        for (i, d) in c.declarations.iter().enumerate() {
            let name = c.name_table[d.name as usize].clone();
            if let Some(old) = seen.get(&name) {
                if old != &hashes[&name] {
                    return Err(GoOrdinaryError::Linkage);
                }
            } else {
                seen.insert(name.clone(), hashes[&name].clone());
                index.insert(name, (s, i));
            }
        }
    }
    let builder = Builder::new()?;
    for (name, hash) in fingerprints(&builder.certificate) {
        if seen.get(&name) != Some(&hash) {
            return Err(GoOrdinaryError::Linkage);
        }
    }
    let mut roots = inputs[1]
        .declarations
        .iter()
        .map(|d| inputs[1].name_table[d.name as usize].clone())
        .collect::<Vec<_>>();
    for name in [
        "CountRangeCompose",
        "CountPairZero",
        "WeightedBitZero",
        "PowerPositive",
        "CountBitComplement",
        "CountPrefixComplementStep",
        "CountComplementBound",
        "CountNoLessExtension",
        "CountPositiveDecomposition",
        "CountNonzeroPositive",
        "TwoMul",
    ] {
        roots.push(format!("Mpk.Go.Practical.Word.Shifts.{name}"));
    }
    for d in 0..=6 {
        for name in ["CountLowComplement", "CountLowBalance", "CountLowBound"] {
            roots.push(format!("Mpk.Go.Practical.Word.Shifts.{name}{d}"));
        }
    }
    for d in 3..=6 {
        for name in ["CountBalance", "CountHighZero", "CountSmallValue"] {
            roots.push(format!("Mpk.Go.Practical.Word.Shifts.{name}{d}"));
        }
    }
    for name in [
        "W64.DigitsValue",
        "PowerTwoZero",
        "PowerTwoSucc",
        "PowerTwoAdd",
    ] {
        roots.push(format!("Mpk.Go.Practical.Word.Numeric.{name}"));
    }
    let mut composer = Composer {
        inputs,
        signed_division_names: false,
        builder,
        index,
        cloned: BTreeMap::new(),
        visiting: BTreeSet::new(),
        terms: BTreeMap::new(),
        levels: BTreeMap::new(),
    };
    for name in composer.dependency_order(&roots)? {
        composer.declaration(&name)?;
    }
    Ok(composer.builder)
}

/// Actual string/byte mathematical prerequisites; original identities are exact.
pub(super) fn string_order_prerequisites(inputs: &[Certificate]) -> Result<Builder> {
    if inputs.len() != 2
        || inputs[0].module != "Mpk.Go.Practical.StringEqualityFoundation"
        || inputs[1].module != "Mpk.Go.Practical.WordOrderFoundation"
    {
        return Err(GoOrdinaryError::Foundation);
    }
    let mut index = BTreeMap::new();
    let mut seen = BTreeMap::new();
    for (source, certificate) in inputs.iter().enumerate() {
        if !certificate.imports.is_empty()
            || !certificate.proof_node_table.is_empty()
            || !certificate.theory_certificates.is_empty()
            || certificate.source_manifest.is_some()
            || certificate.axiom_report.summary.total_axiom_count != 0
            || certificate.declarations.iter().any(|d| {
                !matches!(
                    d.kind,
                    DeclarationKind::Def { .. }
                        | DeclarationKind::Theorem { .. }
                        | DeclarationKind::Inductive { .. }
                        | DeclarationKind::Constructor { .. }
                        | DeclarationKind::Recursor { .. }
                )
            })
        {
            return Err(GoOrdinaryError::Foundation);
        }
        let hashes = fingerprints(certificate);
        for (declaration, d) in certificate.declarations.iter().enumerate() {
            let name = certificate.name_table[d.name as usize].clone();
            if let Some(prior) = seen.get(&name) {
                if prior != &hashes[&name] {
                    return Err(GoOrdinaryError::Linkage);
                }
            } else {
                seen.insert(name.clone(), hashes[&name].clone());
                index.insert(name, (source, declaration));
            }
        }
    }
    let builder = Builder::new()?;
    for (name, hash) in fingerprints(&builder.certificate) {
        if seen.get(&name) != Some(&hash) {
            return Err(GoOrdinaryError::Linkage);
        }
    }
    let mut roots = inputs[0]
        .declarations
        .iter()
        .map(|d| inputs[0].name_table[d.name as usize].clone())
        .collect::<Vec<_>>();
    // Complete old string component plus exact closed byte-comparison roots.
    // No caller-selected roots or source/profile authority enters this union.
    for name in [
        "W8.EqualNatural",
        "W8.UnsignedLessNatural",
        "W8.UnsignedLessEqualNatural",
    ] {
        roots.push(format!("Mpk.Go.Practical.Word.Order.{name}"));
    }
    roots.push("Mpk.Go.Practical.Word.Numeric.ValueCongruence3".into());
    let mut composer = Composer {
        inputs,
        signed_division_names: false,
        builder,
        index,
        cloned: BTreeMap::new(),
        visiting: BTreeSet::new(),
        terms: BTreeMap::new(),
        levels: BTreeMap::new(),
    };
    for name in composer.dependency_order(&roots)? {
        composer.declaration(&name)?;
    }
    Ok(composer.builder)
}
