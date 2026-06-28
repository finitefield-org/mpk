//! Core term, level, declaration, context, reduction, and definitional equality crate.

#![forbid(unsafe_code)]

pub mod level;
pub mod term;

pub use level::{LevelArena, LevelHash, LevelId, LevelNode};
pub use term::{GlobalId, TermArena, TermHash, TermId, TermNode};
