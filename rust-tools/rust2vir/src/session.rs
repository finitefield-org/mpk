use crate::driver_protocol::DriverRequest;

const I686_CFG: &[&str] = &[
    "fmt_debug=\"full\"",
    "overflow_checks",
    "panic=\"abort\"",
    "relocation_model=\"pic\"",
    "target_abi=\"\"",
    "target_arch=\"x86\"",
    "target_endian=\"little\"",
    "target_env=\"gnu\"",
    "target_family=\"unix\"",
    "target_feature=\"fxsr\"",
    "target_feature=\"sse\"",
    "target_feature=\"sse2\"",
    "target_feature=\"x87\"",
    "target_has_atomic",
    "target_has_atomic=\"16\"",
    "target_has_atomic=\"32\"",
    "target_has_atomic=\"64\"",
    "target_has_atomic=\"8\"",
    "target_has_atomic=\"ptr\"",
    "target_has_atomic_equal_alignment=\"16\"",
    "target_has_atomic_equal_alignment=\"32\"",
    "target_has_atomic_equal_alignment=\"8\"",
    "target_has_atomic_equal_alignment=\"ptr\"",
    "target_has_atomic_load_store",
    "target_has_atomic_load_store=\"16\"",
    "target_has_atomic_load_store=\"32\"",
    "target_has_atomic_load_store=\"64\"",
    "target_has_atomic_load_store=\"8\"",
    "target_has_atomic_load_store=\"ptr\"",
    "target_has_reliable_f16",
    "target_has_reliable_f16_math",
    "target_os=\"linux\"",
    "target_pointer_width=\"32\"",
    "target_thread_local",
    "target_vendor=\"unknown\"",
    "unix",
];

const X86_64_CFG: &[&str] = &[
    "fmt_debug=\"full\"",
    "overflow_checks",
    "panic=\"abort\"",
    "relocation_model=\"pic\"",
    "target_abi=\"\"",
    "target_arch=\"x86_64\"",
    "target_endian=\"little\"",
    "target_env=\"gnu\"",
    "target_family=\"unix\"",
    "target_feature=\"fxsr\"",
    "target_feature=\"sse\"",
    "target_feature=\"sse2\"",
    "target_feature=\"x87\"",
    "target_has_atomic",
    "target_has_atomic=\"16\"",
    "target_has_atomic=\"32\"",
    "target_has_atomic=\"64\"",
    "target_has_atomic=\"8\"",
    "target_has_atomic=\"ptr\"",
    "target_has_atomic_equal_alignment=\"16\"",
    "target_has_atomic_equal_alignment=\"32\"",
    "target_has_atomic_equal_alignment=\"64\"",
    "target_has_atomic_equal_alignment=\"8\"",
    "target_has_atomic_equal_alignment=\"ptr\"",
    "target_has_atomic_load_store",
    "target_has_atomic_load_store=\"16\"",
    "target_has_atomic_load_store=\"32\"",
    "target_has_atomic_load_store=\"64\"",
    "target_has_atomic_load_store=\"8\"",
    "target_has_atomic_load_store=\"ptr\"",
    "target_has_reliable_f128",
    "target_has_reliable_f16",
    "target_has_reliable_f16_math",
    "target_os=\"linux\"",
    "target_pointer_width=\"64\"",
    "target_thread_local",
    "target_vendor=\"unknown\"",
    "unix",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectiveSession {
    pub edition: String,
    pub target_id: String,
    pub pointer_width: u8,
    pub panic_strategy: String,
    pub overflow_checks: bool,
    pub debug_assertions: bool,
    pub rustc_opt_level: u8,
    pub mir_opt_level: u8,
    pub enabled_features: Vec<String>,
    pub cfg: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionField {
    Edition,
    Target,
    PointerWidth,
    PanicStrategy,
    OverflowChecks,
    DebugAssertions,
    RustcOptLevel,
    MirOptLevel,
    Features,
    Cfg,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SessionError {
    pub field: SessionField,
}

impl EffectiveSession {
    pub fn validate(&self, request: &DriverRequest) -> Result<(), SessionError> {
        let expected_cfg = target_cfg(request.target()).ok_or(SessionError {
            field: SessionField::Target,
        })?;
        let checks = [
            (self.edition == "2021", SessionField::Edition),
            (self.target_id == request.target(), SessionField::Target),
            (
                self.pointer_width == request.pointer_width(),
                SessionField::PointerWidth,
            ),
            (self.panic_strategy == "abort", SessionField::PanicStrategy),
            (self.overflow_checks, SessionField::OverflowChecks),
            (!self.debug_assertions, SessionField::DebugAssertions),
            (self.rustc_opt_level == 0, SessionField::RustcOptLevel),
            (self.mir_opt_level == 0, SessionField::MirOptLevel),
            (self.enabled_features.is_empty(), SessionField::Features),
            (
                self.cfg
                    .iter()
                    .map(String::as_str)
                    .eq(expected_cfg.iter().copied()),
                SessionField::Cfg,
            ),
        ];
        checks
            .into_iter()
            .find_map(|(matches, field)| (!matches).then_some(SessionError { field }))
            .map_or(Ok(()), Err)
    }
}

pub fn target_cfg(target: &str) -> Option<&'static [&'static str]> {
    match target {
        "i686-unknown-linux-gnu" => Some(I686_CFG),
        "x86_64-unknown-linux-gnu" => Some(X86_64_CFG),
        _ => None,
    }
}
