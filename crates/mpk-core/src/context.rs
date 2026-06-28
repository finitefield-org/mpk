//! Local contexts for de Bruijn-indexed variables and local definitions.

use crate::TermId;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum LocalDecl {
    Binder { ty: TermId },
    Definition { ty: TermId, value: TermId },
}

impl LocalDecl {
    pub fn ty(self) -> TermId {
        match self {
            Self::Binder { ty } | Self::Definition { ty, .. } => ty,
        }
    }

    pub fn value(self) -> Option<TermId> {
        match self {
            Self::Binder { .. } => None,
            Self::Definition { value, .. } => Some(value),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct LocalDefinition {
    pub ty: TermId,
    pub value: TermId,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LocalContext {
    entries: Vec<LocalDecl>,
}

impl LocalContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn push_binder(&mut self, ty: TermId) {
        self.entries.push(LocalDecl::Binder { ty });
    }

    pub fn push_definition(&mut self, ty: TermId, value: TermId) {
        self.entries.push(LocalDecl::Definition { ty, value });
    }

    pub fn pop(&mut self) -> Option<LocalDecl> {
        self.entries.pop()
    }

    pub fn lookup_var(&self, index: u32) -> Option<LocalDecl> {
        let position = self.position_for_de_bruijn(index)?;
        self.entries.get(position).copied()
    }

    pub fn lookup_var_type(&self, index: u32) -> Option<TermId> {
        self.lookup_var(index).map(LocalDecl::ty)
    }

    pub fn lookup_definition(&self, index: u32) -> Option<LocalDefinition> {
        match self.lookup_var(index)? {
            LocalDecl::Binder { .. } => None,
            LocalDecl::Definition { ty, value } => Some(LocalDefinition { ty, value }),
        }
    }

    pub fn iter_outer_to_inner(&self) -> impl DoubleEndedIterator<Item = LocalDecl> + '_ {
        self.entries.iter().copied()
    }

    pub fn iter_inner_to_outer(&self) -> impl DoubleEndedIterator<Item = LocalDecl> + '_ {
        self.entries.iter().rev().copied()
    }

    fn position_for_de_bruijn(&self, index: u32) -> Option<usize> {
        let offset = usize::try_from(index).ok()?;
        let offset = offset.checked_add(1)?;
        self.entries.len().checked_sub(offset)
    }
}

#[cfg(test)]
mod tests {
    use crate::{LevelArena, LocalContext, LocalDecl, LocalDefinition, TermArena};

    fn sort(terms: &mut TermArena, levels: &mut LevelArena, name: &str) -> crate::TermId {
        let level = levels.parse_param(name).expect("valid level param name");
        terms.sort(level)
    }

    #[test]
    fn var_lookup_uses_de_bruijn_order() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let outer_ty = sort(&mut terms, &mut levels, "u");
        let inner_ty = sort(&mut terms, &mut levels, "v");
        let mut context = LocalContext::new();

        context.push_binder(outer_ty);
        context.push_binder(inner_ty);

        assert_eq!(
            context.lookup_var(0),
            Some(LocalDecl::Binder { ty: inner_ty })
        );
        assert_eq!(
            context.lookup_var(1),
            Some(LocalDecl::Binder { ty: outer_ty })
        );
        assert_eq!(context.lookup_var_type(0), Some(inner_ty));
        assert_eq!(context.lookup_var_type(2), None);
    }

    #[test]
    fn local_definition_lookup_returns_value_and_type() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let outer_ty = sort(&mut terms, &mut levels, "u");
        let definition_ty = sort(&mut terms, &mut levels, "v");
        let definition_value = terms.var(0);
        let mut context = LocalContext::new();

        context.push_binder(outer_ty);
        context.push_definition(definition_ty, definition_value);

        assert_eq!(
            context.lookup_var(0),
            Some(LocalDecl::Definition {
                ty: definition_ty,
                value: definition_value,
            })
        );
        assert_eq!(
            context.lookup_definition(0),
            Some(LocalDefinition {
                ty: definition_ty,
                value: definition_value,
            })
        );
        assert_eq!(context.lookup_definition(1), None);
        assert_eq!(context.lookup_definition(2), None);
    }

    #[test]
    fn popping_context_restores_outer_variable_index() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let outer_ty = sort(&mut terms, &mut levels, "u");
        let inner_ty = sort(&mut terms, &mut levels, "v");
        let mut context = LocalContext::new();

        context.push_binder(outer_ty);
        context.push_binder(inner_ty);

        assert_eq!(context.pop(), Some(LocalDecl::Binder { ty: inner_ty }));
        assert_eq!(context.lookup_var_type(0), Some(outer_ty));
        assert_eq!(context.len(), 1);
    }
}
