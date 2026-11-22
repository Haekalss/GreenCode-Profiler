//! Shared HTML rendering helpers: template substitution, badge styling,
//! and label/recommendation builders used by report generators.

pub use super::badges::*;
pub use super::labels::*;
pub use super::recommendations::*;
pub use super::rows::*;

/// Replace every `{{KEY}}` token in `template` with the matching value.
pub fn render(template: &str, vars: &[(&str, String)]) -> String {
    vars.iter()
        .fold(template.to_string(), |html, (key, value)| {
            html.replace(&format!("{{{{{}}}}}", key), value)
        })
}
