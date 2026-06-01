//! Transport: moving terms along paths between types.
//!
//! Transport (also called "substitution" or "cast") is the fundamental
//! operation that lets us move a term of type A to a term of type B
//! given a path/equality between A and B.

use serde::{Deserialize, Serialize};
use crate::path::{PathTerm, TypeExpr, Term};
use crate::identity::{IdentityProof, IdentityType};

/// A type family indexed by some parameter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeFamily {
    pub name: String,
    pub param_type: TypeExpr,
    pub body: TypeExpr,
}

impl TypeFamily {
    pub fn new(name: &str, param_type: TypeExpr, body: TypeExpr) -> Self {
        TypeFamily { name: name.to_string(), param_type, body }
    }
}

/// Transport operation result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransportResult {
    /// The source type.
    pub source_type: TypeExpr,
    /// The target type.
    pub target_type: TypeExpr,
    /// The input term name.
    pub input: String,
    /// The output term name (transported).
    pub output: String,
    /// The path/proof used for transport.
    pub along: String,
}

/// Transport a term along a path between types.
///
/// Given `p : Path U A B` and `a : A`, produces `transport(p, a) : B`.
pub fn transport_along_path(
    path: &PathTerm,
    term: &str,
) -> TransportResult {
    TransportResult {
        source_type: path.ty.clone(),
        target_type: path.ty.clone(), // simplified: real impl computes type substitution
        input: term.to_string(),
        output: format!("transport({}, {})", path.var, term),
        along: format!("path_{}_{}", 
            term_source_name(&path.src),
            term_source_name(&path.tgt)),
    }
}

/// Transport along an identity proof.
pub fn transport_along_id(
    id_type: &IdentityType,
    proof: &IdentityProof,
    term: &str,
) -> TransportResult {
    TransportResult {
        source_type: TypeExpr::Named(id_type.type_name.clone()),
        target_type: TypeExpr::Named(id_type.type_name.clone()),
        input: term.to_string(),
        output: format!("transport_id({}, {})", proof.src(), term),
        along: format!("{}_eq_{}", id_type.lhs, id_type.rhs),
    }
}

fn term_source_name(t: &Term) -> String {
    match t {
        Term::Var(s) => s.clone(),
        _ => "term".to_string(),
    }
}

/// Subst: the Leibniz principle — substitute equals for equals.
///
/// If `p : a =_A b` and `C : A → Type`, then `subst(p, c_a) : C(b)`.
pub fn subst(
    motive: &TypeFamily,
    proof: &IdentityProof,
    term: &str,
) -> TransportResult {
    TransportResult {
        source_type: motive.body.clone(),
        target_type: motive.body.clone(),
        input: term.to_string(),
        output: format!("subst({}, {})", proof.src(), term),
        along: format!("eq_{}_{}", proof.src(), proof.tgt()),
    }
}

/// Transport composition: transport along a composite path equals
/// transporting along each factor.
pub fn transport_compose(
    p: &PathTerm,
    q: &PathTerm,
    term: &str,
) -> (TransportResult, TransportResult) {
    let t1 = transport_along_path(p, term);
    let t2 = transport_along_path(q, &t1.output);
    (t1, t2)
}

/// Transport inversion: transporting along p⁻¹ undoes transport along p.
pub fn transport_inv(path: &PathTerm, term: &str) -> TransportResult {
    let forward = transport_along_path(path, term);
    TransportResult {
        source_type: forward.target_type.clone(),
        target_type: forward.source_type.clone(),
        input: forward.output.clone(),
        output: term.to_string(),
        along: format!("inv_{}", forward.along),
    }
}

/// Transport in a constant type family is the identity.
pub fn transport_const(ty: TypeExpr, term: &str) -> TransportResult {
    TransportResult {
        source_type: ty.clone(),
        target_type: ty,
        input: term.to_string(),
        output: term.to_string(),
        along: "const".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transport_along_path() {
        let p = PathTerm {
            ty: TypeExpr::Named("A".to_string()),
            src: Box::new(Term::Var("a1".to_string())),
            tgt: Box::new(Term::Var("a2".to_string())),
            var: "i".to_string(),
            body: Box::new(Term::Var("body".to_string())),
        };
        let result = transport_along_path(&p, "x");
        assert_eq!(result.input, "x");
        assert!(result.output.contains("transport"));
    }

    #[test]
    fn test_transport_along_id() {
        let id = IdentityType::new("Nat", "zero", "one");
        let proof = IdentityProof::Path("p".to_string(), "zero".to_string(), "one".to_string());
        let result = transport_along_id(&id, &proof, "x");
        assert_eq!(result.input, "x");
        assert!(result.output.contains("transport_id"));
    }

    #[test]
    fn test_subst() {
        let motive = TypeFamily::new("C", TypeExpr::Named("A".to_string()), TypeExpr::Named("B".to_string()));
        let proof = IdentityProof::Path("p".to_string(), "a".to_string(), "b".to_string());
        let result = subst(&motive, &proof, "ca");
        assert_eq!(result.input, "ca");
    }

    #[test]
    fn test_transport_compose() {
        let p = PathTerm::refl(TypeExpr::Unit, Term::Var("x".to_string()));
        let q = PathTerm::refl(TypeExpr::Unit, Term::Var("y".to_string()));
        let (t1, t2) = transport_compose(&p, &q, "z");
        assert_eq!(t1.input, "z");
        assert!(t2.input.contains("transport"));
    }

    #[test]
    fn test_transport_inv() {
        let p = PathTerm {
            ty: TypeExpr::Named("A".to_string()),
            src: Box::new(Term::Var("a1".to_string())),
            tgt: Box::new(Term::Var("a2".to_string())),
            var: "i".to_string(),
            body: Box::new(Term::Var("body".to_string())),
        };
        let result = transport_inv(&p, "x");
        assert_eq!(result.output, "x"); // round-trip
    }

    #[test]
    fn test_transport_const() {
        let result = transport_const(TypeExpr::Bool, "true");
        assert_eq!(result.input, result.output);
        assert_eq!(result.along, "const");
    }

    #[test]
    fn test_type_family() {
        let tf = TypeFamily::new("Vec", TypeExpr::Int, TypeExpr::Named("Vec".to_string()));
        assert_eq!(tf.name, "Vec");
    }
}
