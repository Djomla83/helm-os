//! Independent-review driver: one bounded observation through the public
//! `helm-observe` API, with the capability descriptor numbers printed so a
//! syscall tracer can correlate every later operation with the actual target and
//! capability objects.
//!
//! Usage: `helm-observe-review-driver <root-dir> <plan-json>`
//!
//! It is review instrumentation, not product code, and lives outside the product
//! workspace. It reads only synthetic reviewer fixtures.

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
mod linux_driver {
    use std::os::fd::{AsRawFd as _, OwnedFd};

    use helm_observe::{
        authorize, observe, parse_plan, proc_fd_from_trusted_current_process, root_from_fd,
    };

    fn fail(stage: &str, message: &str) -> ! {
        println!("{{\"kind\":\"error\",\"stage\":\"{stage}\",\"message\":\"{message}\"}}");
        std::process::exit(2)
    }

    pub fn run() {
        let args: Vec<String> = std::env::args().collect();
        if args.len() != 3 {
            fail("args", "expected <root-dir> <plan-json>");
        }

        let root_file = match std::fs::File::open(&args[1]) {
            Ok(f) => f,
            Err(_) => fail("open_root", "cannot open root directory"),
        };
        let proc_file = match std::fs::File::open("/proc/self/fd") {
            Ok(f) => f,
            Err(_) => fail("open_procfs", "cannot open /proc/self/fd"),
        };
        let root_fd = root_file.as_raw_fd();
        let proc_fd = proc_file.as_raw_fd();
        println!("{{\"kind\":\"capabilities\",\"root_fd\":{root_fd},\"proc_fd\":{proc_fd}}}");

        let plan = match parse_plan(args[2].as_bytes()) {
            Ok(p) => p,
            Err(e) => fail("parse_plan", &e.to_string()),
        };
        let root = match root_from_fd("r", OwnedFd::from(root_file)) {
            Ok(r) => r,
            Err(e) => fail("root_from_fd", e.code().as_str()),
        };
        let reopen = match proc_fd_from_trusted_current_process(OwnedFd::from(proc_file)) {
            Ok(c) => c,
            Err(e) => fail("proc_fd", e.code().as_str()),
        };
        let scope = match authorize(plan, vec![root], reopen) {
            Ok(s) => s,
            Err(e) => fail("authorize", &e.to_string()),
        };

        let artifact = observe(&scope);
        let mut out = String::from("{\"kind\":\"observation\",\"targets\":[");
        for (i, t) in artifact.record().targets.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            out.push_str(&format!(
                "{{\"id\":\"{}\",\"code\":\"{}\"}}",
                t.target_id,
                t.outcome.code()
            ));
        }
        out.push_str("],\"artifact_sha256\":\"");
        out.push_str(&artifact.sha256().to_hex());
        out.push_str("\",\"artifact_bytes\":");
        out.push_str(&artifact.exact_bytes().len().to_string());
        out.push('}');
        println!("{out}");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn main() {
    linux_driver::run();
}

/// The observation backend exists only for the supported cohort, so there is
/// nothing honest for this driver to do anywhere else.
#[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
fn main() {
    println!("{{\"kind\":\"error\",\"stage\":\"platform\",\"message\":\"unsupported cohort\"}}");
    std::process::exit(3)
}
