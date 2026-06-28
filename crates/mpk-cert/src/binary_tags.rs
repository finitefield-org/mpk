//! Certificate v0 binary tag assignments.
//!
//! These one-byte tags are part of the canonical `.mpcert` wire format.
//! Changing any value requires a new certificate format version.
//!
//! Level tags:
//!
//! | Tag | Node |
//! |---:|---|
//! | `0x00` | `Zero` |
//! | `0x01` | `Succ` |
//! | `0x02` | `Max` |
//! | `0x03` | `Param` |
//!
//! Term tags:
//!
//! | Tag | Node |
//! |---:|---|
//! | `0x00` | `Sort` |
//! | `0x01` | `Var` |
//! | `0x02` | `Const` |
//! | `0x03` | `App` |
//! | `0x04` | `Lam` |
//! | `0x05` | `Pi` |
//! | `0x06` | `Let` |
//!
//! Proof-node tags:
//!
//! | Tag | Node |
//! |---:|---|
//! | `0x00` | `Exact` |
//! | `0x01` | `Apply` |
//! | `0x02` | `Intro` |
//! | `0x03` | `LetProof` |
//! | `0x04` | `Refl` |
//! | `0x05` | `Rewrite` |
//! | `0x06` | `EqRec` |
//! | `0x07` | `Constructor` |
//! | `0x08` | `Recursor` |
//! | `0x09` | `Conv` |
//! | `0x0a` | `Theory` |
//!
//! Declaration tags:
//!
//! | Tag | Declaration |
//! |---:|---|
//! | `0x00` | `Axiom` |
//! | `0x01` | `Def` |
//! | `0x02` | `Theorem` |
//! | `0x03` | `Inductive` |
//! | `0x04` | `Constructor` |
//! | `0x05` | `Recursor` |
//! | `0x06` | `TheoryPrimitive` |

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[repr(u8)]
pub enum LevelTag {
    Zero = 0x00,
    Succ = 0x01,
    Max = 0x02,
    Param = 0x03,
}

impl LevelTag {
    pub const ALL: [Self; 4] = [Self::Zero, Self::Succ, Self::Max, Self::Param];

    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    pub const fn canonical_name(self) -> &'static str {
        match self {
            Self::Zero => "zero",
            Self::Succ => "succ",
            Self::Max => "max",
            Self::Param => "param",
        }
    }

    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x00 => Some(Self::Zero),
            0x01 => Some(Self::Succ),
            0x02 => Some(Self::Max),
            0x03 => Some(Self::Param),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[repr(u8)]
pub enum TermTag {
    Sort = 0x00,
    Var = 0x01,
    Const = 0x02,
    App = 0x03,
    Lam = 0x04,
    Pi = 0x05,
    Let = 0x06,
}

impl TermTag {
    pub const ALL: [Self; 7] = [
        Self::Sort,
        Self::Var,
        Self::Const,
        Self::App,
        Self::Lam,
        Self::Pi,
        Self::Let,
    ];

    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    pub const fn canonical_name(self) -> &'static str {
        match self {
            Self::Sort => "sort",
            Self::Var => "var",
            Self::Const => "const",
            Self::App => "app",
            Self::Lam => "lam",
            Self::Pi => "pi",
            Self::Let => "let",
        }
    }

    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x00 => Some(Self::Sort),
            0x01 => Some(Self::Var),
            0x02 => Some(Self::Const),
            0x03 => Some(Self::App),
            0x04 => Some(Self::Lam),
            0x05 => Some(Self::Pi),
            0x06 => Some(Self::Let),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[repr(u8)]
pub enum ProofNodeTag {
    Exact = 0x00,
    Apply = 0x01,
    Intro = 0x02,
    LetProof = 0x03,
    Refl = 0x04,
    Rewrite = 0x05,
    EqRec = 0x06,
    Constructor = 0x07,
    Recursor = 0x08,
    Conv = 0x09,
    Theory = 0x0a,
}

impl ProofNodeTag {
    pub const ALL: [Self; 11] = [
        Self::Exact,
        Self::Apply,
        Self::Intro,
        Self::LetProof,
        Self::Refl,
        Self::Rewrite,
        Self::EqRec,
        Self::Constructor,
        Self::Recursor,
        Self::Conv,
        Self::Theory,
    ];

    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    pub const fn canonical_name(self) -> &'static str {
        match self {
            Self::Exact => "exact",
            Self::Apply => "apply",
            Self::Intro => "intro",
            Self::LetProof => "let_proof",
            Self::Refl => "refl",
            Self::Rewrite => "rewrite",
            Self::EqRec => "eq_rec",
            Self::Constructor => "constructor",
            Self::Recursor => "recursor",
            Self::Conv => "conv",
            Self::Theory => "theory",
        }
    }

    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x00 => Some(Self::Exact),
            0x01 => Some(Self::Apply),
            0x02 => Some(Self::Intro),
            0x03 => Some(Self::LetProof),
            0x04 => Some(Self::Refl),
            0x05 => Some(Self::Rewrite),
            0x06 => Some(Self::EqRec),
            0x07 => Some(Self::Constructor),
            0x08 => Some(Self::Recursor),
            0x09 => Some(Self::Conv),
            0x0a => Some(Self::Theory),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[repr(u8)]
