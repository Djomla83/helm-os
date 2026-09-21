"""Repository-level structural checks on the helm-launch unsafe confinement.

These are independent of the crate's own Rust boundary tests and of `cargo`
entirely: they read the tree and the manifests and assert the same facts a
second time, in a second language. They compile nothing, run no launcher and
execute no process.

What they hold the repository to, after the owner authorised HELM-LAUNCH P3:

* `unsafe` exists in exactly one directory, `crates/helm-launch/src/backend`,
  and the single scoped `#![allow(unsafe_code)]` is at that module boundary;
* every other workspace crate still inherits the workspace lint table, whose
  `unsafe_code` stays `forbid`;
* there is exactly one raw syscall implementation, and no `extern` block;
* the child-window file keeps `#![no_implicit_prelude]` and names nothing that
  allocates, locks, formats, prints or panics;
* the test-only fault injection is gated on the feature **and** on
  `debug_assertions` at every `cfg` site;
* no public launch API exists: the backend module is private and re-exported
  nowhere.
"""

from pathlib import Path
import re
import unittest

ROOT = Path(__file__).resolve().parents[2]
CRATE = ROOT / "crates" / "helm-launch"
BACKEND = CRATE / "src" / "backend"

# The one directory the owner authorised for scoped `unsafe`.
BACKEND_FILES = {
    "child.rs",
    "injection.rs",
    "mod.rs",
    "spawn.rs",
    "syscall.rs",
    "tests.rs",
}

# The token, assembled so this file does not trip its own scan.
UNSAFE = "un" + "safe"

LINE_COMMENT = re.compile(r"//[^\n]*")
BLOCK_COMMENT = re.compile(r"/\*.*?\*/", re.S)
RAW_STRING = re.compile(r'r(#*)".*?"\1', re.S)
STRING = re.compile(r'"(?:[^"\\]|\\.)*"', re.S)


def code_only(text: str) -> str:
    """The source with comments and string literals removed."""
    text = BLOCK_COMMENT.sub(" ", text)
    text = LINE_COMMENT.sub(" ", text)
    text = RAW_STRING.sub('""', text)
    text = STRING.sub('""', text)
    return text


def words(text: str) -> set[str]:
    return set(re.findall(r"[A-Za-z_][A-Za-z0-9_]*", text))


def rust_sources(directory: Path) -> dict[str, str]:
    return {
        str(path.relative_to(CRATE)).replace("\\", "/"): path.read_text(encoding="utf-8")
        for path in sorted(directory.rglob("*.rs"))
    }


