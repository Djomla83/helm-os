//! A purpose-built ELF subject for the G2 real-launch integration test.
//!
//! It is not part of the HELM product and nothing in the interface runs it. It
//! exists so the test has a real, bounded, non-graphical Linux executable of
//! its own to admit and launch, without reaching for a system binary and
//! without needing a C toolchain at test time.
//!
//! It writes one marker to standard output, one to standard error, and ends.

fn main() {
    println!("helm-g2-fixture-stdout-marker");
    eprintln!("helm-g2-fixture-stderr-marker");
}
