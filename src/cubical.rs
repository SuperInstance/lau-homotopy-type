//! Cubical types: cubes, face maps, degeneracies, and cubical sets.
//!
//! Models n-dimensional cubes with boundary maps and compositions.

use serde::{Deserialize, Serialize};
use crate::interval::{FaceMap, Degeneracy};

/// An n-dimensional cube is represented by its dimension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Cube {
    pub dim: usize,
}

impl Cube {
    /// The point (0-dimensional cube).
    pub fn point() -> Self { Cube { dim: 0 } }

    /// The interval (1-dimensional cube).
    pub fn interval() -> Self { Cube { dim: 1 } }

    /// The square (2-dimensional cube).
    pub fn square() -> Self { Cube { dim: 2 } }

    /// A cube of given dimension.
    pub fn n(dim: usize) -> Self { Cube { dim } }

    /// The boundary of a cube: all (n-1)-dimensional faces.
    pub fn boundary(&self) -> Vec<CubeFace> {
        if self.dim == 0 {
            return vec![];
        }
        let mut faces = Vec::with_capacity(self.dim * 2);
        for i in 0..self.dim {
            faces.push(CubeFace {
                cube: Cube::n(self.dim - 1),
                face_map: FaceMap::face_0(i),
            });
            faces.push(CubeFace {
                cube: Cube::n(self.dim - 1),
                face_map: FaceMap::face_1(i),
            });
        }
        faces
    }

    /// All degeneracies: projections that drop a dimension.
    pub fn degeneracies(&self) -> Vec<Degeneracy> {
        (0..self.dim).map(Degeneracy::new).collect()
    }
}

/// A face of a cube, obtained by applying a face map.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CubeFace {
    pub cube: Cube,
    pub face_map: FaceMap,
}

/// A cubical type: a presheaf on the cube category.
/// Represented as a mapping from cube dimension to a set of terms.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CubicalType<T: Clone + PartialEq> {
    /// The name of this cubical type.
    pub name: String,
    /// Terms at each dimension. dim 0 = points, dim 1 = paths, etc.
    pub terms: Vec<CubicalTerm<T>>,
}

/// A term in a cubical type, living at a certain dimension.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CubicalTerm<T: Clone + PartialEq> {
    pub dim: usize,
    pub value: T,
    /// Boundary: face maps to lower-dimensional terms.
    pub boundary: Vec<(FaceMap, usize)>, // index into parent's terms
}

impl<T: Clone + PartialEq> CubicalType<T> {
    pub fn new(name: &str) -> Self {
        CubicalType { name: name.to_string(), terms: vec![] }
    }

    /// Add a point (0-dim term).
    pub fn add_point(&mut self, value: T) -> usize {
        let idx = self.terms.len();
        self.terms.push(CubicalTerm { dim: 0, value, boundary: vec![] });
        idx
    }

    /// Add a path (1-dim term) between two points.
    pub fn add_path(&mut self, value: T, src: usize, tgt: usize) -> usize {
        let idx = self.terms.len();
        self.terms.push(CubicalTerm {
            dim: 1,
            value,
            boundary: vec![
                (FaceMap::face_0(0), src),
                (FaceMap::face_1(0), tgt),
            ],
        });
        idx
    }

    /// Add a square (2-dim term) between four paths.
    pub fn add_square(
        &mut self,
        value: T,
        bottom: usize,
        top: usize,
        left: usize,
        right: usize,
    ) -> usize {
        let idx = self.terms.len();
        self.terms.push(CubicalTerm {
            dim: 2,
            value,
            boundary: vec![
                (FaceMap::face_0(1), bottom),
                (FaceMap::face_1(1), top),
                (FaceMap::face_0(0), left),
                (FaceMap::face_1(0), right),
            ],
        });
        idx
    }

    /// Get all terms at a given dimension.
    pub fn terms_at_dim(&self, dim: usize) -> Vec<&CubicalTerm<T>> {
        self.terms.iter().filter(|t| t.dim == dim).collect()
    }
}

/// Composition structure for cubical types.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Composition<T: Clone + PartialEq> {
    /// The cube being filled.
    pub cube: Cube,
    /// The tube: sides of the composition.
    pub tube: Vec<CubicalTerm<T>>,
    /// The cap: the bottom face being extended.
    pub cap: CubicalTerm<T>,
    /// The resulting composed term.
    pub result: CubicalTerm<T>,
}

