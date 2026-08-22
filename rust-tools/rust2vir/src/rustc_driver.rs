use rust2vir_internal::driver_protocol::DriverRequest;
use rust2vir_internal::file_loader::{SnapshotFileLoader, SourceLoaderError};
use rust2vir_internal::mir_access::{
    validate_compatibility, MirAccessTracker, MIR_DIALECT_SHA256, MIR_DIALECT_SUMMARY,
    MIR_PROFILE_ID, MIR_QUERY,
};
use rust2vir_internal::session::EffectiveSession;
use rust2vir_internal::EXPECTED_RUSTC_COMMIT;
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RustcDriverError {
    Source(SourceLoaderError),
    Session,
    MirAdapter,
    Compiler,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CallbackPhase {
    Created,
    Configured,
    RootParsed,
    MirBorrowed,
}

pub fn run_primary(
    arguments: &[String],
    request: &DriverRequest,
    loader: Arc<SnapshotFileLoader>,
) -> Result<(), RustcDriverError> {
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
}

impl PinnedCallbacks<'_> {
    fn reject(&mut self, error: RustcDriverError) -> Compilation {
        if self.failure.is_none() {
            self.failure = Some(error);
        }
        Compilation::Stop
    }

    fn finish(self) -> Result<(), RustcDriverError> {
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
            .map_err(RustcDriverError::Source)
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
        let crate_name_symbol = tcx.crate_name(LOCAL_CRATE);
        let crate_name = crate_name_symbol.as_str();
        let mut matching = tcx.mir_keys(()).iter().copied().filter(|def_id| {
            format!("{crate_name}::{}", tcx.def_path_str(def_id.to_def_id())) == function
        });
        let Some(def_id) = matching.next() else {
            return self.reject(RustcDriverError::MirAdapter);
        };
        if matching.next().is_some() {
            return self.reject(RustcDriverError::MirAdapter);
        }

        let mut access = match MirAccessTracker::new([function.to_owned()]) {
            Ok(access) => access,
            Err(_) => return self.reject(RustcDriverError::MirAdapter),
        };
        if access.force(function, MIR_QUERY).is_err() {
            return self.reject(RustcDriverError::MirAdapter);
        }
        let body = match catch_unwind(AssertUnwindSafe(|| {
            tcx.mir_drops_elaborated_and_const_checked(def_id).borrow()
        })) {
            Ok(body) => body,
            Err(_) => return self.reject(RustcDriverError::MirAdapter),
        };
        if body.phase != MirPhase::Runtime(RuntimePhase::PostCleanup)
            || access.mark_borrowed(function).is_err()
            || access.finish().is_err()
        {
            return self.reject(RustcDriverError::MirAdapter);
        }
        drop(body);
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
