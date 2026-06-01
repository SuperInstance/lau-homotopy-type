//! Interval type for cubical type theory.
//!
//! The interval `I` has two endpoints `i0` and `i1` and represents
//! a continuous line segment used to construct paths and higher cubes.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops;

/// The interval type with two endpoints.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Interval {
    /// Left endpoint (0)
    I0,
    /// Right endpoint (1)
    I1,
    /// A named/de Bruijn variable representing a generic interval point.
    Var(usize),
}

impl Interval {
    /// The left endpoint (0).
    pub fn i0() -> Self { Interval::I0 }

    /// The right endpoint (1).
    pub fn i1() -> Self { Interval::I1 }

    /// Fresh interval variable.
    pub fn var(n: usize) -> Self { Interval::Var(n) }

    /// Is this the left endpoint?
    pub fn is_i0(&self) -> bool { matches!(self, Interval::I0) }

    /// Is this the right endpoint?
    pub fn is_i1(&self) -> bool { matches!(self, Interval::I1) }

    /// Is this a variable?
    pub fn is_var(&self) -> bool { matches!(self, Interval::Var(_)) }

    /// Negate the interval: i0 ↔ i1, vars stay.
    pub fn neg(&self) -> Self {
        match self {
            Interval::I0 => Interval::I1,
            Interval::I1 => Interval::I0,
            Interval::Var(n) => Interval::Var(*n),
        }
    }

    /// Minimum (meet) of two interval points.
    pub fn meet(&self, other: &Self) -> Self {
        match (self, other) {
            (Interval::I0, _) | (_, Interval::I0) => Interval::I0,
            (Interval::I1, x) | (x, Interval::I1) => x.clone(),
            _ => Interval::I0, // conservative for vars
        }
    }

    /// Maximum (join) of two interval points.
    pub fn join(&self, other: &Self) -> Self {
        match (self, other) {
            (Interval::I1, _) | (_, Interval::I1) => Interval::I1,
            (Interval::I0, x) | (x, Interval::I0) => x.clone(),
            _ => Interval::I1, // conservative for vars
        }
    }
}

impl fmt::Display for Interval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Interval::I0 => write!(f, "i0"),
            Interval::I1 => write!(f, "i1"),
            Interval::Var(n) => write!(f, "i_{}", n),
        }
    }
}

impl ops::Not for Interval {
    type Output = Self;
    fn not(self) -> Self { self.neg() }
}

impl ops::BitAnd for Interval {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self { self.meet(&rhs) }
}

impl ops::BitOr for Interval {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self { self.join(&rhs) }
}

/// A face constraint: a mapping from interval variables to endpoints.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Face {
    /// Map from variable index to which endpoint it's constrained to.
    pub constraints: Vec<(usize, Interval)>,
}

impl Face {
    pub fn empty() -> Self { Face { constraints: vec![] } }

    pub fn constrain(var: usize, endpoint: Interval) -> Self {
        Face { constraints: vec![(var, endpoint)] }
    }

    pub fn is_empty(&self) -> bool { self.constraints.is_empty() }

    /// Check if this face is satisfied by a given substitution.
    pub fn satisfied_by(&self, sub: &[(usize, Interval)]) -> bool {
        self.constraints.iter().all(|(v, ep)| {
            sub.iter().any(|(sv, sep)| sv == v && sep == ep)
        })
    }
}

/// Face map: assigns interval endpoints.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FaceMap {
    pub dim: usize,
    pub face_at: Interval,
}

impl FaceMap {
    /// The i-th face map δ_i^0: set dimension i to i0.
    pub fn face_0(dim: usize) -> Self { FaceMap { dim, face_at: Interval::I0 } }

    /// The i-th face map δ_i^1: set dimension i to i1.
    pub fn face_1(dim: usize) -> Self { FaceMap { dim, face_at: Interval::I1 } }
}

/// Degeneracy map: projects away a dimension.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Degeneracy {
    pub dim: usize,
}

impl Degeneracy {
    pub fn new(dim: usize) -> Self { Degeneracy { dim } }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_endpoints() {
        assert!(Interval::i0().is_i0());
        assert!(Interval::i1().is_i1());
        assert!(!Interval::i0().is_i1());
        assert!(!Interval::i1().is_i0());
    }

    #[test]
    fn test_negation() {
        assert_eq!(Interval::i0().neg(), Interval::i1());
        assert_eq!(Interval::i1().neg(), Interval::i0());
        assert_eq!(!Interval::i0(), Interval::i1());
        assert_eq!(!Interval::i1(), Interval::i0());
    }

    #[test]
    fn test_meet_join() {
        assert_eq!(Interval::i0() & Interval::i1(), Interval::i0());
        assert_eq!(Interval::i0() | Interval::i1(), Interval::i1());
        assert_eq!(Interval::i0() & Interval::i0(), Interval::i0());
        assert_eq!(Interval::i1() | Interval::i1(), Interval::i1());
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", Interval::i0()), "i0");
        assert_eq!(format!("{}", Interval::i1()), "i1");
        assert_eq!(format!("{}", Interval::var(3)), "i_3");
    }

    #[test]
    fn test_face_satisfied() {
        let face = Face::constrain(0, Interval::I1);
        assert!(face.satisfied_by(&[(0, Interval::I1)]));
        assert!(!face.satisfied_by(&[(0, Interval::I0)]));
    }

    #[test]
    fn test_face_map() {
        let f0 = FaceMap::face_0(1);
        let f1 = FaceMap::face_1(1);
        assert_eq!(f0.face_at, Interval::I0);
        assert_eq!(f1.face_at, Interval::I1);
    }
}
