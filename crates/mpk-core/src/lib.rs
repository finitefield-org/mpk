//! Core term, level, declaration, context, reduction, and definitional equality crate.

#![forbid(unsafe_code)]

pub mod level;

pub use level::{LevelArena, LevelHash, LevelId, LevelNode};
