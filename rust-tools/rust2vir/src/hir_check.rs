use rust2vir_internal::call_closure::{
    resolve_call_closure, CallClosureError, MAX_CALL_CLOSURE_FUNCTIONS,
};
use rust2vir_internal::contract::ContractType;
use rustc_abi::ExternAbi;
use rustc_ast::ast::{BinOpKind, ByRef, LitKind, Mutability, UnOp};
use rustc_hir as hir;
use rustc_hir::def::{DefKind, Res};
use rustc_hir::intravisit::{self, Visitor};
use rustc_middle::ty::{self, IntTy, Ty, TyCtxt, UintTy};
use rustc_span::def_id::{DefId, LocalDefId, LOCAL_CRATE};
use std::collections::{BTreeSet, HashMap, HashSet};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum HirCheckCode {
    CallClosureLimit,
    AggregateLimit,
    Identifier,
    Item,
    FunctionKind,
    Generic,
    Trait,
    Impl,
    Static,
    Type,
    Drop,
    Pattern,
    Binding,
    ControlFlow,
    Mutation,
    Operation,
    Call,
    Purity,
}

#[allow(dead_code)]
impl HirCheckCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Identifier => "RUST_SUBSET_IDENTIFIER",
            Self::Item => "RUST_SUBSET_ITEM",
            Self::FunctionKind => "RUST_SUBSET_FUNCTION_KIND",
            Self::Generic => "RUST_SUBSET_GENERIC",
            Self::Trait => "RUST_SUBSET_TRAIT",
            Self::Impl => "RUST_SUBSET_IMPL",
            Self::Static => "RUST_SUBSET_STATIC",
            Self::Type => "RUST_SUBSET_TYPE",
            Self::Drop => "RUST_SUBSET_DROP",
            Self::Pattern => "RUST_SUBSET_PATTERN",
            Self::Binding => "RUST_SUBSET_BINDING",
            Self::ControlFlow => "RUST_SUBSET_CONTROL_FLOW",
            Self::Mutation => "RUST_SUBSET_MUTATION",
            Self::Operation => "RUST_SUBSET_OPERATION",
            Self::Call => "RUST_SUBSET_CALL",
            Self::Purity => "RUST_SUBSET_PURITY",
            Self::CallClosureLimit => "RUST_LIMIT_CALL_CLOSURE",
            Self::AggregateLimit => "RUST_LIMIT_AGGREGATE",
        }
    }

    pub fn message(self) -> &'static str {
        match self {
            Self::Identifier => "source identifier is not canonical",
            Self::Item => "item is outside the closed Rust subset",
            Self::FunctionKind => "function kind is outside the closed Rust subset",
            Self::Generic => "generic declarations are not permitted",
            Self::Trait => "traits are not permitted",
            Self::Impl => "implementation items are not permitted",
            Self::Static => "static storage is not permitted",
            Self::Type => "type is outside the closed Rust subset",
            Self::Drop => "values requiring drop glue are not permitted",
            Self::Pattern => "pattern is outside the closed Rust subset",
            Self::Binding => "binding shadows an existing source name",
            Self::ControlFlow => "control flow is outside the closed Rust subset",
            Self::Mutation => "mutation is outside the closed Rust subset",
            Self::Operation => "operation is outside the closed Rust subset",
            Self::Call => "call is not a direct acyclic same-crate free-function call",
            Self::Purity => "operation violates the closed purity profile",
            Self::CallClosureLimit => "call closure exceeds the deterministic function limit",
            Self::AggregateLimit => "aggregate exceeds the deterministic shape limit",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HirFunction {
    pub function_id: String,
    pub parameter_names: Vec<String>,
    pub parameter_types: Vec<ContractType>,
    pub result_type: ContractType,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HirAnalysis {
    pub selected_function: String,
    pub call_closure: Vec<HirFunction>,
}

pub fn analyze_hir(tcx: TyCtxt<'_>, selected_function: &str) -> Result<HirAnalysis, HirCheckCode> {
    let mut functions = HashMap::<LocalDefId, FunctionItem<'_>>::new();
    let mut names = HashMap::<LocalDefId, String>::new();

    let item_ids = tcx.hir_free_items().collect::<Vec<_>>();
    for item_id in &item_ids {
        match tcx.hir_item(*item_id).kind {
            hir::ItemKind::Trait(..) | hir::ItemKind::TraitAlias(..) => {
                return Err(HirCheckCode::Trait);
            }
            hir::ItemKind::Impl(..) => return Err(HirCheckCode::Impl),
            hir::ItemKind::Static(..) => return Err(HirCheckCode::Static),
            _ => {}
        }
    }
    for item_id in item_ids {
        let item = tcx.hir_item(item_id);
        validate_item(tcx, item)?;
        if let hir::ItemKind::Fn {
            sig,
            generics,
            body,
            has_body,
            ..
        } = item.kind
        {
            let def_id = item.owner_id.def_id;
            let function_id = canonical_item_id(tcx, def_id);
            names.insert(def_id, function_id);
            functions.insert(
                def_id,
                FunctionItem {
                    sig,
                    generics,
                    body,
                    has_body,
                },
            );
        }
    }

    let selected = names
        .iter()
        .find_map(|(def_id, name)| (name == selected_function).then_some(*def_id))
        .ok_or(HirCheckCode::Call)?;

    let mut graph = Vec::with_capacity(functions.len());
    let mut scan_errors = HashMap::new();
    for (def_id, function) in &functions {
        let typeck = tcx.typeck(*def_id);
        let mut scanner = CallScanner {
            typeck,
            callees: HashSet::new(),
            error: None,
        };
        scanner.visit_expr(tcx.hir_body(function.body).value);
        if let Some(error) = scanner.error {
            scan_errors.insert(*def_id, error);
        }
        graph.push((
            names[def_id].clone(),
            scanner
                .callees
                .into_iter()
                .filter_map(|callee| callee.as_local())
                .filter_map(|callee| names.get(&callee).cloned())
                .collect::<Vec<_>>(),
        ));
    }

    let closure_names = resolve_call_closure(graph, selected_function, MAX_CALL_CLOSURE_FUNCTIONS)
        .map_err(|error| match error {
            CallClosureError::Limit => HirCheckCode::CallClosureLimit,
            _ => HirCheckCode::Call,
        })?;
    let closure_set = closure_names
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let mut closure = Vec::with_capacity(closure_names.len());
    for (def_id, function_id) in &names {
        if !closure_set.contains(function_id.as_str()) {
            continue;
        }
        if let Some(error) = scan_errors.get(def_id) {
            return Err(*error);
        }
        let function = functions.get(def_id).ok_or(HirCheckCode::Call)?;
        closure.push(validate_function(tcx, *def_id, function, function_id)?);
    }
    closure.sort_by(|left, right| left.function_id.cmp(&right.function_id));

    if !closure
        .iter()
        .any(|function| function.function_id == names[&selected])
    {
        return Err(HirCheckCode::Call);
    }
    Ok(HirAnalysis {
        selected_function: selected_function.to_owned(),
        call_closure: closure,
    })
}

#[derive(Clone, Copy)]
struct FunctionItem<'hir> {
    sig: hir::FnSig<'hir>,
    generics: &'hir hir::Generics<'hir>,
    body: hir::BodyId,
    has_body: bool,
}

fn validate_item(tcx: TyCtxt<'_>, item: &hir::Item<'_>) -> Result<(), HirCheckCode> {
    match item.kind {
        hir::ItemKind::Mod(ident, _) => validate_ident(ident.as_str()),
        hir::ItemKind::Fn {
            sig,
            ident,
            generics,
            has_body,
            ..
        } => {
            validate_ident(ident.as_str())?;
            validate_generics(generics)?;
            if !has_body
                || sig.header.is_async()
                || sig.header.is_const()
                || sig.header.is_unsafe()
                || !matches!(sig.header.abi, ExternAbi::Rust)
                || sig.decl.c_variadic
                || sig.decl.implicit_self.has_implicit_self()
            {
                return Err(HirCheckCode::FunctionKind);
            }
            Ok(())
        }
        hir::ItemKind::Const(ident, generics, source_ty, body) => {
            validate_ident(ident.as_str())?;
            validate_generics(generics)?;
            validate_hir_primitive_type(source_ty)?;
            validate_const(tcx, item.owner_id.def_id, body)
        }
        hir::ItemKind::Struct(ident, generics, data) => {
            validate_ident(ident.as_str())?;
            validate_generics(generics)?;
            validate_struct(tcx, item.owner_id.def_id, data)
        }
        hir::ItemKind::Trait(..) | hir::ItemKind::TraitAlias(..) => Err(HirCheckCode::Trait),
        hir::ItemKind::Impl(..) => Err(HirCheckCode::Impl),
        hir::ItemKind::Static(..) => Err(HirCheckCode::Static),
        hir::ItemKind::Use(..) | hir::ItemKind::ExternCrate(..) if item.span.from_expansion() => {
            Ok(())
        }
        _ => Err(HirCheckCode::Item),
    }
}

fn validate_generics(generics: &hir::Generics<'_>) -> Result<(), HirCheckCode> {
    if generics
        .params
        .iter()
        .all(hir::GenericParam::is_elided_lifetime)
        && generics.predicates.is_empty()
        && !generics.has_where_clause_predicates
    {
        Ok(())
    } else {
        Err(HirCheckCode::Generic)
    }
}

fn validate_const(
    tcx: TyCtxt<'_>,
    def_id: LocalDefId,
    body_id: hir::BodyId,
) -> Result<(), HirCheckCode> {
    let ty = tcx.type_of(def_id).instantiate_identity();
    validate_primitive_type(tcx, def_id, ty)?;
    let value = tcx.hir_body(body_id).value;
    match value.kind {
        hir::ExprKind::Lit(lit) if matches!(lit.node, LitKind::Bool(_) | LitKind::Int(..)) => {
            Ok(())
        }
        hir::ExprKind::Unary(UnOp::Neg, inner)
            if matches!(inner.kind, hir::ExprKind::Lit(lit) if matches!(lit.node, LitKind::Int(..)))
                && is_signed_integer(ty) =>
        {
            Ok(())
        }
        _ => Err(HirCheckCode::Purity),
    }
}

fn validate_struct(
    tcx: TyCtxt<'_>,
    def_id: LocalDefId,
    data: hir::VariantData<'_>,
) -> Result<(), HirCheckCode> {
    let hir::VariantData::Struct { fields, .. } = data else {
        return Err(HirCheckCode::Item);
    };
    if fields.len() > 64 {
        return Err(HirCheckCode::AggregateLimit);
    }
    for field in fields {
        validate_ident(field.ident.as_str())?;
        if field.default.is_some() || !field.safety.is_safe() {
            return Err(HirCheckCode::Item);
        }
        validate_hir_value_type(tcx, field.ty)?;
        let ty = tcx.type_of(field.def_id).instantiate_identity();
        validate_value_type(tcx, def_id, ty, 1)?;
    }
    Ok(())
}

fn validate_function(
    tcx: TyCtxt<'_>,
    def_id: LocalDefId,
    function: &FunctionItem<'_>,
    function_id: &str,
) -> Result<HirFunction, HirCheckCode> {
    validate_generics(function.generics)?;
    if !function.has_body {
        return Err(HirCheckCode::FunctionKind);
    }
    let hir::FnRetTy::Return(_) = function.sig.decl.output else {
        return Err(HirCheckCode::Type);
    };
    let body = tcx.hir_body(function.body);
    if body.params.len() != function.sig.decl.inputs.len() {
        return Err(HirCheckCode::Pattern);
    }
    let typeck = tcx.typeck(def_id);
    let mut validator = BodyValidator {
        tcx,
        def_id,
        typeck,
        names: BTreeSet::new(),
        mutable_bindings: HashSet::new(),
    };
    let mut parameter_names = Vec::with_capacity(body.params.len());
    let mut mutable_parameter = false;
    for parameter in body.params {
        let (binding_id, name, mode) = binding_pattern(parameter.pat)?;
        let mutable = match mode {
            hir::BindingMode(ByRef::No, Mutability::Not) => false,
            hir::BindingMode(ByRef::No, Mutability::Mut) => {
                mutable_parameter = true;
                true
            }
            _ => return Err(HirCheckCode::Pattern),
        };
        validator.add_binding(binding_id, &name, mutable)?;
        validate_value_type(tcx, def_id, typeck.pat_ty(parameter.pat), 0)?;
        parameter_names.push(name);
    }
    for source_ty in function.sig.decl.inputs {
        validate_hir_value_type(tcx, source_ty)?;
    }
    let hir::FnRetTy::Return(source_output) = function.sig.decl.output else {
        return Err(HirCheckCode::Type);
    };
    validate_hir_value_type(tcx, source_output)?;
    let resolved_sig = tcx.fn_sig(def_id).instantiate_identity().skip_binder();
    for ty in resolved_sig.inputs() {
        validate_value_type(tcx, def_id, *ty, 0)?;
    }
    validate_value_type(tcx, def_id, resolved_sig.output(), 0)?;
    let parameter_types = resolved_sig
        .inputs()
        .iter()
        .map(|ty| contract_type(tcx, def_id, *ty))
        .collect::<Result<Vec<_>, _>>()?;
    let result_type = contract_type(tcx, def_id, resolved_sig.output())?;
    validator.validate_expr(body.value)?;
    if mutable_parameter {
        return Err(HirCheckCode::Pattern);
    }
    Ok(HirFunction {
        function_id: function_id.to_owned(),
        parameter_names,
        parameter_types,
        result_type,
    })
}

struct CallScanner<'a, 'tcx> {
    typeck: &'a ty::TypeckResults<'tcx>,
    callees: HashSet<DefId>,
    error: Option<HirCheckCode>,
}

impl<'hir> Visitor<'hir> for CallScanner<'_, '_> {
    fn visit_expr(&mut self, expression: &'hir hir::Expr<'hir>) {
        if let hir::ExprKind::Call(callee, _) = expression.kind {
            let resolution = match callee.kind {
                hir::ExprKind::Path(path) => self.typeck.qpath_res(&path, callee.hir_id),
                _ => Res::Err,
            };
            match resolution {
                Res::Def(DefKind::Fn, def_id) if def_id.is_local() => {
                    self.callees.insert(def_id);
                }
                _ => self.error = Some(HirCheckCode::Call),
            }
        }
        intravisit::walk_expr(self, expression);
    }
}

struct BodyValidator<'a, 'tcx> {
    tcx: TyCtxt<'tcx>,
    def_id: LocalDefId,
    typeck: &'a ty::TypeckResults<'tcx>,
    names: BTreeSet<String>,
    mutable_bindings: HashSet<hir::HirId>,
}

impl BodyValidator<'_, '_> {
    fn validate_expr(&mut self, expression: &hir::Expr<'_>) -> Result<(), HirCheckCode> {
        self.validate_identity_adjustments(expression)?;
        match expression.kind {
            hir::ExprKind::Array(elements) => {
                let ty = self.typeck.expr_ty(expression);
                validate_value_type(self.tcx, self.def_id, ty, 0)?;
                if elements.len() > 256 {
                    return Err(HirCheckCode::AggregateLimit);
                }
                for element in elements {
                    self.validate_expr(element)?;
                }
                Ok(())
            }
            hir::ExprKind::Call(callee, arguments) => {
                self.validate_call_path(callee)?;
                for argument in arguments {
                    self.validate_expr(argument)?;
                }
                Ok(())
            }
            hir::ExprKind::Binary(operator, left, right) => {
                self.validate_expr(left)?;
                self.validate_expr(right)?;
                self.validate_binary(operator.node, left, right)
            }
            hir::ExprKind::Unary(operator, operand) => {
                self.validate_expr(operand)?;
                let ty = self.typeck.expr_ty(operand);
                match operator {
                    UnOp::Not if ty.is_bool() || is_integer(ty) => Ok(()),
                    UnOp::Neg if is_signed_integer(ty) => Ok(()),
                    _ => Err(HirCheckCode::Operation),
                }
            }
            hir::ExprKind::Lit(literal)
                if matches!(literal.node, LitKind::Bool(_) | LitKind::Int(..)) =>
            {
                validate_primitive_type(self.tcx, self.def_id, self.typeck.expr_ty(expression))
            }
            hir::ExprKind::DropTemps(inner) => self.validate_expr(inner),
            hir::ExprKind::If(condition, then_expression, else_expression) => {
                self.validate_expr(condition)?;
                if !self.typeck.expr_ty(condition).is_bool() {
                    return Err(HirCheckCode::Operation);
                }
                self.validate_expr(then_expression)?;
                if let Some(else_expression) = else_expression {
                    self.validate_expr(else_expression)?;
                }
                Ok(())
            }
            hir::ExprKind::Block(block, label) => {
                if label.is_some() {
                    return Err(HirCheckCode::ControlFlow);
                }
                self.validate_block(block)
            }
            hir::ExprKind::Assign(left, right, _) => {
                let hir::ExprKind::Path(path) = left.kind else {
                    return Err(HirCheckCode::Mutation);
                };
                let Res::Local(binding) = self.typeck.qpath_res(&path, left.hir_id) else {
                    return Err(HirCheckCode::Mutation);
                };
                if !self.mutable_bindings.contains(&binding) {
                    return Err(HirCheckCode::Mutation);
                }
                self.validate_expr(right)
            }
            hir::ExprKind::Field(base, ident) => {
                validate_ident(ident.as_str())?;
                self.validate_expr(base)?;
                self.validate_copy_projection(expression)
            }
            hir::ExprKind::Index(base, index, _) => {
                self.validate_expr(base)?;
                self.validate_expr(index)?;
                if !self.typeck.expr_ty(index).is_usize() {
                    return Err(HirCheckCode::Operation);
                }
                self.validate_copy_projection(expression)
            }
            hir::ExprKind::Path(path) => self.validate_value_path(expression, &path),
            hir::ExprKind::Ret(Some(value)) => self.validate_expr(value),
            hir::ExprKind::Ret(None) => Err(HirCheckCode::Type),
            hir::ExprKind::Struct(path, fields, hir::StructTailExpr::None) => {
                self.validate_struct_path(expression, path, fields.len())?;
                let mut names = BTreeSet::new();
                for field in fields {
                    validate_ident(field.ident.as_str())?;
                    if !names.insert(field.ident.as_str()) {
                        return Err(HirCheckCode::Operation);
                    }
                    self.validate_expr(field.expr)?;
                }
                Ok(())
            }
            hir::ExprKind::Loop(..)
            | hir::ExprKind::Match(..)
            | hir::ExprKind::Let(..)
            | hir::ExprKind::Break(..)
            | hir::ExprKind::Continue(..)
            | hir::ExprKind::Become(..)
            | hir::ExprKind::Yield(..) => Err(HirCheckCode::ControlFlow),
            hir::ExprKind::AssignOp(..) => Err(HirCheckCode::Mutation),
            hir::ExprKind::Closure(..) => Err(HirCheckCode::Call),
            hir::ExprKind::InlineAsm(..) => Err(HirCheckCode::Purity),
            hir::ExprKind::MethodCall(..)
            | hir::ExprKind::ConstBlock(..)
            | hir::ExprKind::Use(..)
            | hir::ExprKind::Tup(..)
            | hir::ExprKind::Cast(..)
            | hir::ExprKind::Type(..)
            | hir::ExprKind::AddrOf(..)
            | hir::ExprKind::OffsetOf(..)
            | hir::ExprKind::Repeat(..)
            | hir::ExprKind::UnsafeBinderCast(..)
            | hir::ExprKind::Struct(_, _, _)
            | hir::ExprKind::Lit(..)
            | hir::ExprKind::Err(..) => Err(HirCheckCode::Operation),
        }
    }

    fn validate_block(&mut self, block: &hir::Block<'_>) -> Result<(), HirCheckCode> {
        if block.rules != hir::BlockCheckMode::DefaultBlock || block.targeted_by_break {
            return Err(HirCheckCode::ControlFlow);
        }
        for statement in block.stmts {
            match statement.kind {
                hir::StmtKind::Let(local) => self.validate_local(local)?,
                hir::StmtKind::Expr(expression) | hir::StmtKind::Semi(expression) => {
                    self.validate_expr(expression)?
                }
                hir::StmtKind::Item(_) => return Err(HirCheckCode::Item),
            }
        }
        if let Some(expression) = block.expr {
            self.validate_expr(expression)?;
        }
        Ok(())
    }

    fn validate_identity_adjustments(
        &self,
        expression: &hir::Expr<'_>,
    ) -> Result<(), HirCheckCode> {
        if matches!(
            expression.kind,
            hir::ExprKind::Ret(..)
                | hir::ExprKind::Break(..)
                | hir::ExprKind::Continue(..)
                | hir::ExprKind::Become(..)
                | hir::ExprKind::Yield(..)
        ) {
            return Ok(());
        }
        let mut current = self.typeck.expr_ty(expression);
        for adjustment in self.typeck.expr_adjustments(expression) {
            if current.is_never() && matches!(adjustment.kind, ty::adjustment::Adjust::NeverToAny) {
                current = adjustment.target;
                continue;
            }
            if adjustment.target != current {
                return Err(HirCheckCode::Operation);
            }
            current = adjustment.target;
        }
        Ok(())
    }

    fn validate_local(&mut self, local: &hir::LetStmt<'_>) -> Result<(), HirCheckCode> {
        if local.els.is_some()
            || local.init.is_none()
            || !matches!(local.source, hir::LocalSource::Normal)
        {
            return Err(HirCheckCode::ControlFlow);
        }
        let (binding_id, name, mode) = binding_pattern(local.pat)?;
        let mutable = match mode {
            hir::BindingMode(ByRef::No, Mutability::Not) => false,
            hir::BindingMode(ByRef::No, Mutability::Mut) => true,
            _ => return Err(HirCheckCode::Pattern),
        };
        if let Some(source_ty) = local.ty {
            validate_hir_value_type(self.tcx, source_ty)?;
        }
        validate_value_type(self.tcx, self.def_id, self.typeck.pat_ty(local.pat), 0)?;
        self.validate_expr(local.init.expect("checked initialized local"))?;
        self.add_binding(binding_id, &name, mutable)
    }

    fn add_binding(
        &mut self,
        binding_id: hir::HirId,
        name: &str,
        mutable: bool,
    ) -> Result<(), HirCheckCode> {
        validate_ident(name)?;
        if !self.names.insert(name.to_owned()) {
            return Err(HirCheckCode::Binding);
        }
        if mutable {
            self.mutable_bindings.insert(binding_id);
        }
        Ok(())
    }

    fn validate_call_path(&self, callee: &hir::Expr<'_>) -> Result<(), HirCheckCode> {
        let hir::ExprKind::Path(path) = callee.kind else {
            return Err(HirCheckCode::Call);
        };
        validate_path_shape(&path)?;
        match self.typeck.qpath_res(&path, callee.hir_id) {
            Res::Def(DefKind::Fn, def_id) if def_id.is_local() => Ok(()),
            _ => Err(HirCheckCode::Call),
        }
    }

    fn validate_value_path(
        &self,
        expression: &hir::Expr<'_>,
        path: &hir::QPath<'_>,
    ) -> Result<(), HirCheckCode> {
        validate_path_shape(path)?;
        match self.typeck.qpath_res(path, expression.hir_id) {
            Res::Local(_) => Ok(()),
            Res::Def(DefKind::Const, def_id) if def_id.is_local() => Ok(()),
            Res::Def(DefKind::Static { .. }, _) => Err(HirCheckCode::Purity),
            Res::Def(DefKind::Fn, _) => Err(HirCheckCode::Call),
            Res::Def(_, _) => Err(HirCheckCode::Purity),
            _ => Err(HirCheckCode::Operation),
        }
    }

    fn validate_struct_path(
        &self,
        expression: &hir::Expr<'_>,
        path: &hir::QPath<'_>,
        source_fields: usize,
    ) -> Result<(), HirCheckCode> {
        validate_path_shape(path)?;
        let Res::Def(DefKind::Struct, def_id) = self.typeck.qpath_res(path, expression.hir_id)
        else {
            return Err(HirCheckCode::Type);
        };
        if !def_id.is_local() {
            return Err(HirCheckCode::Purity);
        }
        let ty = self.typeck.expr_ty(expression);
        let ty::Adt(adt, _) = ty.kind() else {
            return Err(HirCheckCode::Type);
        };
        if adt.non_enum_variant().fields.len() != source_fields {
            return Err(HirCheckCode::Operation);
        }
        validate_value_type(self.tcx, self.def_id, ty, 0)
    }

    fn validate_copy_projection(&self, expression: &hir::Expr<'_>) -> Result<(), HirCheckCode> {
        let ty = self.typeck.expr_ty(expression);
        validate_value_type(self.tcx, self.def_id, ty, 0)?;
        let typing_env = ty::TypingEnv::post_analysis(self.tcx, self.def_id);
        if self.tcx.type_is_copy_modulo_regions(typing_env, ty) {
            Ok(())
        } else {
            Err(HirCheckCode::Type)
        }
    }

    fn validate_binary(
        &self,
        operator: BinOpKind,
        left: &hir::Expr<'_>,
        right: &hir::Expr<'_>,
    ) -> Result<(), HirCheckCode> {
        let left_ty = self.typeck.expr_ty(left);
        let right_ty = self.typeck.expr_ty(right);
        let accepted = match operator {
            BinOpKind::And | BinOpKind::Or => left_ty.is_bool() && right_ty.is_bool(),
            BinOpKind::Add
            | BinOpKind::Sub
            | BinOpKind::Mul
            | BinOpKind::Div
            | BinOpKind::Rem
            | BinOpKind::BitXor
            | BinOpKind::BitAnd
            | BinOpKind::BitOr => left_ty == right_ty && is_integer(left_ty),
            BinOpKind::Shl | BinOpKind::Shr => is_integer(left_ty) && is_integer(right_ty),
            BinOpKind::Eq | BinOpKind::Ne => {
                left_ty == right_ty && (left_ty.is_bool() || is_integer(left_ty))
            }
            BinOpKind::Lt | BinOpKind::Le | BinOpKind::Ge | BinOpKind::Gt => {
                left_ty == right_ty && is_integer(left_ty)
            }
        };
        if accepted {
            Ok(())
        } else {
            Err(HirCheckCode::Operation)
        }
    }
}

fn binding_pattern(
    pattern: &hir::Pat<'_>,
) -> Result<(hir::HirId, String, hir::BindingMode), HirCheckCode> {
    match pattern.kind {
        hir::PatKind::Binding(mode, binding_id, ident, None) => {
            Ok((binding_id, ident.as_str().to_owned(), mode))
        }
        _ => Err(HirCheckCode::Pattern),
    }
}

fn validate_path_shape(path: &hir::QPath<'_>) -> Result<(), HirCheckCode> {
    let hir::QPath::Resolved(None, path) = path else {
        return Err(HirCheckCode::Operation);
    };
    if path.is_global() || path.segments.is_empty() {
        return Err(HirCheckCode::Purity);
    }
    for segment in path.segments {
        let name = segment.ident.as_str();
        if !matches!(name, "crate" | "self" | "super") {
            validate_ident(name)?;
        }
        if segment.args.is_some() {
            return Err(HirCheckCode::Generic);
        }
    }
    Ok(())
}

fn validate_hir_primitive_type(source_ty: &hir::Ty<'_>) -> Result<(), HirCheckCode> {
    let hir::TyKind::Path(path) = source_ty.kind else {
        return Err(HirCheckCode::Type);
    };
    let hir::QPath::Resolved(None, path) = path else {
        return Err(HirCheckCode::Type);
    };
    validate_path_shape(&hir::QPath::Resolved(None, path))?;
    let Res::PrimTy(primitive) = path.res else {
        return Err(HirCheckCode::Type);
    };
    if path.segments.len() == 1
        && path.segments[0].ident.as_str() == primitive.name_str()
        && is_hir_primitive(primitive)
    {
        Ok(())
    } else {
        Err(HirCheckCode::Type)
    }
}

fn validate_hir_value_type(tcx: TyCtxt<'_>, source_ty: &hir::Ty<'_>) -> Result<(), HirCheckCode> {
    match source_ty.kind {
        hir::TyKind::Path(path) => {
            let hir::QPath::Resolved(None, resolved) = path else {
                return Err(HirCheckCode::Type);
            };
            validate_path_shape(&path)?;
            match resolved.res {
                Res::PrimTy(primitive)
                    if resolved.segments.len() == 1
                        && resolved.segments[0].ident.as_str() == primitive.name_str()
                        && is_hir_primitive(primitive) =>
                {
                    Ok(())
                }
                Res::Def(DefKind::Struct, def_id) if def_id.is_local() => Ok(()),
                _ => Err(HirCheckCode::Type),
            }
        }
        hir::TyKind::Array(element, length) => {
            validate_hir_value_type(tcx, element)?;
            validate_array_length_source(tcx, length)
        }
        _ => Err(HirCheckCode::Type),
    }
}

fn validate_array_length_source(
    tcx: TyCtxt<'_>,
    length: &hir::ConstArg<'_>,
) -> Result<(), HirCheckCode> {
    let hir::ConstArgKind::Anon(anonymous) = length.kind else {
        return Err(HirCheckCode::Type);
    };
    let mut expression = tcx.hir_body(anonymous.body).value;
    while let hir::ExprKind::DropTemps(inner) = expression.kind {
        expression = inner;
    }
    match expression.kind {
        hir::ExprKind::Lit(literal) if matches!(literal.node, LitKind::Int(..)) => Ok(()),
        hir::ExprKind::Path(path) => {
            validate_path_shape(&path)?;
            let hir::QPath::Resolved(None, resolved) = path else {
                return Err(HirCheckCode::Type);
            };
            let Res::Def(DefKind::Const, def_id) = resolved.res else {
                return Err(HirCheckCode::Type);
            };
            if def_id.is_local() && tcx.type_of(def_id).instantiate_identity().is_usize() {
                Ok(())
            } else {
                Err(HirCheckCode::Type)
            }
        }
        _ => Err(HirCheckCode::Type),
    }
}

fn is_hir_primitive(primitive: hir::PrimTy) -> bool {
    matches!(
        primitive.name_str(),
        "bool" | "i8" | "i16" | "i32" | "i64" | "isize" | "u8" | "u16" | "u32" | "u64" | "usize"
    )
}

fn validate_ident(name: &str) -> Result<(), HirCheckCode> {
    let mut bytes = name.bytes();
    let Some(first) = bytes.next() else {
        return Err(HirCheckCode::Identifier);
    };
    if name == "_"
        || !(first.is_ascii_alphabetic() || first == b'_')
        || !bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    {
        Err(HirCheckCode::Identifier)
    } else {
        Ok(())
    }
}

fn validate_primitive_type<'tcx>(
    tcx: TyCtxt<'tcx>,
    owner: LocalDefId,
    ty: Ty<'tcx>,
) -> Result<(), HirCheckCode> {
    reject_drop(tcx, owner, ty)?;
    if ty.is_bool() || is_integer(ty) {
        Ok(())
    } else {
        Err(HirCheckCode::Type)
    }
}

