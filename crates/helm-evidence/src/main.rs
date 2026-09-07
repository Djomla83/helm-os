#![forbid(unsafe_code)]
use std::{
    io::{self, Write},
    path::Path,
    process::ExitCode,
};

fn main() -> ExitCode {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if !(args.len() == 2 || (args.len() == 3 && args[2] == "--json")) || args[0] != "verify" {
        let _ = writeln!(
            io::stderr(),
            "Usage: helm-evidence verify <bundle-directory> [--json]"
        );
        return ExitCode::from(64);
    }
    let report = helm_evidence::verify(Path::new(&args[1]));
    let mut out = io::stdout().lock();
    let written = if args.len() == 3 {
        serde_json::to_writer_pretty(&mut out, &report)
            .map_err(io::Error::other)
            .and_then(|()| writeln!(out))
    } else {
        helm_evidence::write_human(&mut out, &report)
    };
    if written.is_err() {
        ExitCode::from(74)
    } else {
        ExitCode::from(report.verdict.exit_code())
    }
}
