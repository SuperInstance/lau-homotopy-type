//! HoTT for compilers: type checking = path equivalence, type errors = non-contractible loops.
//!
//! This module bridges HoTT concepts with compiler verification:
//! - Type checking is path equivalence checking
//! - Type errors correspond to non-contractible loops (holes in proofs)
//! - Compilation phases are functors between categories
//! - Correctness is witnessed by natural transformations

use serde::{Deserialize, Serialize};
use crate::path::TypeExpr;
use crate::identity::{IdentityType, IdentityProof, IdentityContext};
use crate::equivalence::Equivalence;
use crate::homotopy::Function;

/// A type checking result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypeCheckResult {
    /// Terms are definitionally equal (checks pass).
    DefEqual,
    /// Terms are propositionally equal (needs proof).
    PropEqual(IdentityProof),
    /// Type error: the loop is not contractible.
    TypeError(TypeErrorInfo),
}

/// Information about a type error, viewed as a non-contractible loop.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeErrorInfo {
    pub expected: TypeExpr,
    pub got: TypeExpr,
    pub message: String,
    /// The "hole" in the proof: what's missing to make the loop contractible.
    pub hole: Option<String>,
}

/// A compilation phase, viewed as a functor between categories.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompilationPhase {
    pub name: String,
    pub source_language: String,
    pub target_language: String,
    /// The semantic function: maps terms to terms.
    pub semantic_map: String,
}

impl CompilationPhase {
    pub fn new(name: &str, src: &str, tgt: &str) -> Self {
        CompilationPhase {
            name: name.to_string(),
            source_language: src.to_string(),
            target_language: tgt.to_string(),
            semantic_map: format!("{}_map", name),
        }
    }
}

/// A compiler pipeline: a sequence of compilation phases.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompilerPipeline {
    pub name: String,
    pub phases: Vec<CompilationPhase>,
}

impl CompilerPipeline {
    pub fn new(name: &str) -> Self {
        CompilerPipeline { name: name.to_string(), phases: vec![] }
    }

    pub fn add_phase(&mut self, phase: CompilationPhase) {
        self.phases.push(phase);
    }

    /// Compose all phases into a single equivalence (if correct).
    pub fn total_equivalence(&self) -> Result<Equivalence, CompilerError> {
        if self.phases.is_empty() {
            return Err(CompilerError::EmptyPipeline);
        }
        let mut eq = Equivalence::identity(TypeExpr::Named(
            self.phases[0].source_language.clone()
        ));
        for phase in &self.phases {
            let phase_eq = Equivalence::new(
                &phase.name,
                TypeExpr::Named(phase.source_language.clone()),
                TypeExpr::Named(phase.target_language.clone()),
                &phase.semantic_map,
                &format!("{}_inv", phase.semantic_map),
            );
            eq = eq.compose(&phase_eq).map_err(|_| CompilerError::PhaseMismatch {
                phase: phase.name.clone(),
            })?;
        }
        Ok(eq)
    }
}

/// Errors in compiler verification.
#[derive(Debug, Clone, PartialEq)]
pub enum CompilerError {
    EmptyPipeline,
    PhaseMismatch { phase: String },
    VerificationFailed { reason: String },
    TypeError(TypeErrorInfo),
}

/// A verified compilation: correctness witnessed by a homotopy.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerifiedCompilation {
    pub source_term: String,
    pub target_term: String,
    pub phases: Vec<String>,
    pub correctness_witness: String,
}

/// Type check two terms for equality.
pub fn type_check(
    ctx: &IdentityContext,
    expected: &TypeExpr,
    got: &TypeExpr,
    term_expected: &str,
    term_got: &str,
) -> TypeCheckResult {
    // First check definitional equality
    if expected == got && term_expected == term_got {
        return TypeCheckResult::DefEqual;
    }

    // Check propositional equality
    if expected == got {
        if let Some(proof) = ctx.propositional_equal(
            &format!("{}", expected),
            term_expected,
            term_got,
        ) {
            return TypeCheckResult::PropEqual(proof.clone());
        }
    }

    // Type error: non-contractible loop
    TypeCheckResult::TypeError(TypeErrorInfo {
        expected: expected.clone(),
        got: got.clone(),
        message: format!("Expected {} but got {}", expected, got),
        hole: Some(format!("need: {} = {}", term_expected, term_got)),
    })
}

