//! Branch-aware VC generation for the first if/else GIR shape.
//!
//! VC-005 handles the diamond-shaped branch emitted by `go2gir` for simple
//! if/else paths: an entry block ending in `Branch`, path blocks ending in
//! either `Return` or `Jump`, and a return join block.

use std::collections::{BTreeMap, BTreeSet};

use crate::expr_encode::{ExprEncoder, MpkExprTerm, STD_BOOL_NOT};
use crate::gir::{GirBlock, GirFunction, GirModule, GirTerminator, GirTerminatorKind, GirValue};
use crate::vc::{VcModule, VcObligation, VcObligationKind};
use crate::wp::{
    encode_requires, initial_environment, substitute_term, validate_contract_references,
    validate_value_reference, WpError, WpGenerator,
};

pub fn generate_branch_vcs(input: &GirModule) -> Result<VcModule, WpError> {
    BranchWpGenerator::new().generate_module(input)
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct BranchWpGenerator;

impl BranchWpGenerator {
    pub fn new() -> Self {
        Self
    }

    pub fn generate_module(self, input: &GirModule) -> Result<VcModule, WpError> {
        let mut output = VcModule::empty_for_gir(input);
        for package in &input.packages {
            for function in &package.functions {
                output.obligations.extend(self.generate_function(function)?);
            }
        }
        Ok(output)
    }

    pub fn generate_function(self, function: &GirFunction) -> Result<Vec<VcObligation>, WpError> {
        reject_common_unsupported(function)?;
        if function.blocks.is_empty() {
            return Err(WpError::UnsupportedBlockCount {
                function_id: function.id.clone(),
                block_count: 0,
            });
        }

        let entry = &function.blocks[0];
        ensure_no_block_parameters(function, entry)?;
        if entry.terminator.kind != GirTerminatorKind::Branch {
            return Err(WpError::UnsupportedTerminator {
                function_id: function.id.clone(),
                block_label: entry.label.clone(),
                kind: entry.terminator.kind,
            });
        }
        ensure_branch_terminator_shape(function, entry, &entry.terminator)?;

        let encoder = ExprEncoder::for_function(function);
        let initial_env = initial_environment(function);
        let assumptions = encode_requires(function, &encoder, &initial_env)?;
        let branch_env =
            WpGenerator::new().execute_block(function, entry, &encoder, initial_env.clone())?;
        let condition = encode_branch_condition(function, entry, &encoder, &branch_env)?;

        let blocks = block_map(function);
        let then_label =
            entry
                .terminator
                .then_label
                .as_deref()
                .ok_or_else(|| WpError::MissingBranchLabel {
                    function_id: function.id.clone(),
                    block_label: entry.label.clone(),
                    label_kind: "then",
                })?;
        let else_label =
            entry
                .terminator
                .else_label
                .as_deref()
                .ok_or_else(|| WpError::MissingBranchLabel {
                    function_id: function.id.clone(),
                    block_label: entry.label.clone(),
                    label_kind: "else",
                })?;

        let mut output = Vec::new();
        let mut then_assumptions = assumptions.clone();
        then_assumptions.push(condition.clone());
        output.extend(self.generate_path(
            function,
            &blocks,
            &encoder,
            &initial_env,
            "then",
            then_label,
            branch_env.clone(),
            then_assumptions,
        )?);

        let mut else_assumptions = assumptions;
        else_assumptions.push(MpkExprTerm::apply(STD_BOOL_NOT, [condition]));
        output.extend(self.generate_path(
            function,
            &blocks,
            &encoder,
            &initial_env,
            "else",
            else_label,
            branch_env,
            else_assumptions,
        )?);

        Ok(output)
    }

    fn generate_path(
        self,
        function: &GirFunction,
        blocks: &BTreeMap<String, &GirBlock>,
        encoder: &ExprEncoder,
        contract_env: &BTreeMap<String, MpkExprTerm>,
        path_name: &'static str,
        start_label: &str,
        env: BTreeMap<String, MpkExprTerm>,
        assumptions: Vec<MpkExprTerm>,
    ) -> Result<Vec<VcObligation>, WpError> {
        let (final_env, result_terms) =
            execute_path_to_return(function, blocks, encoder, start_label, env)?;
        function
            .contracts
            .ensures
            .iter()
            .enumerate()
            .map(|(index, ensure)| {
                validate_contract_references(function, ensure, contract_env, true, "ensures")?;
                let encoded =
                    encoder
                        .encode_contract_expr(ensure)
                        .map_err(|source| WpError::Expression {
                            function_id: function.id.clone(),
                            context: format!("{path_name}.ensures[{index}]"),
                            source,
                        })?;
                Ok(VcObligation {
                    id: format!("{}.{path_name}.post{index}", function.id),
                    function_id: function.id.clone(),
                    kind: VcObligationKind::Postcondition,
                    assumptions: assumptions.clone(),
                    conclusion: substitute_term(&encoded, &final_env, &result_terms),
                })
            })
            .collect()
    }
}

fn reject_common_unsupported(function: &GirFunction) -> Result<(), WpError> {
    if !function.rejected_features.is_empty() {
        return Err(WpError::FunctionHasRejectedFeatures {
            function_id: function.id.clone(),
            rejected_feature_count: function.rejected_features.len(),
        });
    }
    if function.contracts.ensures.is_empty() {
        return Err(WpError::MissingPostcondition {
            function_id: function.id.clone(),
        });
    }
    if !function.contracts.modifies.is_empty() {
        return Err(WpError::NonEmptyModifies {
            function_id: function.id.clone(),
            modifies: function.contracts.modifies.clone(),
        });
    }
    if !function.contracts.loops.is_empty() {
        return Err(WpError::LoopContractsUnsupported {
            function_id: function.id.clone(),
            loop_count: function.contracts.loops.len(),
        });
    }
    Ok(())
}

fn block_map(function: &GirFunction) -> BTreeMap<String, &GirBlock> {
    function
        .blocks
        .iter()
        .map(|block| (block.label.clone(), block))
        .collect()
}

fn execute_path_to_return(
    function: &GirFunction,
    blocks: &BTreeMap<String, &GirBlock>,
    encoder: &ExprEncoder,
    start_label: &str,
    mut env: BTreeMap<String, MpkExprTerm>,
) -> Result<(BTreeMap<String, MpkExprTerm>, BTreeMap<u32, MpkExprTerm>), WpError> {
    let mut label = start_label.to_owned();
    let mut visited = BTreeSet::new();
    loop {
        if !visited.insert(label.clone()) {
            return Err(WpError::CyclicBranchPath {
                function_id: function.id.clone(),
                block_label: label,
            });
        }
        let block = blocks
            .get(&label)
            .copied()
            .ok_or_else(|| WpError::UnknownBlockLabel {
                function_id: function.id.clone(),
                context: "branch path".to_owned(),
                block_label: label.clone(),
            })?;
        ensure_no_block_parameters(function, block)?;
        env = WpGenerator::new().execute_block(function, block, encoder, env)?;

        match block.terminator.kind {
            GirTerminatorKind::Return => {
                ensure_return_terminator_shape(function, block, &block.terminator)?;
                let result_terms =
                    encode_return_values(function, encoder, &env, &block.terminator.values)?;
                return Ok((env, result_terms));
            }
            GirTerminatorKind::Jump => {
                ensure_jump_terminator_shape(function, block, &block.terminator)?;
                label = block
                    .terminator
                    .label
                    .as_ref()
                    .expect("jump shape validates label")
                    .clone();
            }
            kind => {
                return Err(WpError::UnsupportedTerminator {
                    function_id: function.id.clone(),
                    block_label: block.label.clone(),
                    kind,
                });
            }
        }
    }
}

fn encode_branch_condition(
    function: &GirFunction,
    block: &GirBlock,
    encoder: &ExprEncoder,
    env: &BTreeMap<String, MpkExprTerm>,
) -> Result<MpkExprTerm, WpError> {
    let condition =
        block
            .terminator
            .cond
            .as_ref()
            .ok_or_else(|| WpError::MissingBranchCondition {
                function_id: function.id.clone(),
                block_label: block.label.clone(),
            })?;
    validate_value_reference(function, condition, env, "branch condition")?;
    let encoded = encoder
        .encode_value(condition)
        .map_err(|source| WpError::Expression {
            function_id: function.id.clone(),
            context: "branch condition".to_owned(),
            source,
        })?;
    Ok(substitute_term(&encoded, env, &BTreeMap::new()))
}

fn encode_return_values(
    function: &GirFunction,
    encoder: &ExprEncoder,
    env: &BTreeMap<String, MpkExprTerm>,
    values: &[GirValue],
) -> Result<BTreeMap<u32, MpkExprTerm>, WpError> {
    if values.len() != function.results.len() {
        return Err(WpError::ReturnArityMismatch {
            function_id: function.id.clone(),
            expected: function.results.len(),
            actual: values.len(),
        });
    }
    values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            validate_value_reference(function, value, env, format!("return[{index}]"))?;
            let encoded = encoder
                .encode_value(value)
                .map_err(|source| WpError::Expression {
                    function_id: function.id.clone(),
                    context: format!("return[{index}]"),
                    source,
                })?;
            let index = u32::try_from(index).map_err(|_| WpError::ReturnIndexOverflow {
                function_id: function.id.clone(),
                index,
            })?;
            Ok((index, substitute_term(&encoded, env, &BTreeMap::new())))
        })
        .collect()
}

