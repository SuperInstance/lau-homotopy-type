# lau-homotopy-type

A Homotopy Type Theory library bridging algebraic topology, type theory, and compiler verification — implemented in Rust.

Cubical types, path types, identity types, transport, homotopies, equivalences, higher inductive types, fundamental group computation, and HoTT-based compiler verification primitives, all in one crate.

---

## What This Does

This crate provides a computational model of Homotopy Type Theory (HoTT), the foundation that treats types as spaces, terms as points, and equalities as paths. It implements:

- **Cubical type theory** — interval type, face maps, degeneracies, cubes, and Kan filling conditions
- **Path types** — propositional equality as first-class values with composition and inversion
- **Identity types** — the J-rule (path induction), definitional vs. propositional equality
- **Transport** — moving terms along paths between types (substitution)
- **Homotopies** — families of paths between functions, composition, inversion, whiskering
- **Equivalences** — quasi-inverses, half-adjoint equivalences, contractible fibers
- **Univalence** — the axiom that `(A = B) ≃ (A ≃ B)`
- **Higher inductive types (HITs)** — circles, spheres, intervals, suspensions, truncations, pushouts, coequalizers
- **Fundamental groups** — encode-decode proof that π₁(S¹) = ℤ
- **Compiler verification** — type checking as path equivalence, compilation phases as functors

## Key Idea

In HoTT, **equality is a type**, not a metatheoretic judgment. A proof that `a = b` is a *path* from `a` to `b`, and these paths are themselves terms that can be composed, inverted, and transported along. This creates a rich geometry of types where:

- Reflexivity is the constant path
- Symmetry is path inversion
- Transitivity is path composition
- Function extensionality is a homotopy between functions
- The univalence axiom makes equality of types equivalent to equivalence of types

This crate makes those abstractions concrete and executable in Rust.

## Install

Add to your `Cargo.toml`:

```toml
[dependencies]
lau-homotopy-type = "0.1.0"
```

Requires Rust 2021 edition or later.

## Quick Start

```rust
use lau_homotopy_type::prelude::*;

// Create a cubical type with points, paths, and squares
let mut ct: CubicalType<String> = CubicalType::new("MyType");
let a = ct.add_point("a".to_string());
let b = ct.add_point("b".to_string());
let p = ct.add_path("p".to_string(), a, b);
let q = ct.add_path("q".to_string(), a, b);
// A 2-cell (homotopy) between two paths
let sq = ct.add_square("H".to_string(), p, q, a, b);

// Equivalences compose
let e1 = Equivalence::new("f", TypeExpr::Bool, TypeExpr::Int, "fwd", "bwd");
let e2 = Equivalence::new("g", TypeExpr::Int, TypeExpr::Unit, "g_fwd", "g_bwd");
let composed = e1.compose(&e2)?; // Bool ≃ Unit

// Fundamental group of the circle
let proof = CircleFundamentalGroupProof::proof();
assert_eq!(proof.computed_group, "ℤ");

// Compiler pipeline as a chain of equivalences
let mut pipeline = CompilerPipeline::new("my-compiler");
pipeline.add_phase(CompilationPhase::new("parse", "Source", "AST"));
pipeline.add_phase(CompilationPhase::new("codegen", "AST", "Target"));
let total_equiv = pipeline.total_equivalence()?; // Source ≃ Target
```

## API Reference

### Module: `interval`
| Type | Description |
|------|-------------|
| `Interval` | The interval type with endpoints `I0`, `I1`, and variables |
| `Face` | Face constraints mapping interval variables to endpoints |
| `FaceMap` | Assigns interval endpoints for boundary maps |
| `Degeneracy` | Projections that drop a dimension |

### Module: `cubical`
| Type | Description |
|------|-------------|
| `Cube` | An n-dimensional cube |
| `CubeFace` | A face of a cube obtained by applying a face map |
| `CubicalType<T>` | A presheaf on the cube category — terms at each dimension |
| `Composition<T>` | Composition structure for cubical types |
| `DiscreteKan` | Standard Kan filling operations |