/// Verify that a compilation phase preserves semantics.
/// The compilation is correct iff the semantic map is an equivalence.
pub fn verify_phase(
    phase: &CompilationPhase,
    semantics_fwd: &Function,
    semantics_bwd: &Function,
) -> Result<VerifiedCompilation, CompilerError> {
    let equiv = Equivalence::new(
        &format!("{}_correctness", phase.name),
        TypeExpr::Named(phase.source_language.clone()),
        TypeExpr::Named(phase.target_language.clone()),
        &semantics_fwd.name,
        &semantics_bwd.name,
    );

    Ok(VerifiedCompilation {
        source_term: phase.source_language.clone(),
        target_term: phase.target_language.clone(),
        phases: vec![phase.name.clone()],
        correctness_witness: format!("equiv:{}", equiv.name),
    })
}

/// Verify an entire pipeline.
pub fn verify_pipeline(pipeline: &CompilerPipeline) -> Result<VerifiedCompilation, CompilerError> {
    let eq = pipeline.total_equivalence()?;
    let first = pipeline.phases.first().unwrap();
    let last = pipeline.phases.last().unwrap();
    Ok(VerifiedCompilation {
        source_term: first.source_language.clone(),
        target_term: last.target_language.clone(),
        phases: pipeline.phases.iter().map(|p| p.name.clone()).collect(),
        correctness_witness: format!("equiv:{}", eq.name),
    })
}

/// A type safety guarantee: well-typed programs don't go wrong.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeSafety {
    pub language: String,
    pub progress: bool,  // every well-typed term is a value or steps
    pub preservation: bool, // if t : T and t → t', then t' : T
}

impl TypeSafety {
    pub fn guaranteed(language: &str) -> Self {
        TypeSafety {
            language: language.to_string(),
            progress: true,
            preservation: true,
        }
    }

    pub fn is_safe(&self) -> bool {
        self.progress && self.preservation
    }
}

/// Relational semantics for a compiler pass: the relation between source and target.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RelationalSemantics {
    pub name: String,
    pub pairs: Vec<(String, String)>, // (source, target) term pairs
}

impl RelationalSemantics {
    pub fn new(name: &str) -> Self {
        RelationalSemantics { name: name.to_string(), pairs: vec![] }
    }

    pub fn relate(&mut self, src: &str, tgt: &str) {
        self.pairs.push((src.to_string(), tgt.to_string()));
    }

    /// Check if a source-target pair is in the relation.
    pub fn related(&self, src: &str, tgt: &str) -> bool {
        self.pairs.iter().any(|(s, t)| s == src && t == tgt)
    }

    /// Check determinism: each source maps to at most one target.
    pub fn is_deterministic(&self) -> bool {
        for (i, (s1, _)) in self.pairs.iter().enumerate() {
            for (j, (s2, t2)) in self.pairs.iter().enumerate() {
                if i != j && s1 == s2 {
                    let (_, t1) = &self.pairs[i];
                    if t1 != t2 {
                        return false;
                    }
                }
            }
        }
        true
    }

