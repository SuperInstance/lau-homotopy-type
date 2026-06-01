//! Homotopy between functions and natural transformations.
//!
//! A homotopy between functions f, g : A → B is a family of paths
//! `H(x) : f(x) = g(x)` for all x : A. This is the HoTT analogue
//! of a natural transformation.

use serde::{Deserialize, Serialize};
use crate::path::TypeExpr;
use nalgebra::DVector;

/// A function between types (simplified representation).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    pub domain: TypeExpr,
    pub codomain: TypeExpr,
}

impl Function {
    pub fn new(name: &str, domain: TypeExpr, codomain: TypeExpr) -> Self {
        Function { name: name.to_string(), domain, codomain }
    }
}

/// A homotopy between two functions f, g : A → B.
/// Represented as a family of paths H(x) : f(x) = g(x).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Homotopy {
    pub name: String,
    pub f: Function,
    pub g: Function,
    /// The homotopy witness: for each point, a path.
    pub witnesses: Vec<HomotopyWitness>,
}

/// A single witness in a homotopy: a path at a specific point.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HomotopyWitness {
    pub point: String,
    pub path_name: String,
    pub f_value: String,
    pub g_value: String,
}

impl Homotopy {
    /// Create a new homotopy between two functions.
    pub fn new(name: &str, f: Function, g: Function) -> Result<Self, HomotopyError> {
        if f.domain != g.domain || f.codomain != g.codomain {
            return Err(HomotopyError::IncompatibleFunctions);
        }
        Ok(Homotopy {
            name: name.to_string(),
            f,
            g,
            witnesses: vec![],
        })
    }

    /// Add a witness at a point.
    pub fn witness(&mut self, point: &str, f_val: &str, g_val: &str) {
        self.witnesses.push(HomotopyWitness {
            point: point.to_string(),
            path_name: format!("H_{}", point),
            f_value: f_val.to_string(),
            g_value: g_val.to_string(),
        });
    }

    /// Compose two homotopies: if H : f ~ g and K : g ~ h, then K ∘ H : f ~ h.
    pub fn compose(&self, other: &Homotopy) -> Result<Homotopy, HomotopyError> {
        if self.g.name != other.f.name {
            return Err(HomotopyError::CompositionMismatch);
        }
        let mut h = Homotopy::new(
            &format!("{}_then_{}", self.name, other.name),
            self.f.clone(),
            other.g.clone(),
        )?;
        for w in &self.witnesses {
            h.witness(&w.point, &w.f_value, &w.g_value);
        }
        Ok(h)
    }

    /// Invert a homotopy: if H : f ~ g, then H⁻¹ : g ~ f.
    pub fn inverse(&self) -> Homotopy {
        let mut inv = Homotopy {
            name: format!("{}_inv", self.name),
            f: self.g.clone(),
            g: self.f.clone(),
            witnesses: vec![],
        };
        for w in &self.witnesses {
            inv.witnesses.push(HomotopyWitness {
                point: w.point.clone(),
                path_name: format!("{}_inv", w.path_name),
                f_value: w.g_value.clone(),
                g_value: w.f_value.clone(),
            });
        }
        inv
    }

    /// Number of witnesses.
    pub fn num_witnesses(&self) -> usize {
        self.witnesses.len()
    }
}

/// Errors in homotopy operations.
#[derive(Debug, Clone, PartialEq)]
pub enum HomotopyError {
    IncompatibleFunctions,
    CompositionMismatch,
}

/// A natural transformation between functors (categorical perspective).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NaturalTransformation {
    pub name: String,
    pub source_functor: String,
    pub target_functor: String,
    pub components: Vec<(String, String)>, // (object, morphism_name)
}

impl NaturalTransformation {
    pub fn new(name: &str, src: &str, tgt: &str) -> Self {
        NaturalTransformation {
            name: name.to_string(),
            source_functor: src.to_string(),
            target_functor: tgt.to_string(),
            components: vec![],
        }
    }

    pub fn add_component(&mut self, obj: &str, morphism: &str) {
        self.components.push((obj.to_string(), morphism.to_string()));
    }

    /// Vertical composition of natural transformations.
    pub fn vertical_compose(&self, other: &NaturalTransformation) -> Result<NaturalTransformation, HomotopyError> {
        if self.target_functor != other.source_functor {
            return Err(HomotopyError::CompositionMismatch);
        }
        let mut result = NaturalTransformation::new(
            &format!("{}_vcomp_{}", self.name, other.name),
            &self.source_functor,
            &other.target_functor,
        );
        for (obj, _) in &self.components {
            if let Some((_, other_mor)) = other.components.iter().find(|(o, _)| o == obj) {
                result.add_component(obj, other_mor);
            }
        }
        Ok(result)
    }
}

