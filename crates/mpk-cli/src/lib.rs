#![forbid(unsafe_code)]

pub mod policy_profile;
pub mod policy_report;
pub mod policy_scan;
pub mod policy_schema;
pub mod policy_verify;
pub mod program_certificate;
#[doc(hidden)]
pub mod reference_checker;

extern crate self as mpk_cli;

pub mod frontend_protocol;
mod frontend_registry;
mod frontend_runner;
mod frontend_sandbox;
#[doc(hidden)]
pub mod successor_frontend_protocol;
#[doc(hidden)]
pub mod successor_frontend_runner;
#[doc(hidden)]
pub mod successor_policy;
#[doc(hidden)]
pub mod successor_release_bundle;

#[doc(hidden)]
pub fn run_frontend_sandbox_bootstrap(arguments: &[String]) -> u8 {
    frontend_sandbox::run_bootstrap(arguments)
}

#[doc(hidden)]
pub fn run_frontend_sandbox_probe() -> u8 {
    frontend_sandbox::run_probe()
}

pub mod ai_explain;
