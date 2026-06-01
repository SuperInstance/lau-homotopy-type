//! Higher Inductive Types (HITs): circles, spheres, intervals, suspensions, truncations.
//!
//! HITs generalize inductive types by allowing path/face constructors
//! in addition to point constructors, generating non-trivial homotopy types.

use serde::{Deserialize, Serialize};
use crate::path::{TypeExpr, Term, PathTerm};

/// The circle S¹: one point (base) and one loop (loop).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Circle {
    /// The base point.
    pub base: String,
    /// The loop path: base = base.
    pub loop_path: PathTerm,
}

impl Circle {
    /// Construct the circle with its point and path constructors.
    pub fn new() -> Self {
        Circle {
            base: "base".to_string(),
            loop_path: PathTerm {
                ty: TypeExpr::Named("S¹".to_string()),
                src: Box::new(Term::Var("base".to_string())),
                tgt: Box::new(Term::Var("base".to_string())),
                var: "i".to_string(),
                body: Box::new(Term::Var("loop".to_string())),
            },
        }
    }

    /// The recursion principle for S¹.
    /// Given b : B and l : b = b, produce a map S¹ → B.
    pub fn rec(&self, b: &str, l: &PathTerm) -> CircleRec {
        CircleRec {
            base_image: b.to_string(),
            loop_image: l.clone(),
        }
    }
}

impl Default for Circle {
    fn default() -> Self { Self::new() }
}

/// Result of circle recursion.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CircleRec {
    pub base_image: String,
    pub loop_image: PathTerm,
}

/// The n-sphere Sⁿ.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sphere {
    pub dimension: usize,
    pub base: String,
    /// For S⁰: two points. For Sⁿ (n≥1): base + one n-dimensional cell.
    pub constructors: Vec<SphereConstructor>,
}

/// A constructor for a sphere: either a point or a higher cell.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SphereConstructor {
    Point(String),
    /// Cell of dimension n, connecting boundary to base.
    Cell { dim: usize, name: String },
}

impl Sphere {
    /// S⁰: two points, north and south.
    pub fn s0() -> Self {
        Sphere {
            dimension: 0,
            base: "north".to_string(),
            constructors: vec![
                SphereConstructor::Point("north".to_string()),
                SphereConstructor::Point("south".to_string()),
            ],
        }
    }

    /// S¹: the circle.
    pub fn s1() -> Self {
        Sphere {
            dimension: 1,
            base: "base".to_string(),
            constructors: vec![
                SphereConstructor::Point("base".to_string()),
                SphereConstructor::Cell { dim: 1, name: "loop".to_string() },
            ],
        }
    }

    /// S²: base point + 2-cell (like the surface of a globe).
    pub fn s2() -> Self {
        Sphere {
            dimension: 2,
            base: "base".to_string(),
            constructors: vec![
                SphereConstructor::Point("base".to_string()),
                SphereConstructor::Cell { dim: 2, name: "surf".to_string() },
            ],
        }
    }

    /// Generic Sⁿ.
    pub fn sn(n: usize) -> Self {
        if n == 0 { return Self::s0(); }
        Sphere {
            dimension: n,
            base: "base".to_string(),
            constructors: vec![
                SphereConstructor::Point("base".to_string()),
                SphereConstructor::Cell { dim: n, name: format!("cell_{}", n) },
            ],
        }
    }

    /// Suspension: ΣX has two points (north, south) and a meridian for each point of X.
    pub fn suspend(base_type: &str) -> Suspension {
        Suspension {
            base_type: base_type.to_string(),
            north: "north".to_string(),
            south: "south".to_string(),
            meridians: vec![],
        }
    }
}

/// The interval as a HIT: two endpoints and a path between them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntervalHIT {
    pub left: String,
    pub right: String,
    pub seg: PathTerm,
}

