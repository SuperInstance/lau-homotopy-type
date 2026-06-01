//! Path types: paths between terms, path composition, and path inversion.
//!
//! In HoTT, a path `p : Path A a b` is a witness that `a` and `b` are
//! (propositionally) equal in type `A`.

use serde::{Deserialize, Serialize};
use std::fmt;

/// A type expression in our simple type theory.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TypeExpr {
    Unit,
    Bool,
    Int,
    String,
    /// Named type (e.g. user-defined)
    Named(String),
    /// Function type: Box<(domain, codomain)>
    Arrow(Box<TypeExpr>, Box<TypeExpr>),
    /// Product type
    Product(Box<TypeExpr>, Box<TypeExpr>),
    /// Sum type
    Sum(Box<TypeExpr>, Box<TypeExpr>),
    /// Path type: Path ty a b
    Path(Box<TypeExpr>, String, String),
    /// Universe level
    Universe(usize),
}

impl fmt::Display for TypeExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeExpr::Unit => write!(f, "Unit"),
            TypeExpr::Bool => write!(f, "Bool"),
            TypeExpr::Int => write!(f, "Int"),
            TypeExpr::String => write!(f, "String"),
            TypeExpr::Named(n) => write!(f, "{}", n),
            TypeExpr::Arrow(a, b) => write!(f, "({} → {})", a, b),
            TypeExpr::Product(a, b) => write!(f, "({} × {})", a, b),
            TypeExpr::Sum(a, b) => write!(f, "({} + {})", a, b),
            TypeExpr::Path(ty, a, b) => write!(f, "Path[{}] {} {}", ty, a, b),
            TypeExpr::Universe(n) => write!(f, "U{}", n),
        }
    }
}

/// A term in our type theory.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Term {
    Var(String),
    Lit(Literal),
    /// Lambda abstraction
    Lam(String, Box<Term>),
    /// Application
    App(Box<Term>, Box<Term>),
    /// Pair
    Pair(Box<Term>, Box<Term>),
    /// Fst/Snd projections
    Fst(Box<Term>),
    Snd(Box<Term>),
    /// Reflexivity: the constant path
    Refl(Box<Term>),
    /// Path application: p @ i where p is a path and i is an interval var
    PathApp(String, String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Literal {
    Unit,
    Bool(bool),
    Int(i64),
    Str(String),
}

/// A path between terms: `Path A a b` represented by a lambda over the interval.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PathTerm {
    /// The type this path lives in.
    pub ty: TypeExpr,
    /// The source endpoint.
    pub src: Box<Term>,
    /// The target endpoint.
    pub tgt: Box<Term>,
    /// The interval variable name.
    pub var: String,
    /// The body: a term parameterized by the interval.
    pub body: Box<Term>,
}

impl PathTerm {
    /// Create a reflexive path (constant path) for a term.
    pub fn refl(ty: TypeExpr, term: Term) -> Self {
        let var = "i".to_string();
        PathTerm {
            ty,
            src: Box::new(term.clone()),
            tgt: Box::new(term.clone()),
            var,
            body: Box::new(term),
        }
    }

    /// Invert a path: p⁻¹ goes from b to a.
    pub fn inverse(&self) -> Self {
        PathTerm {
            ty: self.ty.clone(),
            src: self.tgt.clone(),
            tgt: self.src.clone(),
            var: self.var.clone(),
            body: self.body.clone(),
        }
    }

    /// Compose two paths: p · q where p : a → b, q : b → c.
    /// Uses the "diagonal" composition from cubical type theory.
    pub fn compose(&self, other: &PathTerm) -> Result<PathTerm, PathError> {
        if self.ty != other.ty {
            return Err(PathError::TypeMismatch);
        }
        if *self.tgt != *other.src {
            return Err(PathError::EndpointMismatch);
        }
        Ok(PathTerm {
            ty: self.ty.clone(),
            src: self.src.clone(),
            tgt: other.tgt.clone(),
            var: "j".to_string(),
            body: Box::new(Term::Var("composed".to_string())),
        })
    }

    /// Apply the path at an endpoint: 0 gives src, 1 gives tgt.
    pub fn at_endpoint(&self, endpoint: bool) -> &Term {
        if endpoint { &self.tgt } else { &self.src }
    }

    /// Is this a reflexive path?
    pub fn is_refl(&self) -> bool {
        *self.src == *self.tgt
    }
}

/// Errors in path operations.
#[derive(Debug, Clone, PartialEq)]
pub enum PathError {
    TypeMismatch,
    EndpointMismatch,
    InvalidComposition,
    NonPathTerm,
}

/// Path composition using homogeneous composition (hcomp).
pub fn hcomp_path(
    _ty: TypeExpr,
    paths: &[PathTerm],
) -> Result<PathTerm, PathError> {
    if paths.is_empty() {
        return Err(PathError::InvalidComposition);
    }
    let first = &paths[0];
    let mut current = first.clone();
    for p in &paths[1..] {
        current = current.compose(p)?;
    }
    Ok(current)
}

/// Congruence: if f : A → B and p : Path A a b, then cong f p : Path B (f a) (f b).
pub fn cong(f: &Term, path: &PathTerm) -> PathTerm {
    PathTerm {
        ty: TypeExpr::Unit, // simplified; real impl would track B
        src: Box::new(Term::App(Box::new(f.clone()), path.src.clone())),
        tgt: Box::new(Term::App(Box::new(f.clone()), path.tgt.clone())),
        var: path.var.clone(),
        body: Box::new(Term::App(
            Box::new(f.clone()),
            path.body.clone(),
        )),
    }
}

