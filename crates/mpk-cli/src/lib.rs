#![forbid(unsafe_code)]

pub mod policy_callsite;
pub mod policy_evidence;
pub mod policy_report;
pub mod policy_scan;
pub mod policy_schema;
pub mod policy_verify;

#[cfg(test)]
extern crate self as mpk_cli;
#[cfg(test)]
mod ai_explain_v1;

pub mod frontend_protocol;
#[allow(dead_code)]
mod frontend_registry;
#[allow(dead_code)]
mod frontend_runner;
#[allow(dead_code)]
mod frontend_sandbox;

#[doc(hidden)]
pub fn run_frontend_sandbox_bootstrap(arguments: &[String]) -> u8 {
    frontend_sandbox::run_bootstrap(arguments)
}

#[doc(hidden)]
pub fn run_frontend_sandbox_probe() -> u8 {
    frontend_sandbox::run_probe()
}

#[cfg(feature = "vertex-ai")]
pub mod ai_explain;
#[cfg(feature = "vertex-ai")]
pub mod vertex_ai;
