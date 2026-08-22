use rust2vir_internal::driver_protocol::DriverRequest;
use rust2vir_internal::file_loader::{SnapshotFileLoader, SourceLoaderError};
use rust2vir_internal::mir_access::{
    validate_compatibility, MirAccessTracker, MIR_DIALECT_SHA256, MIR_DIALECT_SUMMARY,
    MIR_PROFILE_ID, MIR_QUERY,
};
use rust2vir_internal::session::EffectiveSession;
use rust2vir_internal::EXPECTED_RUSTC_COMMIT;
use rust2vir_internal::{
    contract::{ContractError, ContractFunction, ContractSet},
    contract_typecheck::attach_contracts,
};
use rustc_ast as ast;
use rustc_driver::{Callbacks, Compilation};
use rustc_interface::interface::{Compiler, Config};
use rustc_middle::mir::{MirPhase, RuntimePhase};
use rustc_middle::ty::TyCtxt;
use rustc_session::config::OptLevel;
use rustc_span::def_id::LOCAL_CRATE;
use rustc_span::edition::Edition;
use rustc_span::source_map::FileLoader;
use rustc_target::spec::PanicStrategy;
use std::io;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::Path;
use std::sync::Arc;

#[path = "hir_check.rs"]
mod hir_check;
#[path = "mir_lower.rs"]
mod mir_lower;

pub use hir_check::{HirAnalysis, HirCheckCode};
pub use mir_lower::{MirError, MirLowering};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RustcDriverError {
    Source(SourceLoaderError),
    Session,
    Subset(HirCheckCode),
    Contract(ContractError),
    Mir(MirError),
    MirAdapter,
    Compiler,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PrimaryAnalysis {
    pub hir: HirAnalysis,
    pub contracts: ContractSet,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CallbackPhase {
    Created,
    Configured,
    RootParsed,
    MirBorrowed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)]
enum AnalysisMode {
    Hir,
    Contracts,
    Lower,
}

#[allow(dead_code)]
struct CallbackResult {
    primary: PrimaryAnalysis,
    lowering: Option<MirLowering>,
}

#[allow(dead_code)]
pub fn run_primary(
    arguments: &[String],
    request: &DriverRequest,
    loader: Arc<SnapshotFileLoader>,
) -> Result<MirLowering, RustcDriverError> {
    lower_primary(arguments, request, loader)
}

#[allow(dead_code)]
pub fn analyze_primary(
    arguments: &[String],
    request: &DriverRequest,
    loader: Arc<SnapshotFileLoader>,
) -> Result<PrimaryAnalysis, RustcDriverError> {
    analyze_with_mode(arguments, request, loader, AnalysisMode::Contracts)
        .map(|result| result.primary)
}

pub fn lower_primary(
    arguments: &[String],
    request: &DriverRequest,
    loader: Arc<SnapshotFileLoader>,
) -> Result<MirLowering, RustcDriverError> {
    analyze_with_mode(arguments, request, loader, AnalysisMode::Lower)?
        .lowering
        .ok_or(RustcDriverError::MirAdapter)
}

#[cfg(test)]
#[allow(dead_code)]
pub fn analyze_hir_primary(
    arguments: &[String],
    request: &DriverRequest,
    loader: Arc<SnapshotFileLoader>,
) -> Result<HirAnalysis, RustcDriverError> {
    analyze_with_mode(arguments, request, loader, AnalysisMode::Hir)
        .map(|result| result.primary.hir)
}

fn analyze_with_mode(
    arguments: &[String],
    request: &DriverRequest,
    loader: Arc<SnapshotFileLoader>,
    mode: AnalysisMode,
) -> Result<CallbackResult, RustcDriverError> {
    validate_compatibility(
        MIR_PROFILE_ID,
        EXPECTED_RUSTC_COMMIT,
        MIR_QUERY,
        MIR_DIALECT_SUMMARY,
        MIR_DIALECT_SHA256,
    )
    .map_err(|_| RustcDriverError::MirAdapter)?;
    let mut callbacks = PinnedCallbacks {
        request,
        loader,
        phase: CallbackPhase::Created,
        failure: None,
        analysis: None,
        lowering: None,
        mode,
    };
    if rustc_driver::catch_fatal_errors(|| rustc_driver::run_compiler(arguments, &mut callbacks))
        .is_err()
    {
        return Err(callbacks
            .failure
            .or_else(|| callbacks.loader.failure().map(RustcDriverError::Source))
            .unwrap_or(RustcDriverError::Compiler));
    }
    callbacks.finish()
}

struct PinnedCallbacks<'a> {
    request: &'a DriverRequest,
    loader: Arc<SnapshotFileLoader>,
    phase: CallbackPhase,
    failure: Option<RustcDriverError>,
    analysis: Option<PrimaryAnalysis>,
    lowering: Option<MirLowering>,
    mode: AnalysisMode,
}

