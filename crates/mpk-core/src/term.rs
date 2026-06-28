//! Core term arena, spine application, and structural hashing.

use std::collections::HashMap;

use crate::{GlobalId, LevelArena, LevelId};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct TermId(u32);

impl TermId {
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum TermNode {
    Sort(LevelId),
    Var(u32),
    Const {
        global: GlobalId,
        levels: Vec<LevelId>,
    },
    App {
        function: TermId,
        arguments: Vec<TermId>,
    },
    Lam {
        ty: TermId,
        body: TermId,
    },
    Pi {
        ty: TermId,
        body: TermId,
    },
    Let {
        ty: TermId,
        value: TermId,
        body: TermId,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct TermHash(u64);

impl TermHash {
    pub fn as_u64(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Default)]
pub struct TermArena {
    nodes: Vec<TermNode>,
    interned: HashMap<TermNode, TermId>,
}

impl TermArena {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn node(&self, id: TermId) -> &TermNode {
        &self.nodes[id.index()]
    }

    pub fn iter_topological(&self) -> impl Iterator<Item = (TermId, &TermNode)> {
        self.nodes
            .iter()
            .enumerate()
            .map(|(index, node)| (TermId(index as u32), node))
    }

    pub fn dependencies(&self, id: TermId) -> Vec<TermId> {
        match self.node(id) {
            TermNode::Sort(_) | TermNode::Var(_) | TermNode::Const { .. } => Vec::new(),
            TermNode::App {
                function,
                arguments,
            } => {
                let mut deps = Vec::with_capacity(arguments.len() + 1);
                deps.push(*function);
                deps.extend(arguments.iter().copied());
                deps
            }
            TermNode::Lam { ty, body } | TermNode::Pi { ty, body } => vec![*ty, *body],
            TermNode::Let { ty, value, body } => vec![*ty, *value, *body],
        }
    }

    pub fn sort(&mut self, level: LevelId) -> TermId {
        self.intern(TermNode::Sort(level))
    }

    pub fn var(&mut self, index: u32) -> TermId {
        self.intern(TermNode::Var(index))
    }

    pub fn constant<I>(&mut self, global: GlobalId, levels: I) -> TermId
    where
        I: IntoIterator<Item = LevelId>,
    {
        self.intern(TermNode::Const {
            global,
            levels: levels.into_iter().collect(),
        })
    }

    pub fn app<I>(&mut self, function: TermId, arguments: I) -> TermId
    where
        I: IntoIterator<Item = TermId>,
    {
        let arguments: Vec<_> = arguments.into_iter().collect();
        if arguments.is_empty() {
            return function;
        }

        let (function, mut flattened_arguments) = match self.node(function).clone() {
            TermNode::App {
                function,
                arguments,
            } => (function, arguments),
            _ => (function, Vec::new()),
        };
        flattened_arguments.extend(arguments);

        self.intern(TermNode::App {
            function,
            arguments: flattened_arguments,
        })
    }

    pub fn lam(&mut self, ty: TermId, body: TermId) -> TermId {
        self.intern(TermNode::Lam { ty, body })
    }

    pub fn pi(&mut self, ty: TermId, body: TermId) -> TermId {
        self.intern(TermNode::Pi { ty, body })
    }

    pub fn let_term(&mut self, ty: TermId, value: TermId, body: TermId) -> TermId {
        self.intern(TermNode::Let { ty, value, body })
    }

    pub fn structural_hash(&mut self, levels: &mut LevelArena, id: TermId) -> TermHash {
        let node = self.node(id).clone();
        let mut hasher = StableTermHasher::new();
        match node {
            TermNode::Sort(level) => {
                hasher.write_u8(0);
                hasher.write_u64(levels.stable_hash(level).as_u64());
            }
            TermNode::Var(index) => {
                hasher.write_u8(1);
                hasher.write_u64(u64::from(index));
            }
            TermNode::Const { global, levels: ls } => {
                hasher.write_u8(2);
                hasher.write_u64(u64::from(global.as_u32()));
                hasher.write_u64(ls.len() as u64);
                for level in ls {
                    hasher.write_u64(levels.stable_hash(level).as_u64());
                }
            }
            TermNode::App {
                function,
                arguments,
            } => {
                hasher.write_u8(3);
                hasher.write_u64(self.structural_hash(levels, function).as_u64());
                hasher.write_u64(arguments.len() as u64);
                for argument in arguments {
                    hasher.write_u64(self.structural_hash(levels, argument).as_u64());
                }
            }
            TermNode::Lam { ty, body } => {
                hasher.write_u8(4);
                hasher.write_u64(self.structural_hash(levels, ty).as_u64());
                hasher.write_u64(self.structural_hash(levels, body).as_u64());
            }
            TermNode::Pi { ty, body } => {
                hasher.write_u8(5);
                hasher.write_u64(self.structural_hash(levels, ty).as_u64());
                hasher.write_u64(self.structural_hash(levels, body).as_u64());
            }
            TermNode::Let { ty, value, body } => {
                hasher.write_u8(6);
                hasher.write_u64(self.structural_hash(levels, ty).as_u64());
                hasher.write_u64(self.structural_hash(levels, value).as_u64());
                hasher.write_u64(self.structural_hash(levels, body).as_u64());
            }
        }

        TermHash(hasher.finish())
    }

    fn intern(&mut self, node: TermNode) -> TermId {
        if let Some(id) = self.interned.get(&node) {
            return *id;
        }

        let index = u32::try_from(self.nodes.len()).expect("term arena exceeded u32 ids");
        let id = TermId(index);
        self.nodes.push(node.clone());
        self.interned.insert(node, id);
        id
    }
}

struct StableTermHasher(u64);

impl StableTermHasher {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;

    fn new() -> Self {
        Self(Self::OFFSET)
    }

    fn write_u8(&mut self, byte: u8) {
        self.0 ^= u64::from(byte);
        self.0 = self.0.wrapping_mul(Self::PRIME);
    }

    fn write_u64(&mut self, value: u64) {
        self.write_bytes(&value.to_le_bytes());
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.write_u8(*byte);
        }
    }

    fn finish(self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::{LevelArena, NameResolver, TermArena, TermNode};

    fn param(levels: &mut LevelArena, name: &str) -> crate::LevelId {
        levels.parse_param(name).expect("valid level param name")
    }

    #[test]
    fn interns_identical_terms() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let u = param(&mut levels, "u");

        let first_sort = terms.sort(u);
        let second_sort = terms.sort(u);
        let first_var = terms.var(0);
        let second_var = terms.var(0);

        assert_eq!(first_sort, second_sort);
        assert_eq!(first_var, second_var);
        assert_eq!(terms.len(), 2);
    }

    #[test]
    fn empty_application_returns_function() {
        let mut terms = TermArena::new();
        let f = terms.var(0);

        assert_eq!(terms.app(f, Vec::new()), f);
    }

    #[test]
    fn application_uses_spine_form() {
        let mut terms = TermArena::new();
        let f = terms.var(2);
        let x = terms.var(1);
        let y = terms.var(0);

        let fx = terms.app(f, [x]);
        let fxy = terms.app(fx, [y]);

        assert_eq!(
            terms.node(fxy),
            &TermNode::App {
                function: f,
                arguments: vec![x, y],
            }
        );
    }

    #[test]
    fn terms_are_topologically_inspectable() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let u = param(&mut levels, "u");
        let ty = terms.sort(u);
        let var = terms.var(0);
        let lam = terms.lam(ty, var);
        let pi = terms.pi(ty, ty);
        let _let_term = terms.let_term(ty, lam, pi);

        for (id, _) in terms.iter_topological() {
            for dependency in terms.dependencies(id) {
                assert!(
                    dependency.index() < id.index(),
                    "term {id:?} depends on non-earlier term {dependency:?}"
                );
            }
        }
    }

    #[test]
    fn structural_hash_is_stable_for_interned_terms() {
        let mut levels = LevelArena::new();
        let mut terms = TermArena::new();
        let u = param(&mut levels, "u");
        let ty = terms.sort(u);
        let var = terms.var(0);

        let first = terms.lam(ty, var);
        let second = terms.lam(ty, var);
        let first_hash = terms.structural_hash(&mut levels, first);
        let second_hash = terms.structural_hash(&mut levels, second);

        assert_eq!(first, second);
        assert_eq!(first_hash, second_hash);
        assert_ne!(first_hash, terms.structural_hash(&mut levels, ty));
    }

    #[test]
    fn const_hash_includes_level_arguments() {
        let mut levels = LevelArena::new();
        let mut names = NameResolver::new();
        let mut terms = TermArena::new();
        let u = param(&mut levels, "u");
        let v = param(&mut levels, "v");
        let core_id = names.register("Core.Id").expect("valid global name");

        let const_u = terms.constant(core_id, [u]);
        let const_v = terms.constant(core_id, [v]);
        let const_u_hash = terms.structural_hash(&mut levels, const_u);
        let const_v_hash = terms.structural_hash(&mut levels, const_v);

        assert_ne!(const_u, const_v);
        assert_ne!(const_u_hash, const_v_hash);
    }
}