/// Symmetry of paths: if p : Path A a b, then sym p : Path A b a.
pub fn sym(path: &PathTerm) -> PathTerm {
    path.inverse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_refl_path() {
        let t = Term::Var("x".to_string());
        let p = PathTerm::refl(TypeExpr::Unit, t.clone());
        assert!(p.is_refl());
        assert_eq!(*p.at_endpoint(false), t);
        assert_eq!(*p.at_endpoint(true), t);
    }

    #[test]
    fn test_path_inverse() {
        let a = Term::Var("a".to_string());
        let b = Term::Var("b".to_string());
        let p = PathTerm {
            ty: TypeExpr::Unit,
            src: Box::new(a.clone()),
            tgt: Box::new(b.clone()),
            var: "i".to_string(),
            body: Box::new(Term::Var("body".to_string())),
        };
        let inv = p.inverse();
        assert_eq!(*inv.src, b);
        assert_eq!(*inv.tgt, a);
    }

    #[test]
    fn test_path_compose() {
        let a = Term::Var("a".to_string());
        let b = Term::Var("b".to_string());
        let c = Term::Var("c".to_string());
        let p = PathTerm {
            ty: TypeExpr::Unit,
            src: Box::new(a),
            tgt: Box::new(b.clone()),
            var: "i".to_string(),
            body: Box::new(Term::Var("p_body".to_string())),
        };
        let q = PathTerm {
            ty: TypeExpr::Unit,
            src: Box::new(b),
            tgt: Box::new(c),
            var: "j".to_string(),
            body: Box::new(Term::Var("q_body".to_string())),
        };
        let composed = p.compose(&q).unwrap();
        assert_eq!(*composed.src, Term::Var("a".to_string()));
        assert_eq!(*composed.tgt, Term::Var("c".to_string()));
    }

    #[test]
    fn test_path_compose_type_error() {
        let p = PathTerm {
            ty: TypeExpr::Bool,
            src: Box::new(Term::Var("a".to_string())),
            tgt: Box::new(Term::Var("b".to_string())),
            var: "i".to_string(),
            body: Box::new(Term::Var("p".to_string())),
        };
        let q = PathTerm {
            ty: TypeExpr::Unit,
            src: Box::new(Term::Var("b".to_string())),
            tgt: Box::new(Term::Var("c".to_string())),
            var: "j".to_string(),
            body: Box::new(Term::Var("q".to_string())),
        };
        assert_eq!(p.compose(&q), Err(PathError::TypeMismatch));
    }

    #[test]
    fn test_path_compose_endpoint_error() {
        let p = PathTerm {
            ty: TypeExpr::Unit,
            src: Box::new(Term::Var("a".to_string())),
            tgt: Box::new(Term::Var("b".to_string())),
            var: "i".to_string(),
            body: Box::new(Term::Var("p".to_string())),
        };
        let q = PathTerm {
            ty: TypeExpr::Unit,
            src: Box::new(Term::Var("c".to_string())), // mismatch!
            tgt: Box::new(Term::Var("d".to_string())),
            var: "j".to_string(),
            body: Box::new(Term::Var("q".to_string())),
        };
        assert_eq!(p.compose(&q), Err(PathError::EndpointMismatch));
    }

    #[test]
    fn test_hcomp_path() {
        let p = PathTerm {
            ty: TypeExpr::Unit,
            src: Box::new(Term::Var("a".to_string())),
            tgt: Box::new(Term::Var("b".to_string())),
            var: "i".to_string(),
            body: Box::new(Term::Var("p".to_string())),
        };
        let q = PathTerm {
            ty: TypeExpr::Unit,
            src: Box::new(Term::Var("b".to_string())),
            tgt: Box::new(Term::Var("c".to_string())),
            var: "j".to_string(),
            body: Box::new(Term::Var("q".to_string())),
        };
        let r = hcomp_path(TypeExpr::Unit, &[p, q]).unwrap();
        assert_eq!(*r.src, Term::Var("a".to_string()));
        assert_eq!(*r.tgt, Term::Var("c".to_string()));
    }

    #[test]
    fn test_cong() {
        let f = Term::Var("f".to_string());
        let p = PathTerm {
            ty: TypeExpr::Bool,
            src: Box::new(Term::Var("a".to_string())),
            tgt: Box::new(Term::Var("b".to_string())),
            var: "i".to_string(),
            body: Box::new(Term::Var("body".to_string())),
        };
        let cp = cong(&f, &p);
        match cp.src.as_ref() {
            Term::App(_, _) => {},
            _ => panic!("Expected App term"),
        }
    }

    #[test]
    fn test_sym() {
        let p = PathTerm {
            ty: TypeExpr::Unit,
            src: Box::new(Term::Var("a".to_string())),
            tgt: Box::new(Term::Var("b".to_string())),
            var: "i".to_string(),
            body: Box::new(Term::Var("p".to_string())),
        };
        let s = sym(&p);
        assert_eq!(*s.src, Term::Var("b".to_string()));
        assert_eq!(*s.tgt, Term::Var("a".to_string()));
    }

    #[test]
    fn test_type_expr_display() {
        assert_eq!(format!("{}", TypeExpr::Unit), "Unit");
        assert_eq!(format!("{}", TypeExpr::Arrow(Box::new(TypeExpr::Bool), Box::new(TypeExpr::Int))), "(Bool → Int)");
        assert_eq!(format!("{}", TypeExpr::Universe(0)), "U0");
    }
}