class UnsafeConfinementTests(unittest.TestCase):
    def setUp(self):
        self.sources = rust_sources(CRATE / "src")
        self.tests = rust_sources(CRATE / "tests")

    def test_backend_directory_holds_exactly_the_expected_files(self):
        self.assertTrue(BACKEND.is_dir(), "the authorised backend directory is missing")
        found = {path.name for path in BACKEND.iterdir() if path.is_file()}
        self.assertEqual(found, BACKEND_FILES)
        subdirectories = [path.name for path in BACKEND.iterdir() if path.is_dir()]
        self.assertEqual(subdirectories, [], "the backend gained a subdirectory")
        # P4 owns the public lifecycle module. It must exist, and it must be a
        # sibling of the backend rather than a member of it: the unsafe boundary
        # is this directory, and the public launch is deliberately outside it.
        self.assertTrue((CRATE / "src" / "launch.rs").is_file())
        self.assertNotIn("launch.rs", found)

    def test_unsafe_appears_as_code_only_under_the_backend(self):
        # The product tree. The Rust boundary test covers `tests/` too, with a
        # full Rust lexer; this scan deliberately stays on `src/`, because the
        # boundary tests themselves quote the very spellings they forbid.
        naming = {
            name for name, text in self.sources.items() if UNSAFE in words(code_only(text))
        }
        for name in naming:
            self.assertTrue(
                name.startswith("src/backend/"),
                f"{name} uses the {UNSAFE} token outside the authorised backend",
            )
        self.assertTrue(naming, "the scan found nothing, so it is not scanning")
        # The backend's own test module observes the backend; it does not
        # extend it.
        self.assertNotIn("src/backend/tests.rs", naming)

    def test_exactly_one_scoped_allow_at_the_backend_boundary(self):
        relaxing = []
        for name, text in self.sources.items():
            compact = "".join(code_only(text).split())
            if f"allow({UNSAFE}_code" in compact or f"expect({UNSAFE}_code" in compact:
                relaxing.append(name)
        self.assertEqual(relaxing, ["src/backend/mod.rs"])
        # No test source relaxes the lint. The check is on an attribute at the
        # start of a line, so a boundary test that quotes the spelling inside an
        # assertion is not mistaken for one that applies it.
        for name, text in self.tests.items():
            for line in text.splitlines():
                stripped = line.strip()
                self.assertFalse(
                    stripped.startswith(f"#![allow({UNSAFE}_code")
                    or stripped.startswith(f"#[allow({UNSAFE}_code"),
                    f"{name} relaxes the {UNSAFE} lint",
                )
        boundary = "".join(code_only(self.sources["src/backend/mod.rs"]).split())
        self.assertIn(f"#![allow({UNSAFE}_code)]", boundary)
        root = "".join(code_only(self.sources["src/lib.rs"]).split())
        self.assertIn(f"#![deny({UNSAFE}_code,{UNSAFE}_op_in_{UNSAFE}_fn)]", root)
        self.assertNotIn("forbid(", root)

    def test_the_workspace_still_forbids_unsafe_for_every_other_member(self):
        workspace = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
        self.assertIn(f'{UNSAFE}_code = "forbid"', workspace)
        members = re.findall(r'^\s*"(crates/[^"]+)",?\s*$', workspace, re.M)
        self.assertIn("crates/helm-launch", members)
        for member in members:
            manifest = (ROOT / member / "Cargo.toml").read_text(encoding="utf-8")
            declared = [
                line.strip()
                for line in manifest.splitlines()
                if not line.lstrip().startswith("#")
            ]
            if member == "crates/helm-launch":
                self.assertNotIn("workspace = true", declared)
                self.assertIn(f'{UNSAFE}_code = "deny"', declared)
                self.assertNotIn(f'{UNSAFE}_code = "forbid"', declared)
            else:
                self.assertIn("workspace = true", declared, member)

    def test_exactly_one_raw_syscall_implementation_and_no_extern_block(self):
        with_assembly = [
            name for name, text in self.sources.items() if "asm" in words(code_only(text))
        ]
        self.assertEqual(with_assembly, ["src/backend/syscall.rs"])
        shim = code_only(self.sources["src/backend/syscall.rs"])
        self.assertEqual(shim.count("asm!("), 1)
        for name, text in self.sources.items():
            found = words(code_only(text))
            for forbidden in ("extern", "global_asm", "naked_asm", "transmute"):
                self.assertNotIn(forbidden, found, f"{name} uses {forbidden}")

    def test_the_child_window_keeps_its_closed_vocabulary(self):
        child = self.sources["src/backend/child.rs"]
        self.assertIn("#![no_implicit_prelude]", child)
        found = words(code_only(child))
        for forbidden in (
            "std",
            "alloc",
            "rustix",
            "libc",
            "Vec",
            "String",
            "Box",
            "Mutex",
            "format",
            "println",
            "panic",
            "unwrap",
            "thread",
            "Duration",
            "fork",
            "kill",
            "Command",
        ):
            self.assertNotIn(forbidden, found, f"the child path names {forbidden}")
        self.assertIn("syscall", found)
        self.assertIn("ChildPlan", found)

    def test_the_only_process_group_signal_is_the_guarded_p4_sweep(self):
        """P4 activates one guarded group sweep; nothing else may signal a group.

        Before P4 no group signal existed at all. The claim is now bounded
        rather than absent: exactly one call site, in the launch slice, and
        still none anywhere under the unsafe boundary.
        """
        sites = {}
        for name, text in self.sources.items():
            code = code_only(text)
            found = words(code)
            for forbidden in ("killpg", "tgkill", "tkill"):
                self.assertNotIn(forbidden, found, f"{name} names {forbidden}")
            if name.startswith("src/backend/") and name != "src/backend/tests.rs":
                self.assertNotIn("kill", found, f"{name} names a pid-valued kill")
                self.assertNotIn(
                    "kill_process_group",
                    found,
                    f"{name} sweeps a process group; that belongs to src/launch.rs",
                )
            count = "".join(code.split()).count("kill_process_group(")
            if count:
                sites[name] = count
        self.assertEqual(
            sites,
            {"src/launch.rs": 1},
            "the guarded group sweep must have exactly one call site",
        )
        launch = "".join(code_only(self.sources["src/launch.rs"]).split())
        self.assertIn("kill_process_group(group,Signal::KILL)", launch)
        # It is reached only from the model's own action, after the
        # non-consuming probe the model demands.
        self.assertIn("Action::SweepGroup=>self.sweep_group()", launch)
        self.assertIn("WaitIdOptions::NOWAIT", launch)

    def test_fault_injection_is_gated_on_the_feature_and_on_debug_assertions(self):
        gate = 'all(feature="test-fault-injection",debug_assertions)'
        for name, text in self.sources.items():
            compact = "".join(text.split())
            conditions = compact.count('feature="test-fault-injection"')
            self.assertEqual(
                conditions,
                compact.count(gate),
                f"{name} has a fault-injection condition without debug_assertions",
            )
        manifest = (CRATE / "Cargo.toml").read_text(encoding="utf-8")
        self.assertIn("test-fault-injection = []", manifest)
        self.assertNotIn("default = [", manifest)

    def test_the_public_launch_api_is_exactly_the_authorised_pair(self):
        """P4 publishes `launch` and `LaunchOutcome`, cohort-gated, and nothing else.

        The backend stays private: no public module, no re-export, and no
        backend type in the public surface.
        """
        root = "".join(code_only(self.sources["src/lib.rs"]).split())
        self.assertEqual(root.count("modbackend;"), 1)
        self.assertNotIn("pubmodbackend", root)
        self.assertNotIn("pubusebackend", root)
        self.assertNotIn("pubmodlaunch", root)
        for name in ("SpawnedChild", "PreparedLaunch", "ChildHandle", "MinimalLaunch"):
            self.assertNotIn(f"pubuse{name}", root)
        # Exactly one public launch re-export, and it is cohort-gated.
        self.assertEqual(root.count("pubuselaunch::{LaunchOutcome,launch};"), 1)
        gate = '#[cfg(all(target_os="linux",target_arch="x86_64"))]'
        raw = "".join(
            line
            for line in self.sources["src/lib.rs"].splitlines()
            if not line.lstrip().startswith("//")
        )
        raw = "".join(raw.split())
        for gated in ("modlaunch;", "pubuselaunch::{LaunchOutcome,launch};"):
            at = raw.find(gated)
            self.assertNotEqual(at, -1, f"{gated} is missing")
            self.assertTrue(
                raw[:at].endswith(gate),
                f"{gated} is not immediately preceded by the cohort gate",
            )
        for name, text in self.sources.items():
            if not name.startswith("src/backend/"):
                continue
            for line in code_only(text).splitlines():
                stripped = line.strip()
                self.assertFalse(
                    stripped.startswith("pub ") and not stripped.startswith("pub("),
                    f"{name} declares a bare public item: {stripped}",
                )

    def test_the_crate_declares_only_the_two_vetted_cohort_dependencies(self):
        manifest = (CRATE / "Cargo.toml").read_text(encoding="utf-8")
        self.assertIn('rustix = { version = "=1.1.4"', manifest)
        self.assertIn('libc = { version = "=0.2.189"', manifest)
        # P4 adds exactly one rustix feature, `event`, for the observation
        # loop. `time` stays off: monotonic deadlines use std::time::Instant.
        self.assertEqual(
            manifest.count(
                'features = ["std", "fs", "process", "pipe", "event"]'
            ),
            2,
        )
        self.assertNotIn('"time"', manifest)
        self.assertNotIn('"runtime"', manifest)
        self.assertNotIn("linux-raw-sys", manifest)
        self.assertFalse((CRATE / "build.rs").exists(), "a build script appeared")
        lockfile = (ROOT / "Cargo.lock").read_text(encoding="utf-8")
        self.assertEqual(lockfile.count('name = "libc"'), 1)
        self.assertEqual(lockfile.count('name = "rustix"'), 1)
        self.assertIn('version = "0.2.189"', lockfile)


if __name__ == "__main__":
    unittest.main()
