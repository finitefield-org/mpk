//! Exact original-source entry environments. This component initializes only
//! actual entry variables. Local declarations have no entry cell; execution,
//! storage permissions, source/IR correspondence and entry validity stay due.
use super::syntax::*;
use super::{error_layout, source_error_layout, Builder, GoOrdinaryError, Result, MAX_BYTES};
use crate::go_practical_type_keys::{
    derive_go_source_entry_layouts, GoSourceEntryBindingOrigin, GoSourceEntryLayout,
    GoSourceSelectionRef, GoSourceValueDomain,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

pub struct GoSourceEntryCatalogue {
    layouts: Vec<GoSourceEntryLayout>,
    builder: Builder,
    source: String,
    cases: Vec<String>,
    prior: Vec<u8>,
    scope: Value,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceEntryRow {
    layout: GoSourceEntryLayout,
    module: String,
    source: String,
    binding: String,
    cases: Vec<String>,
    prior: Vec<u8>,
    certificate: Vec<u8>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceEntryAssembly {
    functions: Vec<(GoSourceSelectionRef, String)>,
    source: String,
    cases: Vec<String>,
    prior: Vec<u8>,
    certificate: Vec<u8>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceEntryAssemblies {
    parts: Vec<GoSourceEntryAssembly>,
    packing_snapshots: Vec<Vec<u8>>,
}
impl GoSourceEntryAssemblies {
    pub fn parts(&self) -> &[GoSourceEntryAssembly] {
        &self.parts
    }
    /// Exact construction states at a packing limit. These are incomplete,
    /// untrusted snapshots, not canonical certificates or checker acceptances.
    pub fn packing_snapshots(&self) -> &[Vec<u8>] {
        &self.packing_snapshots
    }
}
impl GoSourceEntryAssembly {
    pub fn function_modules(&self) -> &[(GoSourceSelectionRef, String)] {
        &self.functions
    }
    pub fn source_layout_module_name(&self) -> &str {
        &self.source
    }
    pub fn case_type_ids(&self) -> &[String] {
        &self.cases
    }
    pub fn prior_certificate_bytes(&self) -> &[u8] {
        &self.prior
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
}
impl GoSourceEntryRow {
    pub fn layout(&self) -> &GoSourceEntryLayout {
        &self.layout
    }
    pub fn module_name(&self) -> &str {
        &self.module
    }
    pub fn source_layout_module_name(&self) -> &str {
        &self.source
    }
    pub fn source_binding_sha256(&self) -> &str {
        &self.binding
    }
    pub fn case_type_ids(&self) -> &[String] {
        &self.cases
    }
    pub fn prior_certificate_bytes(&self) -> &[u8] {
        &self.prior
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
}
fn layout_value(layout: &GoSourceEntryLayout) -> Value {
    let reference = layout.function();
    json!({"function":[reference.source_object_id(),reference.node()],
        "inputs":layout.inputs().iter().map(|s|json!([s.type_id(),s.binding_index()])).collect::<Vec<_>>(),
        "results":layout.results().iter().map(|s|json!([s.type_id(),s.binding_index()])).collect::<Vec<_>>(),
        "bindings":layout.bindings().iter().map(|r|{let b=r.binding();json!({"source":b.source_object_id(),"package":b.package_id(),"declaration":b.declaration_node(),"name_node":b.name_node(),"name":b.name(),"visible_from":b.visible_from(),"type":r.type_id(),"origin":match r.origin(){GoSourceEntryBindingOrigin::Input(n)=>json!(["input",n]),GoSourceEntryBindingOrigin::NamedResult(n)=>json!(["named_result",n]),GoSourceEntryBindingOrigin::Local=>json!(["local"])}})}).collect::<Vec<_>>()})
}

pub fn generate_go_practical_source_entry_catalogue(
    domain: &GoSourceValueDomain,
) -> Result<GoSourceEntryCatalogue> {
    let layouts = derive_go_source_entry_layouts(domain).map_err(|_| GoOrdinaryError::Linkage)?;
    let selection = domain.selection();
    let sources = selection
        .packages()
        .iter()
        .flat_map(|p| p.source_objects.iter())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .map(|id| {
            selection
                .source(id)
                .map(|p| json!([p.source().object(), p.source().bytes()]))
                .ok_or(GoOrdinaryError::Linkage)
        })
        .collect::<Result<Vec<_>>>()?;
    let scope = json!({"sources":sources,"packages":selection.packages(),
        "roots":selection.roots().iter().map(|r|json!([r.source_object_id(),r.node()])).collect::<Vec<_>>(),
        "types":domain.types().collect::<Vec<_>>(),
        "underlying":domain.types().filter_map(|t|domain.underlying(t.id()).map(|u|json!([t.id(),u]))).collect::<Vec<_>>(),
        "cases":domain.error_cases().iter().map(|c|c.value_type_id()).collect::<Vec<_>>(),
        "layouts":layouts.iter().map(layout_value).collect::<Vec<_>>()});
    let (builder, source, cases) = source_error_layout::source_type_builder(domain)?;
    let prior = builder.clone().finish()?;
    Ok(GoSourceEntryCatalogue {
        layouts,
        builder,
        source,
        cases,
        prior,
        scope,
    })
}
impl GoSourceEntryCatalogue {
    pub fn layouts(&self) -> &[GoSourceEntryLayout] {
        &self.layouts
    }
    fn identity(&self, layout: &GoSourceEntryLayout) -> Result<(String, String)> {
        let mut h = Sha256::new();
        h.update(b"MPK-PRIVATE-GO-SOURCE-ENTRY-1\0");
        h.update(
            serde_json::to_vec(&(&self.scope, layout_value(layout)))
                .map_err(|_| GoOrdinaryError::Linkage)?,
        );
        let binding = format!("{:x}", h.finalize());
        let module = format!("Mpk.Go.Practical.SourceEntry.S{binding}");
        Ok((binding, module))
    }
    pub fn generate_row(&self, function: &GoSourceSelectionRef) -> Result<GoSourceEntryRow> {
        let layout = self
            .layouts
            .iter()
            .find(|r| r.function() == function)
            .ok_or(GoOrdinaryError::Linkage)?;
        let (binding, module) = self.identity(layout)?;
        let mut builder = self.builder.clone();
        Entry {
            builder: &mut builder,
            layout,
            module: &module,
            source: &self.source,
        }
        .append()?;
        builder.certificate.module = module.clone();
        Ok(GoSourceEntryRow {
            layout: layout.clone(),
            module,
            source: self.source.clone(),
            binding,
            cases: self.cases.clone(),
            prior: self.prior.clone(),
            certificate: builder.finish()?,
        })
    }
    /// Every original selected/transitive body in one ordinary context. This
    /// accepts no caller-supplied function subset, declaration or source model.
    pub fn generate_assembly(&self) -> Result<GoSourceEntryAssembly> {
        let mut builder = self.builder.clone();
        let mut functions = vec![];
        for layout in &self.layouts {
            let (_, module) = self.identity(layout)?;
            Entry {
                builder: &mut builder,
                layout,
                module: &module,
                source: &self.source,
            }
            .append()?;
            functions.push((layout.function().clone(), module));
        }
        let hash = format!(
            "{:x}",
            Sha256::digest(serde_json::to_vec(&self.scope).map_err(|_| GoOrdinaryError::Linkage)?)
        );
        builder.certificate.module = format!("Mpk.Go.Practical.SourceEntryAssembly.S{hash}");
        Ok(GoSourceEntryAssembly {
            functions,
            source: self.source.clone(),
            cases: self.cases.clone(),
            prior: self.prior.clone(),
            certificate: builder.finish()?,
        })
    }
    pub fn import_assembly(&self, bytes: &[u8]) -> Result<GoSourceEntryAssembly> {
        if bytes.len() > MAX_BYTES {
            return Err(GoOrdinaryError::Limit);
        }
        let expected = self.generate_assembly()?;
        if bytes != expected.certificate_bytes() {
            return Err(GoOrdinaryError::Linkage);
        }
        Ok(expected)
    }
    /// Pack every complete original row without changing source scope or any
    /// checker cap. Every part retains the complete source/type/U foundation.
    /// Parts cover all original declarations; the caller chooses no subset.
    pub fn generate_assemblies(&self) -> Result<GoSourceEntryAssemblies> {
        let mut builder = self.builder.clone();
        let mut functions = vec![];
        let mut parts = vec![];
        let mut packing_snapshots = vec![];
        for layout in &self.layouts {
            let (_, module) = self.identity(layout)?;
            let mut next = builder.clone();
            next.certificate.module = self.part_module(parts.len())?;
            let appended = Entry {
                builder: &mut next,
                layout,
                module: &module,
                source: &self.source,
            }
            .append();
            let fits = match appended {
                Ok(()) => match next.clone().finish() {
                    Ok(_) => true,
                    Err(GoOrdinaryError::Limit) => false,
                    Err(e) => return Err(e),
                },
                Err(GoOrdinaryError::Limit) => false,
                Err(e) => return Err(e),
            };
            if !fits {
                packing_snapshots.push(mpk_cert::encode_certificate(&next.certificate));
                if functions.is_empty() {
                    return Err(GoOrdinaryError::Limit);
                }
                parts.push(self.finish_part(builder, functions, parts.len())?);
                builder = self.builder.clone();
                Entry {
                    builder: &mut builder,
                    layout,
                    module: &module,
                    source: &self.source,
                }
                .append()?;
                functions = vec![(layout.function().clone(), module)];
            } else {
                builder = next;
                functions.push((layout.function().clone(), module));
            }
        }
        if !functions.is_empty() {
            parts.push(self.finish_part(builder, functions, parts.len())?);
        }
        Ok(GoSourceEntryAssemblies {
            parts,
            packing_snapshots,
        })
    }
    fn finish_part(
        &self,
        mut builder: Builder,
        functions: Vec<(GoSourceSelectionRef, String)>,
        part: usize,
    ) -> Result<GoSourceEntryAssembly> {
        builder.certificate.module = self.part_module(part)?;
        Ok(GoSourceEntryAssembly {
            functions,
            source: self.source.clone(),
            cases: self.cases.clone(),
            prior: self.prior.clone(),
            certificate: builder.finish()?,
        })
    }
    fn part_module(&self, part: usize) -> Result<String> {
        let hash = format!(
            "{:x}",
            Sha256::digest(serde_json::to_vec(&self.scope).map_err(|_| GoOrdinaryError::Linkage)?)
        );
        Ok(format!(
            "Mpk.Go.Practical.SourceEntryAssembly.S{hash}.Part{part}"
        ))
    }
    pub fn import_row(
        &self,
        bytes: &[u8],
        function: &GoSourceSelectionRef,
    ) -> Result<GoSourceEntryRow> {
        if bytes.len() > MAX_BYTES {
            return Err(GoOrdinaryError::Limit);
        }
        let expected = self.generate_row(function)?;
        if bytes != expected.certificate_bytes() {
            return Err(GoOrdinaryError::Linkage);
        }
        Ok(expected)
    }
}

struct Entry<'a> {
    builder: &'a mut Builder,
    layout: &'a GoSourceEntryLayout,
    module: &'a str,
    source: &'a str,
}
impl Entry<'_> {
    fn own(&self, name: &str, args: Vec<Expr>) -> Expr {
        call(&format!("{}.{name}", self.module), args)
    }
    fn carrier(&self, id: &str) -> Expr {
        constant(&format!("{}.T{id}.Carrier", self.source))
    }
    fn zero(&self, id: &str) -> Expr {
        constant(&format!("{}.T{id}.Zero", self.source))
    }
    fn emit(&mut self, name: &str, args: &[Binder], ty: Expr, body: Expr, law: bool) -> Result<()> {
        emit(
            self.builder,
            &format!("{}.{name}", self.module),
            args,
            ty,
            body,
            law,
        )
    }
    fn append(&mut self) -> Result<()> {
        let layout = self.layout;
        let input_types = layout
            .inputs()
            .iter()
            .map(|s| self.carrier(s.type_id()))
            .collect::<Vec<_>>();
        let active = layout
            .bindings()
            .iter()
            .enumerate()
            .filter(|(_, b)| b.exists_at_entry())
            .collect::<Vec<_>>();
        let env_types = active
            .iter()
            .map(|(_, b)| self.carrier(b.type_id()))
            .collect::<Vec<_>>();
        let args_type = error_layout::product(&input_types);
        let env_type = error_layout::product(&env_types);
        self.emit("Arguments", &[], Expr::Sort0, args_type.clone(), false)?;
        self.emit("Environment", &[], Expr::Sort0, env_type.clone(), false)?;
        let args = [("arguments", args_type.clone())];
        let values = active
            .iter()
            .map(|(_, binding)| match binding.origin() {
                GoSourceEntryBindingOrigin::Input(i) => {
                    Ok(error_layout::get(&input_types, var("arguments"), i))
                }
                GoSourceEntryBindingOrigin::NamedResult(_) => Ok(self.zero(binding.type_id())),
                GoSourceEntryBindingOrigin::Local => Err(GoOrdinaryError::Linkage),
            })
            .collect::<Result<Vec<_>>>()?;
        self.emit(
            "Initialize",
            &args,
            env_type.clone(),
            error_layout::make(&env_types, &values),
            false,
        )?;
        if active.is_empty() {
            self.emit(
                "EmptyEnvironment",
                &args,
                eq(
                    boolean(),
                    self.own("Initialize", vec![var("arguments")]),
                    bit(false),
                ),
                refl(boolean(), bit(false)),
                true,
            )?;
        }
        for (i, binding) in layout.bindings().iter().enumerate() {
            let present = bit(binding.exists_at_entry());
            self.emit(
                &format!("ExistsAtEntry{i}"),
                &[],
                boolean(),
                present.clone(),
                false,
            )?;
            self.emit(
                &format!("EntryPresence{i}"),
                &[],
                eq(
                    boolean(),
                    self.own(&format!("ExistsAtEntry{i}"), vec![]),
                    present.clone(),
                ),
                refl(boolean(), present),
                true,
            )?;
            if let Some(position) = active.iter().position(|(index, _)| *index == i) {
                let ty = self.carrier(binding.type_id());
                self.emit(
                    &format!("Binding{i}"),
                    &[("environment", env_type.clone())],
                    ty.clone(),
                    error_layout::get(&env_types, var("environment"), position),
                    false,
                )?;
                self.emit(
                    &format!("InitializedBinding{i}"),
                    &args,
                    eq(
                        ty.clone(),
                        self.own(
                            &format!("Binding{i}"),
                            vec![self.own("Initialize", vec![var("arguments")])],
                        ),
                        values[position].clone(),
                    ),
                    refl(ty, values[position].clone()),
                    true,
                )?;
            }
        }
        let parameters = input_types
            .iter()
            .enumerate()
            .map(|(i, t)| (format!("_argument{i}"), t.clone()))
            .collect::<Vec<_>>();
        let arguments = parameters
            .iter()
            .map(|(name, _)| var(name))
            .collect::<Vec<_>>();
        let mut packed = error_layout::make(&input_types, &arguments);
        let mut packed_type = args_type;
        for (name, ty) in parameters.iter().rev() {
            packed = lam(name, ty.clone(), packed);
            packed_type = pi(name, ty.clone(), packed_type);
        }
        self.emit("PackArguments", &[], packed_type, packed, false)?;
        // Share the proof abstraction, keeping every original 0..N parameter
        // and every original fully expanded PackedArgument theorem type.
        if input_types.len() > 1 {
            let pred = pi("_value", error_layout::product(&input_types), Expr::Sort0);
            let mut quantified = app(
                var("_predicate"),
                vec![self.own("PackArguments", arguments.clone())],
            );
            let mut universal = app(
                var("_evidence"),
                vec![self.own("PackArguments", arguments.clone())],
            );
            for (name, ty) in parameters.iter().rev() {
                quantified = pi(name, ty.clone(), quantified);
                universal = lam(name, ty.clone(), universal);
            }
            self.emit(
                "ForAllArguments",
                &[("_predicate", pred.clone())],
                Expr::Sort0,
                quantified,
                false,
            )?;
            self.emit(
                "ForAllArgumentsProof",
                &[
                    ("_predicate", pred),
                    (
                        "_evidence",
                        pi(
                            "_value",
                            error_layout::product(&input_types),
                            app(var("_predicate"), vec![var("_value")]),
                        ),
                    ),
                ],
                self.own("ForAllArguments", vec![var("_predicate")]),
                universal,
                false,
            )?;
        }
        if input_types.is_empty() {
            self.emit(
                "EmptyArguments",
                &[],
                eq(boolean(), self.own("PackArguments", vec![]), bit(false)),
                refl(boolean(), bit(false)),
                true,
            )?;
        }
        for (i, ty) in input_types.iter().enumerate() {
            self.emit(
                &format!("Argument{i}"),
                &args,
                ty.clone(),
                error_layout::get(&input_types, var("arguments"), i),
                false,
            )?;
            let mut claim = eq(
                ty.clone(),
                self.own(
                    &format!("Argument{i}"),
                    vec![self.own("PackArguments", arguments.clone())],
                ),
                arguments[i].clone(),
            );
            let mut proof = refl(ty.clone(), arguments[i].clone());
            for (name, ty) in parameters.iter().rev() {
                claim = pi(name, ty.clone(), claim);
                proof = lam(name, ty.clone(), proof);
            }
            if input_types.len() > 1 {
                let projection = self.own(&format!("Argument{i}"), vec![var("_values")]);
                let predicate = lam(
                    "_values",
                    error_layout::product(&input_types),
                    eq(ty.clone(), projection.clone(), projection.clone()),
                );
                let evidence = lam(
                    "_values",
                    error_layout::product(&input_types),
                    refl(ty.clone(), projection),
                );
                proof = self.own("ForAllArgumentsProof", vec![predicate, evidence]);
            }
            self.emit(&format!("PackedArgument{i}"), &[], claim, proof, true)?;
        }
        Ok(())
    }
}