impl IntervalHIT {
    pub fn new() -> Self {
        IntervalHIT {
            left: "i0".to_string(),
            right: "i1".to_string(),
            seg: PathTerm {
                ty: TypeExpr::Named("Interval".to_string()),
                src: Box::new(Term::Var("i0".to_string())),
                tgt: Box::new(Term::Var("i1".to_string())),
                var: "t".to_string(),
                body: Box::new(Term::Var("seg".to_string())),
            },
        }
    }

    /// Recursion: given a, b : B and p : a = b, produce Interval → B.
    pub fn rec(&self, a: &str, b: &str, p: &PathTerm) -> IntervalRec {
        IntervalRec {
            left_image: a.to_string(),
            right_image: b.to_string(),
            seg_image: p.clone(),
        }
    }
}

impl Default for IntervalHIT {
    fn default() -> Self { Self::new() }
}

/// Result of interval recursion.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntervalRec {
    pub left_image: String,
    pub right_image: String,
    pub seg_image: PathTerm,
}

/// Suspension of a type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Suspension {
    pub base_type: String,
    pub north: String,
    pub south: String,
    pub meridians: Vec<(String, String)>, // (point_name, meridian_name)
}

impl Suspension {
    pub fn add_meridian(&mut self, point: &str) {
        let idx = self.meridians.len();
        self.meridians.push((point.to_string(), format!("merid_{}", idx)));
    }

    /// Suspension of Sⁿ gives Sⁿ⁺¹.
    pub fn suspend_sphere(n: usize) -> Self {
        let sn = Sphere::sn(n);
        let mut susp = Suspension {
            base_type: format!("S{}", n),
            north: "north".to_string(),
            south: "south".to_string(),
            meridians: vec![],
        };
        for c in &sn.constructors {
            if let SphereConstructor::Point(name) = c {
                susp.add_meridian(name);
            }
        }
        susp
    }
}

/// Truncation: ‖A‖ₙ is the n-truncation of A.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Truncation {
    pub original_type: String,
    pub truncation_level: isize, // -1 = contractible, 0 = hProp, 1 = hSet, etc.
}

impl Truncation {
    pub fn new(type_name: &str, level: isize) -> Self {
        Truncation {
            original_type: type_name.to_string(),
            truncation_level: level,
        }
    }

    /// Is this a proposition (0-truncated)?
    pub fn is_prop(&self) -> bool { self.truncation_level == 0 }

    /// Is this a set (1-truncated)?
    pub fn is_set(&self) -> bool { self.truncation_level == 1 }

    /// Is this contractible (-1-truncated)?
    pub fn is_contractible(&self) -> bool { self.truncation_level == -1 }

    /// The unit type is the (-2)-truncation (contractible).
    pub fn unit() -> Self {
        Truncation { original_type: "Unit".to_string(), truncation_level: -2 }
    }

    /// Truncation level name.
    pub fn level_name(&self) -> &'static str {
        match self.truncation_level {
            -2 => "contractible",
            -1 => "hProp",
            0 => "hProp",
            1 => "hSet",
            2 => "hGroupoid",
            _ => "n-type",
        }
    }
}

/// Propositional truncation: ‖A‖ (0-truncation).
pub fn prop_truncation(type_name: &str) -> Truncation {
    Truncation::new(type_name, 0)
}

/// Pushout: given f : A → B, g : A → C, the pushout B +ᴬ C.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pushout {
    pub left: String,
    pub right: String,
    pub glue: Vec<String>,
}

impl Pushout {
    pub fn new(left: &str, right: &str) -> Self {
        Pushout {
            left: left.to_string(),
            right: right.to_string(),
            glue: vec![],
        }
    }

    pub fn add_glue(&mut self, name: &str) {
        self.glue.push(name.to_string());
    }
}

/// Coequalizer: given f, g : A → B, identify f(a) and g(a) for all a.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Coequalizer {
    pub f: String,
    pub g: String,
    pub identified: Vec<(String, String)>,
}

