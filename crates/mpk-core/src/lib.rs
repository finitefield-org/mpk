//! Core term, level, declaration, context, reduction, and definitional equality crate.

#![forbid(unsafe_code)]

pub mod context;
pub mod level;
pub mod name;
pub mod term;

pub use context::{LocalContext, LocalDecl, LocalDefinition};
pub use level::{LevelArena, LevelHash, LevelId, LevelNode};
pub use name::{GlobalId, Name, NameError, NameResolver};
pub use term::{TermArena, TermHash, TermId, TermNode};
