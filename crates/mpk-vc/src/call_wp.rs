//! Contract-bound static-call analysis for the unified VIR WP engine.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::vir::{VirFunction, VirInstruction, VirModule, VirUnit};
use crate::vir_canonical::contract_hash;

pub const VC_FUNCTION_DECLARATION_PREFIX: &str = "VC.Function.f";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProgramDeclarationKind {
    Contract,
    PanicFree,
}

impl ProgramDeclarationKind {
    const fn suffix(self) -> &'static str {
        match self {
            Self::Contract => "contract",
            Self::PanicFree => "panic_free",
        }
    }
}

pub fn program_declaration_name(function_id: &str, kind: ProgramDeclarationKind) -> String {
    const LOWERCASE_HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(function_id.len().saturating_mul(2));
    for byte in function_id.as_bytes() {
        encoded.push(char::from(LOWERCASE_HEX[usize::from(*byte >> 4)]));
        encoded.push(char::from(LOWERCASE_HEX[usize::from(*byte & 0x0f)]));
    }
    format!(
        "{VC_FUNCTION_DECLARATION_PREFIX}{encoded}.{}",
        kind.suffix()
    )
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProgramCallDependencies {
    pub direct_callees: Vec<String>,
    pub contract_dependencies: Vec<String>,
    pub panic_free_dependencies: Vec<String>,
}

#[derive(Debug)]
pub(crate) struct ProgramCallGraph<'a> {
    functions: BTreeMap<&'a str, (&'a VirUnit, &'a VirFunction)>,
    order: Vec<(&'a VirUnit, &'a VirFunction)>,
    direct_callees: BTreeMap<&'a str, Vec<&'a str>>,
}