impl Coequalizer {
    pub fn new(f: &str, g: &str) -> Self {
        Coequalizer {
            f: f.to_string(),
            g: g.to_string(),
            identified: vec![],
        }
    }

    pub fn identify(&mut self, a: &str, b: &str) {
        self.identified.push((a.to_string(), b.to_string()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circle() {
        let c = Circle::new();
        assert_eq!(c.base, "base");
        assert!(c.loop_path.is_refl()); // loop: base = base
    }

    #[test]
    fn test_circle_rec() {
        let c = Circle::new();
        let loop_img = PathTerm::refl(TypeExpr::Unit, Term::Var("x".to_string()));
        let rec = c.rec("x", &loop_img);
        assert_eq!(rec.base_image, "x");
    }

    #[test]
    fn test_sphere_s0() {
        let s = Sphere::s0();
        assert_eq!(s.dimension, 0);
        assert_eq!(s.constructors.len(), 2);
    }

    #[test]
    fn test_sphere_s1() {
        let s = Sphere::s1();
        assert_eq!(s.dimension, 1);
        assert_eq!(s.constructors.len(), 2);
    }

    #[test]
    fn test_sphere_s2() {
        let s = Sphere::s2();
        assert_eq!(s.dimension, 2);
        let cell = &s.constructors[1];
        match cell {
            SphereConstructor::Cell { dim, name } => {
                assert_eq!(*dim, 2);
                assert_eq!(name, "surf");
            },
            _ => panic!("Expected cell constructor"),
        }
    }

    #[test]
    fn test_sphere_sn() {
        let s5 = Sphere::sn(5);
        assert_eq!(s5.dimension, 5);
    }

    #[test]
    fn test_interval_hit() {
        let i = IntervalHIT::new();
        assert_eq!(i.left, "i0");
        assert_eq!(i.right, "i1");
    }

    #[test]
    fn test_interval_rec() {
        let i = IntervalHIT::new();
        let p = PathTerm {
            ty: TypeExpr::Unit,
            src: Box::new(Term::Var("a".to_string())),
            tgt: Box::new(Term::Var("b".to_string())),
            var: "t".to_string(),
            body: Box::new(Term::Var("p".to_string())),
        };
        let rec = i.rec("a", "b", &p);
        assert_eq!(rec.left_image, "a");
        assert_eq!(rec.right_image, "b");
    }

    #[test]
    fn test_suspension() {
        let mut susp = Sphere::suspend("S0");
        susp.add_meridian("north");
        susp.add_meridian("south");
        assert_eq!(susp.meridians.len(), 2);
    }

    #[test]
    fn test_suspend_sphere() {
        let susp = Suspension::suspend_sphere(0);
        assert_eq!(susp.north, "north");
        assert_eq!(susp.south, "south");
    }

    #[test]
    fn test_truncation_levels() {
        let prop = Truncation::new("A", 0);
        let set = Truncation::new("A", 1);
        let contract = Truncation::new("A", -1);
        assert!(prop.is_prop());
        assert!(set.is_set());
        assert!(contract.is_contractible());
        assert!(!prop.is_set());
    }

    #[test]
    fn test_truncation_level_names() {
        assert_eq!(Truncation::unit().level_name(), "contractible");
        assert_eq!(Truncation::new("A", 0).level_name(), "hProp");
        assert_eq!(Truncation::new("A", 1).level_name(), "hSet");
    }

    #[test]
    fn test_pushout() {
        let mut po = Pushout::new("B", "C");
        po.add_glue("g1");
        po.add_glue("g2");
        assert_eq!(po.glue.len(), 2);
    }

    #[test]
    fn test_coequalizer() {
        let mut coeq = Coequalizer::new("f", "g");
        coeq.identify("a", "b");
        coeq.identify("c", "d");
        assert_eq!(coeq.identified.len(), 2);
    }
}