/// Kan filling condition: every open box has a filler.
pub trait CubicalKan<T: Clone + PartialEq> {
    /// Compose a path from a tube and cap.
    fn compose(
        &self,
        ty: &mut CubicalType<T>,
        tube: &[usize],
        cap: usize,
    ) -> usize;

    /// Fill an open box.
    fn fill(
        &self,
        ty: &mut CubicalType<T>,
        tube: &[usize],
        cap: usize,
    ) -> usize;
}

/// Standard Kan operations for discrete cubical types.
pub struct DiscreteKan;

impl<T: Clone + PartialEq + Default> CubicalKan<T> for DiscreteKan {
    fn compose(
        &self,
        ty: &mut CubicalType<T>,
        tube: &[usize],
        cap: usize,
    ) -> usize {
        let cap_term = &ty.terms[cap];
        let dim = cap_term.dim + 1;
        let idx = ty.terms.len();
        let value = cap_term.value.clone();
        let boundary: Vec<(FaceMap, usize)> = tube.iter()
            .enumerate()
            .flat_map(|(i, &t)| vec![
                (FaceMap::face_0(i), t),
                (FaceMap::face_1(i), t),
            ])
            .chain(std::iter::once((FaceMap::face_0(dim - 1), cap)))
            .collect();
        ty.terms.push(CubicalTerm { dim, value, boundary });
        idx
    }

    fn fill(
        &self,
        ty: &mut CubicalType<T>,
        tube: &[usize],
        cap: usize,
    ) -> usize {
        // Fill is compose + one more dimension
        self.compose(ty, tube, cap)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cube_dimensions() {
        assert_eq!(Cube::point().dim, 0);
        assert_eq!(Cube::interval().dim, 1);
        assert_eq!(Cube::square().dim, 2);
        assert_eq!(Cube::n(3).dim, 3);
    }

    #[test]
    fn test_cube_boundary() {
        let b = Cube::interval().boundary();
        assert_eq!(b.len(), 2); // two endpoints
        let b = Cube::square().boundary();
        assert_eq!(b.len(), 4); // four edges
        assert!(Cube::point().boundary().is_empty());
    }

    #[test]
    fn test_cubical_type_point() {
        let mut ct: CubicalType<String> = CubicalType::new("Test");
        let p = ct.add_point("base".to_string());
        assert_eq!(ct.terms[p].dim, 0);
        assert_eq!(ct.terms[p].value, "base");
    }

    #[test]
    fn test_cubical_type_path() {
        let mut ct: CubicalType<String> = CubicalType::new("Test");
        let a = ct.add_point("a".to_string());
        let b = ct.add_point("b".to_string());
        let p = ct.add_path("p".to_string(), a, b);
        assert_eq!(ct.terms[p].dim, 1);
        assert_eq!(ct.terms[p].boundary.len(), 2);
    }

    #[test]
    fn test_cubical_type_square() {
        let mut ct: CubicalType<String> = CubicalType::new("Test");
        let a = ct.add_point("a".to_string());
        let b = ct.add_point("b".to_string());
        let c = ct.add_point("c".to_string());
        let d = ct.add_point("d".to_string());
        let p = ct.add_path("p".to_string(), a, b);
        let q = ct.add_path("q".to_string(), c, d);
        let r = ct.add_path("r".to_string(), a, c);
        let s = ct.add_path("s".to_string(), b, d);
        let sq = ct.add_square("sq".to_string(), p, q, r, s);
        assert_eq!(ct.terms[sq].dim, 2);
        assert_eq!(ct.terms[sq].boundary.len(), 4);
    }

    #[test]
    fn test_terms_at_dim() {
        let mut ct: CubicalType<i32> = CubicalType::new("Test");
        ct.add_point(0);
        ct.add_point(1);
        let _a = &ct.terms[0];
        let _b = &ct.terms[1];
        ct.add_path(2, 0, 1);
        assert_eq!(ct.terms_at_dim(0).len(), 2);
        assert_eq!(ct.terms_at_dim(1).len(), 1);
    }

    #[test]
    fn test_degeneracies() {
        let degs = Cube::square().degeneracies();
        assert_eq!(degs.len(), 2);
    }

    #[test]
    fn test_kan_compose() {
        let mut ct: CubicalType<String> = CubicalType::new("Test");
        let a = ct.add_point("a".to_string());
        let b = ct.add_point("b".to_string());
        let cap = ct.add_path("cap".to_string(), a, b);
        let kan = DiscreteKan;
        let result = kan.compose(&mut ct, &[a, b], cap);
        assert_eq!(ct.terms[result].dim, 2);
    }
}
