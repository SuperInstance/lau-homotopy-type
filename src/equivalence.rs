//! Equivalences: contractible fibers, half-adjoint equivalences, univalence.
//!
//! An equivalence `f : A ≃ B` is a function with a quasi-inverse,
//! plus coherence conditions. The univalence axiom states that
//! `(A = B) ≃ (A ≃ B)` for types in a universe.

use serde::{Deserialize, Serialize};
use crate::path::TypeExpr;
use crate::homotopy::Function;

/// A quasi-inverse: a function with left and right inverses.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuasiInverse {
    pub f: Function,
    pub g: Function,
    /// Homotopy g∘f ~ id
    pub left_inv: String,
    /// Homotopy f∘g ~ id
    pub right_inv: String,
}

/// An equivalence between types A and B.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Equivalence {
    pub name: String,
    pub source_type: TypeExpr,
    pub target_type: TypeExpr,
    pub forward: String,
    pub backward: String,
    pub left_triangle: String, // homotopy g∘f ~ id
    pub right_triangle: String, // homotopy f∘g ~ id
}

impl Equivalence {
    /// Create an equivalence from forward/backward maps.
    pub fn new(
        name: &str,
        src: TypeExpr,
        tgt: TypeExpr,
        fwd: &str,
        bwd: &str,
    ) -> Self {
        Equivalence {
            name: name.to_string(),
            source_type: src,
            target_type: tgt,
            forward: fwd.to_string(),
            backward: bwd.to_string(),
            left_triangle: format!("{}_left_tri", name),
            right_triangle: format!("{}_right_tri", name),
        }
    }

    /// The identity equivalence: A ≃ A.
    pub fn identity(ty: TypeExpr) -> Self {
        Equivalence::new("id_equiv", ty.clone(), ty, "id", "id")
    }

    /// Compose two equivalences: if A ≃ B and B ≃ C, then A ≃ C.
    pub fn compose(&self, other: &Equivalence) -> Result<Equivalence, EquivError> {
        if self.target_type != other.source_type {
            return Err(EquivError::TypeMismatch);
        }
        Ok(Equivalence::new(
            &format!("{}_∘{}", other.name, self.name),
            self.source_type.clone(),
            other.target_type.clone(),
            &format!("{}∘{}", other.forward, self.forward),
            &format!("{}∘{}", self.backward, other.backward),
        ))
    }

    /// Inverse equivalence: if A ≃ B, then B ≃ A.
    pub fn inverse(&self) -> Equivalence {
        Equivalence::new(
            &format!("{}_inv", self.name),
            self.target_type.clone(),
            self.source_type.clone(),
            &self.backward,
            &self.forward,
        )
    }

    /// Is this the identity equivalence?
    pub fn is_identity(&self) -> bool {
        self.forward == "id" && self.backward == "id"
    }
}

/// Errors in equivalence operations.
#[derive(Debug, Clone, PartialEq)]
pub enum EquivError {
    TypeMismatch,
    NotContractible,
}

/// A fiber of a function f : A → B over a point b : B is { a : A | f(a) = b }.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Fiber {
    pub function_name: String,
    pub base_point: String,
    pub elements: Vec<String>,
}

impl Fiber {
    pub fn new(func: &str, base: &str) -> Self {
        Fiber {
            function_name: func.to_string(),
            base_point: base.to_string(),
            elements: vec![],
        }
    }

    pub fn add_element(&mut self, elem: &str) {
        self.elements.push(elem.to_string());
    }

    /// Is this fiber contractible (has exactly one element up to homotopy)?
    pub fn is_contractible(&self) -> bool {
        self.elements.len() == 1
    }
}

/// Check if a function is an equivalence by checking contractible fibers.
pub fn is_equivalence_by_fibers(fibers: &[Fiber]) -> bool {
    fibers.iter().all(|f| f.is_contractible())
}

/// Half-adjoint equivalence: an equivalence with an extra coherence.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HalfAdjointEquiv {
    pub equiv: Equivalence,
    /// The adjustment homotopy.
    pub adjustment: String,
}

impl HalfAdjointEquiv {
    /// Promote a quasi-inverse to a half-adjoint equivalence.
    pub fn from_quasi_inverse(qi: &QuasiInverse) -> HalfAdjointEquiv {
        HalfAdjointEquiv {
            equiv: Equivalence::new(
                &format!("hae_{}", qi.f.name),
                qi.f.domain.clone(),
                qi.f.codomain.clone(),
                &qi.f.name,
                &qi.g.name,
            ),
            adjustment: format!("adj_{}", qi.f.name),
        }
    }
}

/// Univalence: the map (A = B) → (A ≃ B) is an equivalence.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Univalence {
    pub universe_level: usize,
}

impl Univalence {
    pub fn new(level: usize) -> Self {
        Univalence { universe_level: level }
    }

    /// The univalence map: idtoeqv : (A = B) → (A ≃ B).
    pub fn id_to_equiv(&self, a: &TypeExpr, b: &TypeExpr) -> Equivalence {
        Equivalence::new(
            "idtoeqv",
            a.clone(),
            b.clone(),
            "transport",
            "transport_inv",
        )
    }

