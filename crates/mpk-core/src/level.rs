//! Universe level arena and normalization.

use std::collections::HashMap;

use crate::{Name, NameError};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct LevelId(u32);

impl LevelId {
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum LevelNode {
    Zero,
    Succ(LevelId),
    Max(LevelId, LevelId),
    Param(Name),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct LevelHash(u64);

impl LevelHash {
    pub fn as_u64(self) -> u64 {
        self.0
    }
}

#[derive(Debug)]
pub struct LevelArena {
    nodes: Vec<LevelNode>,
    interned: HashMap<LevelNode, LevelId>,
    hash_cache: Vec<Option<LevelHash>>,
}

impl Default for LevelArena {
    fn default() -> Self {
        let mut arena = Self {
            nodes: Vec::new(),
            interned: HashMap::new(),
            hash_cache: Vec::new(),
        };
        let zero = arena.intern_normalized(LevelNode::Zero);
        debug_assert_eq!(zero.index(), 0);
        arena
    }
}

impl LevelArena {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn zero(&self) -> LevelId {
        LevelId(0)
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn node(&self, id: LevelId) -> &LevelNode {
        &self.nodes[id.index()]
    }

    pub fn param(&mut self, name: Name) -> LevelId {
        self.intern_normalized(LevelNode::Param(name))
    }

    pub fn parse_param(&mut self, name: impl AsRef<str>) -> Result<LevelId, NameError> {
        Ok(self.param(Name::parse(name)?))
    }

    pub fn succ(&mut self, inner: LevelId) -> LevelId {
        let inner = self.normalize(inner);
        self.intern_normalized(LevelNode::Succ(inner))
    }

    pub fn max(&mut self, lhs: LevelId, rhs: LevelId) -> LevelId {
        let lhs = self.normalize(lhs);
        let rhs = self.normalize(rhs);
        let mut terms = Vec::new();
        self.collect_max_terms(lhs, &mut terms);
        self.collect_max_terms(rhs, &mut terms);

        terms.sort_by_cached_key(|id| self.canonical_key(*id));
        terms.dedup();

        let mut terms = terms.into_iter();
        let Some(first) = terms.next() else {
            return self.zero();
        };

        terms.fold(first, |acc, next| {
            self.intern_normalized(LevelNode::Max(acc, next))
        })
    }

    pub fn normalize(&mut self, id: LevelId) -> LevelId {
        match self.node(id).clone() {
            LevelNode::Zero => self.zero(),
            LevelNode::Succ(inner) => self.succ(inner),
            LevelNode::Max(lhs, rhs) => self.max(lhs, rhs),
            LevelNode::Param(name) => self.param(name),
        }
    }

    pub fn stable_hash(&mut self, id: LevelId) -> LevelHash {
        let id = self.normalize(id);
        self.stable_hash_normalized(id)
    }

    fn intern_normalized(&mut self, node: LevelNode) -> LevelId {
        if let Some(id) = self.interned.get(&node) {
            return *id;
        }

        let index = u32::try_from(self.nodes.len()).expect("level arena exceeded u32 ids");
        let id = LevelId(index);
        self.nodes.push(node.clone());
        self.interned.insert(node, id);
        self.hash_cache.push(None);
        id
    }

    fn collect_max_terms(&self, id: LevelId, terms: &mut Vec<LevelId>) {
        match self.node(id) {
            LevelNode::Zero => {}
            LevelNode::Max(lhs, rhs) => {
                self.collect_max_terms(*lhs, terms);
                self.collect_max_terms(*rhs, terms);
            }
            LevelNode::Succ(_) | LevelNode::Param(_) => terms.push(id),
        }
    }

    fn canonical_key(&self, id: LevelId) -> String {
        match self.node(id) {
            LevelNode::Zero => "0".to_owned(),
            LevelNode::Succ(inner) => format!("s({})", self.canonical_key(*inner)),
            LevelNode::Max(lhs, rhs) => {
                format!(
                    "m({},{})",
                    self.canonical_key(*lhs),
                    self.canonical_key(*rhs)
                )
            }
            LevelNode::Param(name) => format!("p:{}:{}", name.as_str().len(), name.as_str()),
        }
    }

    fn stable_hash_normalized(&mut self, id: LevelId) -> LevelHash {
        if let Some(hash) = self.hash_cache[id.index()] {
            return hash;
        }

        let node = self.node(id).clone();
        let mut hasher = StableLevelHasher::new();
        match node {
            LevelNode::Zero => hasher.write_u8(0),
            LevelNode::Succ(inner) => {
                hasher.write_u8(1);
                hasher.write_u64(self.stable_hash_normalized(inner).as_u64());
            }
            LevelNode::Max(lhs, rhs) => {
                hasher.write_u8(2);
                hasher.write_u64(self.stable_hash_normalized(lhs).as_u64());
                hasher.write_u64(self.stable_hash_normalized(rhs).as_u64());
            }
            LevelNode::Param(name) => {
                hasher.write_u8(3);
                hasher.write_u64(name.as_str().len() as u64);
                hasher.write_bytes(name.as_str().as_bytes());
            }
        }

        let hash = LevelHash(hasher.finish());
        self.hash_cache[id.index()] = Some(hash);
        hash
    }
}

struct StableLevelHasher(u64);

impl StableLevelHasher {
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
    use super::{LevelArena, LevelNode};

    fn param(arena: &mut LevelArena, name: &str) -> super::LevelId {
        arena.parse_param(name).expect("valid level param name")
    }

    #[test]
    fn zero_is_canonical() {
        let arena = LevelArena::new();

        assert_eq!(arena.zero().index(), 0);
        assert_eq!(arena.len(), 1);
        assert_eq!(arena.node(arena.zero()), &LevelNode::Zero);
    }

    #[test]
    fn max_drops_zero_and_deduplicates_terms() {
        let mut arena = LevelArena::new();
        let u = param(&mut arena, "u");
        let max_with_zero = arena.max(u, arena.zero());
        let duplicate = arena.max(u, u);

        assert_eq!(max_with_zero, u);
        assert_eq!(duplicate, u);
    }

    #[test]
    fn max_normalization_is_order_independent() {
        let mut arena = LevelArena::new();
        let u = param(&mut arena, "u");
        let v = param(&mut arena, "v");

        let lhs = arena.max(u, v);
        let rhs = arena.max(v, u);

        assert_eq!(lhs, rhs);
    }

    #[test]
    fn nested_max_flattens_and_sorts_terms() {
        let mut arena = LevelArena::new();
        let u = param(&mut arena, "u");
        let v = param(&mut arena, "v");
        let w = param(&mut arena, "w");
        let uv = arena.max(u, v);
        let nested = arena.max(w, uv);
        let wv = arena.max(w, v);
        let reordered = arena.max(wv, u);

        assert_eq!(nested, reordered);
        match arena.node(nested) {
            LevelNode::Max(_, _) => {}
            node => panic!("expected normalized max tree, got {node:?}"),
        }
    }

    #[test]
    fn succ_normalizes_its_inner_level() {
        let mut arena = LevelArena::new();
        let u = param(&mut arena, "u");
        let with_zero = arena.max(arena.zero(), u);

        assert_eq!(with_zero, u);
        assert_eq!(arena.succ(with_zero), arena.succ(u));
    }

    #[test]
    fn stable_hash_uses_normal_form() {
        let mut arena = LevelArena::new();
        let u = param(&mut arena, "u");
        let v = param(&mut arena, "v");
        let first = arena.max(u, v);
        let second = arena.max(v, u);

        let first_hash = arena.stable_hash(first);
        let second_hash = arena.stable_hash(second);
        let succ_first = arena.succ(first);
        let succ_first_hash = arena.stable_hash(succ_first);

        assert_eq!(first, second);
        assert_eq!(first_hash, second_hash);
        assert_ne!(first_hash, succ_first_hash);
    }
}