fn validate_value_type<'tcx>(
    tcx: TyCtxt<'tcx>,
    owner: LocalDefId,
    ty: Ty<'tcx>,
    depth: usize,
) -> Result<(), HirCheckCode> {
    reject_drop(tcx, owner, ty)?;
    if ty.is_bool() || is_integer(ty) {
        return Ok(());
    }
    match ty.kind() {
        ty::Array(element, length) => {
            if depth >= 16 {
                return Err(HirCheckCode::AggregateLimit);
            }
            let typing_env = ty::TypingEnv::post_analysis(tcx, owner);
            let evaluated_length = length.try_to_target_usize(tcx).or_else(|| {
                let ty::ConstKind::Unevaluated(unevaluated) = length.kind() else {
                    return None;
                };
                rustc_middle::mir::Const::Unevaluated(
                    rustc_middle::mir::UnevaluatedConst::new(unevaluated.def, unevaluated.args),
                    tcx.types.usize,
                )
                .try_eval_target_usize(tcx, typing_env)
            });
            match evaluated_length {
                Some(length) if length <= 256 => {}
                Some(_) => return Err(HirCheckCode::AggregateLimit),
                None => return Err(HirCheckCode::Type),
            }
            validate_value_type(tcx, owner, *element, depth + 1)
        }
        ty::Adt(adt, arguments)
            if adt.is_struct() && adt.did().is_local() && arguments.is_empty() =>
        {
            if depth >= 16 || adt.non_enum_variant().fields.len() > 64 {
                return Err(HirCheckCode::AggregateLimit);
            }
            for field in &adt.non_enum_variant().fields {
                validate_value_type(tcx, owner, field.ty(tcx, arguments), depth + 1)?;
            }
            Ok(())
        }
        _ => Err(HirCheckCode::Type),
    }
}