fn ensure_no_block_parameters(function: &GirFunction, block: &GirBlock) -> Result<(), WpError> {
    if block.parameters.is_empty() {
        return Ok(());
    }
    Err(WpError::BlockParametersUnsupported {
        function_id: function.id.clone(),
        block_label: block.label.clone(),
        parameter_count: block.parameters.len(),
    })
}

fn ensure_branch_terminator_shape(
    function: &GirFunction,
    block: &GirBlock,
    terminator: &GirTerminator,
) -> Result<(), WpError> {
    let reason = first_present([
        (!terminator.values.is_empty(), "Branch cannot have values"),
        (terminator.label.is_some(), "Branch cannot have label"),
        (!terminator.args.is_empty(), "Branch cannot have args"),
        (terminator.reason.is_some(), "Branch cannot have reason"),
    ]);
    reject_bad_terminator_shape(function, block, terminator.kind, reason)
}

fn ensure_jump_terminator_shape(
    function: &GirFunction,
    block: &GirBlock,
    terminator: &GirTerminator,
) -> Result<(), WpError> {
    let reason = first_present([
        (!terminator.values.is_empty(), "Jump cannot have values"),
        (terminator.cond.is_some(), "Jump cannot have cond"),
        (terminator.label.is_none(), "Jump must have label"),
        (
            terminator.then_label.is_some(),
            "Jump cannot have then_label",
        ),
        (
            terminator.else_label.is_some(),
            "Jump cannot have else_label",
        ),
        (!terminator.args.is_empty(), "Jump cannot have args"),
        (terminator.reason.is_some(), "Jump cannot have reason"),
    ]);
    reject_bad_terminator_shape(function, block, terminator.kind, reason)
}