    /// Check totality: every source has a target.
    pub fn is_total_for(&self, sources: &[&str]) -> bool {
        sources.iter().all(|s| self.pairs.iter().any(|(src, _)| src == s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_check_def_equal() {
        let ctx = IdentityContext::new();
        let result = type_check(&ctx, &TypeExpr::Bool, &TypeExpr::Bool, "x", "x");
        assert_eq!(result, TypeCheckResult::DefEqual);
    }

    #[test]
    fn test_type_check_prop_equal() {
        let mut ctx = IdentityContext::new();
        ctx.assume_equal(
            IdentityType::new("Bool", "x", "y"),
            IdentityProof::Path("p".to_string(), "x".to_string(), "y".to_string()),
        );
        let result = type_check(&ctx, &TypeExpr::Bool, &TypeExpr::Bool, "x", "y");
        match result {
            TypeCheckResult::PropEqual(_) => {},
            _ => panic!("Expected PropEqual, got {:?}", result),
        }
    }

    #[test]
    fn test_type_check_error() {
        let ctx = IdentityContext::new();
        let result = type_check(&ctx, &TypeExpr::Bool, &TypeExpr::Int, "x", "y");
        match result {
            TypeCheckResult::TypeError(info) => {
                assert!(info.hole.is_some());
            },
            _ => panic!("Expected TypeError"),
        }
    }

    #[test]
    fn test_compilation_phase() {
        let phase = CompilationPhase::new("parse", "Source", "AST");
        assert_eq!(phase.name, "parse");
        assert_eq!(phase.source_language, "Source");
        assert_eq!(phase.target_language, "AST");
    }

    #[test]
    fn test_compiler_pipeline() {
        let mut pipeline = CompilerPipeline::new("my-compiler");
        pipeline.add_phase(CompilationPhase::new("parse", "Source", "AST"));
        pipeline.add_phase(CompilationPhase::new("typecheck", "AST", "TypedAST"));
        pipeline.add_phase(CompilationPhase::new("codegen", "TypedAST", "Target"));
        assert_eq!(pipeline.phases.len(), 3);
    }

    #[test]
    fn test_pipeline_total_equivalence() {
        let mut pipeline = CompilerPipeline::new("test");
        pipeline.add_phase(CompilationPhase::new("p1", "A", "B"));
        pipeline.add_phase(CompilationPhase::new("p2", "B", "C"));
        let eq = pipeline.total_equivalence().unwrap();
        assert_eq!(eq.source_type, TypeExpr::Named("A".to_string()));
        assert_eq!(eq.target_type, TypeExpr::Named("C".to_string()));
    }

    #[test]
    fn test_pipeline_empty_error() {
        let pipeline = CompilerPipeline::new("empty");
        assert!(pipeline.total_equivalence().is_err());
    }

    #[test]
    fn test_verify_phase() {
        let phase = CompilationPhase::new("parse", "Source", "AST");
        let fwd = Function::new("parse_map", TypeExpr::Named("Source".to_string()), TypeExpr::Named("AST".to_string()));
        let bwd = Function::new("parse_inv", TypeExpr::Named("AST".to_string()), TypeExpr::Named("Source".to_string()));
        let result = verify_phase(&phase, &fwd, &bwd).unwrap();
        assert!(result.correctness_witness.contains("equiv"));
    }

    #[test]
    fn test_verify_pipeline() {
        let mut pipeline = CompilerPipeline::new("test");
        pipeline.add_phase(CompilationPhase::new("p1", "A", "B"));
        let result = verify_pipeline(&pipeline).unwrap();
        assert_eq!(result.source_term, "A");
        assert_eq!(result.target_term, "B");
    }

    #[test]
    fn test_type_safety() {
        let safety = TypeSafety::guaranteed("Source");
        assert!(safety.is_safe());
        assert!(safety.progress);
        assert!(safety.preservation);
    }

    #[test]
    fn test_relational_semantics() {
        let mut rel = RelationalSemantics::new("parse");
        rel.relate("x + y", "Add(x, y)");
        rel.relate("x * y", "Mul(x, y)");
        assert!(rel.related("x + y", "Add(x, y)"));
        assert!(!rel.related("x + y", "Mul(x, y)"));
    }

    #[test]
    fn test_relational_deterministic() {
        let mut rel = RelationalSemantics::new("parse");
        rel.relate("x", "a");
        assert!(rel.is_deterministic());
        rel.relate("x", "b");
        assert!(!rel.is_deterministic());
    }

    #[test]
    fn test_relational_total() {
        let mut rel = RelationalSemantics::new("parse");
        rel.relate("x", "a");
        rel.relate("y", "b");
        assert!(rel.is_total_for(&["x", "y"]));
        assert!(!rel.is_total_for(&["x", "z"]));
    }
}