pub enum DeclarationTag {
    Axiom = 0x00,
    Def = 0x01,
    Theorem = 0x02,
    Inductive = 0x03,
    Constructor = 0x04,
    Recursor = 0x05,
    TheoryPrimitive = 0x06,
}

impl DeclarationTag {
    pub const ALL: [Self; 7] = [
        Self::Axiom,
        Self::Def,
        Self::Theorem,
        Self::Inductive,
        Self::Constructor,
        Self::Recursor,
        Self::TheoryPrimitive,
    ];

    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    pub const fn canonical_name(self) -> &'static str {
        match self {
            Self::Axiom => "axiom",
            Self::Def => "def",
            Self::Theorem => "theorem",
            Self::Inductive => "inductive",
            Self::Constructor => "constructor",
            Self::Recursor => "recursor",
            Self::TheoryPrimitive => "theory_primitive",
        }
    }

    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x00 => Some(Self::Axiom),
            0x01 => Some(Self::Def),
            0x02 => Some(Self::Theorem),
            0x03 => Some(Self::Inductive),
            0x04 => Some(Self::Constructor),
            0x05 => Some(Self::Recursor),
            0x06 => Some(Self::TheoryPrimitive),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{DeclarationTag, LevelTag, ProofNodeTag, TermTag};

    fn assert_unique(codes: &[u8]) {
        let mut sorted = codes.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), codes.len());
    }

    #[test]
    fn level_tags_match_certificate_v0() {
        let table = LevelTag::ALL.map(|tag| (tag.as_u8(), tag.canonical_name()));

        assert_eq!(
            table,
            [
                (0x00, "zero"),
                (0x01, "succ"),
                (0x02, "max"),
                (0x03, "param"),
            ]
        );
        assert_unique(&LevelTag::ALL.map(LevelTag::as_u8));
        for tag in LevelTag::ALL {
            assert_eq!(LevelTag::from_u8(tag.as_u8()), Some(tag));
        }
        assert_eq!(LevelTag::from_u8(0x04), None);
    }

    #[test]
    fn term_tags_match_certificate_v0() {
        let table = TermTag::ALL.map(|tag| (tag.as_u8(), tag.canonical_name()));

        assert_eq!(
            table,
            [
                (0x00, "sort"),
                (0x01, "var"),
                (0x02, "const"),
                (0x03, "app"),
                (0x04, "lam"),
                (0x05, "pi"),
                (0x06, "let"),
            ]
        );
        assert_unique(&TermTag::ALL.map(TermTag::as_u8));
        for tag in TermTag::ALL {
            assert_eq!(TermTag::from_u8(tag.as_u8()), Some(tag));
        }
        assert_eq!(TermTag::from_u8(0x07), None);
    }

    #[test]
    fn proof_node_tags_match_certificate_v0() {
        let table = ProofNodeTag::ALL.map(|tag| (tag.as_u8(), tag.canonical_name()));

        assert_eq!(
            table,
            [
                (0x00, "exact"),
                (0x01, "apply"),
                (0x02, "intro"),
                (0x03, "let_proof"),
                (0x04, "refl"),
                (0x05, "rewrite"),
                (0x06, "eq_rec"),
                (0x07, "constructor"),
                (0x08, "recursor"),
                (0x09, "conv"),
                (0x0a, "theory"),
            ]
        );
        assert_unique(&ProofNodeTag::ALL.map(ProofNodeTag::as_u8));
        for tag in ProofNodeTag::ALL {
            assert_eq!(ProofNodeTag::from_u8(tag.as_u8()), Some(tag));
        }
        assert_eq!(ProofNodeTag::from_u8(0x0b), None);
    }

    #[test]
    fn declaration_tags_match_core_v0() {
        let table = DeclarationTag::ALL.map(|tag| (tag.as_u8(), tag.canonical_name()));

        assert_eq!(
            table,
            [
                (0x00, "axiom"),
                (0x01, "def"),
                (0x02, "theorem"),
                (0x03, "inductive"),
                (0x04, "constructor"),
                (0x05, "recursor"),
                (0x06, "theory_primitive"),
            ]
        );
        assert_unique(&DeclarationTag::ALL.map(DeclarationTag::as_u8));
        for tag in DeclarationTag::ALL {
            assert_eq!(DeclarationTag::from_u8(tag.as_u8()), Some(tag));
        }
        assert_eq!(DeclarationTag::from_u8(0x07), None);
    }
}
