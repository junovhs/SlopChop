// src/graph/locality/mod.rs
//! Law of Locality enforcement for topological integrity.

pub mod classifier;
pub mod coupling;
pub mod distance;
pub mod types;
pub mod validator;

pub use classifier::{classify, ClassifierConfig};
pub use coupling::compute_coupling;
pub use distance::compute_distance;
pub use types::{Coupling, EdgeVerdict, LocalityEdge, NodeIdentity, PassReason};
pub use validator::{validate_edge, validate_graph, ValidationReport, ValidatorConfig};