### Module: `path`
| Type | Description |
|------|-------------|
| `TypeExpr` | Type expressions (Unit, Bool, Int, Arrow, Product, Sum, Path, Universe) |
| `Term` | Terms (variables, lambdas, applications, pairs, projections, paths) |
| `PathTerm` | A path `Path A a b` parameterized by the interval |
| `hcomp_path()` | Homogeneous composition of paths |
| `cong()` | Congruence: apply a function to a path |
| `sym()` | Symmetry: invert a path |

### Module: `identity`
| Type | Description |
|------|-------------|
| `IdentityType` | The identity type `Id_A(a, b)` |
| `IdentityProof` | Proofs: Refl, Path, Compose, Inv, Cong, Transport |
| `IdentityContext` | Tracks known equalities in scope |
| `j_rule()` | The J-rule (path induction eliminator) |

### Module: `transport`
| Type | Description |
|------|-------------|
| `TypeFamily` | A type family indexed by a parameter |
| `TransportResult` | Result of a transport operation |
| `transport_along_path()` | Move a term along a path between types |
| `subst()` | Leibniz substitution: equals for equals |
| `transport_compose()` | Transport along composite paths |
| `transport_inv()` | Transport along inverted paths |

### Module: `homotopy`
| Type | Description |
|------|-------------|
| `Function` | A function between types |
| `Homotopy` | A homotopy `H : f ~ g` between functions |
| `NaturalTransformation` | Natural transformation between functors |
| `left_whisker()` | Left whiskering: compose a homotopy with a function |
| `linear_homotopy()` | Linear interpolation `H(x,t) = (1-t)f(x) + t·g(x)` |

### Module: `equivalence`
| Type | Description |
|------|-------------|
| `Equivalence` | An equivalence `A ≃ B` with forward/backward maps |
| `QuasiInverse` | A function with left and right inverses |
| `HalfAdjointEquiv` | An equivalence with extra coherence |
| `Fiber` | The fiber of a function over a point |
| `Univalence` | The univalence axiom implementation |
| `funext()` | Function extensionality |

### Module: `hots` (Higher Inductive Types)
| Type | Description |
|------|-------------|
| `Circle` | S¹: one point `base` and one `loop` |
| `Sphere` | Sⁿ: base point + n-dimensional cell |
| `IntervalHIT` | Two endpoints and a segment |
| `Suspension` | Suspension of a type |
| `Truncation` | n-truncation `‖A‖ₙ` |
| `Pushout` | Pushout `B +ᴬ C` |
| `Coequalizer` | Coequalizer identifying `f(a) = g(a)` |

### Module: `fundamental`
| Type | Description |
|------|-------------|
| `LoopElement` | Elements of π₁(S¹): `Refl` or `Pow(n)` |
| `FundamentalGroup` | The fundamental group π₁(X, x₀) |
| `CircleCode` | The encoding function for the encode-decode proof |
| `UniversalCover` | The universal cover ℝ → S¹ |
| `CircleFundamentalGroupProof` | Complete proof that π₁(S¹) = ℤ |

### Module: `hott_compiler`
| Type | Description |
|------|-------------|
| `TypeCheckResult` | DefEqual, PropEqual, or TypeError |
| `CompilationPhase` | A compilation phase as a functor |
| `CompilerPipeline` | A chain of compilation phases |
| `VerifiedCompilation` | Correctness witnessed by an equivalence |
| `TypeSafety` | Progress + preservation guarantees |
| `RelationalSemantics` | Source-target relation with determinism/totality checks |

## How It Works

The library is organized in layers:

1. **Interval** (`interval.rs`): The abstract interval `I` with endpoints `i0`, `i1`. Face maps `δ_i^0`, `δ_i^1` constrain dimensions, and degeneracy maps project dimensions away. This is the substrate on which all higher-dimensional structure is built.

2. **Cubical types** (`cubical.rs`): n-dimensional cubes with boundary maps and compositions. `CubicalType<T>` stores terms at each dimension — points (dim 0), paths (dim 1), squares (dim 2), and higher. The `DiscreteKan` implementation provides the Kan filling condition: every open box has a filler.

3. **Paths** (`path.rs`): `PathTerm` represents a path `p : Path A a b` as a lambda over the interval variable. Paths compose via diagonal composition, invert by swapping endpoints, and can be applied at endpoints (0 → source, 1 → target).

