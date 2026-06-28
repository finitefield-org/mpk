//! Core term, level, declaration, context, reduction, and definitional equality crate.

#![forbid(unsafe_code)]

pub mod context;
pub mod level;
pub mod name;
pub mod reduce;
pub mod subst;
pub mod term;

pub use context::{LocalContext, LocalDecl, LocalDefinition};
pub use level::{LevelArena, LevelHash, LevelId, LevelNode};
pub use name::{GlobalId, Name, NameError, NameResolver};
pub use reduce::{whnf, whnf_with_fuel, ReduceError, DEFAULT_WHNF_FUEL};
pub use subst::{beta_substitute, lift, lift_from, substitute, substitute_top, SubstError};
pub use term::{TermArena, TermHash, TermId, TermNode};
