//! Core term, level, declaration, context, reduction, and definitional equality crate.

#![forbid(unsafe_code)]

pub mod context;
pub mod decl_check;
pub mod defeq;
pub mod env;
pub mod error;
pub mod inductive;
pub mod infer;
pub mod level;
pub mod name;
pub mod positivity;
pub mod reduce;
pub mod subst;
pub mod term;

pub use context::{LocalContext, LocalDecl, LocalDefinition};
pub use decl_check::{check_theorem, register_checked_theorem};
pub use defeq::{definitionally_equal, definitionally_equal_with_fuel, DEFAULT_DEFEQ_FUEL};
pub use env::{Declaration, DeclarationKind, DefinitionReducibility, Environment};
pub use error::{CoreError, CoreErrorCode, CoreLocation, CoreLocationPart};
pub use inductive::{
    export_registered_inductive, register_mvp_inductive, ConstructorSignature, ExportedInductive,
    ExportedInductiveDeclaration, InductiveSpec, MvpInductiveShape, RecursorSignature,
    RegisteredInductive,
};
pub use infer::{check, infer, infer_sort};
pub use level::{LevelArena, LevelHash, LevelId, LevelNode};
pub use name::{GlobalId, Name, NameError, NameResolver};
pub use positivity::{check_mvp_positivity, PositivityErrorKind};
pub use reduce::{whnf, whnf_with_fuel, ReduceError, DEFAULT_WHNF_FUEL};
pub use subst::{beta_substitute, lift, lift_from, substitute, substitute_top, SubstError};
pub use term::{TermArena, TermHash, TermId, TermNode};