4. **Identity types** (`identity.rs`): Propositional equality as a type. `IdentityProof` supports reflexivity, explicit paths, composition (transitivity), inversion (symmetry), congruence, and transport. The J-rule is the eliminator: given a motive and a base case at reflexivity, it produces a proof for any equality.

5. **Transport** (`transport.rs`): The fundamental substitution operation. Given a path between types, move a term from one type to the other. Transport composes (`transport(p·q, a) = transport(q, transport(p, a))`) and inverts (`transport(p⁻¹, transport(p, a)) = a`). In a constant type family, transport is the identity.

6. **Homotopies** (`homotopy.rs`): A homotopy `H : f ~ g` between functions is a family of paths `H(x) : f(x) = g(x)`. Homotopies compose (vertical composition), invert, and whisker (pre/post-compose with a function). The module also includes natural transformations (categorical perspective) and numerical linear homotopies via `nalgebra`.

7. **Equivalences** (`equivalence.rs`): An equivalence `A ≃ B` is a function with a quasi-inverse plus coherence conditions. The module provides composition, inversion, half-adjoint promotion, fiber contractibility checking, the univalence axiom (`idtoeqv` and `ua`), and function extensionality.

8. **Higher inductive types** (`hots.rs`): HITs generate non-trivial homotopy types by adding path/face constructors. The circle `S¹` has one point and one loop. The sphere `Sⁿ` has a base point and one n-cell. Suspensions, truncations, pushouts, and coequalizers are also provided.

9. **Fundamental group** (`fundamental.rs`): The encode-decode method proves π₁(S¹) = ℤ. The encoding function maps loop elements to paths in the code fiber ℤ, and decoding maps them back. Round-trip verification confirms the isomorphism. The universal cover ℝ → S¹ provides the simply-connected covering space.

10. **Compiler verification** (`hott_compiler.rs`): Type checking becomes path equivalence checking — type errors correspond to non-contractible loops (holes in proofs). Compilation phases are functors, and correctness is witnessed by natural transformations. `RelationalSemantics` provides source-target relations with determinism and totality checks.

## The Math

**Homotopy Type Theory** (HoTT) is a foundation for mathematics that interprets Martin-Löf type theory through the lens of homotopy theory. Key correspondences:

| Type theory | Homotopy theory |
|-------------|----------------|
| Type | Space |
| Term `a : A` | Point `a ∈ A` |
| Identity type `a =_A b` | Path space from `a` to `b` |
| Reflexivity `refl_a` | Constant path |
| Function `f : A → B` | Continuous map |
| Homotopy `H : f ~ g` | Continuous deformation |
| Equivalence `A ≃ B` | Homotopy equivalence |
| Higher inductive type | Cell complex |

The **univalence axiom** states that the canonical map `(A = B) → (A ≃ B)` is itself an equivalence. This means equality of types *is* equivalence of types — a powerful principle that lets us transport any construction along equivalences.

The **Kan condition** ensures that every "open box" (partial cube with missing face) can be filled. This is the key property that makes cubical type theory computational — univalence is not just an axiom but a function you can run.

The **encode-decode method** for π₁(S¹) = ℤ works by:
1. Defining a type family `Code : S¹ → Type` with `Code(base) = ℤ`
2. Proving `encode ∘ decode = id` and `decode ∘ encode = id`
3. Concluding `π₁(S¹, base) ≅ Code(base) ≅ ℤ`

## Tests

The crate contains **100 unit tests** covering:
- Cube boundary and dimension calculations
- Cubical type construction (points, paths, squares)
- Path composition, inversion, and congruence
- Identity proofs (reflexivity, composition, inversion, J-rule)
- Transport round-trips and composition
- Homotopy operations (compose, invert, whisker)
- Equivalence composition, inversion, and fiber contractibility
- Univalence witnesses and function extensionality
- HIT construction (circles, spheres, intervals, suspensions, truncations)
- Fundamental group encode-decode proof
- Compiler pipeline verification and type safety

Run with:

```bash
cargo test
```

## License

MIT
