#![forbid(unsafe_code)]

pub mod policy_profile;
#[allow(dead_code)]
mod policy_scan;
#[allow(dead_code)]
mod policy_schema;
pub mod program_certificate;
#[doc(hidden)]
pub mod reference_checker;

extern crate self as mpk_cli;

#[allow(dead_code)]
pub mod frontend_protocol;
mod frontend_registry;
#[allow(dead_code)]
mod frontend_runner;
mod frontend_sandbox;
#[cfg(target_os = "linux")]
#[allow(dead_code)]
mod go_practical_analysis_root_binding;
#[cfg(target_os = "linux")]
#[allow(dead_code)]
mod go_practical_capture_binding;
#[allow(dead_code)]
mod go_practical_checked_entries;
#[allow(dead_code)]
mod go_practical_checked_invocations;
#[allow(dead_code)]
mod go_practical_installed_compiler;
#[cfg(target_os = "linux")]
#[allow(dead_code)]
mod go_practical_source_roots;
pub mod successor_ai_explain;
pub mod successor_cli;
pub mod successor_frontend_protocol;
pub mod successor_frontend_runner;
pub mod successor_policy;
pub mod successor_release_bundle;

#[doc(hidden)]
pub fn run_frontend_sandbox_bootstrap(arguments: &[String]) -> u8 {
    frontend_sandbox::run_bootstrap(arguments)
}

#[doc(hidden)]
pub fn run_frontend_sandbox_probe() -> u8 {
    frontend_sandbox::run_probe()
}

mod ai_explain;
