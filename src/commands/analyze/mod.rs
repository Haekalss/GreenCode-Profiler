//! `greencode analyze`: static code-quality analysis (CCN, MI, NCLOC, smells).

pub mod output;
pub mod targets;

use crate::analyzer::CodeQualityAnalyzer;
use crate::cli::CliFlags;
use crate::reporter::ReportGenerator;

pub fn execute(flags: CliFlags) {
    let Some(targets) = targets::resolve_targets(&flags) else {
        return;
    };

    output::print_header(&targets, &flags);
    let report = build_analyzer(targets, &flags).run_analysis();
    output::print_overview(&report, &flags);
    let html_path = output::write_reports(&report, &flags);

    if !flags.no_open {
        ReportGenerator::open_in_browser(&html_path);
    }
}

fn build_analyzer(targets: Vec<String>, flags: &CliFlags) -> CodeQualityAnalyzer {
    if flags.has_filter() {
        CodeQualityAnalyzer::with_file_filters(
            targets,
            flags.filters.clone(),
            flags.excludes.clone(),
        )
    } else {
        CodeQualityAnalyzer::new(targets)
    }
}
