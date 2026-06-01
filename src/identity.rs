//! Identity types: propositional equality vs definitional equality.
//!
//! In HoTT, the identity type `Id_A(a, b)` (or `a =_A b`) is the type
//! of witnesses that `a` and `b` are equal. Unlike definitional/judgmental
//! equality (which is a metatheoretic relation), propositional equality
//! is a type that can be inhabited by terms.

use serde::{Deserialize, Serialize};
use crate::path::Term;

/// The nature of equality between two terms.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Equality {
    /// Definitional (judgmental) equality: the terms compute to the same normal form.
    Definitional,
    /// Propositional equality: there exists a path/proof connecting them.
    Propositional(IdentityProof),
}

/// A proof of propositional equality.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IdentityProof {
    /// Reflexivity: a = a.
    Refl(String),
    /// A path between terms.
    Path(String, String, String), // name, src, tgt
    /// Composed proof.
    Compose(Box<IdentityProof>, Box<IdentityProof>),
    /// Inverted proof.
    Inv(Box<IdentityProof>),
    /// By congruence.
    Cong(String, Box<IdentityProof>),
    /// By transport.
    Transport(String, Box<IdentityProof>),
}

impl IdentityProof {
    /// Source of the equality proof.
    pub fn src(&self) -> &str {
        match self {
            IdentityProof::Refl(a) => a,
            IdentityProof::Path(_, s, _) => s,
            IdentityProof::Compose(p, _) => p.src(),
            IdentityProof::Inv(p) => p.tgt(),
            IdentityProof::Cong(_, p) => p.src(),
            IdentityProof::Transport(_, p) => p.src(),
        }
    }

    /// Target of the equality proof.
    pub fn tgt(&self) -> &str {
        match self {
            IdentityProof::Refl(a) => a,
            IdentityProof::Path(_, _, t) => t,
            IdentityProof::Compose(_, q) => q.tgt(),
            IdentityProof::Inv(p) => p.src(),
            IdentityProof::Cong(_, p) => p.tgt(),
            IdentityProof::Transport(_, p) => p.tgt(),
        }
    }

    /// Is this proof reflexive?
    pub fn is_refl(&self) -> bool {
        matches!(self, IdentityProof::Refl(_))
    }
}

/// Identity type: `Id_A(a, b)`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IdentityType {
    pub type_name: String,
    pub lhs: String,
    pub rhs: String,
}

impl IdentityType {
    pub fn new(type_name: &str, lhs: &str, rhs: &str) -> Self {
        IdentityType {
            type_name: type_name.to_string(),
            lhs: lhs.to_string(),
            rhs: rhs.to_string(),
        }
    }

    /// Is this a reflexive identity type?
    pub fn is_refl(&self) -> bool {
        self.lhs == self.rhs
    }
}

/// An identity context: tracks known equalities in scope.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IdentityContext {
    /// Known propositional equalities.
    pub equalities: Vec<(IdentityType, IdentityProof)>,
}

impl IdentityContext {
    pub fn new() -> Self { IdentityContext::default() }

    /// Assert an equality with proof.
    pub fn assume_equal(&mut self, id_type: IdentityType, proof: IdentityProof) {
        self.equalities.push((id_type, proof));
    }

    /// Check if two terms are definitionally equal (same name).
    pub fn definitional_equal(&self, a: &str, b: &str) -> bool {
        a == b
    }

    /// Check if two terms are propositionally equal (find a proof).
    pub fn propositional_equal(&self, type_name: &str, a: &str, b: &str) -> Option<&IdentityProof> {
        self.equalities.iter()
            .find(|(id, _)| {
                id.type_name == type_name &&
                ((id.lhs == a && id.rhs == b) || (id.lhs == b && id.rhs == a))
            })
            .map(|(_, proof)| proof)
    }

    /// Compose two equality proofs: transitivity.
    pub fn compose_proofs(&self, p: IdentityProof, q: IdentityProof) -> Result<IdentityProof, IdentityError> {
        if p.tgt() != q.src() {
            return Err(IdentityError::EndpointMismatch);
        }
        Ok(IdentityProof::Compose(Box::new(p), Box::new(q)))
    }

    /// Invert an equality proof: symmetry.
    pub fn invert_proof(&self, p: IdentityProof) -> IdentityProof {
        IdentityProof::Inv(Box::new(p))
    }

    /// Check if a type is a proposition (at most one inhabitant).
    pub fn is_proposition(&self, type_name: &str) -> bool {
        // A type is a proposition if all identity proofs are reflexivity
        let relevant: Vec<_> = self.equalities.iter()
            .filter(|(id, _)| id.type_name == type_name)
            .collect();
        relevant.iter().all(|(_, proof)| proof.is_refl())
    }
}

/// Errors in identity operations.
#[derive(Debug, Clone, PartialEq)]
pub enum IdentityError {
    EndpointMismatch,
    NotEqual,
    TypeMismatch,
}