    /// Apply univalence: from an equivalence, get an equality.
    pub fn ua(&self, equiv: &Equivalence) -> UnivalenceWitness {
        UnivalenceWitness {
            source: equiv.source_type.clone(),
            target: equiv.target_type.clone(),
            equivalence_name: equiv.name.clone(),
            universe: self.universe_level,
        }
    }
}

/// Witness of univalence: an equality between types arising from an equivalence.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnivalenceWitness {
    pub source: TypeExpr,
    pub target: TypeExpr,
    pub equivalence_name: String,
    pub universe: usize,
}

/// Function extensionality: if two functions are pointwise equal, they are equal.
pub fn funext(
    f: &Function,
    g: &Function,
    pointwise_paths: &[String],
) -> Result<FunExtWitness, EquivError> {
    if f.domain != g.domain || f.codomain != g.codomain {
        return Err(EquivError::TypeMismatch);
    }
    Ok(FunExtWitness {
        f: f.name.clone(),
        g: g.name.clone(),
        num_points: pointwise_paths.len(),
    })
}

/// Witness of function extensionality.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunExtWitness {
    pub f: String,
    pub g: String,
    pub num_points: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_equivalence() {
        let e = Equivalence::identity(TypeExpr::Bool);
        assert!(e.is_identity());
        assert_eq!(e.source_type, TypeExpr::Bool);
        assert_eq!(e.target_type, TypeExpr::Bool);
    }

    #[test]
    fn test_compose_equivalences() {
        let e1 = Equivalence::new("e1", TypeExpr::Bool, TypeExpr::Int, "f", "g");
        let e2 = Equivalence::new("e2", TypeExpr::Int, TypeExpr::Unit, "h", "k");
        let composed = e1.compose(&e2).unwrap();
        assert_eq!(composed.source_type, TypeExpr::Bool);
        assert_eq!(composed.target_type, TypeExpr::Unit);
    }

    #[test]
    fn test_compose_equivalences_error() {
        let e1 = Equivalence::new("e1", TypeExpr::Bool, TypeExpr::Int, "f", "g");
        let e2 = Equivalence::new("e2", TypeExpr::Bool, TypeExpr::Unit, "h", "k");
        assert!(e1.compose(&e2).is_err());
    }

    #[test]
    fn test_inverse_equivalence() {
        let e = Equivalence::new("e", TypeExpr::Bool, TypeExpr::Int, "f", "g");
        let inv = e.inverse();
        assert_eq!(inv.source_type, TypeExpr::Int);
        assert_eq!(inv.target_type, TypeExpr::Bool);
        assert_eq!(inv.forward, "g");
        assert_eq!(inv.backward, "f");
    }

    #[test]
    fn test_fiber_contractible() {
        let mut fib = Fiber::new("f", "b");
        fib.add_element("a");
        assert!(fib.is_contractible());
        fib.add_element("a'");
        assert!(!fib.is_contractible());
    }

    #[test]
    fn test_is_equivalence_by_fibers() {
        let mut f1 = Fiber::new("f", "b1");
        f1.add_element("a1");
        let mut f2 = Fiber::new("f", "b2");
        f2.add_element("a2");
        assert!(is_equivalence_by_fibers(&[f1, f2]));

        let mut f3 = Fiber::new("f", "b3");
        f3.add_element("a3");
        f3.add_element("a3'");
        assert!(!is_equivalence_by_fibers(&[f3]));
    }

    #[test]
    fn test_half_adjoint_equiv() {
        let f = Function::new("f", TypeExpr::Bool, TypeExpr::Bool);
        let g = Function::new("g", TypeExpr::Bool, TypeExpr::Bool);
        let qi = QuasiInverse { f, g, left_inv: "H1".to_string(), right_inv: "H2".to_string() };
        let hae = HalfAdjointEquiv::from_quasi_inverse(&qi);
        assert!(hae.equiv.name.contains("hae"));
    }

    #[test]
    fn test_univalence() {
        let ua = Univalence::new(0);
        let e = ua.id_to_equiv(&TypeExpr::Bool, &TypeExpr::Int);
        assert_eq!(e.source_type, TypeExpr::Bool);
        assert_eq!(e.target_type, TypeExpr::Int);

        let witness = ua.ua(&e);
        assert_eq!(witness.universe, 0);
        assert_eq!(witness.source, TypeExpr::Bool);
    }

    #[test]
    fn test_funext() {
        let f = Function::new("f", TypeExpr::Unit, TypeExpr::Bool);
        let g = Function::new("g", TypeExpr::Unit, TypeExpr::Bool);
        let witness = funext(&f, &g, &["p1".to_string(), "p2".to_string()]).unwrap();
        assert_eq!(witness.num_points, 2);
    }

    #[test]
    fn test_funext_error() {
        let f = Function::new("f", TypeExpr::Unit, TypeExpr::Bool);
        let g = Function::new("g", TypeExpr::Bool, TypeExpr::Unit);
        assert!(funext(&f, &g, &[]).is_err());
    }
}
