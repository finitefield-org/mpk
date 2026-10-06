//! Closed Boolean and cube proofs that preserve staged state computation.
//! Every rewrite is an ordinary equality; selector variables remain universal.
use super::ownership_proofs::{equality, publish_theorem};
use super::*;
const EQ: &str = "Std.Eq";
const REFL: &str = "Std.Eq.refl";

struct Normalizer<'a> {
    b: &'a mut Builder,
    heads: BTreeMap<u32, u32>,
    substitutions: BTreeMap<(u32, u32, u32), u32>,
    shifts: BTreeMap<(u32, u32, u32), u32>,
    proofs: BTreeMap<u32, (bool, u32, u32)>,
    word_values: BTreeMap<u32, u32>,
    staged_words: BTreeMap<(u32, u32), (u32, u32)>,
    staged_values: BTreeSet<(u32, u32)>,
    word_heads: BTreeMap<u32, u32>,
    word_busy: BTreeSet<(u32, u32)>,
}
impl Normalizer<'_> {
    // Builder.app preserves the frozen insertion order and nested application
    // syntax. Flatten locally before deciding which closed operation to prove.
    fn flat_application(&mut self, mut function: u32, mut arguments: Vec<u32>) -> R<u32> {
        while let TermNode::App {
            function: inner,
            arguments: prefix,
        } = self.b.c.term_table[function as usize].clone()
        {
            let mut all = prefix;
            all.append(&mut arguments);
            arguments = all;
            function = inner;
        }
        self.b.app(function, arguments)
    }

    fn word_transport(&mut self, _function: u32) -> R<String> {
        let name = format!("{PREFIX}.ClosedCubeProof.BoolTransport.Generic");
        if self.b.globals.contains_key(&name) {
            return Ok(name);
        }
        let cube = self.b.cube(5)?;
        let boolean = self.b.boolean;
        // a,b,c,d : C5; a=c; b=d; r : Bool; WordEqual(c,d)=r.
        let a = self.b.var(3)?;
        let c = self.b.var(1)?;
        let left_ty = call(self.b, EQ, vec![cube, a, c])?;
        let b = self.b.var(3)?;
        let d = self.b.var(1)?;
        let right_ty = call(self.b, EQ, vec![cube, b, d])?;
        let c = self.b.var(4)?;
        let d = self.b.var(3)?;
        let result = self.b.var(0)?;
        let function = self.b.var(7)?;
        let comparison = self.b.app(function, vec![c, d])?;
        let comparison_ty = call(self.b, EQ, vec![boolean, comparison, result])?;
        let function_tail = self.b.pi(cube, boolean)?;
        let function_type = self.b.pi(cube, function_tail)?;
        let binders = [
            function_type,
            cube,
            cube,
            cube,
            cube,
            left_ty,
            right_ty,
            boolean,
            comparison_ty,
        ];
        let a = self.b.var(7)?;
        let b = self.b.var(6)?;
        let c = self.b.var(5)?;
        let d = self.b.var(4)?;
        let left_eq = self.b.var(3)?;
        let right_eq = self.b.var(2)?;
        let result = self.b.var(1)?;
        let comparison_eq = self.b.var(0)?;
        let variable = self.b.var(0)?;
        let inner_b = self.b.var(7)?;
        let function = self.b.var(9)?;
        let body = self.b.app(function, vec![variable, inner_b])?;
        let left_fun = self.b.lam(cube, body)?;
        let left = call(
            self.b,
            "Std.Eq.congr",
            vec![cube, boolean, left_fun, a, c, left_eq],
        )?;
        let inner_c = self.b.var(6)?;
        let body = self.b.app(function, vec![inner_c, variable])?;
        let right_fun = self.b.lam(cube, body)?;
        let right = call(
            self.b,
            "Std.Eq.congr",
            vec![cube, boolean, right_fun, b, d, right_eq],
        )?;
        let function = self.b.var(8)?;
        let original = self.b.app(function, vec![a, b])?;
        let middle = self.b.app(function, vec![c, b])?;
        let canonical = self.b.app(function, vec![c, d])?;
        let joined = call(
            self.b,
            "Std.Eq.trans",
            vec![boolean, original, middle, canonical, left, right],
        )?;
        let mut proof = call(
            self.b,
            "Std.Eq.trans",
            vec![boolean, original, canonical, result, joined, comparison_eq],
        )?;
        let mut ty = call(self.b, EQ, vec![boolean, original, result])?;
        for binder in binders.into_iter().rev() {
            proof = self.b.lam(binder, proof)?;
            ty = self.b.pi(binder, ty)?;
        }
        self.publish(&name, ty, proof)?;
        Ok(name)
    }
    fn bool_step(&mut self, selected: bool) -> R<String> {
        let name = format!("{PREFIX}.ClosedCubeProof.BoolStep.{selected}");
        if self.b.globals.contains_key(&name) {
            return Ok(name);
        }
        // For arbitrary branches n/y, condition c and result r:
        // c = selected -> branch = r -> rec n y c = r.
        // Sharing this ordinary theorem avoids re-inferring a fresh lambda
        // containing both complete concrete branches at every reduction step.
        let boolean = self.b.boolean;
        let chosen = bit(self.b, selected)?;
        let condition = self.b.var(1)?;
        let condition_ty = call(self.b, EQ, vec![boolean, condition, chosen])?;
        let branch = self.b.var(if selected { 3 } else { 4 })?;
        let result = self.b.var(1)?;
        let branch_ty = call(self.b, EQ, vec![boolean, branch, result])?;
        let binders = [boolean, boolean, boolean, boolean, condition_ty, branch_ty];
        let no = self.b.var(5)?;
        let yes = self.b.var(4)?;
        let condition = self.b.var(3)?;
        let result = self.b.var(2)?;
        let condition_proof = self.b.var(1)?;
        let branch_proof = self.b.var(0)?;
        let expression = call(self.b, "Std.Bool.rec", vec![no, yes, condition])?;
        let branch = if selected { yes } else { no };
        let inner_no = self.b.var(6)?;
        let inner_yes = self.b.var(5)?;
        let variable = self.b.var(0)?;
        let body = call(self.b, "Std.Bool.rec", vec![inner_no, inner_yes, variable])?;
        let fun = self.b.lam(boolean, body)?;
        let congr = call(
            self.b,
            "Std.Eq.congr",
            vec![boolean, boolean, fun, condition, chosen, condition_proof],
        )?;
        let mut proof = call(
            self.b,
            "Std.Eq.trans",
            vec![boolean, expression, branch, result, congr, branch_proof],
        )?;
        let mut ty = call(self.b, EQ, vec![boolean, expression, result])?;
        for binder in binders.into_iter().rev() {
            proof = self.b.lam(binder, proof)?;
            ty = self.b.pi(binder, ty)?;
        }
        self.publish(&name, ty, proof)?;
        Ok(name)
    }
    fn shift(&mut self, t: u32, amount: u32, cutoff: u32) -> R<u32> {
        if amount == 0 {
            return Ok(t);
        }
        if let Some(r) = self.shifts.get(&(t, amount, cutoff)) {
            return Ok(*r);
        }
        let node = match self.b.c.term_table[t as usize].clone() {
            TermNode::Var(i) if i >= cutoff => {
                TermNode::Var(i.checked_add(amount).ok_or(OrdinaryCarrierError::Limit)?)
            }
            TermNode::App {
                function,
                arguments,
            } => TermNode::App {
                function: self.shift(function, amount, cutoff)?,
                arguments: arguments
                    .into_iter()
                    .map(|a| self.shift(a, amount, cutoff))
                    .collect::<R<Vec<_>>>()?,
            },
            TermNode::Lam { ty, body } => TermNode::Lam {
                ty: self.shift(ty, amount, cutoff)?,
                body: self.shift(body, amount, cutoff + 1)?,
            },
            TermNode::Pi { ty, body } => TermNode::Pi {
                ty: self.shift(ty, amount, cutoff)?,
                body: self.shift(body, amount, cutoff + 1)?,
            },
            TermNode::Let { ty, value, body } => TermNode::Let {
                ty: self.shift(ty, amount, cutoff)?,
                value: self.shift(value, amount, cutoff)?,
                body: self.shift(body, amount, cutoff + 1)?,
            },
            node => node,
        };
        let result = self.b.term(node)?;
        self.shifts.insert((t, amount, cutoff), result);
        Ok(result)
    }
    fn substitute(&mut self, t: u32, depth: u32, value: u32) -> R<u32> {
        if let Some(r) = self.substitutions.get(&(t, depth, value)) {
            return Ok(*r);
        }
        let node = self.b.c.term_table[t as usize].clone();
        let node = match node {
            TermNode::Var(i) if i == depth => {
                // Word-wide congruence also opens applications under the five
                // selector binders. Preserve free selectors without capture.
                let shifted = self.shift(value, depth, 0)?;
                self.substitutions.insert((t, depth, value), shifted);
                return Ok(shifted);
            }
            TermNode::Var(i) if i > depth => TermNode::Var(i - 1),
            TermNode::App {
                function,
                arguments,
            } => TermNode::App {
                function: self.substitute(function, depth, value)?,
                arguments: arguments
                    .into_iter()
                    .map(|a| self.substitute(a, depth, value))
                    .collect::<R<Vec<_>>>()?,
            },
            TermNode::Lam { ty, body } => TermNode::Lam {
                ty: self.substitute(ty, depth, value)?,
                body: self.substitute(body, depth + 1, value)?,
            },
            TermNode::Pi { ty, body } => TermNode::Pi {
                ty: self.substitute(ty, depth, value)?,
                body: self.substitute(body, depth + 1, value)?,
            },
            TermNode::Let { ty, value: v, body } => TermNode::Let {
                ty: self.substitute(ty, depth, value)?,
                value: self.substitute(v, depth, value)?,
                body: self.substitute(body, depth + 1, value)?,
            },
            node => node,
        };
        let result = self.b.term(node)?;
        self.substitutions.insert((t, depth, value), result);
        Ok(result)
    }
    fn head(&mut self, t: u32) -> R<u32> {
        if let TermNode::App {
            function,
            arguments,
        } = self.b.c.term_table[t as usize].clone()
        {
            if matches!(self.b.c.term_table[function as usize], TermNode::App { .. }) {
                let flat = self.flat_application(function, arguments)?;
                return self.head(flat);
            }
        }
        if self.word_call(t).is_some() {
            return Ok(t);
        }
        if let Some(r) = self.heads.get(&t) {
            return Ok(*r);
        }
        let node = self.b.c.term_table[t as usize].clone();
        let result = match node {
            TermNode::Const { global, .. } => match self.b.c.declarations[global as usize].kind {
                DeclarationKind::Def {
                    value,
                    reducibility: DefinitionReducibility::Reducible,
                    ..
                } => self.head(value)?,
                _ => t,
            },
            TermNode::Let { ty, .. } if self.cube_depth(ty).is_some_and(|d| d > 0) => t,
            TermNode::Let { value, body, .. } => {
                let reduced = self.substitute(body, 0, value)?;
                self.head(reduced)?
            }
            TermNode::App {
                function,
                arguments,
            } => {
                let function = self.head(function)?;
                if let TermNode::Lam { body, .. } = self.b.c.term_table[function as usize] {
                    let reduced = self.substitute(body, 0, arguments[0])?;
                    let reduced = self.b.app(reduced, arguments[1..].to_vec())?;
                    self.head(reduced)?
                } else if let TermNode::Let { ty, value, body } =
                    self.b.c.term_table[function as usize]
                {
                    let args = arguments
                        .into_iter()
                        .map(|a| self.shift(a, 1, 0))
                        .collect::<R<Vec<_>>>()?;
                    let body = self.b.app(body, args)?;
                    self.b.term(TermNode::Let { ty, value, body })?
                } else if matches!(self.b.c.term_table[function as usize], TermNode::Const {global,..} if self.b.c.name_table[self.b.c.declarations[global as usize].name as usize] == "Std.Bool.rec")
                    && arguments.len() == 3
                {
                    // Only a literal constructor major is an ordinary iota
                    // conversion. Computed conditions still need explicit proof.
                    let branch = match self.b.c.term_table[arguments[2] as usize] {
                        TermNode::Const { global, .. } => match self.b.c.name_table
                            [self.b.c.declarations[global as usize].name as usize]
                            .as_str()
                        {
                            "Std.Bool.false" => Some(arguments[0]),
                            "Std.Bool.true" => Some(arguments[1]),
                            _ => None,
                        },
                        _ => None,
                    };
                    if let Some(branch) = branch {
                        self.head(branch)?
                    } else {
                        self.b.app(function, arguments)?
                    }
                } else {
                    self.b.app(function, arguments)?
                }
            }
            _ => t,
        };
        self.heads.insert(t, result);
        Ok(result)
    }
    fn word_call(&self, t: u32) -> Option<(u32, Vec<u32>)> {
        let TermNode::App {
            function,
            arguments,
        } = &self.b.c.term_table[t as usize]
        else {
            return None;
        };
        let TermNode::Const { global, .. } = self.b.c.term_table[*function as usize] else {
            return None;
        };
        let d = &self.b.c.declarations[global as usize];
        let name = &self.b.c.name_table[d.name as usize];
        let selected = name == &format!("{PREFIX}.OwnershipFlow.WordEqual")
            || (name == &format!("{PREFIX}.OrderedFold.Less")
                || name.starts_with(&format!("{PREFIX}.StructuralScalar."))
                    && name.ends_with(".Result"));
        if !selected || arguments.len() != 2 {
            return None;
        }
        let DeclarationKind::Def { ty, .. } = d.kind else {
            return None;
        };
        let TermNode::Pi {
            ty: left,
            body: tail,
        } = self.b.c.term_table[ty as usize]
        else {
            return None;
        };
        let TermNode::Pi {
            ty: right,
            body: result,
        } = self.b.c.term_table[tail as usize]
        else {
            return None;
        };
        if !self.cube_type(left, 5) || !self.cube_type(right, 5) || !self.cube_type(result, 0) {
            return None;
        }
        Some((*function, arguments.clone()))
    }
    fn normalize(&mut self, term: u32) -> R<(bool, u32, u32)> {
        let h = self.head(term)?;
        if let Some(&result) = self.proofs.get(&h) {
            return Ok(result);
        }
        if let Some((function, arguments)) = self.word_call(h) {
            let (left, left_proof) = self.closed_word(arguments[0])?;
            let (right, right_proof) = self.closed_word(arguments[1])?;
            let canonical = vec![left, right];
            let result = if canonical == arguments {
                let TermNode::Const { global, .. } = self.b.c.term_table[function as usize] else {
                    return Err(OrdinaryCarrierError::Linkage);
                };
                let DeclarationKind::Def { value, .. } =
                    self.b.c.declarations[global as usize].kind
                else {
                    return Err(OrdinaryCarrierError::Linkage);
                };
                let expanded = self.b.app(value, canonical.clone())?;
                if self.scalar_result(function) {
                    self.closed_result(function, arguments.clone())?
                } else {
                    self.normalize(expanded)?
                }
            } else {
                let shared = self.b.app(function, canonical)?;
                let (value, reduced, _) = self.normalize(shared)?;
                let expected = bit(self.b, value)?;
                let lemma = self.word_transport(function)?;
                let proof = call(
                    self.b,
                    &lemma,
                    vec![
                        function,
                        arguments[0],
                        arguments[1],
                        left,
                        right,
                        left_proof,
                        right_proof,
                        expected,
                        reduced,
                    ],
                )?;
                let ty = call(self.b, EQ, vec![self.b.boolean, h, expected])?;
                let name = format!(
                    "{PREFIX}.ClosedCubeProof.BooleanWord.N{}",
                    self.b.c.declarations.len()
                );
                self.publish(&name, ty, proof)?;
                (value, self.b.constant(&name)?, 1)
            };
            self.proofs.insert(h, result);
            return Ok(result);
        }
        let result = match self.b.c.term_table[h as usize].clone() {
            TermNode::Const { global, .. } => {
                let name =
                    &self.b.c.name_table[self.b.c.declarations[global as usize].name as usize];
                let value = match name.as_str() {
                    "Std.Bool.true" => true,
                    "Std.Bool.false" => false,
                    _ => return Err(OrdinaryCarrierError::Linkage),
                };
                (value, call(self.b, REFL, vec![self.b.boolean, h])?, 1)
            }
            TermNode::App {
                function,
                arguments,
            } => {
                let TermNode::Const { global, .. } = self.b.c.term_table[function as usize] else {
                    return Err(OrdinaryCarrierError::Linkage);
                };
                if self.b.c.name_table[self.b.c.declarations[global as usize].name as usize]
                    != "Std.Bool.rec"
                    || arguments.len() != 3
                {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                let (selected, condition_proof, condition_cost) = self.normalize(arguments[2])?;
                let (value, branch_proof, branch_cost) =
                    self.normalize(arguments[usize::from(selected)])?;
                let expected = bit(self.b, value)?;
                let lemma = self.bool_step(selected)?;
                let proof = call(
                    self.b,
                    &lemma,
                    vec![
                        arguments[0],
                        arguments[1],
                        arguments[2],
                        expected,
                        condition_proof,
                        branch_proof,
                    ],
                )?;
                (
                    value,
                    proof,
                    condition_cost.saturating_add(branch_cost).saturating_add(1),
                )
            }
            TermNode::Let {
                ty,
                value: state,
                body,
            } => {
                let depth = self
                    .cube_depth(ty)
                    .filter(|d| *d > 0)
                    .ok_or(OrdinaryCarrierError::Linkage)?;
                self.closed_read(depth, h, ty, state, body)?
            }
            _ => return Err(OrdinaryCarrierError::Linkage),
        };
        let (value, proof, cost) = result;
        let result = if cost >= 4 {
            let expected = bit(self.b, value)?;
            let ty = call(self.b, EQ, vec![self.b.boolean, h, expected])?;
            let name = format!(
                "{PREFIX}.ClosedCubeProof.Step.N{}",
                self.b.c.declarations.len()
            );
            self.publish(&name, ty, proof)?;
            (value, self.b.constant(&name)?, 1)
        } else {
            result
        };
        self.proofs.insert(h, result);
        Ok(result)
    }
    fn theorem(&mut self, definition: &str, expected: bool) -> R<String> {
        let value = self.b.constant(definition)?;
        let (actual, proof, _) = self.normalize(value)?;
        if actual != expected {
            return Err(OrdinaryCarrierError::Linkage);
        }
        let expected = bit(self.b, expected)?;
        let ty = call(self.b, EQ, vec![self.b.boolean, value, expected])?;
        let name = format!("{definition}.Proof");
        self.publish(&name, ty, proof)?;
        Ok(name)
    }
    fn publish(&mut self, name: &str, ty: u32, proof: u32) -> R<()> {
        publish_theorem(self.b, name, ty, proof)
    }
}
impl Normalizer<'_> {
    fn cube_type(&self, t: u32, depth: u32) -> bool {
        if depth == 0 {
            return matches!(self.b.c.term_table[t as usize], TermNode::Const { global, .. } if self.b.c.name_table[self.b.c.declarations[global as usize].name as usize] == "Std.Bool");
        }
        matches!(self.b.c.term_table[t as usize], TermNode::Pi { ty, body } if self.cube_type(ty,0) && self.cube_type(body,depth-1))
    }
    fn closed_term(&self, t: u32, depth: u32, seen: &mut BTreeSet<(u32, u32)>) -> bool {
        if !seen.insert((t, depth)) {
            return true;
        }
        match &self.b.c.term_table[t as usize] {
            TermNode::Var(i) => *i < depth,
            TermNode::Const { .. } | TermNode::Sort(_) => true,
            TermNode::App {
                function,
                arguments,
            } => {
                self.closed_term(*function, depth, seen)
                    && arguments.iter().all(|a| self.closed_term(*a, depth, seen))
            }
            TermNode::Lam { ty, body } | TermNode::Pi { ty, body } => {
                self.closed_term(*ty, depth, seen) && self.closed_term(*body, depth + 1, seen)
            }
            TermNode::Let { ty, value, body } => {
                self.closed_term(*ty, depth, seen)
                    && self.closed_term(*value, depth, seen)
                    && self.closed_term(*body, depth + 1, seen)
            }
        }
    }
    fn word_special(&self, depth: u32, t: u32) -> Option<(u32, Vec<u32>, bool)> {
        let TermNode::App {
            function,
            arguments,
        } = &self.b.c.term_table[t as usize]
        else {
            return None;
        };
        let TermNode::Const { global, .. } = self.b.c.term_table[*function as usize] else {
            return None;
        };
        let name = &self.b.c.name_table[self.b.c.declarations[global as usize].name as usize];
        if name == &format!("{PREFIX}.Cube.D{depth}.Mux") && arguments.len() == 3 {
            Some((*function, arguments.clone(), true))
        } else if depth == 5
            && name.starts_with(&format!("{PREFIX}.DomainCount."))
            && name.ends_with(".Result")
            && arguments.len() == 2
        {
            Some((*function, arguments.clone(), false))
        } else {
            None
        }
    }
    // Preserve Let sharing and complete C5 operations until their arguments
    // have their own checked equalities. This is only a syntactic head walk.
    fn word_head(&mut self, t: u32) -> R<u32> {
        if let TermNode::App {
            function,
            arguments,
        } = self.b.c.term_table[t as usize].clone()
        {
            if matches!(self.b.c.term_table[function as usize], TermNode::App { .. }) {
                let flat = self.flat_application(function, arguments)?;
                return self.word_head(flat);
            }
        }
        if self.state_call(t).is_some()
            || (1..=18).any(|d| self.word_special(d, t).is_some())
            || self.compose_call(t).is_some()
        {
            return Ok(t);
        }
        if let Some(&h) = self.word_heads.get(&t) {
            return Ok(h);
        }
        let h = match self.b.c.term_table[t as usize].clone() {
            TermNode::Const { global, .. } => match self.b.c.declarations[global as usize].kind {
                DeclarationKind::Def {
                    value,
                    reducibility: DefinitionReducibility::Reducible,
                    ..
                } => self.word_head(value)?,
                _ => t,
            },
            TermNode::App {
                function,
                arguments,
            } => {
                let f = self.word_head(function)?;
                if let TermNode::Lam { body, .. } = self.b.c.term_table[f as usize] {
                    let body = self.substitute(body, 0, arguments[0])?;
                    let body = self.b.app(body, arguments[1..].to_vec())?;
                    self.word_head(body)?
                } else {
                    self.b.app(f, arguments)?
                }
            }
            _ => t,
        };
        self.word_heads.insert(t, h);
        Ok(h)
    }
    // A literal word may have any unchanged Bool-selector decision expression;
    // no selector is assigned a value and no extensional equality is assumed.
    fn selector_expression(&mut self, depth: u32, t: u32, seen: &mut BTreeSet<u32>) -> bool {
        if !seen.insert(t) {
            return true;
        }
        let h = match self.word_head(t) {
            Ok(h) => h,
            Err(_) => return false,
        };
        if h != t {
            return self.selector_expression(depth, h, seen);
        }
        match self.b.c.term_table[t as usize].clone() {
            TermNode::Var(i) => i < depth,
            TermNode::Const { global, .. } => matches!(
                self.b.c.name_table[self.b.c.declarations[global as usize].name as usize].as_str(),
                "Std.Bool.false" | "Std.Bool.true"
            ),
            TermNode::App {
                function,
                arguments,
            } => {
                let TermNode::Const { global, .. } = self.b.c.term_table[function as usize] else {
                    return false;
                };
                let name =
                    &self.b.c.name_table[self.b.c.declarations[global as usize].name as usize];
                if name == "Std.Bool.rec" && arguments.len() == 3 {
                    arguments
                        .iter()
                        .all(|a| self.selector_expression(depth, *a, seen))
                } else if matches!(
                    name.as_str(),
                    "Std.Bool.not" | "Std.Bool.and" | "Std.Bool.or" | "Std.Bool.xor"
                ) {
                    match self.head(t) {
                        Ok(h) if h != t => self.selector_expression(depth, h, seen),
                        _ => false,
                    }
                } else {
                    false
                }
            }
            _ => false,
        }
    }
    fn literal_closed_word(&mut self, depth: u32, h: u32) -> R<Option<u32>> {
        let mut body = h;
        for _ in 0..depth {
            let TermNode::Lam { ty, body: next } = self.b.c.term_table[body as usize] else {
                return Ok(None);
            };
            if !self.cube_type(ty, 0) {
                return Ok(None);
            }
            body = next;
        }
        if !self.selector_expression(depth, body, &mut BTreeSet::new()) {
            return Ok(None);
        }
        if let Some(&v) = self.word_values.get(&h) {
            return Ok(Some(v));
        }
        let name = format!(
            "{PREFIX}.ClosedCubeProof.Literal.N{}",
            self.b.c.declarations.len()
        );
        let cube = self.b.cube(depth)?;
        self.b.define(&name, cube, h)?;
        let v = self.b.constant(&name)?;
        self.word_values.insert(h, v);
        self.staged_values.insert((depth, v));
        Ok(Some(v))
    }
    fn word_equality(
        &mut self,
        depth: u32,
        left: u32,
        right: u32,
        proof: u32,
        part: &str,
    ) -> R<u32> {
        let cube = self.b.cube(depth)?;
        let ty = call(self.b, EQ, vec![cube, left, right])?;
        let name = format!(
            "{PREFIX}.ClosedCubeProof.{part}.N{}",
            self.b.c.declarations.len()
        );
        self.publish(&name, ty, proof)?;
        self.b.constant(&name)
    }
    fn word_return_transport(&mut self, depth: u32, function: u32) -> R<String> {
        let name = format!("{PREFIX}.ClosedCubeProof.CubeTransport.D{depth}.F{function}");
        if self.b.globals.contains_key(&name) {
            return Ok(name);
        }
        let cube = self.b.cube(depth)?;
        let a = self.b.var(3)?;
        let c = self.b.var(1)?;
        let left_ty = call(self.b, EQ, vec![cube, a, c])?;
        let b = self.b.var(3)?;
        let d = self.b.var(1)?;
        let right_ty = call(self.b, EQ, vec![cube, b, d])?;
        let binders = [cube, cube, cube, cube, left_ty, right_ty];
        let a = self.b.var(5)?;
        let b = self.b.var(4)?;
        let c = self.b.var(3)?;
        let d = self.b.var(2)?;
        let left_eq = self.b.var(1)?;
        let right_eq = self.b.var(0)?;
        let variable = self.b.var(0)?;
        let inner_b = self.b.var(5)?;
        let body = self.b.app(function, vec![variable, inner_b])?;
        let left_fun = self.b.lam(cube, body)?;
        let left = call(
            self.b,
            "Std.Eq.congr",
            vec![cube, cube, left_fun, a, c, left_eq],
        )?;
        let inner_c = self.b.var(4)?;
        let body = self.b.app(function, vec![inner_c, variable])?;
        let right_fun = self.b.lam(cube, body)?;
        let right = call(
            self.b,
            "Std.Eq.congr",
            vec![cube, cube, right_fun, b, d, right_eq],
        )?;
        let original = self.b.app(function, vec![a, b])?;
        let middle = self.b.app(function, vec![c, b])?;
        let canonical = self.b.app(function, vec![c, d])?;
        let mut proof = call(
            self.b,
            "Std.Eq.trans",
            vec![cube, original, middle, canonical, left, right],
        )?;
        let mut ty = call(self.b, EQ, vec![cube, original, canonical])?;
        for binder in binders.into_iter().rev() {
            proof = self.b.lam(binder, proof)?;
            ty = self.b.pi(binder, ty)?;
        }
        self.publish(&name, ty, proof)?;
        Ok(name)
    }
    fn closed_cube(&mut self, depth: u32, term: u32) -> R<(u32, u32)> {
        if !self.closed_term(term, 0, &mut BTreeSet::new()) {
            return Err(OrdinaryCarrierError::Linkage);
        }
        if self.staged_values.contains(&(depth, term)) {
            let cube = self.b.cube(depth)?;
            return Ok((term, call(self.b, REFL, vec![cube, term])?));
        }
        if let Some(&r) = self.staged_words.get(&(depth, term)) {
            return Ok(r);
        }
        if !self.word_busy.insert((depth, term)) {
            return Err(OrdinaryCarrierError::Linkage);
        }
        let h = self.word_head(term)?;
        let cube = self.b.cube(depth)?;
        let result = if let Some((function, arguments, source_value)) = self.state_call(h) {
            self.closed_state(depth, term, function, arguments, source_value)?
        } else if let Some((compose_depth, function, arguments)) = self.compose_call(h) {
            if compose_depth != depth {
                return Err(OrdinaryCarrierError::Linkage);
            }
            let fs = self.b.app(arguments[0], vec![arguments[2]])?;
            let (middle, p) = self.closed_cube(depth, fs)?;
            let gm = self.b.app(arguments[1], vec![middle])?;
            let (canonical, q) = self.closed_cube(depth, gm)?;
            let lemma = self.compose_transport(depth, function)?;
            let proof = call(
                self.b,
                &lemma,
                vec![
                    arguments[0],
                    arguments[1],
                    arguments[2],
                    middle,
                    canonical,
                    p,
                    q,
                ],
            )?;
            (
                canonical,
                self.word_equality(depth, term, canonical, proof, "Composition")?,
            )
        } else if let Some(v) = self.literal_closed_word(depth, h)? {
            let p = call(self.b, REFL, vec![cube, v])?;
            (v, self.word_equality(depth, term, v, p, "LiteralProof")?)
        } else if let Some((function, arguments, is_mux)) = self.word_special(depth, h) {
            if is_mux {
                let (selected, condition_proof, _) = self.normalize(arguments[0])?;
                let chosen = bit(self.b, selected)?;
                let selected_word = arguments[if selected { 1 } else { 2 }];
                let (canonical, rest) = self.closed_cube(depth, selected_word)?;
                let parameter = self.b.var(0)?;
                let yes = self.shift(arguments[1], 1, 0)?;
                let no = self.shift(arguments[2], 1, 0)?;
                let body = self.b.app(function, vec![parameter, yes, no])?;
                let fun = self.b.lam(self.b.boolean, body)?;
                let rewritten = self
                    .b
                    .app(function, vec![chosen, arguments[1], arguments[2]])?;
                let first = call(
                    self.b,
                    "Std.Eq.congr",
                    vec![
                        self.b.boolean,
                        cube,
                        fun,
                        arguments[0],
                        chosen,
                        condition_proof,
                    ],
                )?;
                let first = self.word_equality(depth, term, rewritten, first, "MuxCondition")?;
                let word = self.b.var(depth)?;
                let selectors = (0..depth)
                    .rev()
                    .map(|i| self.b.var(i))
                    .collect::<R<Vec<_>>>()?;
                let body = self.b.app(word, selectors)?;
                let body = self.b.wrap_selectors(depth, body)?;
                let eta = self.b.lam(cube, body)?;
                let second = call(
                    self.b,
                    "Std.Eq.congr",
                    vec![cube, cube, eta, selected_word, canonical, rest],
                )?;
                let second =
                    self.word_equality(depth, rewritten, canonical, second, "MuxResult")?;
                let p = call(
                    self.b,
                    "Std.Eq.trans",
                    vec![cube, term, rewritten, canonical, first, second],
                )?;
                (
                    canonical,
                    self.word_equality(depth, term, canonical, p, "Mux")?,
                )
            } else {
                let (left, left_p) = self.closed_cube(depth, arguments[0])?;
                let (right, right_p) = self.closed_cube(depth, arguments[1])?;
                let computed = self.b.app(function, vec![left, right])?;
                let transport = self.word_return_transport(depth, function)?;
                let first = call(
                    self.b,
                    &transport,
                    vec![arguments[0], arguments[1], left, right, left_p, right_p],
                )?;
                let first =
                    self.word_equality(depth, term, computed, first, "AdditionArguments")?;
                let TermNode::Const { global, .. } = self.b.c.term_table[function as usize] else {
                    return Err(OrdinaryCarrierError::Linkage);
                };
                let DeclarationKind::Def { value, .. } =
                    self.b.c.declarations[global as usize].kind
                else {
                    return Err(OrdinaryCarrierError::Linkage);
                };
                let expanded = self.b.app(value, vec![left, right])?;
                let (canonical, rest) = self.closed_cube(depth, expanded)?;
                let p = call(
                    self.b,
                    "Std.Eq.trans",
                    vec![cube, term, computed, canonical, first, rest],
                )?;
                (
                    canonical,
                    self.word_equality(depth, term, canonical, p, "Addition")?,
                )
            }
        } else if let TermNode::Let { ty, value, body } = self.b.c.term_table[h as usize] {
            if !self.cube_type(ty, 0) {
                let value_depth = self
                    .cube_depth(ty)
                    .filter(|d| *d > 0)
                    .ok_or(OrdinaryCarrierError::Linkage)?;
                let (literal, state_proof) = self.closed_cube(value_depth, value)?;
                let fun = self.b.lam(ty, body)?;
                let rewritten = self.b.app(fun, vec![literal])?;
                let first = call(
                    self.b,
                    "Std.Eq.congr",
                    vec![ty, cube, fun, value, literal, state_proof],
                )?;
                let first = self.word_equality(depth, term, rewritten, first, "StateCondition")?;
                let reduced = self.substitute(body, 0, literal)?;
                let (canonical, rest) = self.closed_cube(depth, reduced)?;
                let proof = call(
                    self.b,
                    "Std.Eq.trans",
                    vec![cube, term, rewritten, canonical, first, rest],
                )?;
                (
                    canonical,
                    self.word_equality(depth, term, canonical, proof, "StateResult")?,
                )
            } else {
                if !self.closed_term(value, 0, &mut BTreeSet::new()) {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                let (selected, gate_proof, _) = self.normalize(value)?;
                let literal = bit(self.b, selected)?;
                let fun = self.b.lam(ty, body)?;
                let fun_ty = self.b.pi(ty, cube)?;
                let name = format!(
                    "{PREFIX}.ClosedCubeProof.GateFunction.N{}",
                    self.b.c.declarations.len()
                );
                self.b.define(&name, fun_ty, fun)?;
                let fun = self.b.constant(&name)?;
                let rewritten = self.b.app(fun, vec![literal])?;
                let first = call(
                    self.b,
                    "Std.Eq.congr",
                    vec![ty, cube, fun, value, literal, gate_proof],
                )?;
                let first = self.word_equality(depth, term, rewritten, first, "GateCondition")?;
                let reduced = self.substitute(body, 0, literal)?;
                let (canonical, rest) = self.closed_cube(depth, reduced)?;
                let p = call(
                    self.b,
                    "Std.Eq.trans",
                    vec![cube, term, rewritten, canonical, first, rest],
                )?;
                (
                    canonical,
                    self.word_equality(depth, term, canonical, p, "GateResult")?,
                )
            }
        } else {
            return Err(OrdinaryCarrierError::Linkage);
        };
        self.word_busy.remove(&(depth, term));
        self.staged_words.insert((depth, term), result);
        Ok(result)
    }
}

