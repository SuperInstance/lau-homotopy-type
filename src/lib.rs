//! # lau-homotopy-type
//!
//! A Homotopy Type Theory library bridging algebraic topology, compilers, and type theory.
//!
//! Provides cubical types, path types, identity types, transport, homotopies,
//! equivalences, higher inductive types, fundamental group computation, and
//! HoTT-based compiler verification primitives.

pub mod interval;
pub mod cubical;
pub mod path;
pub mod identity;
pub mod transport;
pub mod homotopy;
pub mod equivalence;
pub mod hots;
pub mod fundamental;
pub mod hott_compiler;

pub mod prelude {
    pub use crate::interval::*;
    pub use crate::cubical::*;
    pub use crate::path::*;
    pub use crate::identity::*;
    pub use crate::transport::*;
    pub use crate::homotopy::*;
    pub use crate::equivalence::*;
    pub use crate::hots::*;
    pub use crate::fundamental::*;
    pub use crate::hott_compiler::*;
}