/// J-rule (path induction): the eliminator for identity types.
///
/// Given a motive C that produces a type for each `y` and `p : x = y`,
/// and a base case `c : C(x, refl)`, we get a function
/// `J(c, y, p) : C(y, p)` for any `y` and `p : x = y`.
pub fn j_rule(
    motive_name: &str,
    base_case: &str,
    target: &str,
    _proof: &IdentityProof,
) -> JResult {
    JResult {
        motive: motive_name.to_string(),
        base: base_case.to_string(),
        target: target.to_string(),
        result: format!("J({}, {}, {})", motive_name, base_case, target),
    }
}

/// Result of applying the J-rule.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JResult {
    pub motive: String,
    pub base: String,
    pub target: String,
    pub result: String,
}

/// Definitional equality checker for simple terms.
pub fn definitionally_equal(a: &Term, b: &Term) -> bool {
    a == b
}

/// Computes the normal form of an identity proof for definitional checking.
pub fn normalize_proof(proof: &IdentityProof) -> IdentityProof {
    match proof {
        IdentityProof::Refl(_) => proof.clone(),
        IdentityProof::Path(_, _, _) => proof.clone(),
        IdentityProof::Compose(p, q) => {
            let pn = normalize_proof(p);
            let qn = normalize_proof(q);
            if pn.is_refl() { qn }
            else if qn.is_refl() { pn }
            else { IdentityProof::Compose(Box::new(pn), Box::new(qn)) }
        },
        IdentityProof::Inv(p) => {
            let pn = normalize_proof(p);
            if pn.is_refl() { pn } // inv(refl) = refl
            else { IdentityProof::Inv(Box::new(pn)) }
        },
        _ => proof.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_type_refl() {
        let id = IdentityType::new("Nat", "zero", "zero");
        assert!(id.is_refl());
    }

    #[test]
    fn test_identity_type_not_refl() {
        let id = IdentityType::new("Nat", "zero", "succ zero");
        assert!(!id.is_refl());
    }

    #[test]
    fn test_refl_proof() {
        let proof = IdentityProof::Refl("x".to_string());
        assert!(proof.is_refl());
        assert_eq!(proof.src(), "x");
        assert_eq!(proof.tgt(), "x");
    }

    #[test]
    fn test_path_proof() {
        let proof = IdentityProof::Path("p".to_string(), "a".to_string(), "b".to_string());
        assert_eq!(proof.src(), "a");
        assert_eq!(proof.tgt(), "b");
    }

    #[test]
    fn test_compose_proofs() {
        let ctx = IdentityContext::new();
        let p = IdentityProof::Path("p".to_string(), "a".to_string(), "b".to_string());
        let q = IdentityProof::Path("q".to_string(), "b".to_string(), "c".to_string());
        let r = ctx.compose_proofs(p, q).unwrap();
        assert_eq!(r.src(), "a");
        assert_eq!(r.tgt(), "c");
    }

    #[test]
    fn test_compose_proofs_error() {
        let ctx = IdentityContext::new();
        let p = IdentityProof::Path("p".to_string(), "a".to_string(), "b".to_string());
        let q = IdentityProof::Path("q".to_string(), "c".to_string(), "d".to_string());
        assert_eq!(ctx.compose_proofs(p, q), Err(IdentityError::EndpointMismatch));
    }

    #[test]
    fn test_invert_proof() {
        let ctx = IdentityContext::new();
        let p = IdentityProof::Path("p".to_string(), "a".to_string(), "b".to_string());
        let inv = ctx.invert_proof(p);
        assert_eq!(inv.src(), "b");
        assert_eq!(inv.tgt(), "a");
    }

    #[test]
    fn test_propositional_equal() {
        let mut ctx = IdentityContext::new();
        ctx.assume_equal(
            IdentityType::new("Nat", "a", "b"),
            IdentityProof::Path("p".to_string(), "a".to_string(), "b".to_string()),
        );
        assert!(ctx.propositional_equal("Nat", "a", "b").is_some());
        assert!(ctx.propositional_equal("Nat", "b", "a").is_some());
        assert!(ctx.propositional_equal("Nat", "a", "c").is_none());
    }

    #[test]
    fn test_j_rule() {
        let result = j_rule("C", "base", "y", &IdentityProof::Refl("x".to_string()));
        assert_eq!(result.result, "J(C, base, y)");
    }

    #[test]
    fn test_normalize_refl() {
        let proof = IdentityProof::Refl("x".to_string());
        assert_eq!(normalize_proof(&proof), proof);
    }

    #[test]
    fn test_normalize_compose_refl() {
        let p = IdentityProof::Refl("x".to_string());
        let q = IdentityProof::Path("q".to_string(), "x".to_string(), "y".to_string());
        let composed = IdentityProof::Compose(Box::new(p), Box::new(q.clone()));
        let norm = normalize_proof(&composed);
        assert_eq!(norm, q);
    }

    #[test]
    fn test_normalize_inv_refl() {
        let p = IdentityProof::Refl("x".to_string());
        let inv = IdentityProof::Inv(Box::new(p));
        assert!(normalize_proof(&inv).is_refl());
    }
}