fn reject_drop<'tcx>(
    tcx: TyCtxt<'tcx>,
    owner: LocalDefId,
    ty: Ty<'tcx>,
) -> Result<(), HirCheckCode> {
    let typing_env = ty::TypingEnv::post_analysis(tcx, owner);
    if ty.needs_drop(tcx, typing_env) {
        Err(HirCheckCode::Drop)
    } else {
        Ok(())
    }
}

fn contract_type<'tcx>(
    tcx: TyCtxt<'tcx>,
    owner: LocalDefId,
    ty: Ty<'tcx>,
) -> Result<ContractType, HirCheckCode> {
    match ty.kind() {
        ty::Bool => Ok(ContractType::Bool),
        ty::Int(integer) => Ok(ContractType::BitVector {
            width: match integer {
                IntTy::I8 => 8,
                IntTy::I16 => 16,
                IntTy::I32 => 32,
                IntTy::I64 => 64,
                IntTy::Isize => u8::try_from(tcx.sess.target.pointer_width).unwrap_or(0),
                _ => return Err(HirCheckCode::Type),
            },
            signed: true,
        }),
        ty::Uint(integer) => Ok(ContractType::BitVector {
            width: match integer {
                UintTy::U8 => 8,
                UintTy::U16 => 16,
                UintTy::U32 => 32,
                UintTy::U64 => 64,
                UintTy::Usize => u8::try_from(tcx.sess.target.pointer_width).unwrap_or(0),
                _ => return Err(HirCheckCode::Type),
            },
            signed: false,
        }),
        ty::Array(element, length) => {
            let typing_env = ty::TypingEnv::post_analysis(tcx, owner);
            let length = length.try_to_target_usize(tcx).or_else(|| {
                let ty::ConstKind::Unevaluated(unevaluated) = length.kind() else {
                    return None;
                };
                rustc_middle::mir::Const::Unevaluated(
                    rustc_middle::mir::UnevaluatedConst::new(unevaluated.def, unevaluated.args),
                    tcx.types.usize,
                )
                .try_eval_target_usize(tcx, typing_env)
            });
            Ok(ContractType::Array {
                element: Box::new(contract_type(tcx, owner, *element)?),
                length: length.ok_or(HirCheckCode::Type)?,
            })
        }
        ty::Adt(adt, arguments)
            if adt.is_struct() && adt.did().is_local() && arguments.is_empty() =>
        {
            Ok(ContractType::Struct {
                id: canonical_item_id(tcx, adt.did().as_local().ok_or(HirCheckCode::Type)?),
            })
        }
        _ => Err(HirCheckCode::Type),
    }
}

fn is_integer(ty: Ty<'_>) -> bool {
    matches!(
        ty.kind(),
        ty::Int(IntTy::I8 | IntTy::I16 | IntTy::I32 | IntTy::I64 | IntTy::Isize)
            | ty::Uint(UintTy::U8 | UintTy::U16 | UintTy::U32 | UintTy::U64 | UintTy::Usize)
    )
}

fn is_signed_integer(ty: Ty<'_>) -> bool {
    matches!(
        ty.kind(),
        ty::Int(IntTy::I8 | IntTy::I16 | IntTy::I32 | IntTy::I64 | IntTy::Isize)
    )
}

fn canonical_item_id(tcx: TyCtxt<'_>, def_id: LocalDefId) -> String {
    format!(
        "{}::{}",
        tcx.crate_name(LOCAL_CRATE).as_str(),
        tcx.def_path_str(def_id.to_def_id())
    )
}