impl<'a> ProgramCallGraph<'a> {
    pub(crate) fn analyze(module: &'a VirModule) -> Result<Self, CallWpError> {
        let mut functions = BTreeMap::new();
        for unit in &module.units {
            for function in &unit.functions {
                if functions
                    .insert(function.id.as_str(), (unit, function))
                    .is_some()
                {
                    return Err(CallWpError::DuplicateFunction {
                        function_id: function.id.clone(),
                    });
                }
                let actual = contract_hash(&function.contracts).map_err(|error| {
                    CallWpError::ContractHash {
                        function_id: function.id.clone(),
                        detail: error.to_string(),
                    }
                })?;
                if actual != function.contracts.contract_hash {
                    return Err(CallWpError::ContractHashMismatch {
                        function_id: function.id.clone(),
                    });
                }
                if function.contracts.semantic_profile != module.semantic_profile
                    || function.contracts.semantic_parameters != module.semantic_parameters
                {
                    return Err(CallWpError::SemanticContextMismatch {
                        caller: function.id.clone(),
                        callee: function.id.clone(),
                    });
                }
            }
        }

        let mut direct_callees = BTreeMap::new();
        for (caller_id, (_, caller)) in &functions {
            let mut direct = BTreeSet::new();
            for instruction in caller
                .blocks
                .iter()
                .flat_map(|block| block.instructions.iter())
            {
                let VirInstruction::CallStatic {
                    function,
                    contract_hash: repeated_hash,
                    ..
                } = instruction
                else {
                    continue;
                };
                let Some((_, callee)) = functions.get(function.as_str()) else {
                    return Err(CallWpError::UnknownCallee {
                        caller: (*caller_id).to_owned(),
                        callee: function.clone(),
                    });
                };
                if callee.contracts.semantic_profile != caller.contracts.semantic_profile
                    || callee.contracts.semantic_parameters != caller.contracts.semantic_parameters
                {
                    return Err(CallWpError::SemanticContextMismatch {
                        caller: (*caller_id).to_owned(),
                        callee: function.clone(),
                    });
                }
                let actual = contract_hash(&callee.contracts).map_err(|error| {
                    CallWpError::ContractHash {
                        function_id: function.clone(),
                        detail: error.to_string(),
                    }
                })?;
                if actual != *repeated_hash {
                    return Err(CallWpError::CalleeContractHashMismatch {
                        caller: (*caller_id).to_owned(),
                        callee: function.clone(),
                    });
                }
                direct.insert(function.as_str());
            }
            direct_callees.insert(*caller_id, direct.into_iter().collect::<Vec<_>>());
        }

        let mut callers_by_callee = BTreeMap::<&str, Vec<&str>>::new();
        let mut remaining = BTreeMap::<&str, usize>::new();
        for (caller, callees) in &direct_callees {
            remaining.insert(*caller, callees.len());
            for callee in callees {
                callers_by_callee.entry(*callee).or_default().push(*caller);
            }
        }
        let mut ready = remaining
            .iter()
            .filter_map(|(function, count)| (*count == 0).then_some(*function))
            .collect::<BTreeSet<_>>();
        let mut order = Vec::with_capacity(functions.len());
        while let Some(function) = ready.pop_first() {
            order.push(function);
            for caller in callers_by_callee.get(function).into_iter().flatten() {
                let count =
                    remaining
                        .get_mut(caller)
                        .ok_or_else(|| CallWpError::UnknownFunction {
                            function_id: (*caller).to_owned(),
                        })?;
                *count = count
                    .checked_sub(1)
                    .ok_or_else(|| CallWpError::CounterOverflow {
                        context: "remaining callee count".to_owned(),
                    })?;
                if *count == 0 {
                    ready.insert(caller);
                }
            }
        }
        if order.len() != functions.len() {
            return Err(CallWpError::CallCycle);
        }
        let order = order
            .into_iter()
            .map(|function_id| {
                functions
                    .get(function_id)
                    .copied()
                    .ok_or_else(|| CallWpError::UnknownFunction {
                        function_id: function_id.to_owned(),
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            functions,
            order,
            direct_callees,
        })
    }

    pub(crate) fn ordered_functions(
        &self,
    ) -> impl Iterator<Item = (&'a VirUnit, &'a VirFunction)> + '_ {
        self.order.iter().copied()
    }

    pub(crate) fn resolve(
        &self,
        function_id: &str,
    ) -> Result<(&'a VirUnit, &'a VirFunction), CallWpError> {
        self.functions
            .get(function_id)
            .copied()
            .ok_or_else(|| CallWpError::UnknownFunction {
                function_id: function_id.to_owned(),
            })
    }

    pub(crate) fn dependencies(
        &self,
        function_id: &str,
    ) -> Result<ProgramCallDependencies, CallWpError> {
        let direct =
            self.direct_callees
                .get(function_id)
                .ok_or_else(|| CallWpError::UnknownFunction {
                    function_id: function_id.to_owned(),
                })?;
        let direct_callees = direct
            .iter()
            .map(|callee| (*callee).to_owned())
            .collect::<Vec<_>>();
        let mut contract_dependencies = direct
            .iter()
            .map(|callee| program_declaration_name(callee, ProgramDeclarationKind::Contract))
            .collect::<Vec<_>>();
        contract_dependencies.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
        let mut panic_free_dependencies = vec![program_declaration_name(
            function_id,
            ProgramDeclarationKind::Contract,
        )];
        for callee in direct {
            panic_free_dependencies.push(program_declaration_name(
                callee,
                ProgramDeclarationKind::Contract,
            ));
            panic_free_dependencies.push(program_declaration_name(
                callee,
                ProgramDeclarationKind::PanicFree,
            ));
        }
        panic_free_dependencies.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
        panic_free_dependencies.dedup();
        Ok(ProgramCallDependencies {
            direct_callees,
            contract_dependencies,
            panic_free_dependencies,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CallWpError {
    DuplicateFunction { function_id: String },
    UnknownFunction { function_id: String },
    UnknownCallee { caller: String, callee: String },
    ContractHash { function_id: String, detail: String },
    ContractHashMismatch { function_id: String },
    CalleeContractHashMismatch { caller: String, callee: String },
    SemanticContextMismatch { caller: String, callee: String },
    CallCycle,
    CounterOverflow { context: String },
}

impl fmt::Display for CallWpError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateFunction { function_id } => {
                write!(formatter, "duplicate function {function_id}")
            }
            Self::UnknownFunction { function_id } => {
                write!(formatter, "unknown function {function_id}")
            }
            Self::UnknownCallee { caller, callee } => {
                write!(formatter, "unknown callee {callee} in {caller}")
            }
            Self::ContractHash {
                function_id,
                detail,
            } => write!(
                formatter,
                "contract hash failed for {function_id}: {detail}"
            ),
            Self::ContractHashMismatch { function_id } => {
                write!(formatter, "contract hash mismatch for {function_id}")
            }
            Self::CalleeContractHashMismatch { caller, callee } => {
                write!(
                    formatter,
                    "callee contract hash mismatch for {caller} -> {callee}"
                )
            }
            Self::SemanticContextMismatch { caller, callee } => {
                write!(
                    formatter,
                    "semantic context mismatch for {caller} -> {callee}"
                )
            }
            Self::CallCycle => formatter.write_str("reachable VIR call graph is cyclic"),
            Self::CounterOverflow { context } => write!(formatter, "counter overflow: {context}"),
        }
    }
}

impl std::error::Error for CallWpError {}
