//! Closed selection of original auxiliary Go build inputs; no C/asm semantics.
use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum GoSourceAuxiliaryKind {
    C,
    Cxx,
    ObjectiveC,
    Header,
    Fortran,
    Assembly,
    Swig,
    SwigCxx,
    SystemObject,
}
impl GoSourceAuxiliaryKind {
    pub(super) fn filename(name: &[u8]) -> Option<Self> {
        let at = name.iter().rposition(|b| *b == b'.')?;
        Some(match &name[at + 1..] {
            b"c" => Self::C,
            b"cc" | b"cpp" | b"cxx" => Self::Cxx,
            b"m" => Self::ObjectiveC,
            b"h" | b"hh" | b"hpp" | b"hxx" => Self::Header,
            b"f" | b"F" | b"for" | b"f90" => Self::Fortran,
            b"s" | b"S" | b"sx" => Self::Assembly,
            b"swig" => Self::Swig,
            b"swigcxx" => Self::SwigCxx,
            b"syso" => Self::SystemObject,
            _ => return None,
        })
    }
}

pub(super) fn select(source: &GoTypeSourceBuffer, kind: GoSourceAuxiliaryKind) -> bool {
    let name = source.object().path.rsplit('/').next().unwrap();
    // Cgo is fixed disabled. Native Go sends preprocessed assembly to the
    // ignored-other list even when its target/header conditions match.
    if kind == GoSourceAuxiliaryKind::Assembly && (name.ends_with(".S") || name.ends_with(".sx")) {
        return false;
    }
    let header = if kind == GoSourceAuxiliaryKind::SystemObject {
        Some(&b""[..])
    } else {
        header::comment_prefix(source.bytes())
    };
    // A malformed modern constraint yields nil native file info. ImportDir
    // ignores that non-Go error and retains the exact file as ignored-other.
    source_build::auxiliary_match(source, header).unwrap_or(false)
}
