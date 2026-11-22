//! Recommendations engine for modules, endpoints, files, and code smells.

pub mod perf;
pub mod quality;

pub use perf::{endpoint_recommendation, module_recommendation};
pub use quality::{file_recommendation, smell_recommendation};