impl PinnedCallbacks<'_> {
    fn reject(&mut self, error: RustcDriverError) -> Compilation {
        if self.failure.is_none() {
            self.failure = Some(error);
        }
        Compilation::Stop
    }

    fn finish(self) -> Result<CallbackResult, RustcDriverError> {
        if let Some(error) = self.failure {
            return Err(error);
        }
        if self.phase != CallbackPhase::MirBorrowed {
            return Err(self
                .loader
                .failure()
                .map(RustcDriverError::Source)
                .unwrap_or(RustcDriverError::Compiler));
        }
        self.loader
            .verify_inventory()
            .map_err(RustcDriverError::Source)?;
        let primary = self.analysis.ok_or(RustcDriverError::MirAdapter)?;
        if self.mode == AnalysisMode::Lower && self.lowering.is_none() {
            return Err(RustcDriverError::MirAdapter);
        }
        Ok(CallbackResult {
            primary,
            lowering: self.lowering,
        })
    }
}

impl Callbacks for PinnedCallbacks<'_> {
    fn config(&mut self, config: &mut Config) {
        if self.phase != CallbackPhase::Created || config.file_loader.is_some() {
            self.failure = Some(RustcDriverError::MirAdapter);
        } else {
            self.phase = CallbackPhase::Configured;
        }
        config.file_loader = Some(Box::new(RustcSnapshotLoader(Arc::clone(&self.loader))));
    }

    fn after_crate_root_parsing(
        &mut self,
        _compiler: &Compiler,
        _krate: &mut ast::Crate,
    ) -> Compilation {
        if self.phase != CallbackPhase::Configured {
            return self.reject(RustcDriverError::MirAdapter);
        }
        let path = self.loader.crate_root_path();
        let bytes = self.loader.crate_root_bytes();
        if let Err(error) = self.loader.validate_root_ast(&path, &bytes) {
            return self.reject(RustcDriverError::Source(error));
        }
        self.phase = CallbackPhase::RootParsed;
        Compilation::Continue
    }

    fn after_analysis<'tcx>(&mut self, _compiler: &Compiler, tcx: TyCtxt<'tcx>) -> Compilation {
        if self.phase != CallbackPhase::RootParsed {
            return self.reject(RustcDriverError::MirAdapter);
        }
        let effective = effective_session(tcx);
        if effective.validate(self.request).is_err() {
            return self.reject(RustcDriverError::Session);
        }

        let function = self.request.selection().2;
        let analysis = match hir_check::analyze_hir(tcx, function) {
            Ok(analysis) => analysis,
            Err(error) => return self.reject(RustcDriverError::Subset(error)),
        };
        let contracts = if self.mode != AnalysisMode::Hir {
            let signatures = analysis
                .call_closure
                .iter()
                .map(|function| ContractFunction {
                    function_id: function.function_id.clone(),
                    parameter_names: function.parameter_names.clone(),
                    parameter_types: function.parameter_types.clone(),
                    result_type: function.result_type.clone(),
                })
                .collect::<Vec<_>>();
            match attach_contracts(
                self.loader.contract_inputs(),
                &signatures,
                self.request.target(),
                self.request.pointer_width(),
            ) {
                Ok(contracts) => contracts,
                Err(error) => return self.reject(RustcDriverError::Contract(error)),
            }
        } else {
            ContractSet::default()
        };

        let crate_name_symbol = tcx.crate_name(LOCAL_CRATE);
        let crate_name = crate_name_symbol.as_str();
        let mut definitions = analysis
            .call_closure
            .iter()
            .map(|function| function.function_id.as_str())
            .map(|function_id| {
                let mut matching = tcx.mir_keys(()).iter().copied().filter(|def_id| {
                    format!("{crate_name}::{}", tcx.def_path_str(def_id.to_def_id())) == function_id
                });
                let result = matching.next().filter(|_| matching.next().is_none());
                (function_id, result)
            })
            .collect::<Vec<_>>();
        if definitions.iter().any(|(_, def_id)| def_id.is_none()) {
            return self.reject(RustcDriverError::MirAdapter);
        }

        let mut access = match MirAccessTracker::new(
            analysis
                .call_closure
                .iter()
                .map(|function| function.function_id.clone()),
        ) {
            Ok(access) => access,
            Err(_) => return self.reject(RustcDriverError::MirAdapter),
        };
        definitions.sort_by_key(|(function_id, _)| *function_id);
        let mut lowered = Vec::new();
        for (function_id, def_id) in definitions {
            if access.force(function_id, MIR_QUERY).is_err() {
                return self.reject(RustcDriverError::MirAdapter);
            }
            let body = match catch_unwind(AssertUnwindSafe(|| {
                tcx.mir_drops_elaborated_and_const_checked(def_id.expect("checked definition"))
                    .borrow()
            })) {
                Ok(body) => body,
                Err(_) => return self.reject(RustcDriverError::MirAdapter),
            };
            if body.phase != MirPhase::Runtime(RuntimePhase::PostCleanup)
                || access.mark_borrowed(function_id).is_err()
            {
                return self.reject(RustcDriverError::MirAdapter);
            }
            if self.mode == AnalysisMode::Lower {
                let function = analysis
                    .call_closure
                    .iter()
                    .find(|function| function.function_id == function_id)
                    .expect("definition was built from analyzed closure");
                let contract = contracts
                    .get(function_id)
                    .expect("contract attachment covers the analyzed closure");
                match mir_lower::lower_function(
                    tcx,
                    def_id.expect("checked definition"),
                    &body,
                    function,
                    &contract.value,
                    &self.loader,
                ) {
                    Ok(function) => lowered.push(function),
                    Err(error) => return self.reject(RustcDriverError::Mir(error)),
                }
            }
            drop(body);
        }
        if access.finish().is_err() {
            return self.reject(RustcDriverError::MirAdapter);
        }
        if self.mode == AnalysisMode::Lower {
            self.lowering = match mir_lower::finish_module(self.request, lowered) {
                Ok(lowering) => Some(lowering),
                Err(error) => return self.reject(RustcDriverError::Mir(error)),
            };
        }
        self.analysis = Some(PrimaryAnalysis {
            hir: analysis,
            contracts,
        });
        self.phase = CallbackPhase::MirBorrowed;
        Compilation::Stop
    }
}

