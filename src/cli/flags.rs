//! Command-line flags definitions and parser.

#[derive(Debug, Default)]
pub struct CliFlags {
    pub positional: Vec<String>,
    pub limit: usize,
    pub force: bool,
    pub no_open: bool,
    pub strict: bool,
    pub users: u64,
    pub requests_per_user: u64,
    pub cores: u32,
    pub window_sec: f64,
    pub filters: Vec<String>,
    pub excludes: Vec<String>,
    pub output: Option<String>,
}

impl CliFlags {
    pub fn has_filter(&self) -> bool {
        !self.filters.is_empty() || !self.excludes.is_empty()
    }
}

pub fn parse_flags(args: &[String]) -> CliFlags {
    let mut flags = CliFlags::default();
    let mut index = 0;
    while index < args.len() {
        let arg = args[index].as_str();
        let consumed = match arg {
            "--limit" | "-n" => assign(&mut flags.limit, args, index),
            "--filter" | "-F" => push(&mut flags.filters, args, index),
            "--exclude" | "-X" => push(&mut flags.excludes, args, index),
            "--output" | "-o" => assign_option(&mut flags.output, args, index),
            "--users" | "-u" => assign(&mut flags.users, args, index),
            "--requests-per-user" | "--rpu" => assign(&mut flags.requests_per_user, args, index),
            "--cores" | "-c" => assign(&mut flags.cores, args, index),
            "--window" | "-w" => assign(&mut flags.window_sec, args, index),
            "--strict" => {
                flags.strict = true;
                None
            }
            "--force" | "-f" => {
                flags.force = true;
                None
            }
            "--no-open" => {
                flags.no_open = true;
                None
            }
            _ => {
                record_positional(&mut flags, arg);
                None
            }
        };
        index += consumed.unwrap_or(1);
    }
    flags
}

fn next_arg(args: &[String], index: usize) -> Option<String> {
    args.get(index + 1).cloned()
}

fn assign<T: std::str::FromStr>(target: &mut T, args: &[String], index: usize) -> Option<usize> {
    let value = next_arg(args, index)?;
    if let Ok(parsed) = value.parse() {
        *target = parsed;
    }
    Some(2)
}

fn push(target: &mut Vec<String>, args: &[String], index: usize) -> Option<usize> {
    target.push(next_arg(args, index)?);
    Some(2)
}

fn assign_option(target: &mut Option<String>, args: &[String], index: usize) -> Option<usize> {
    *target = Some(next_arg(args, index)?);
    Some(2)
}

fn record_positional(flags: &mut CliFlags, arg: &str) {
    if !arg.starts_with('-') {
        flags.positional.push(arg.to_string());
        return;
    }
    println!("[GreenProfiler] Flag tidak dikenal diabaikan: {}", arg);
}