fn ensure_return_terminator_shape(
    function: &GirFunction,
    block: &GirBlock,
    terminator: &GirTerminator,
) -> Result<(), WpError> {
    let reason = first_present([
        (terminator.cond.is_some(), "Return cannot have cond"),
        (terminator.label.is_some(), "Return cannot have label"),
        (
            terminator.then_label.is_some(),
            "Return cannot have then_label",
        ),
        (
            terminator.else_label.is_some(),
            "Return cannot have else_label",
        ),
        (!terminator.args.is_empty(), "Return cannot have args"),
        (terminator.reason.is_some(), "Return cannot have reason"),
    ]);
    reject_bad_terminator_shape(function, block, terminator.kind, reason)
}

fn reject_bad_terminator_shape(
    function: &GirFunction,
    block: &GirBlock,
    kind: GirTerminatorKind,
    reason: Option<&'static str>,
) -> Result<(), WpError> {
    if let Some(reason) = reason {
        return Err(WpError::UnsupportedTerminatorShape {
            function_id: function.id.clone(),
            block_label: block.label.clone(),
            kind,
            reason,
        });
    }
    Ok(())
}

fn first_present<const N: usize>(checks: [(bool, &'static str); N]) -> Option<&'static str> {
    checks
        .into_iter()
        .find_map(|(present, reason)| present.then_some(reason))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr_encode::{STD_BITVEC_MODULE, STD_BOOL_NOT, STD_EQ};
    use crate::gir::{import_gir_json, GirInstructionKind};

    fn generate(input: &str) -> Result<VcModule, WpError> {
        let gir = import_gir_json(input).expect("GIR imports");
        generate_branch_vcs(&gir)
    }

    fn max64_gir_json() -> &'static str {
        r#"{"schema_version":"mpk.gir.v0","packages":[{"package_path":"example/pkg","name":"example","functions":[{"id":"example/pkg.Max64","package":"example/pkg","name":"Max64","params":[{"name":"a","type":{"kind":"bv","width":64,"signed":true}},{"name":"b","type":{"kind":"bv","width":64,"signed":true}}],"results":[{"name":"result0","type":{"kind":"bv","width":64,"signed":true}}],"locals":[{"name":"max","type":{"kind":"bv","width":64,"signed":true}}],"blocks":[{"label":"entry","parameters":[],"instructions":[{"id":"t0","kind":"Copy","type":{"kind":"bv","width":64,"signed":true},"target":"max","value":{"var":"a"}},{"id":"t1","kind":"BinOp","op":"signed_gt","type":{"kind":"bool"},"lhs":{"var":"b"},"rhs":{"var":"max"}}],"terminator":{"kind":"Branch","cond":{"var":"t1"},"then_label":"if_then_0","else_label":"if_after_2"}},{"label":"if_then_0","parameters":[],"instructions":[{"id":"t2","kind":"Copy","type":{"kind":"bv","width":64,"signed":true},"target":"max","value":{"var":"b"}}],"terminator":{"kind":"Jump","label":"if_after_2"}},{"label":"if_after_2","parameters":[],"instructions":[],"terminator":{"kind":"Return","values":[{"var":"max"}]}}],"contracts":{"requires":[],"ensures":[{"op":"signed_ge","lhs":{"result":0},"rhs":{"var":"a"}},{"op":"signed_ge","lhs":{"result":0},"rhs":{"var":"b"}},{"op":"or","args":[{"op":"eq","lhs":{"result":0},"rhs":{"var":"a"}},{"op":"eq","lhs":{"result":0},"rhs":{"var":"b"}}]}],"modifies":[],"loops":[]},"supported_features":["params","locals","blocks","binops","if","return"],"rejected_features":[]}]}],"gir_hash":"maxhash"}"#
    }

    fn var(name: &str) -> MpkExprTerm {
        MpkExprTerm::Var {
            name: name.to_owned(),
        }
    }

    fn apply(function: impl Into<String>, args: Vec<MpkExprTerm>) -> MpkExprTerm {
        MpkExprTerm::Apply {
            function: function.into(),
            args,
        }
    }

    fn condition() -> MpkExprTerm {
        apply(
            format!("{STD_BITVEC_MODULE}.BV64.sgt"),
            vec![var("b"), var("a")],
        )
    }

    fn sge(lhs: MpkExprTerm, rhs: MpkExprTerm) -> MpkExprTerm {
        apply(format!("{STD_BITVEC_MODULE}.BV64.sge"), vec![lhs, rhs])
    }

    fn eq(lhs: MpkExprTerm, rhs: MpkExprTerm) -> MpkExprTerm {
        apply(STD_EQ, vec![lhs, rhs])
    }

    #[test]
    fn max64_branch_vcs_are_produced_for_both_paths() {
        let module = generate(max64_gir_json()).expect("branch VCs generate");

        assert_eq!(module.source_gir_hash.as_deref(), Some("maxhash"));
        assert_eq!(module.obligations.len(), 6);
        assert_eq!(module.obligations[0].id, "example/pkg.Max64.then.post0");
        assert_eq!(module.obligations[3].id, "example/pkg.Max64.else.post0");
        assert_eq!(module.obligations[0].assumptions, vec![condition()]);
        assert_eq!(
            module.obligations[3].assumptions,
            vec![MpkExprTerm::apply(STD_BOOL_NOT, [condition()])]
        );
        assert_eq!(module.obligations[0].conclusion, sge(var("b"), var("a")));
        assert_eq!(module.obligations[1].conclusion, sge(var("b"), var("b")));
        assert_eq!(module.obligations[3].conclusion, sge(var("a"), var("a")));
        assert_eq!(module.obligations[4].conclusion, sge(var("a"), var("b")));
        assert_eq!(
            module.obligations[2].conclusion,
            apply(
                "Std.Bool.or",
                vec![eq(var("b"), var("a")), eq(var("b"), var("b"))],
            )
        );
        assert_eq!(
            module.obligations[5].conclusion,
            apply(
                "Std.Bool.or",
                vec![eq(var("a"), var("a")), eq(var("a"), var("b"))],
            )
        );
    }

    #[test]
    fn rejects_missing_branch_condition() {
        let error = generate(
            r#"{"schema_version":"mpk.gir.v0","packages":[{"package_path":"example/pkg","name":"example","functions":[{"id":"example/pkg.BadBranch","package":"example/pkg","name":"BadBranch","params":[],"results":[],"locals":[],"blocks":[{"label":"entry","parameters":[],"instructions":[],"terminator":{"kind":"Branch","then_label":"then","else_label":"else"}},{"label":"then","parameters":[],"instructions":[],"terminator":{"kind":"Return","values":[]}},{"label":"else","parameters":[],"instructions":[],"terminator":{"kind":"Return","values":[]}}],"contracts":{"requires":[],"ensures":[{"bool":true}],"modifies":[],"loops":[]},"supported_features":[],"rejected_features":[]}]}]}"#,
        )
        .expect_err("missing branch condition rejects");

        assert_eq!(
            error,
            WpError::MissingBranchCondition {
                function_id: "example/pkg.BadBranch".to_owned(),
                block_label: "entry".to_owned(),
            }
        );
    }

    #[test]
    fn rejects_unknown_branch_target() {
        let error = generate(
            r#"{"schema_version":"mpk.gir.v0","packages":[{"package_path":"example/pkg","name":"example","functions":[{"id":"example/pkg.UnknownTarget","package":"example/pkg","name":"UnknownTarget","params":[{"name":"cond","type":{"kind":"bool"}}],"results":[],"locals":[],"blocks":[{"label":"entry","parameters":[],"instructions":[],"terminator":{"kind":"Branch","cond":{"var":"cond"},"then_label":"missing","else_label":"else"}},{"label":"else","parameters":[],"instructions":[],"terminator":{"kind":"Return","values":[]}}],"contracts":{"requires":[],"ensures":[{"bool":true}],"modifies":[],"loops":[]},"supported_features":[],"rejected_features":[]}]}]}"#,
        )
        .expect_err("unknown target rejects");

        assert_eq!(
            error,
            WpError::UnknownBlockLabel {
                function_id: "example/pkg.UnknownTarget".to_owned(),
                context: "branch path".to_owned(),
                block_label: "missing".to_owned(),
            }
        );
    }

    #[test]
    fn rejects_nested_branch_inside_path() {
        let error = generate(
            r#"{"schema_version":"mpk.gir.v0","packages":[{"package_path":"example/pkg","name":"example","functions":[{"id":"example/pkg.Nested","package":"example/pkg","name":"Nested","params":[{"name":"cond","type":{"kind":"bool"}}],"results":[],"locals":[],"blocks":[{"label":"entry","parameters":[],"instructions":[],"terminator":{"kind":"Branch","cond":{"var":"cond"},"then_label":"then","else_label":"else"}},{"label":"then","parameters":[],"instructions":[],"terminator":{"kind":"Branch","cond":{"var":"cond"},"then_label":"else","else_label":"else"}},{"label":"else","parameters":[],"instructions":[],"terminator":{"kind":"Return","values":[]}}],"contracts":{"requires":[],"ensures":[{"bool":true}],"modifies":[],"loops":[]},"supported_features":[],"rejected_features":[]}]}]}"#,
        )
        .expect_err("nested branch rejects");

        assert_eq!(
            error,
            WpError::UnsupportedTerminator {
                function_id: "example/pkg.Nested".to_owned(),
                block_label: "then".to_owned(),
                kind: GirTerminatorKind::Branch,
            }
        );
    }

    #[test]
    fn rejects_unsupported_instruction_shape_in_branch_path() {
        let error = generate(
            r#"{"schema_version":"mpk.gir.v0","packages":[{"package_path":"example/pkg","name":"example","functions":[{"id":"example/pkg.BadInstruction","package":"example/pkg","name":"BadInstruction","params":[{"name":"cond","type":{"kind":"bool"}},{"name":"a","type":{"kind":"bv","width":64,"signed":true}}],"results":[{"name":"result0","type":{"kind":"bv","width":64,"signed":true}}],"locals":[{"name":"max","type":{"kind":"bv","width":64,"signed":true}}],"blocks":[{"label":"entry","parameters":[],"instructions":[],"terminator":{"kind":"Branch","cond":{"var":"cond"},"then_label":"then","else_label":"else"}},{"label":"then","parameters":[],"instructions":[{"id":"bad","kind":"Copy","op":"eq","type":{"kind":"bv","width":64,"signed":true},"target":"max","value":{"var":"a"}}],"terminator":{"kind":"Return","values":[{"var":"max"}]}},{"label":"else","parameters":[],"instructions":[],"terminator":{"kind":"Return","values":[{"var":"a"}]}}],"contracts":{"requires":[],"ensures":[{"bool":true}],"modifies":[],"loops":[]},"supported_features":[],"rejected_features":[]}]}]}"#,
        )
        .expect_err("bad instruction shape rejects");

        assert_eq!(
            error,
            WpError::UnsupportedInstructionShape {
                function_id: "example/pkg.BadInstruction".to_owned(),
                block_label: "then".to_owned(),
                instruction_id: "bad".to_owned(),
                kind: GirInstructionKind::Copy,
                reason: "Copy cannot have op",
            }
        );
    }
}