/// Whiskering: compose a homotopy with a function.
/// If H : f ~ g and h : B → C, then h ∘ H : h∘f ~ h∘g (left whisker).
pub fn left_whisker(h_func: &Function, hom: &Homotopy) -> Homotopy {
    let whiskered = Function::new(
        &format!("{}_after_{}", h_func.name, hom.f.name),
        hom.f.domain.clone(),
        h_func.codomain.clone(),
    );
    let mut result = Homotopy {
        name: format!("{}_whisker_{}", h_func.name, hom.name),
        f: whiskered.clone(),
        g: Function::new(
            &format!("{}_after_{}", h_func.name, hom.g.name),
            hom.g.domain.clone(),
            h_func.codomain.clone(),
        ),
        witnesses: vec![],
    };
    for w in &hom.witnesses {
        result.witness(&w.point, &format!("{}({})", h_func.name, w.f_value), &format!("{}({})", h_func.name, w.g_value));
    }
    result
}

/// Compute a homotopy as a continuous deformation using linear interpolation.
/// Maps R^n → R^m functions and computes the homotopy H(x,t) = (1-t)f(x) + t·g(x).
pub fn linear_homotopy(
    f_values: &[f64],
    g_values: &[f64],
    t: f64,
) -> DVector<f64> {
    let f = DVector::from_vec(f_values.to_vec());
    let g = DVector::from_vec(g_values.to_vec());
    (1.0 - t) * &f + t * &g
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_homotopy_create() {
        let f = Function::new("f", TypeExpr::Unit, TypeExpr::Bool);
        let g = Function::new("g", TypeExpr::Unit, TypeExpr::Bool);
        let h = Homotopy::new("H", f, g).unwrap();
        assert_eq!(h.name, "H");
        assert_eq!(h.num_witnesses(), 0);
    }

    #[test]
    fn test_homotopy_witness() {
        let f = Function::new("f", TypeExpr::Unit, TypeExpr::Bool);
        let g = Function::new("g", TypeExpr::Unit, TypeExpr::Bool);
        let mut h = Homotopy::new("H", f, g).unwrap();
        h.witness("x", "true", "true");
        assert_eq!(h.num_witnesses(), 1);
    }

    #[test]
    fn test_homotopy_inverse() {
        let f = Function::new("f", TypeExpr::Unit, TypeExpr::Unit);
        let g = Function::new("g", TypeExpr::Unit, TypeExpr::Unit);
        let mut h = Homotopy::new("H", f, g).unwrap();
        h.witness("x", "a", "b");
        let inv = h.inverse();
        assert_eq!(inv.f.name, "g");
        assert_eq!(inv.g.name, "f");
        assert_eq!(inv.witnesses[0].f_value, "b");
        assert_eq!(inv.witnesses[0].g_value, "a");
    }

    #[test]
    fn test_homotopy_compose() {
        let f = Function::new("f", TypeExpr::Unit, TypeExpr::Unit);
        let g = Function::new("g", TypeExpr::Unit, TypeExpr::Unit);
        let h2 = Function::new("h", TypeExpr::Unit, TypeExpr::Unit);
        let mut h1 = Homotopy::new("H1", f, g.clone()).unwrap();
        h1.witness("x", "a", "b");
        let h2 = Homotopy::new("H2", g, h2).unwrap();
        let composed = h1.compose(&h2).unwrap();
        assert_eq!(composed.f.name, "f");
        assert_eq!(composed.g.name, "h");
    }

    #[test]
    fn test_homotopy_incompatible() {
        let f = Function::new("f", TypeExpr::Unit, TypeExpr::Bool);
        let g = Function::new("g", TypeExpr::Bool, TypeExpr::Bool);
        assert!(Homotopy::new("H", f, g).is_err());
    }

    #[test]
    fn test_natural_transformation() {
        let mut nt = NaturalTransformation::new("alpha", "F", "G");
        nt.add_component("X", "alpha_X");
        nt.add_component("Y", "alpha_Y");
        assert_eq!(nt.components.len(), 2);
    }

    #[test]
    fn test_vertical_compose() {
        let mut nt1 = NaturalTransformation::new("alpha", "F", "G");
        nt1.add_component("X", "alpha_X");
        let mut nt2 = NaturalTransformation::new("beta", "G", "H");
        nt2.add_component("X", "beta_X");
        let vcomp = nt1.vertical_compose(&nt2).unwrap();
        assert_eq!(vcomp.source_functor, "F");
        assert_eq!(vcomp.target_functor, "H");
    }

    #[test]
    fn test_left_whisker() {
        let f = Function::new("f", TypeExpr::Unit, TypeExpr::Unit);
        let g = Function::new("g", TypeExpr::Unit, TypeExpr::Unit);
        let h_func = Function::new("h", TypeExpr::Unit, TypeExpr::Unit);
        let mut hom = Homotopy::new("H", f, g).unwrap();
        hom.witness("x", "a", "b");
        let whiskered = left_whisker(&h_func, &hom);
        assert!(whiskered.name.contains("whisker"));
        assert_eq!(whiskered.num_witnesses(), 1);
    }

    #[test]
    fn test_linear_homotopy() {
        let f = vec![1.0, 2.0];
        let g = vec![3.0, 6.0];
        let h0 = linear_homotopy(&f, &g, 0.0);
        let h1 = linear_homotopy(&f, &g, 1.0);
        let h_half = linear_homotopy(&f, &g, 0.5);
        assert_eq!(h0[0], 1.0);
        assert_eq!(h1[0], 3.0);
        assert_eq!(h_half[0], 2.0);
        assert_eq!(h_half[1], 4.0);
    }
}