impl Normalizer<'_> {
    fn cube_depth(&self, ty: u32) -> Option<u32> {
        (0..=18).find(|d| self.cube_type(ty, *d))
    }
    fn closed_word(&mut self, term: u32) -> R<(u32, u32)> {
        self.closed_cube(5, term)
    }
    fn compose_call(&self, term: u32) -> Option<(u32, u32, Vec<u32>)> {
        let TermNode::App {
            function,
            arguments,
        } = &self.b.c.term_table[term as usize]
        else {
            return None;
        };
        if arguments.len() != 3 {
            return None;
        }
        let TermNode::Const { global, .. } = self.b.c.term_table[*function as usize] else {
            return None;
        };
        let name = &self.b.c.name_table[self.b.c.declarations[global as usize].name as usize];
        let depth = name
            .strip_prefix(&format!("{PREFIX}.Cube.D"))?
            .strip_suffix(".Compose")?
            .parse()
            .ok()?;
        Some((depth, *function, arguments.clone()))
    }
    fn compose_transport(&mut self, depth: u32, function: u32) -> R<String> {
        let name = format!("{PREFIX}.ClosedCubeProof.Compose.D{depth}");
        if self.b.globals.contains_key(&name) {
            return Ok(name);
        }
        let cube = self.b.cube(depth)?;
        let transformer = self.b.pi(cube, cube)?;
        let f = self.b.var(4)?;
        let s = self.b.var(2)?;
        let m = self.b.var(1)?;
        let fs = self.b.app(f, vec![s])?;
        let p_ty = call(self.b, EQ, vec![cube, fs, m])?;
        let g = self.b.var(4)?;
        let m = self.b.var(2)?;
        let t = self.b.var(1)?;
        let gm = self.b.app(g, vec![m])?;
        let q_ty = call(self.b, EQ, vec![cube, gm, t])?;
        let binders = [transformer, transformer, cube, cube, cube, p_ty, q_ty];
        let f = self.b.var(6)?;
        let g = self.b.var(5)?;
        let s = self.b.var(4)?;
        let m = self.b.var(3)?;
        let t = self.b.var(2)?;
        let p = self.b.var(1)?;
        let q = self.b.var(0)?;
        let fs = self.b.app(f, vec![s])?;
        let middle = self.b.app(g, vec![m])?;
        let composed = self.b.app(function, vec![f, g, s])?;
        let congr = call(self.b, "Std.Eq.congr", vec![cube, cube, g, fs, m, p])?;
        let mut proof = call(
            self.b,
            "Std.Eq.trans",
            vec![cube, composed, middle, t, congr, q],
        )?;
        let mut ty = call(self.b, EQ, vec![cube, composed, t])?;
        for binder in binders.into_iter().rev() {
            proof = self.b.lam(binder, proof)?;
            ty = self.b.pi(binder, ty)?;
        }
        self.publish(&name, ty, proof)?;
        Ok(name)
    }
}
impl Normalizer<'_> {
    fn state_call(&self, t: u32) -> Option<(u32, Vec<u32>, u32)> {
        let TermNode::App {
            function,
            arguments,
        } = &self.b.c.term_table[t as usize]
        else {
            return None;
        };
        if arguments.len() != 2 {
            return None;
        }
        let TermNode::Const { global, .. } = self.b.c.term_table[*function as usize] else {
            return None;
        };
        let d = &self.b.c.declarations[global as usize];
        let name = &self.b.c.name_table[d.name as usize];
        if !name.starts_with(&format!("{PREFIX}.StructuralScalar.")) || !name.ends_with(".State") {
            return None;
        }
        let DeclarationKind::Def { ty, value, .. } = d.kind else {
            return None;
        };
        let TermNode::Pi {
            ty: left,
            body: tail,
        } = self.b.c.term_table[ty as usize]
        else {
            return None;
        };
        let TermNode::Pi { ty: right, .. } = self.b.c.term_table[tail as usize] else {
            return None;
        };
        if !self.cube_type(left, 5) || !self.cube_type(right, 5) {
            return None;
        }
        Some((*function, arguments.clone(), value))
    }
    fn state_function_transport(&mut self, depth: u32) -> R<String> {
        let name = format!("{PREFIX}.ClosedCubeProof.StateFunctionTransport.D{depth}");
        if self.b.globals.contains_key(&name) {
            return Ok(name);
        }
        let input = self.b.cube(5)?;
        let cube = self.b.cube(depth)?;
        let tail = self.b.pi(input, cube)?;
        let function_type = self.b.pi(input, tail)?;
        let f = self.b.var(4)?;
        let g = self.b.var(3)?;
        let p_ty = call(self.b, EQ, vec![function_type, f, g])?;
        let g = self.b.var(4)?;
        let a = self.b.var(3)?;
        let b = self.b.var(2)?;
        let r = self.b.var(1)?;
        let ga = self.b.app(g, vec![a, b])?;
        let q_ty = call(self.b, EQ, vec![cube, ga, r])?;
        let binders = [function_type, function_type, input, input, cube, p_ty, q_ty];
        let f = self.b.var(6)?;
        let g = self.b.var(5)?;
        let a = self.b.var(4)?;
        let b = self.b.var(3)?;
        let r = self.b.var(2)?;
        let p = self.b.var(1)?;
        let q = self.b.var(0)?;
        let fa = self.b.app(f, vec![a, b])?;
        let ga = self.b.app(g, vec![a, b])?;
        let inner_f = self.b.var(0)?;
        let inner_a = self.b.var(5)?;
        let inner_b = self.b.var(4)?;
        let body = self.b.app(inner_f, vec![inner_a, inner_b])?;
        let apply = self.b.lam(function_type, body)?;
        let first = call(
            self.b,
            "Std.Eq.congr",
            vec![function_type, cube, apply, f, g, p],
        )?;
        let mut proof = call(self.b, "Std.Eq.trans", vec![cube, fa, ga, r, first, q])?;
        let mut ty = call(self.b, EQ, vec![cube, fa, r])?;
        for binder in binders.into_iter().rev() {
            proof = self.b.lam(binder, proof)?;
            ty = self.b.pi(binder, ty)?;
        }
        self.publish(&name, ty, proof)?;
        Ok(name)
    }
    fn closed_state(
        &mut self,
        depth: u32,
        term: u32,
        function: u32,
        args: Vec<u32>,
        source: u32,
    ) -> R<(u32, u32)> {
        let TermNode::Lam {
            ty: first_ty,
            body: tail,
        } = self.b.c.term_table[source as usize]
        else {
            return Err(OrdinaryCarrierError::Linkage);
        };
        let TermNode::Lam {
            ty: second_ty,
            body,
        } = self.b.c.term_table[tail as usize]
        else {
            return Err(OrdinaryCarrierError::Linkage);
        };
        let TermNode::App {
            function: composed,
            arguments: initial,
        } = self.b.c.term_table[body as usize].clone()
        else {
            return Err(OrdinaryCarrierError::Linkage);
        };
        if initial.len() != 1 {
            return Err(OrdinaryCarrierError::Linkage);
        }
        let TermNode::App {
            function: compose,
            arguments: steps,
        } = self.b.c.term_table[composed as usize].clone()
        else {
            return Err(OrdinaryCarrierError::Linkage);
        };
        if steps.len() != 2
            || steps
                .iter()
                .any(|s| !self.closed_term(*s, 0, &mut BTreeSet::new()))
        {
            return Err(OrdinaryCarrierError::Linkage);
        }
        let input = self.b.cube(5)?;
        let cube = self.b.cube(depth)?;
        let transformer = self.b.pi(cube, cube)?;
        let f = self.b.var(3)?;
        let g = self.b.var(2)?;
        let generic_composed = self.b.term(TermNode::App {
            function: compose,
            arguments: vec![f, g],
        })?;
        let generic_body = self.b.term(TermNode::App {
            function: generic_composed,
            arguments: initial,
        })?;
        let generic_tail = self.b.lam(second_ty, generic_body)?;
        let template = self.b.lam(first_ty, generic_tail)?;
        // Generalize the original lambda syntax while the block functions are
        // variables. Its beta conversion then ends at a neutral application.
        let before_p = self.shift(template, 3, 0)?;
        let a = self.b.var(2)?;
        let b = self.b.var(1)?;
        let r = self.b.var(0)?;
        let app = self.b.app(before_p, vec![a, b])?;
        let expanded = self.word_head(app)?;
        let p_ty = call(self.b, EQ, vec![cube, expanded, r])?;
        let binders = [transformer, transformer, input, input, cube, p_ty];
        let template = self.shift(template, 4, 0)?;
        let a = self.b.var(3)?;
        let b = self.b.var(2)?;
        let r = self.b.var(1)?;
        let proof = self.b.var(0)?;
        let app = self.b.app(template, vec![a, b])?;
        let mut beta_ty = call(self.b, EQ, vec![cube, app, r])?;
        let mut beta_proof = proof;
        for binder in binders.into_iter().rev() {
            beta_ty = self.b.pi(binder, beta_ty)?;
            beta_proof = self.b.lam(binder, beta_proof)?;
        }
        let beta_name = format!("{PREFIX}.ClosedCubeProof.StateBeta.F{function}");
        if !self.b.globals.contains_key(&beta_name) {
            self.publish(&beta_name, beta_ty, beta_proof)?;
        }
        let applied = self.b.app(source, args.clone())?;
        let h = self.word_head(applied)?;
        let (canonical, p) = self.closed_cube(depth, h)?;
        let beta = call(
            self.b,
            &beta_name,
            vec![steps[0], steps[1], args[0], args[1], canonical, p],
        )?;
        let tail = self.b.pi(input, cube)?;
        let function_type = self.b.pi(input, tail)?;
        let unfold_ty = call(self.b, EQ, vec![function_type, function, source])?;
        let unfold_proof = call(self.b, REFL, vec![function_type, function])?;
        let unfold_name = format!("{PREFIX}.ClosedCubeProof.StateUnfold.F{function}");
        if !self.b.globals.contains_key(&unfold_name) {
            self.publish(&unfold_name, unfold_ty, unfold_proof)?;
        }
        let unfold = self.b.constant(&unfold_name)?;
        let transport = self.state_function_transport(depth)?;
        let proof = call(
            self.b,
            &transport,
            vec![function, source, args[0], args[1], canonical, unfold, beta],
        )?;
        Ok((
            canonical,
            self.word_equality(depth, term, canonical, proof, "State")?,
        ))
    }
}
impl Normalizer<'_> {
    fn read_transport(&mut self, depth: u32, cube: u32, body: u32) -> R<String> {
        if !self.closed_term(body, 1, &mut BTreeSet::new()) {
            return Err(OrdinaryCarrierError::Linkage);
        }
        let name = format!("{PREFIX}.ClosedCubeProof.Read.D{depth}.B{body}");
        if self.b.globals.contains_key(&name) {
            return Ok(name);
        }
        let boolean = self.b.boolean;
        let s = self.b.var(2)?;
        let c = self.b.var(1)?;
        let p_ty = call(self.b, EQ, vec![cube, s, c])?;
        let c = self.b.var(2)?;
        let r = self.b.var(1)?;
        let value = self.substitute(body, 0, c)?;
        let q_ty = call(self.b, EQ, vec![boolean, value, r])?;
        let binders = [cube, cube, boolean, p_ty, q_ty];
        let s = self.b.var(4)?;
        let c = self.b.var(3)?;
        let r = self.b.var(2)?;
        let p = self.b.var(1)?;
        let q = self.b.var(0)?;
        let left = self.b.term(TermNode::Let {
            ty: cube,
            value: s,
            body,
        })?;
        let fun = self.b.lam(cube, body)?;
        let middle = self.substitute(body, 0, c)?;
        let first = call(self.b, "Std.Eq.congr", vec![cube, boolean, fun, s, c, p])?;
        let mut proof = call(
            self.b,
            "Std.Eq.trans",
            vec![boolean, left, middle, r, first, q],
        )?;
        let mut ty = call(self.b, EQ, vec![boolean, left, r])?;
        for binder in binders.into_iter().rev() {
            proof = self.b.lam(binder, proof)?;
            ty = self.b.pi(binder, ty)?;
        }
        self.publish(&name, ty, proof)?;
        Ok(name)
    }
    fn closed_read(
        &mut self,
        depth: u32,
        _term: u32,
        cube: u32,
        state: u32,
        body: u32,
    ) -> R<(bool, u32, u32)> {
        let (canonical, state_proof) = self.closed_cube(depth, state)?;
        let reduced = self.substitute(body, 0, canonical)?;
        let (value, rest, _) = self.normalize(reduced)?;
        let expected = bit(self.b, value)?;
        let lemma = self.read_transport(depth, cube, body)?;
        let proof = call(
            self.b,
            &lemma,
            vec![state, canonical, expected, state_proof, rest],
        )?;
        Ok((value, proof, 4))
    }
    fn scalar_result(&self, function: u32) -> bool {
        let TermNode::Const { global, .. } = self.b.c.term_table[function as usize] else {
            return false;
        };
        let name = &self.b.c.name_table[self.b.c.declarations[global as usize].name as usize];
        name.starts_with(&format!("{PREFIX}.StructuralScalar.")) && name.ends_with(".Result")
    }
    fn closed_result(&mut self, function: u32, args: Vec<u32>) -> R<(bool, u32, u32)> {
        let TermNode::Const { global, .. } = self.b.c.term_table[function as usize] else {
            return Err(OrdinaryCarrierError::Linkage);
        };
        let DeclarationKind::Def {
            ty: function_type,
            value: source,
            ..
        } = self.b.c.declarations[global as usize].kind
        else {
            return Err(OrdinaryCarrierError::Linkage);
        };
        let TermNode::Lam {
            ty: left_ty,
            body: tail,
        } = self.b.c.term_table[source as usize]
        else {
            return Err(OrdinaryCarrierError::Linkage);
        };
        let TermNode::Lam { ty: right_ty, body } = self.b.c.term_table[tail as usize] else {
            return Err(OrdinaryCarrierError::Linkage);
        };
        let TermNode::Let {
            ty: cube,
            value: state,
            body: read,
        } = self.b.c.term_table[body as usize]
        else {
            return Err(OrdinaryCarrierError::Linkage);
        };
        let TermNode::App {
            function: state_fn,
            arguments: state_args,
        } = self.b.c.term_table[state as usize].clone()
        else {
            return Err(OrdinaryCarrierError::Linkage);
        };
        if state_args.len() != 2
            || !self.closed_term(state_fn, 0, &mut BTreeSet::new())
            || !self.closed_term(read, 1, &mut BTreeSet::new())
        {
            return Err(OrdinaryCarrierError::Linkage);
        }
        let input = self.b.cube(5)?;
        let boolean = self.b.boolean;
        let tail = self.b.pi(input, cube)?;
        let state_type = self.b.pi(input, tail)?;
        let f = self.b.var(2)?;
        let generic_state = self.b.term(TermNode::App {
            function: f,
            arguments: state_args,
        })?;
        let generic_body = self.b.term(TermNode::Let {
            ty: cube,
            value: generic_state,
            body: read,
        })?;
        let tail = self.b.lam(right_ty, generic_body)?;
        let template = self.b.lam(left_ty, tail)?;
        let fn_before_p = self.shift(template, 3, 0)?;
        let a = self.b.var(2)?;
        let b = self.b.var(1)?;
        let r = self.b.var(0)?;
        let app = self.b.app(fn_before_p, vec![a, b])?;
        let expanded = self.head(app)?;
        let p_ty = call(self.b, EQ, vec![boolean, expanded, r])?;
        let binders = [state_type, input, input, boolean, p_ty];
        let template = self.shift(template, 4, 0)?;
        let a = self.b.var(3)?;
        let b = self.b.var(2)?;
        let r = self.b.var(1)?;
        let p = self.b.var(0)?;
        let app = self.b.app(template, vec![a, b])?;
        let mut beta_ty = call(self.b, EQ, vec![boolean, app, r])?;
        let mut beta_proof = p;
        for binder in binders.into_iter().rev() {
            beta_ty = self.b.pi(binder, beta_ty)?;
            beta_proof = self.b.lam(binder, beta_proof)?;
        }
        let beta_name = format!("{PREFIX}.ClosedCubeProof.ResultBeta.F{function}");
        if !self.b.globals.contains_key(&beta_name) {
            self.publish(&beta_name, beta_ty, beta_proof)?;
        }
        let applied = self.b.app(source, args.clone())?;
        let (value, p, _) = self.normalize(applied)?;
        let expected = bit(self.b, value)?;
        let beta = call(
            self.b,
            &beta_name,
            vec![state_fn, args[0], args[1], expected, p],
        )?;
        let unfold_ty = call(self.b, EQ, vec![function_type, function, source])?;
        let unfold_proof = call(self.b, REFL, vec![function_type, function])?;
        let unfold_name = format!("{PREFIX}.ClosedCubeProof.ResultUnfold.F{function}");
        if !self.b.globals.contains_key(&unfold_name) {
            self.publish(&unfold_name, unfold_ty, unfold_proof)?;
        }
        let unfold = self.b.constant(&unfold_name)?;
        let transport = self.state_function_transport(0)?;
        let proof = call(
            self.b,
            &transport,
            vec![function, source, args[0], args[1], expected, unfold, beta],
        )?;
        Ok((value, proof, 1))
    }
}

/// Generate exact equality proofs for closed original goals. A certificate
/// remains a candidate until it passes the unchanged source-free checkers.
pub(super) fn prove_closed_boolean_definitions(
    b: &mut Builder,
    definitions: &[String],
) -> R<Vec<String>> {
    if definitions.is_empty() {
        return Ok(vec![]);
    }
    equality(b)?;
    let mut n = Normalizer {
        b,
        heads: BTreeMap::new(),
        substitutions: BTreeMap::new(),
        shifts: BTreeMap::new(),
        proofs: BTreeMap::new(),
        word_values: BTreeMap::new(),
        staged_words: BTreeMap::new(),
        staged_values: BTreeSet::new(),
        word_heads: BTreeMap::new(),
        word_busy: BTreeSet::new(),
    };
    definitions
        .iter()
        .map(|definition| {
            let term = n.b.constant(definition)?;
            if !n.closed_term(term, 0, &mut BTreeSet::new()) {
                return Err(OrdinaryCarrierError::Linkage);
            }
            n.theorem(definition, true)
        })
        .collect()
}