fn effective_session(tcx: TyCtxt<'_>) -> EffectiveSession {
    let mut enabled_features = Vec::new();
    let mut cfg = Vec::new();
    for &(name, value) in &tcx.sess.psess.config {
        let name = name.as_str();
        if name == "feature" {
            if let Some(value) = value {
                enabled_features.push(value.as_str().to_owned());
            }
        } else {
            cfg.push(value.map_or_else(
                || name.to_owned(),
                |value| format!("{name}=\"{}\"", value.as_str()),
            ));
        }
    }
    enabled_features.sort();
    cfg.sort();
    EffectiveSession {
        edition: match tcx.sess.edition() {
            Edition::Edition2021 => "2021",
            _ => "other",
        }
        .to_owned(),
        target_id: tcx.sess.opts.target_triple.tuple().to_owned(),
        pointer_width: u8::try_from(tcx.sess.target.pointer_width).unwrap_or(0),
        panic_strategy: match tcx.sess.panic_strategy() {
            PanicStrategy::Abort => "abort",
            PanicStrategy::Unwind => "unwind",
        }
        .to_owned(),
        overflow_checks: tcx.sess.overflow_checks(),
        debug_assertions: tcx.sess.opts.debug_assertions,
        rustc_opt_level: match tcx.sess.opts.optimize {
            OptLevel::No => 0,
            _ => u8::MAX,
        },
        mir_opt_level: u8::try_from(tcx.sess.mir_opt_level()).unwrap_or(u8::MAX),
        enabled_features,
        cfg,
    }
}

struct RustcSnapshotLoader(Arc<SnapshotFileLoader>);

impl FileLoader for RustcSnapshotLoader {
    fn file_exists(&self, path: &Path) -> bool {
        self.0.file_exists(path)
    }

    fn read_file(&self, path: &Path) -> io::Result<String> {
        self.0.read_file(path)
    }

    fn read_binary_file(&self, path: &Path) -> io::Result<Arc<[u8]>> {
        self.0.read_binary_file(path)
    }
}
