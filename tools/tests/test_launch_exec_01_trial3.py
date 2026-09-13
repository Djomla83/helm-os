"""Trial #3 correction candidate: regressions for the bounded Trial #2 findings.

Scope, and nothing else: X2b/X2c/X4, T1, S4, M2, E4, E6/E6c, O6/O7 and R3, plus
the liveness revalidation support P1/P2/P4 need. N3 stays conditional.

The reviewed candidate is now the Trial #3 freeze (trial-003): FROZEN and NOT_RUN,
with no Trial #3 D-7. The Trial3Freeze tests bind the working tree directly to
the Trial #3 SOURCE-HASHES.json; the superseded NOT_FROZEN record is provenance
only. Nothing here poses a
LAUNCH-EXEC case, runs launcher_spike, or executes a helper or a generated ELF.
Observations are fabricated in-process. The POSIX-only tests use scratch FIFOs
and symlinks, Python stand-ins and isolated fork/waitid probes that are not
LAUNCH-EXEC cases. The compiler-backed test compiles helper_report.c into a
scratch directory and reads the result as bytes only. No test constructs a
driver.Authorisation.

Trial #2 is immutable. Its replay here runs the modules frozen at ba41a3f, in a
separate interpreter, over the preserved journal, and must still produce
59 PASS / 6 FAIL / 6 INVALID / 1 BLOCKED and MECHANISM_REJECTED.
"""
import ast
import base64
import contextlib
import errno
import hashlib
import json
import os
import pathlib
import re
import shutil
import stat
import select
import subprocess
import sys
import tempfile
import unittest

HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE.parents[1]
EXP = ROOT / "docs" / "experiments" / "launch-exec-01"
for _path in (str(EXP), str(HERE)):
    if _path not in sys.path:
        sys.path.insert(0, _path)

import checker              # noqa: E402
import driver               # noqa: E402
import evidence             # noqa: E402
import frozen_cases as fc   # noqa: E402
import harness              # noqa: E402
import make_fixtures        # noqa: E402
import observations as ob   # noqa: E402
import oracles              # noqa: E402
import run_launch_exec_01 as runner   # noqa: E402
import test_launch_exec_01_delta as D   # noqa: E402  fabrication helpers only

PASS, FAIL, INVALID, BLOCKED = (checker.PASS, checker.FAIL, checker.INVALID,
                                checker.BLOCKED)
POSIX = unittest.skipUnless(os.name == "posix",
                            "FIFOs, symlinks and POSIX modes; Linux CI and WSL run it")
LINUX = unittest.skipUnless(sys.platform.startswith("linux"),
                            "reads /proc; Linux CI and WSL run it")
DRIVER_SOURCE = (EXP / "driver.py").read_text(encoding="utf-8")
DRIVER_TREE = ast.parse(DRIVER_SOURCE)
TRIAL2_DIR = (ROOT / "docs" / "experiments" / "evidence"
              / "LAUNCH-EXEC-01-TRIAL-002-2026-09-11")
MANIFEST = json.loads((EXP / "SOURCE-HASHES.json").read_text(encoding="utf-8"))
CANDIDATE = json.loads((EXP / "TRIAL-3-CORRECTION-CANDIDATE.json").read_text(
    encoding="utf-8"))
TRIAL2_FREEZE = "ba41a3f12be411058ed50e78bcd1c7e22afb7ae4"


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def git_blob(data):
    return hashlib.sha1(b"blob %d\0" % len(data) + data).hexdigest()


def function(name, tree=DRIVER_TREE):
    for node in ast.walk(tree):
        if isinstance(node, ast.FunctionDef) and node.name == name:
            return node
    raise KeyError(name)


def source_of(name, tree=DRIVER_TREE):
    return ast.unparse(function(name, tree))


def score(plan, obs):
    record = driver.evaluate(plan, obs)
    return checker.score_case(plan.case, record)[0], record


def full_report(marker="helper_report", declared_exit=fc.S4_EXIT_CODE):
    rep = D.report(marker)
    rep["requested"] = {"stdout": 0, "stderr": 0, "exit": declared_exit}
    return rep


def capture(data, completeness="CompleteAtEof"):
    """A stream block with its retained prefix, as launcher_spike.c prints it."""
    return {"bytes_drained": len(data), "drained_sha256": sha256(data),
            "completeness": completeness, "capture_prefix_length": len(data),
            "capture_prefix_truncated": False,
            "capture_prefix_base64": base64.b64encode(data).decode("ascii")}


class SafetyBarrier(unittest.TestCase):
    def test_this_suite_constructs_no_authorisation(self):
        text = pathlib.Path(__file__).read_text(encoding="utf-8")
        self.assertNotIn("Authorisation" + "(True)", text)
        self.assertNotIn("driver." + "observe(", text)
        self.assertNotIn("driver." + "pose(", text)
        self.assertNotIn("_launch_" + "and_observe(", text)


# ======================================================= Trial #2 is immutable
TRIAL2_EVIDENCE_SHA256 = {
    "preflight.json":
        "4669c8490811ef5a3bd0fff812c877ee46e7d7c0a830670456e0bc0b5019a892",
    "build-identity.json":
        "cbf015a64b7b5bf0034fb6e638bbfdaaccbc9110a95236ea58f06efac45bb21b",
    "journal.jsonl":
        "451e471d68ab8bb6509eca3974ab54bcfa08b89ecf37bd99f7541dfd664a6558",
    "evidence.json":
        "c105223789a5f8576035fa06cd348b71bda78b34132c13e0785fb75e37fdfb9f",
    "runner-stdout.json":
        "c105223789a5f8576035fa06cd348b71bda78b34132c13e0785fb75e37fdfb9f",
}

# Committed blobs of the Trial #2 records this correction must not touch.
TRIAL2_RECORD_BLOBS = {
    "docs/implementation/HELM-LAUNCH-EXEC-01-TRIAL-002-RESULT-REVIEW.md":
        "f6593f5cb29c9407b4b07376a337b75cab61ace3",
    "docs/implementation/HELM-LAUNCH-EXEC-01-TRIAL-002-POSTMORTEM-DIAGNOSTICS.md":
        "17cafc675695edb50fccf073cc0086dada2547f2",
    "docs/experiments/evidence/LAUNCH-EXEC-01-TRIAL-002-2026-09-11/PROVENANCE.md":
        "f9021abe6429c36089c4f90e8ce6e36f608c8b73",
}

# The Trial #2 manifest is no longer the working SOURCE-HASHES.json: the Trial #3
# freeze replaced it. It stays addressable at the Trial #2 freeze, byte for byte.
MANIFEST_PATH = "docs/experiments/launch-exec-01/SOURCE-HASHES.json"
TRIAL2_MANIFEST_BLOB = "6f000fac9a48625d3c9def18e16ae7ce1b61efa8"
TRIAL2_MANIFEST_SHA256 = "616dc6b340c5453a013259554b10fd997a90c990da3395449ba3497f186c9a94"

# The frozen modules the historical replay needs, from the Trial #2 freeze.
REPLAY_MODULES = ("checker.py", "frozen_cases.py", "journal.py", "evidence.py",
                  "observations.py")

REPLAY = r'''
import json, sys
sys.path.insert(0, sys.argv[1])
import checker, frozen_cases, journal
records, torn = journal.read(sys.argv[2])
state = journal.replay(records)
report = checker.report(journal.recovered_records(state))
first = next(r for r in records if r.get("kind") == "case_pose_started")
sys.stdout.write(json.dumps({
    "torn": torn,
    "project": journal.project_status(state),
    "aggregate": report["aggregate"], "counts": report["counts"],
    "detail": report["detail"], "statuses": report["statuses"],
    "trial_ends": sum(1 for r in records if r.get("kind") == "trial_end"),
    "first_pose_started": [first["case"], first["n"]],
    "pose_started": len(state["pose_started"]),
    "entered": len(state["entered"]), "completed": len(state["completed"]),
    "predictions": {n: frozen_cases.BY_NAME[n]["predict"] for n in ("T1", "S4")},
}, sort_keys=True))
'''


def git_show(revision, path):
    git = shutil.which("git")
    if git is None:
        raise unittest.SkipTest("git is needed to read the frozen Trial #2 sources")
    root = str(ROOT).replace("\\", "/")
    proc = subprocess.run([git, "-C", str(ROOT), "-c", "safe.directory=" + root,
                           "show", "%s:%s" % (revision, path)],
                          capture_output=True, timeout=60)
    if proc.returncode != 0:
        raise AssertionError("git show %s:%s failed: %s" % (
            revision, path, proc.stderr.decode("utf-8", "replace")))
    return proc.stdout


class Trial2StaysImmutable(unittest.TestCase):
    def test_the_preserved_evidence_is_byte_exact(self):
        for name, digest in TRIAL2_EVIDENCE_SHA256.items():
            self.assertEqual(sha256((TRIAL2_DIR / name).read_bytes()), digest, name)
        self.assertEqual(sorted(p.name for p in TRIAL2_DIR.iterdir()),
                         sorted(list(TRIAL2_EVIDENCE_SHA256) + ["PROVENANCE.md"]))

    def test_the_trial_2_records_and_manifest_are_the_committed_blobs(self):
        for path, blob in TRIAL2_RECORD_BLOBS.items():
            self.assertEqual(git_blob((ROOT / path).read_bytes()), blob, path)
        historical = git_show(TRIAL2_FREEZE, MANIFEST_PATH)
        self.assertEqual(git_blob(historical), TRIAL2_MANIFEST_BLOB)
        self.assertEqual(sha256(historical), TRIAL2_MANIFEST_SHA256)
        for record in (CANDIDATE["trial_2"], MANIFEST["trial_2"]):
            self.assertEqual((record["manifest_git_blob"], record["manifest_sha256"]),
                             (TRIAL2_MANIFEST_BLOB, TRIAL2_MANIFEST_SHA256))
        self.assertNotEqual(sha256((EXP / "SOURCE-HASHES.json").read_bytes()),
                            TRIAL2_MANIFEST_SHA256)

    def test_the_historical_replay_with_the_ba41a3f_checker_is_unchanged(self):
        historical = json.loads(git_show(TRIAL2_FREEZE, MANIFEST_PATH))
        tmp = pathlib.Path(tempfile.mkdtemp(prefix="trial2-replay-"))
        try:
            for name in REPLAY_MODULES:
                data = git_show(TRIAL2_FREEZE, "docs/experiments/launch-exec-01/"
                                + name)
                # The replay runs exactly the bytes Trial #2 executed.
                self.assertEqual(sha256(data), historical["sha256"][name], name)
                (tmp / name).write_bytes(data)
            proc = subprocess.run(
                [sys.executable, "-I", "-B", "-c", REPLAY, str(tmp),
                 str(TRIAL2_DIR / "journal.jsonl")],
                capture_output=True, timeout=120, cwd=str(tmp))
            self.assertEqual(proc.returncode, 0, proc.stderr.decode("utf-8",
                                                                   "replace"))
            got = json.loads(proc.stdout.decode("utf-8"))
        finally:
            shutil.rmtree(tmp, ignore_errors=True)
        self.assertEqual(got["aggregate"], "MECHANISM_REJECTED")
        self.assertEqual(got["counts"], {"PASS": 59, "FAIL": 6, "INVALID": 6,
                                         "BLOCKED": 1})
        self.assertEqual(got["detail"]["failing_cases"],
                         ["X2b", "X2c", "X4", "T1", "S4", "M2"])
        published = json.loads((TRIAL2_DIR / "evidence.json").read_text(
            encoding="utf-8"))
        self.assertEqual(got["aggregate"], published["aggregate"])
        self.assertEqual(got["counts"], published["counts"])
        self.assertEqual(got["detail"], published["detail"])
        self.assertEqual(len(got["statuses"]), 72)
        for name, entry in published["cases"].items():
            self.assertEqual(got["statuses"][name],
                             {"status": entry["status"], "reason": entry["reason"]},
                             name)
        # D-7 consumed at E1's durable pose start; exactly one valid trial.
        self.assertIs(got["torn"], False)
        self.assertEqual(got["project"]["trial_status"], "TRIAL_COMPLETED")
        self.assertIs(got["project"]["d7_consumed"], True)
        self.assertEqual(got["first_pose_started"], ["E1", 4])
        self.assertEqual((got["trial_ends"], got["entered"], got["completed"],
                          got["pose_started"]), (1, 72, 72, 68))
        # The frozen predictions the candidate corrects, read from the freeze.
        for case, pair in CANDIDATE["contract_delta"]["predictions"].items():
            self.assertEqual(got["predictions"][case], pair["trial_2"], case)


class CandidateRecord(unittest.TestCase):
    """The NOT_FROZEN correction record, superseded by the Trial #3 freeze.

    Provenance only: it grants nothing, is not hashed, and active freeze
    validation never reads it (see Trial3Freeze)."""

    def test_it_is_superseded_provenance_and_grants_nothing(self):
        self.assertEqual(CANDIDATE["record"], "TRIAL_3_CORRECTION_CANDIDATE")
        self.assertEqual(CANDIDATE["state"], "SUPERSEDED_BY_TRIAL_003_FREEZE")
        self.assertIs(CANDIDATE["trial_3_authorised"], False)
        self.assertIsNone(CANDIDATE["trial_3_d7"])
        self.assertIs(CANDIDATE["trial_3_executed"], False)
        self.assertIs(CANDIDATE["build"]["build_7_binds_candidate"], False)
        trial2 = CANDIDATE["trial_2"]
        self.assertEqual(trial2["freeze"], TRIAL2_FREEZE)
        self.assertEqual((trial2["aggregate"], trial2["d7"],
                          trial2["valid_trial_count"], trial2["rerun"]),
                         ("MECHANISM_REJECTED", "CONSUMED", 1, "FORBIDDEN"))
        self.assertEqual(trial2["counts"], {"PASS": 59, "FAIL": 6, "INVALID": 6,
                                            "BLOCKED": 1})
        self.assertNotIn("freeze", CANDIDATE["record"].lower())
        self.assertNotIn("sha256", CANDIDATE)
        superseded = CANDIDATE["superseded_by"]
        self.assertEqual((superseded["trial"], superseded["manifest"],
                          superseded["freeze_state"], superseded["authority"]),
                         ("trial-003", "SOURCE-HASHES.json", "FROZEN", "none"))
        self.assertEqual(superseded["manifest_sha256"],
                         sha256((EXP / "SOURCE-HASHES.json").read_bytes()))
        self.assertEqual(MANIFEST["correction_candidate_record"]["state"],
                         CANDIDATE["state"])

    def test_its_historical_drift_is_the_difference_between_the_two_manifests(self):
        # Committed data against committed data: the Trial #2 manifest at
        # ba41a3f and the Trial #3 manifest. Never the files under test.
        historical = json.loads(git_show(TRIAL2_FREEZE, MANIFEST_PATH))
        drift = sorted(name for name, digest in historical["sha256"].items()
                       if MANIFEST["sha256"][name] != digest)
        freeze_step = ["README.md", "run_launch_exec_01.py"]
        self.assertEqual(sorted(set(drift) - set(freeze_step)),
                         CANDIDATE["sources_differing_from_trial_2_freeze"])
        self.assertLessEqual(set(freeze_step), set(drift))
        documents = sorted(path for path, digest
                           in historical["definition_sha256"].items()
                           if MANIFEST["definition_sha256"][path] != digest)
        self.assertEqual(documents,
                         CANDIDATE["definition_files_differing_from_trial_2_freeze"])
        self.assertEqual(sorted(n for n in drift if n.endswith(".c")),
                         CANDIDATE["c_sources_changed"])

    def test_the_runner_never_reads_the_superseded_record(self):
        runner_source = (EXP / "run_launch_exec_01.py").read_text(encoding="utf-8")
        self.assertNotIn("TRIAL-3-CORRECTION-CANDIDATE", runner_source)
        self.assertIn('SOURCE_HASHES = HERE / "SOURCE-HASHES.json"', runner_source)

    def test_the_scope_is_the_bounded_finding_set(self):
        scope = CANDIDATE["scope"]
        self.assertEqual(sorted(scope["corrected_findings"]),
                         sorted(["X2b", "X2c", "X4", "T1", "S4", "M2", "E4", "E6",
                                 "E6c", "O6", "O7", "R3"]))
        self.assertEqual(scope["revalidation_support"], ["P1", "P2", "P4"])
        summary = fc.summary()
        self.assertEqual(scope["membership_unchanged"],
                         {k: summary[k] for k in ("conditional", "mandatory",
                                                  "recorded", "total")})

    def test_the_declared_contract_delta_is_the_code(self):
        delta = CANDIDATE["contract_delta"]
        for case, pair in delta["predictions"].items():
            self.assertEqual(fc.BY_NAME[case]["predict"], pair["candidate"], case)
        for rule, entry in delta["rules_added"].items():
            self.assertIn(rule, ob.RULES)
            self.assertEqual(sorted(p.case for p in driver._PLAN_LIST
                                    if p.rule == rule), entry["cases"])
        s4 = driver.CASE_PLANS["S4"]
        declared = delta["plans_changed"]["S4"]
        self.assertEqual(list(s4.channels), declared["channels"])
        self.assertEqual(list(s4.helper_args), declared["helper_args"])
        self.assertEqual(list(s4.spike_flags), declared["spike_flags"])
        self.assertEqual(s4.posed_when, declared["posed_when"])
        self.assertEqual(s4.expected_marker, declared["expected_marker"])
        self.assertEqual(list(s4.assertions), declared["assertions"])
        self.assertEqual(s4.rule, declared["rule"])
        self.assertEqual({name: int(mode, 8) for name, mode
                          in delta["fixture_modes"].items()},
                         make_fixtures.FIXTURE_MODES)
        self.assertEqual(set(delta["receipt_fields_added"]["wait_si_code"]),
                         set(ob.SPIKE_WAIT_SI_CODES))
        self.assertEqual(delta["fixture_signal_diagnostic"]["line"].split("<")[0],
                         ob.FIXTURE_SIGNAL_FAILED.decode("ascii"))


# ============================================ X2b / X2c / X4: fixture modes
class FixtureModes(unittest.TestCase):
    EXECUTED = ("script_fixture.sh", "unloadable_in_cohort.elf")
    NEVER_EXECUTED = ("helper_foreign.elf", "magic_only.bin")

    def test_every_fixture_declares_exactly_one_mode(self):
        self.assertEqual(set(make_fixtures.FIXTURE_MODES),
                         set(make_fixtures.FIXTURES))
        for name, mode in make_fixtures.FIXTURE_MODES.items():
            self.assertIsInstance(mode, int, name)
            self.assertEqual(mode & ~0o777, 0, name)

    def test_executed_fixtures_are_executable_and_the_others_are_not(self):
        for name in self.EXECUTED:
            self.assertEqual(make_fixtures.FIXTURE_MODES[name], 0o755, name)
        for name in self.NEVER_EXECUTED:
            self.assertEqual(make_fixtures.FIXTURE_MODES[name], 0o644, name)
            self.assertEqual(make_fixtures.FIXTURE_MODES[name] & 0o111, 0, name)
        users = {p.case: p.setup for p in driver._PLAN_LIST
                 if p.setup in ("script_fixture", "unloadable_in_cohort")}
        self.assertEqual(sorted(users), ["X2", "X2b", "X2c", "X4"])
        for case in ("X2b", "X2c", "X4"):
            self.assertIn("--bypass-admission", driver.CASE_PLANS[case].spike_flags)

    def test_the_bytes_are_trial_2s_whatever_the_mode(self):
        identity = json.loads((TRIAL2_DIR / "build-identity.json").read_text(
            encoding="utf-8"))["artefacts"]
        for name, digest in make_fixtures.digests().items():
            self.assertEqual(identity[name]["sha256"], digest, name)
            self.assertEqual(identity[name]["size"],
                             len(make_fixtures.FIXTURES[name]), name)

    def test_modes_are_applied_per_fixture_not_by_a_broad_chmod(self):
        tree = ast.parse((EXP / "make_fixtures.py").read_text(encoding="utf-8"))
        chmods = [n for n in ast.walk(function("write", tree))
                  if isinstance(n, ast.Call) and ast.unparse(n.func) == "os.chmod"]
        self.assertEqual(len(chmods), 1)
        self.assertEqual(ast.unparse(chmods[0].args[1]), "FIXTURE_MODES[name]")
        self.assertNotIn("0o755", source_of("write", tree))

    @POSIX
    def test_write_applies_each_declared_mode_under_any_umask(self):
        for umask in (0o000, 0o022, 0o077, 0o777):
            tmp = pathlib.Path(tempfile.mkdtemp(prefix="fixture-modes-"))
            # The directory exists first: the umask under test is the one the
            # fixture FILES are created under.
            (tmp / "build").mkdir()
            old = os.umask(umask)
            try:
                make_fixtures.write(str(tmp / "build"))
            finally:
                os.umask(old)
            try:
                for name, data in make_fixtures.FIXTURES.items():
                    path = tmp / "build" / name
                    self.assertEqual(stat.S_IMODE(path.stat().st_mode),
                                     make_fixtures.FIXTURE_MODES[name],
                                     (oct(umask), name))
                    self.assertEqual(path.read_bytes(), data, name)
            finally:
                shutil.rmtree(tmp, ignore_errors=True)

    @POSIX
    def test_a_rewrite_restores_the_declared_mode(self):
        tmp = pathlib.Path(tempfile.mkdtemp(prefix="fixture-rewrite-"))
        try:
            make_fixtures.write(str(tmp))
            os.chmod(tmp / "script_fixture.sh", 0o600)
            os.chmod(tmp / "magic_only.bin", 0o755)
            make_fixtures.write(str(tmp))
            self.assertEqual(stat.S_IMODE((tmp / "script_fixture.sh").stat().st_mode),
                             0o755)
            self.assertEqual(stat.S_IMODE((tmp / "magic_only.bin").stat().st_mode),
                             0o644)
        finally:
            shutil.rmtree(tmp, ignore_errors=True)

    @POSIX
    def test_a_fixture_without_its_declared_mode_does_not_pose(self):
        tmp = pathlib.Path(tempfile.mkdtemp(prefix="fixture-posing-"))
        try:
            make_fixtures.write(str(tmp))
            ctx = driver.TrialContext(build=str(tmp), work=str(tmp), preflight={},
                                      freeze={}, sanitiser=evidence.Sanitiser())
            for case in ("E8", "X2", "X2b", "X2c", "X4"):
                plan = driver.CASE_PLANS[case]
                built = driver.SETUPS[plan.setup](ctx, plan)
                self.assertNotIn("not_posed", built, case)
                self.assertTrue(os.access(built["exec_path"], os.R_OK), case)
            os.chmod(tmp / "script_fixture.sh", 0o644)
            os.chmod(tmp / "unloadable_in_cohort.elf", 0o644)
            for case in ("X2b", "X2c", "X4"):
                plan = driver.CASE_PLANS[case]
                built = driver.SETUPS[plan.setup](ctx, plan)
                self.assertIn("declared mode 0o755", built["not_posed"], case)
                record = dict(built, launch_returned=None)
                self.assertEqual(checker.score_case(case, record)[0], INVALID)
        finally:
            shutil.rmtree(tmp, ignore_errors=True)


# ================================================== T1: the qualified timeout
class T1QualifiedTimeout(unittest.TestCase):
    plan = driver.CASE_PLANS["T1"]
    TOKEN = "TimedOut:KilledByLauncher:SIGTERM"

    def t1(self, **over):
        fields = dict(process_disposition="TimedOut",
                      timeout_disposition="KilledByLauncher", term_signal=15,
                      exit_code=-1, wait_si_code="CLD_KILLED",
                      launcher_signal_issued=True)
        fields.update(over)
        obs = D.obs_for(self.plan, spike=D.receipt(**fields), rep=full_report(),
                        elapsed_ms=2006)
        return score(self.plan, obs)

    def test_the_prediction_is_the_exact_qualified_token(self):
        self.assertEqual(fc.BY_NAME["T1"]["predict"], self.TOKEN)
        self.assertEqual(fc.BY_NAME["T1"]["cls"], fc.MANDATORY)
        self.assertIsNone(fc.BY_NAME["T1"]["safe"])
        # The checker compares a single prediction for equality, never a prefix.
        self.assertIn('outcome != spec["predict"]',
                      (EXP / "checker.py").read_text(encoding="utf-8"))

    def test_the_intended_sigterm_sequence_passes(self):
        status, record = self.t1()
        self.assertEqual(status, PASS, record)
        self.assertEqual(record["outcome"], self.TOKEN)

    def test_every_other_termination_path_fails(self):
        for over in ({"term_signal": 9},
                     {"timeout_disposition": "ExitedDuringGrace", "exit_code": 9,
                      "term_signal": -1, "wait_si_code": "CLD_EXITED"},
                     {"timeout_disposition": "TerminationFailed", "term_signal": -1,
                      "wait_si_code": ""},
                     {"timeout_disposition": "", "term_signal": -1},
                     {"process_disposition": "Signaled", "timeout_disposition": ""},
                     {"process_disposition": "Exited", "timeout_disposition": "",
                      "exit_code": 0, "term_signal": -1,
                      "wait_si_code": "CLD_EXITED"}):
            status, record = self.t1(**over)
            self.assertEqual(status, FAIL, (over, record))
            self.assertNotEqual(record["outcome"], self.TOKEN)

    def test_the_trial_2_observation_now_matches_and_the_bare_token_fails(self):
        # Trial #2 observed exactly this token; the candidate expects it.
        self.assertEqual(self.t1(wait_si_code=None)[0], PASS)
        status, record = self.t1(timeout_disposition="")
        self.assertEqual((status, record["outcome"]), (FAIL, "TimedOut"))

    def test_a_classification_contradicting_the_sub_disposition_is_invalid(self):
        status, record = self.t1(wait_si_code="CLD_EXITED")
        self.assertEqual(status, INVALID, record)

    def test_the_construction_takes_the_sigterm_path(self):
        self.assertEqual(self.plan.helper_args, ("--sleep-ms", "60000"))
        self.assertEqual((self.plan.timeout_ms, self.plan.grace_ms), (2000, 2000))
        self.assertEqual(self.plan.rule, "process_disposition")
        # The bare TimedOut is unreachable: every timeout branch of the
        # launcher assigns a non-empty sub-disposition.
        spike = (EXP / "launcher_spike.c").read_text(encoding="utf-8")
        branch = spike.split("else if (timed_out) {", 1)[1].split(
            'else if (info.si_code == CLD_EXITED) { disposition = "Exited"; }', 1)[0]
        for sub in ("TerminationFailed", "ExitedDuringGrace", "KilledByLauncher"):
            self.assertIn('timeout_disposition = "%s"' % sub, branch)
        self.assertEqual(branch.count("timeout_disposition = "), 3)


# ======================================== S4: conservative exec evidence
class S4ConservativeExecEvidence(unittest.TestCase):
    plan = driver.CASE_PLANS["S4"]

    def s4(self, spike=None, rep=None, **over):
        spike = D.receipt(exit_code=fc.S4_EXIT_CODE, wait_si_code="CLD_EXITED") \
            if spike is None else spike
        rep = full_report() if rep is None else rep
        return score(self.plan, D.obs_for(self.plan, spike=spike, rep=rep, **over))

    def test_the_case_and_plan(self):
        spec = fc.BY_NAME["S4"]
        self.assertEqual((spec["cls"], spec["predict"], spec["instant_reject"]),
                         (fc.MANDATORY, "ExecStatusIndeterminate", True))
        self.assertEqual(self.plan.rule, "launcher_receipt_claim")
        self.assertEqual(self.plan.channels, (driver.CH_RECEIPT, driver.CH_REPORT))
        self.assertEqual(self.plan.helper_args, ("--exit", str(fc.S4_EXIT_CODE)))
        self.assertEqual(self.plan.spike_flags,
                         ("--post-fork-delay-ms", str(fc.EXEC_RACE_DELAY_MS)))
        # R-M1: no posed check reads the report; it is a result-side fact.
        self.assertIsNone(self.plan.posed_when)
        self.assertEqual(self.plan.assertions,
                         ("executed_marker", "helper_exit_corroborated"))
        self.assertFalse(any("--exit-immediately" in p.helper_args
                             for p in driver._PLAN_LIST))

    def test_a_conservative_claim_with_independent_evidence_passes(self):
        status, record = self.s4()
        self.assertEqual(status, PASS, record)
        self.assertEqual(record["outcome"], "ExecStatusIndeterminate")
        for name in self.plan.assertions:
            self.assertEqual(record["assertions"][name]["result"],
                             ob.ASSERTION_HOLDS, name)
        self.assertNotIn("posed_check", record["posing_evidence"])
        self.assertEqual(record["posing_evidence"]["measured"]["report_states"],
                         {ob.REPORT_COMPLETE: 1})

    def test_the_launcher_claim_never_reads_the_independent_evidence(self):
        receipt = D.receipt(exit_code=fc.S4_EXIT_CODE, wait_si_code="CLD_EXITED")
        with_report = D.obs_for(self.plan, spike=receipt, rep=full_report())
        without = D.obs_for(self.plan, spike=receipt, rep=None)
        payload = D.obs_for(self.plan, spike=receipt, payload_is_recipe=True)
        for obs in (with_report, without, payload):
            self.assertEqual(ob.derive("launcher_receipt_claim", obs)[0],
                             "ExecStatusIndeterminate")
        # A receipt asserting its own exec success changes nothing.
        boastful = dict(receipt, exec_confirmed=True, exec_reached=True)
        self.assertEqual(ob.derive("launcher_receipt_claim", D.obs_for(
            self.plan, spike=boastful, rep=full_report()))[0],
                         "ExecStatusIndeterminate")
        body = ast.unparse(function("rule_launcher_receipt_claim",
                                    ast.parse((EXP / "observations.py").read_text(
                                        encoding="utf-8"))))
        self.assertIn("exec_confirmation(spike, None)", body)
        self.assertIn("report=None", body)
        launcher = (EXP / "launcher_spike.c").read_text(encoding="utf-8")
        accepted = launcher.split('printf("{\\"admission\\":\\"accepted\\",', 1)[1]
        self.assertNotIn("exec_confirmed", accepted.split("rejected_acquisition_arm")[0])

    def test_missing_or_uninterpretable_independent_evidence_is_invalid(self):
        for state in (ob.REPORT_ABSENT, ob.REPORT_TRUNCATED, ob.REPORT_MALFORMED,
                      ob.REPORT_STREAM_INCOMPLETE):
            obs = D.obs_for(self.plan, spike=D.receipt(exit_code=7), rep=None,
                            report_state=state)
            status, record = score(self.plan, obs)
            self.assertEqual(status, INVALID, state)
            self.assertIn("an executed-identity assertion could not be observed",
                          record["not_posed"])
        for over in ({"spike": None}, {"exit_code": "7"},
                     {"wait_si_code": "CLD_KILLED"}):
            if "spike" in over:
                obs = D.obs_for(self.plan, rep=full_report())
                obs["spike"] = None
                obs["repeat_observations"] = [obs]
                status = score(self.plan, obs)[0]
            else:
                status = self.s4(spike=D.receipt(**dict(
                    {"exit_code": 7, "wait_si_code": "CLD_EXITED"}, **over)))[0]
            self.assertEqual(status, INVALID, over)
        self.assertEqual(self.s4(rep=D.report())[0], INVALID)   # no declared exit

    def test_contradictory_evidence_fails(self):
        rows = (
            (dict(spike=D.receipt(exit_code=3, wait_si_code="CLD_EXITED")),
             "helper_exit_contradicted"),
            (dict(rep=full_report(declared_exit=3)), "helper_exit_contradicted"),
            (dict(rep=full_report(marker="helper_alt")), "executed_body_mismatch"),
            (dict(spike=D.receipt(process_disposition="Signaled", term_signal=9,
                                  exit_code=-1, wait_si_code="CLD_KILLED")),
             "helper_exit_contradicted"),
            (dict(spike=D.receipt(process_disposition="ExecStatusIndeterminate",
                                  exit_code=-1)), "helper_exit_contradicted"),
            (dict(spike=D.receipt(process_disposition="ExecFailed",
                                  exec_failed_stage="EXEC", exec_failed_errno=13,
                                  exit_code=127)), None),
            (dict(spike=D.receipt(process_disposition="ExitStatusUnobservable",
                                  exit_code=-1, wait_errno=10)), None),
        )
        for over, token in rows:
            status, record = self.s4(**over)
            self.assertEqual(status, FAIL, (over, record))
            if token is not None:
                self.assertEqual(record["outcome"], token, over)
        self.assertEqual(self.s4(launch_returned=False)[0], FAIL)

    def test_s4_and_s5_differ_by_their_evidence_not_their_token(self):
        s5 = driver.CASE_PLANS["S5"]
        self.assertEqual(fc.BY_NAME["S5"]["predict"], fc.BY_NAME["S4"]["predict"])
        self.assertEqual(s5.posed_when, "no_helper_report")
        self.assertEqual(s5.rule, "process_disposition")
        receipt = D.receipt(exit_code=0)
        self.assertEqual(driver.POSED_CHECKS["no_helper_report"](
            D.obs_for(s5, spike=receipt)), True)
        self.assertNotIn("helper_report_complete", driver.POSED_CHECKS)


# ============================================ M2: the direct-child clone3
THREAD_FLAGS = ("CLONE_VM|CLONE_FS|CLONE_FILES|CLONE_SIGHAND|CLONE_THREAD|"
                "CLONE_SYSVSEM|CLONE_SETTLS|CLONE_PARENT_SETTID|CLONE_CHILD_CLEARTID")
CHILD_WINDOW = (
    "dup2(5, 0) = 0", "dup2(6, 1) = 1", "dup2(7, 2) = 2",
    "fcntl(0, F_SETFD, 0) = 0", "fchdir(8) = 0", "close_range(3, 7, 0) = 0",
    "setpgid(0, 0) = 0", "rt_sigprocmask(SIG_SETMASK, [], NULL, 8) = 0",
    "rt_sigaction(SIGHUP, {sa_handler=SIG_DFL}, NULL, 8) = 0",
    "prctl(PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) = 0",
    'execveat(3, "", ["helper_report"], [], AT_EMPTY_PATH) = 0',
)
CHILD_CALLS = [line.split("(", 1)[0] for line in CHILD_WINDOW]
THREAD_LIFE = (
    "rseq(0x7f00, 32, 0, 0x53053053) = 0", "set_robust_list(0x7f10, 24) = 0",
    "rt_sigprocmask(SIG_SETMASK, [], NULL, 8) = 0",
    "mmap(NULL, 134217728, PROT_NONE, MAP_PRIVATE|MAP_ANONYMOUS, -1, 0) = 0x7f",
    "madvise(0x7f20, 8368128, MADV_DONTNEED) = 0",
)


def clone3_line(task, flags, ret, pidfd=None):
    out = " => {pidfd=[%d]}" % pidfd if pidfd is not None else ""
    signal_name = "0" if "CLONE_THREAD" in flags else "SIGCHLD"
    return ("%d clone3({flags=%s, pidfd=0x7ffd0000, exit_signal=%s}%s, 88) = %s"
            % (task, flags, signal_name, out, ret))


def threaded_trace(child=222, threads=(301, 302, 303), process_clone=None,
                   extra_child=()):
    lines = [clone3_line(111, THREAD_FLAGS, tid) for tid in threads]
    lines += ["%d %s" % (tid, THREAD_LIFE[0]) for tid in threads]
    lines.append(process_clone if process_clone is not None
                 else clone3_line(111, "CLONE_PIDFD", child, pidfd=4))
    calls = list(CHILD_WINDOW)
    for position, call in enumerate(extra_child):
        calls.insert(4, call)
    for index, call in enumerate(calls):
        lines.append("%d %s" % (child, call))
        tid = threads[index % len(threads)]
        lines.append("%d %s" % (tid, THREAD_LIFE[1 + index % (len(THREAD_LIFE) - 1)]))
    lines += ["%d exit(0) = ?" % tid for tid in threads]
    lines.append("111 waitid(P_PIDFD, 4, {si_signo=SIGCHLD, si_code=CLD_EXITED, "
                 "si_pid=%d, si_status=0}, WEXITED, NULL) = 0" % child)
    return "\n".join(lines) + "\n"


def control_trace(child=223):
    lines = [clone3_line(111, "CLONE_PIDFD", child, pidfd=4)]
    lines += ["%d %s" % (child, call) for call in CHILD_WINDOW]
    return "\n".join(lines) + "\n"


class M2DirectChildWindow(unittest.TestCase):
    def test_thread_clones_before_the_process_clone_are_not_the_child(self):
        window = ob.parse_strace_child_window(threaded_trace())
        self.assertEqual(window["child_pid"], 222)
        self.assertEqual(window["child_syscalls"], CHILD_CALLS)
        self.assertEqual(window["stage_sequence"],
                         ["DUP2", "CLEAR_CLOEXEC", "CHDIR", "CLOSE_RANGE", "SETPGID",
                          "SIGMASK", "SIGACTION", "NO_NEW_PRIVS", "EXEC"])
        self.assertIs(window["integrity_ok"], True)
        for thread_call in ("rseq", "set_robust_list", "madvise", "exit"):
            self.assertNotIn(thread_call, window["child_syscalls"])
        records = ob.join_trace_fragments(threaded_trace())["records"]
        index, why = ob.select_direct_child_clone(records)
        self.assertIsNone(why)
        self.assertEqual(records[index]["text"].count("CLONE_PIDFD"), 1)
        first_clone = next(i for i, r in enumerate(records) if r["call"] == "clone3")
        self.assertNotEqual(index, first_clone, "Trial #2 anchored on this thread")

    def test_the_published_acquisition_facts_describe_the_same_clone(self):
        facts = ob.parse_pidfd_acquisition(threaded_trace())
        self.assertEqual(facts["clone3_call_count"], 4)
        self.assertEqual(facts["clone3_flags"], ["CLONE_PIDFD"])
        self.assertIs(facts["clone_pidfd_flag"], True)
        self.assertEqual((facts["clone3_return"], facts["pidfd_from_clone3"]),
                         (222, 4))
        self.assertEqual(ob.normalise_acquisition(facts)["direct_child_pidfd"],
                         "DIRECT_CHILD_PIDFD")
        # M3's requirement still counts every clone3 and refuses several.
        self.assertIsNone(ob.derive("pidfd_acquired_atomically",
                                    {"acquisition": facts})[0])
        lone_thread = ob.parse_pidfd_acquisition(clone3_line(111, THREAD_FLAGS, 301))
        self.assertEqual((lone_thread["clone3_call_count"],
                          lone_thread["clone3_return"]), (1, 301))

    def test_the_arms_compare_equal_and_a_real_difference_still_fails(self):
        threaded = ob.parse_strace_child_window(threaded_trace())
        control = ob.parse_strace_child_window(control_trace())
        obs = {"trace": threaded,
               "single_threaded_child_syscalls": control["child_syscalls"]}
        self.assertEqual(ob.derive("identical_to_single_threaded_arm", obs)[0],
                         "identical_to_single_threaded_arm")
        differing = ob.parse_strace_child_window(threaded_trace(
            extra_child=("mmap(NULL, 4096, PROT_READ, MAP_PRIVATE, -1, 0) = 0x7f",)))
        obs["trace"] = differing
        self.assertEqual(ob.derive("identical_to_single_threaded_arm", obs)[0],
                         "differs_from_single_threaded_arm")

    def test_a_thread_created_after_the_process_clone_changes_nothing(self):
        text = threaded_trace().replace(
            "111 waitid(", clone3_line(111, THREAD_FLAGS, 399) + "\n111 waitid(")
        self.assertEqual(ob.parse_strace_child_window(text)["child_syscalls"],
                         CHILD_CALLS)

    def test_no_qualifying_process_clone_fails_safe(self):
        only_threads = "\n".join([clone3_line(111, THREAD_FLAGS, 301),
                                  clone3_line(111, THREAD_FLAGS, 302),
                                  "301 rseq(0x7f00, 32, 0, 0) = 0"]) + "\n"
        self.assertIsNone(ob.parse_strace_child_window(only_threads))
        lone_thread = clone3_line(111, THREAD_FLAGS, 301) + "\n301 exit(0) = ?\n"
        self.assertIsNone(ob.parse_strace_child_window(lone_thread))
        self.assertEqual(ob.select_direct_child_clone([])[0], None)

    def test_malformed_or_ambiguous_candidates_fail_safe(self):
        good = clone3_line(111, "CLONE_PIDFD", 222, pidfd=4)
        variants = {
            "undecodable flags among several": "111 clone3(0x7ffd1234, 88) = 222",
            "no CLONE_PIDFD among thread clones":
                clone3_line(111, "0", 222).replace("flags=0", "flags=CLONE_PARENT"),
            "unbounded record": good.rsplit(", 88) = 222", 1)[0],
            "return not observed": good.replace("= 222", "= ?"),
            "observed error": good.replace("= 222", "= -1 EPERM (Operation not "
                                                   "permitted)"),
        }
        for label, line in variants.items():
            self.assertIsNone(ob.parse_strace_child_window(
                threaded_trace(process_clone=line)), label)
        two = threaded_trace().replace(
            good, good + "\n" + clone3_line(111, "CLONE_PIDFD", 333, pidfd=5))
        self.assertIsNone(ob.parse_strace_child_window(two), "two process clones")
        unfinished = threaded_trace(process_clone=good.replace(
            " => {pidfd=[4]}, 88) = 222", " <unfinished ...>"))
        self.assertIsNone(ob.parse_strace_child_window(unfinished), "never resumed")

    def test_a_split_process_clone_is_rejoined_before_it_is_selected(self):
        split = ("111 clone3({flags=CLONE_PIDFD, pidfd=0x7ffd0000, "
                 "exit_signal=SIGCHLD} <unfinished ...>\n"
                 "301 set_robust_list(0x7f10, 24) = 0\n"
                 "111 <... clone3 resumed> => {pidfd=[4]}, 88) = 222")
        window = ob.parse_strace_child_window(threaded_trace(process_clone=split))
        self.assertEqual(window["child_syscalls"], CHILD_CALLS)
        self.assertIs(window["integrity_ok"], True)

    def test_numeric_flags_decode_only_the_two_selection_bits(self):
        names = ob.clone3_flag_names
        self.assertEqual(names("clone3({flags=0x1000, exit_signal=SIGCHLD}, 88)"),
                         frozenset({"CLONE_PIDFD"}))
        self.assertEqual(names("clone3({flags=CLONE_VM|0x10000}, 88)"),
                         frozenset({"CLONE_VM", "CLONE_THREAD"}))
        self.assertIsNone(names("clone3({flags=0x1g}, 88)"))
        self.assertIsNone(names("clone3({exit_signal=SIGCHLD}, 88)"))

    def test_the_single_threaded_shape_is_unchanged(self):
        window = ob.parse_strace_child_window(control_trace())
        self.assertEqual(window["child_syscalls"], CHILD_CALLS)
        # A lone clone3 without CLONE_PIDFD still names its child, so M3 keeps
        # its own decisive FAIL for the missing flag instead of an INVALID.
        text = control_trace().replace("flags=CLONE_PIDFD", "flags=CLONE_PARENT")
        self.assertEqual(ob.parse_strace_child_window(text)["child_syscalls"],
                         CHILD_CALLS)
        self.assertEqual(ob.derive("pidfd_acquired_atomically",
                                   {"acquisition": ob.parse_pidfd_acquisition(text)})[0],
                         "pidfd_not_acquired_atomically")

    def test_structural_validation_still_comes_first(self):
        text = threaded_trace() + "302 <... futex resumed>) = 0\n"
        window = ob.parse_strace_child_window(text)
        self.assertIs(window["integrity_ok"], False)
        self.assertFalse(checker.valid_trace_record(window)[0])


# ===================================================== E4: resolved target
@POSIX
class E4ResolvedSymlinkTarget(unittest.TestCase):
    BODY = b"\x7fELF" + b"REPORT" * 64
    ALT = b"\x7fELF" + b"ALTERN" * 64

    def setUp(self):
        self.root = pathlib.Path(tempfile.mkdtemp(prefix="e4-"))
        self.cwd = os.getcwd()
        os.chdir(self.root)
        # Trial #2's shape: the runner passed a RELATIVE, nested build directory.
        self.build = pathlib.Path("target") / "launch-exec-01"
        self.build.mkdir(parents=True)
        (self.build / "helper_report").write_bytes(self.BODY)
        (self.build / "helper_alt").write_bytes(self.ALT)
        self.identity = harness.build_identity(str(self.build))
        self.ctx = driver.TrialContext(build=str(self.build), work=str(self.build),
                                       preflight={}, freeze={},
                                       sanitiser=evidence.Sanitiser(),
                                       build_identity=self.identity)
        self.plan = driver.CASE_PLANS["E4"]

    def tearDown(self):
        os.chdir(self.cwd)
        shutil.rmtree(self.root, ignore_errors=True)

    def test_the_link_targets_the_canonical_build_artefact(self):
        built = driver.SETUPS["symlink_retarget"](self.ctx, self.plan)
        target = os.readlink(built["exec_path"])
        self.assertTrue(os.path.isabs(target))
        self.assertEqual(target, os.path.realpath(self.build / "helper_report"))
        self.assertEqual(target.count("launch-exec-01"), 1, "self-nested")
        self.assertEqual(pathlib.Path(built["exec_path"]).read_bytes(), self.BODY)
        action, replacement = built["post_pin"]
        self.assertEqual(action, "retarget_symlink")
        self.assertTrue(os.path.isabs(replacement))
        self.assertEqual(pathlib.Path(replacement).read_bytes(), self.ALT)

    def test_the_binding_still_names_the_built_artefact(self):
        built = driver.SETUPS["symlink_retarget"](self.ctx, self.plan)
        fact = driver.bind_build_identity(self.ctx, self.plan, built)
        self.assertIs(fact["bound"], True, fact)
        self.assertEqual((fact["classification"], fact["base_artefact"]),
                         (driver.SYMLINK_TO_BASE, "helper_report"))
        self.assertEqual(fact["object_sha256"], self.identity["helper_report"]["sha256"])

    def test_the_link_means_the_same_file_from_another_directory(self):
        built = driver.SETUPS["symlink_retarget"](self.ctx, self.plan)
        link = self.root / built["exec_path"]
        elsewhere = self.root / "a" / "b" / "c"
        elsewhere.mkdir(parents=True)
        os.chdir(elsewhere)
        self.assertEqual(link.read_bytes(), self.BODY)
        self.assertEqual(os.path.realpath(link),
                         os.path.realpath(self.root / self.build / "helper_report"))

    def test_trial_2s_relative_target_dangles_and_now_says_so(self):
        link = self.build / "E4_link"
        link.symlink_to(self.build / "helper_report")     # the frozen construction
        self.assertFalse(link.exists())
        fact = driver.bind_build_identity(self.ctx, self.plan,
                                          {"exec_path": str(link)})
        self.assertIs(fact["bound"], False)
        self.assertIn("dangling", fact["detail"])
        self.assertIsNone(fact["object_sha256"])

    def test_an_identical_copy_elsewhere_is_not_the_artefact(self):
        copy = self.root / "elsewhere" / "helper_report"
        copy.parent.mkdir()
        copy.write_bytes(self.BODY)
        link = self.build / "E4_link"
        link.symlink_to(copy)
        fact = driver.bind_build_identity(self.ctx, self.plan,
                                          {"exec_path": str(link)})
        self.assertIs(fact["bound"], False)
        self.assertIn("does not resolve to the build artefact", fact["detail"])

    def test_the_retarget_lands_only_through_the_link(self):
        built = driver.SETUPS["symlink_retarget"](self.ctx, self.plan)
        fact = driver.POST_PIN_ACTIONS["retarget_symlink"](
            self.ctx, self.plan, built, built["post_pin"][1])
        self.assertIs(fact["landed"], True, fact)
        self.assertEqual(fact["path_body_sha256_after"], sha256(self.ALT))
        again = driver.SETUPS["symlink_retarget"](self.ctx, self.plan)
        relative = os.path.relpath(again["post_pin"][1])
        fact = driver.POST_PIN_ACTIONS["retarget_symlink"](
            self.ctx, self.plan, again, relative)
        self.assertIs(fact["landed"], False, fact)


# ===================================== E6 / E6c: one contiguous marker object
REGION = (driver.MARKER_GUARD_LO + b"helper_report".ljust(16, b"\0")
          + driver.MARKER_GUARD_HI)


def elf_symbols(data):
    """``{name: (file_offset, size)}`` for defined symbols, read as data only."""
    import struct
    if data[:4] != b"\x7fELF" or data[4] != 2 or data[5] != 1:
        raise ValueError("not an ELF64 little-endian image")
    shoff = struct.unpack_from("<Q", data, 0x28)[0]
    shentsize, shnum = struct.unpack_from("<HH", data, 0x3A)
    sections = [struct.unpack_from("<IIQQQQIIQQ", data, shoff + i * shentsize)
                for i in range(shnum)]
    symtab = next(s for s in sections if s[1] == 2)             # SHT_SYMTAB
    strtab = sections[symtab[6]]
    out = {}
    for j in range(symtab[5] // symtab[9]):
        name, _, _, shndx, value, size = struct.unpack_from(
            "<IBBHQQ", data, symtab[4] + j * symtab[9])
        if shndx == 0 or shndx >= len(sections) or sections[shndx][1] == 8:
            continue
        start = strtab[4] + name
        label = data[start:data.index(b"\0", start)].decode("ascii", "replace")
        out[label] = (sections[shndx][4] + value - sections[shndx][3], size)
    return out


def scratch_compiler():
    return (os.environ.get("HELM_LAUNCH_EXEC_01_TEST_CC") or shutil.which("cc")
            or shutil.which("gcc"))


class E6ContiguousMarkerRegion(unittest.TestCase):
    PAD = b"\0" * 100

    def image(self, *parts):
        return b"\x7fELF" + self.PAD + b"".join(parts) + self.PAD

    def test_exactly_one_region_is_located_at_its_marker_bytes(self):
        data = self.image(REGION)
        offset, why = driver.locate_marker_region(data)
        self.assertIsNone(why)
        self.assertEqual(offset, 4 + len(self.PAD) + len(driver.MARKER_GUARD_LO))
        self.assertEqual(data[offset:offset + driver.MARKER_REGION_BYTES],
                         b"helper_report\0\0\0")
        at_end = b"x" + REGION
        self.assertEqual(driver.locate_marker_region(at_end)[0], 10)

    def test_trial_2s_reversed_padded_layout_is_zero_regions(self):
        # nm at ba41a3f: KRAM-MLEH, 15 padding bytes, the marker, HELM-MARK.
        layout = (driver.MARKER_GUARD_HI + b"\0" * 15 + b"helper_report\0\0\0"
                  + driver.MARKER_GUARD_LO)
        offset, why = driver.locate_marker_region(self.image(layout))
        self.assertIsNone(offset)
        self.assertIn("no complete guarded region (1 low guard(s), 1 high guard(s))",
                      why)

    def test_zero_or_several_regions_or_a_stray_guard_do_not_pose(self):
        rows = ((self.image(), "no complete guarded region (0 low"),
                (self.image(REGION, self.PAD, REGION),
                 "2 complete guarded regions"),
                (self.image(REGION, self.PAD, driver.MARKER_GUARD_LO),
                 "outside the one guarded region"),
                (self.image(driver.MARKER_GUARD_HI, self.PAD, REGION),
                 "outside the one guarded region"),
                (self.image(driver.MARKER_GUARD_LO, b"\0" * 15,
                            driver.MARKER_GUARD_HI), "no complete guarded region"))
        for data, reason in rows:
            offset, why = driver.locate_marker_region(data)
            self.assertIsNone(offset, reason)
            self.assertIn(reason, why)

    def test_e6_and_e6c_share_one_posing_primitive(self):
        for setup in ("_setup_mutate_marker", "_setup_shared_writable_mapping"):
            self.assertIn("_marker_region_of(info)", source_of(setup), setup)
        callers = sorted({fn.name for fn in ast.walk(DRIVER_TREE)
                          if isinstance(fn, ast.FunctionDef)
                          for node in ast.walk(fn) if isinstance(node, ast.Call)
                          and ast.unparse(node.func) == "locate_marker_region"})
        self.assertEqual(callers, ["_marker_region_of", "find_marker_region"])

    def test_helper_report_declares_the_region_as_one_object(self):
        source = (EXP / "helper_report.c").read_text(encoding="utf-8")
        for gone in ("g_marker_guard_lo", "g_marker_guard_hi", "g_marker[16]"):
            self.assertNotIn(gone, source)
        declarations = re.findall(
            r"volatile unsigned char g_marker_region\[[^\]]*\]\s*=\s*\{(.*?)\};",
            source, re.S)
        self.assertEqual(len(declarations), 1)
        items = [item.strip() for item in declarations[0].split(",") if item.strip()]
        data = bytes(ord(item[1]) if item.startswith("'") else int(item, 0)
                     for item in items)
        self.assertEqual(data, REGION)
        self.assertIn("g_marker_region[MARKER_GUARD_BYTES + mi]", source)
        self.assertIn("_Static_assert(sizeof(g_marker_region)", source)

    @POSIX
    def test_the_setups_pose_from_the_region_or_refuse_with_the_reason(self):
        tmp = pathlib.Path(tempfile.mkdtemp(prefix="e6-setup-"))
        try:
            (tmp / "helper_report").write_bytes(self.image(REGION))
            ctx = driver.TrialContext(build=str(tmp), work=str(tmp), preflight={},
                                      freeze={}, sanitiser=evidence.Sanitiser(),
                                      build_identity=harness.build_identity(str(tmp)))
            offset = driver.locate_marker_region(self.image(REGION))[0]
            e6 = driver.SETUPS["mutate_marker_in_place"](ctx, driver.CASE_PLANS["E6"])
            self.assertEqual(e6["post_pin"], ("pwrite_marker", offset))
            self.assertEqual(e6["mutated_marker"], b"MUTATED" + b"\0" * 9)
            e6c = driver.SETUPS["shared_writable_mapping"](ctx,
                                                           driver.CASE_PLANS["E6c"])
            self.assertEqual(e6c["post_pin"], ("mmap_write", offset))
            (tmp / "helper_report").write_bytes(self.image(REGION, REGION))
            ctx.build_identity = harness.build_identity(str(tmp))
            for setup, case in (("mutate_marker_in_place", "E6"),
                                ("shared_writable_mapping", "E6c")):
                built = driver.SETUPS[setup](ctx, driver.CASE_PLANS[case])
                self.assertIn("exactly one is required", built["not_posed"], case)
        finally:
            shutil.rmtree(tmp, ignore_errors=True)

    @POSIX
    def test_the_mutation_touches_exactly_the_marker_bytes(self):
        tmp = pathlib.Path(tempfile.mkdtemp(prefix="e6-mutate-"))
        try:
            original = self.image(REGION)
            path = tmp / "E6_helper_report"
            path.write_bytes(original)
            offset = driver.locate_marker_region(original)[0]
            built = {"exec_path": str(path),
                     "mutated_marker": driver._marker_bytes(ob.E6_MUTATED_MARKER)}
            fact = driver.POST_PIN_ACTIONS["pwrite_marker"](
                None, driver.CASE_PLANS["E6"], built, offset)
            self.assertIs(fact["landed"], True, fact)
            self.assertIs(fact["outside_region_unchanged"], True)
            mutated = path.read_bytes()
            changed = [i for i in range(len(original)) if original[i] != mutated[i]]
            self.assertTrue(changed)
            self.assertGreaterEqual(min(changed), offset)
            self.assertLess(max(changed), offset + driver.MARKER_REGION_BYTES)
            self.assertEqual(driver.locate_marker_region(mutated)[0], offset)
        finally:
            shutil.rmtree(tmp, ignore_errors=True)

    def test_a_compiled_scratch_helper_has_one_region_in_one_symbol(self):
        cc = scratch_compiler()
        if cc is None:
            self.skipTest("no C compiler; the compile-only workflow and WSL run it")
        tmp = pathlib.Path(tempfile.mkdtemp(prefix="e6-compile-"))
        try:
            image = tmp / "helper_report-inspection-only"
            proc = subprocess.run([cc, "-O2", "-Wall", "-Wextra", "-static", "-o",
                                   str(image), str(EXP / "helper_report.c")],
                                  capture_output=True, timeout=600)
            self.assertEqual(proc.returncode, 0, proc.stderr.decode("utf-8", "replace"))
            self.assertNotIn(b"warning:", proc.stderr)
            data = image.read_bytes()        # read as data; never executed
        finally:
            shutil.rmtree(tmp, ignore_errors=True)
        offset, why = driver.locate_marker_region(data)
        self.assertIsNone(why)
        self.assertEqual(data.count(driver.MARKER_GUARD_LO), 1)
        self.assertEqual(data.count(driver.MARKER_GUARD_HI), 1)
        start = offset - len(driver.MARKER_GUARD_LO)
        self.assertEqual(data[start:start + len(REGION)], REGION)
        symbols = elf_symbols(data)
        self.assertEqual(symbols["g_marker_region"], (start, len(REGION)))
        for gone in ("g_marker_guard_lo", "g_marker_guard_hi", "g_marker"):
            self.assertNotIn(gone, symbols)
        marker = driver._marker_bytes(ob.E6_MUTATED_MARKER)
        mutated = data[:offset] + marker + data[offset + len(marker):]
        self.assertTrue(driver._outside_region_unchanged(data, mutated, offset,
                                                         len(marker)))
        self.assertEqual(driver.locate_marker_region(mutated)[0], offset)


# ==================================== O6 / O7 and P1 / P2 / P4: FIFO paths
FORKLIKE = r'''
import os, sys
fifo, workdir = sys.argv[1], sys.argv[2]
os.chdir(workdir)          # the launcher's child fchdirs before execveat
try:
    f = os.open(fifo, os.O_WRONLY | os.O_CLOEXEC)
except OSError as exc:
    os.write(2, b"HELM-LAUNCH-EXEC-01-FIXTURE-SIGNAL-FAILED:open:%d\n" % exc.errno)
    sys.exit(0)
os.write(f, b"L")
os.close(f)
'''


@POSIX
class HelperFacingFifoPaths(unittest.TestCase):
    def setUp(self):
        self.root = pathlib.Path(tempfile.mkdtemp(prefix="fifo-paths-"))
        self.cwd = os.getcwd()
        os.chdir(self.root)
        self.build = pathlib.Path("target") / "launch-exec-01"   # relative, as in Trial #2
        self.build.mkdir(parents=True)
        self.work = self.root / "work-dir-capability"
        self.work.mkdir()
        self.ctx = driver.TrialContext(build=str(self.build), work=str(self.build),
                                       preflight={}, freeze={},
                                       sanitiser=evidence.Sanitiser())
        self.armed = []

    def tearDown(self):
        for built in self.armed:
            driver.disarm_fixture_signal(built)
        os.chdir(self.cwd)
        shutil.rmtree(self.root, ignore_errors=True)

    def setup(self, case):
        plan = driver.CASE_PLANS[case]
        built = driver.SETUPS[plan.setup](self.ctx, plan)
        self.armed.append(built)
        return plan, built

    def standin(self, path):
        return subprocess.run([sys.executable, "-I", "-c", FORKLIKE, path,
                               str(self.work)], capture_output=True, timeout=60)

    def test_every_fork_helper_fixture_hands_an_absolute_case_private_path(self):
        users = sorted(p.case for p in driver._PLAN_LIST
                       if p.setup in driver.FORK_HELPER_SETUPS)
        self.assertEqual(users, ["O6", "O7", "P1", "P2", "P3", "P4", "T5"])
        canonical = pathlib.Path(os.path.realpath(self.build))
        for case in users:
            plan, built = self.setup(case)
            path = driver._helper_fifo_argument(built)
            self.assertTrue(os.path.isabs(path), case)
            self.assertEqual(pathlib.Path(path).parent, canonical, case)
            self.assertTrue(pathlib.Path(path).name.startswith(case + "."), case)
            key = ("fixture_signal_fifo" if plan.setup == "fork_helper_prearmed"
                   else "liveness_fifo")
            self.assertEqual(built[key], path, case)

    def test_the_path_names_the_same_fifo_after_a_change_of_directory(self):
        for case in ("O6", "O7", "P1", "P2", "P4"):
            plan, built = self.setup(case)
            if plan.setup == "fork_helper_prearmed":
                facts, why = driver._arm_fixture_signal(built, ())
                self.assertIsNone(why, case)
                self.assertIs(facts["helper_path_absolute"], True, case)
                self.assertIs(facts["helper_path_is_armed_fifo"], True, case)
            path = driver._helper_fifo_argument(built)
            before = os.stat(path)
            os.chdir(self.work)
            try:
                after = os.stat(path)
            finally:
                os.chdir(self.root)
            self.assertTrue(stat.S_ISFIFO(after.st_mode), case)
            self.assertEqual((after.st_dev, after.st_ino),
                             (before.st_dev, before.st_ino), case)
            self.assertEqual(stat.S_IMODE(after.st_mode), 0o600, case)

    def test_a_relative_or_foreign_helper_path_is_refused_before_launch(self):
        _, built = self.setup("O7")
        built["extra_helper_args"] = ("--liveness-fifo",
                                      os.path.relpath(built["fixture_signal_fifo"]))
        facts, why = driver._arm_fixture_signal(built, ())
        self.assertIsNone(facts)
        self.assertIn("not an absolute path to the armed FIFO", why)
        other = self.root / "someone-elses.fifo"
        os.mkfifo(str(other), 0o600)
        built["extra_helper_args"] = ("--liveness-fifo", str(other))
        facts, why = driver._arm_fixture_signal(built, ())
        self.assertIsNone(facts)
        self.assertIn("not an absolute path to the armed FIFO", why)

    def test_a_stand_in_that_changes_directory_still_signals(self):
        _, built = self.setup("O6")
        _, why = driver._arm_fixture_signal(built, ())
        self.assertIsNone(why)
        proc = self.standin(driver._helper_fifo_argument(built))
        self.assertEqual(proc.stderr, b"")
        self.assertIs(driver._read_fixture_signal(built, timeout_ms=2000), True)

    def test_trial_2s_relative_path_breaks_after_the_change_and_is_diagnosed(self):
        _, built = self.setup("O7")
        _, why = driver._arm_fixture_signal(built, ())
        self.assertIsNone(why)
        proc = self.standin(os.path.relpath(built["fixture_signal_fifo"]))
        self.assertIs(driver._read_fixture_signal(built, timeout_ms=300), False)
        diagnostic = ob.fixture_signal_diagnostic(capture(proc.stderr))
        self.assertEqual(diagnostic, {"reported": True, "failed_step": "open",
                                      "errno": "ENOENT",
                                      "errno_number": errno.ENOENT})

    def test_the_p_series_rendezvous_works_across_the_change(self):
        plan, built = self.setup("P2")
        writer = subprocess.Popen([sys.executable, "-I", "-c", FORKLIKE,
                                   driver._helper_fifo_argument(built),
                                   str(self.work)], stderr=subprocess.PIPE)
        try:
            alive = driver._descendant_alive(built, plan, timeout_ms=10000)
        finally:
            _, err = writer.communicate(timeout=60)
        self.assertEqual(err, b"")
        self.assertIs(alive, True)

    def test_a_p_series_node_is_fresh_so_no_earlier_signal_is_read(self):
        plan, built = self.setup("P1")
        path = built["liveness_fifo"]
        stale = os.open(path, os.O_RDWR | os.O_NONBLOCK)
        try:
            os.write(stale, b"L")                   # buffered in the OLD node
            _, again = self.setup("P1")
            self.assertEqual(again["liveness_fifo"], path)
            self.assertIs(driver._descendant_alive(again, plan, timeout_ms=200),
                          False)
        finally:
            os.close(stale)


class FixtureSignalDiagnostic(unittest.TestCase):
    PREFIX = ob.FIXTURE_SIGNAL_FAILED

    def test_the_line_is_reduced_to_a_normalised_fact(self):
        rows = ((self.PREFIX + b"open:2\n", {"reported": True, "failed_step": "open",
                                             "errno": "ENOENT", "errno_number": 2}),
                (b"noise\n" + self.PREFIX + b"write:32",
                 {"reported": True, "failed_step": "write", "errno": None,
                  "errno_number": 32}),
                (self.PREFIX + b"/tmp/private/O7.fixture-signal\n",
                 {"reported": True, "failed_step": None, "errno": None,
                  "errno_number": None}),
                (self.PREFIX + b"unlink:2\n",
                 {"reported": True, "failed_step": None, "errno": None,
                  "errno_number": None}),
                (b"stderr without a diagnostic\n", {"reported": False}))
        for data, want in rows:
            self.assertEqual(ob.fixture_signal_diagnostic(capture(data)), want, data)
        self.assertIsNone(ob.fixture_signal_diagnostic({"completeness": "CompleteAtEof"}))
        self.assertIsNone(ob.fixture_signal_diagnostic(None))

    def o7(self, signalled, diagnostic):
        plan = driver.CASE_PLANS["O7"]
        spike = D.receipt(exit_code=42, wait_si_code="CLD_EXITED",
                          stderr={"bytes_drained": 0,
                                  "drained_sha256": oracles.digest_of(b""),
                                  "completeness": "WriterRetainedAfterChildExit"})
        obs = D.obs_for(plan, spike=spike, fixture_descendant_signalled=signalled,
                        fixture_signal_diagnostic=diagnostic)
        obs["exec_confirmation"] = ob.exec_confirmation(
            spike, None, None, obs["report_state"], fixture_signalled=signalled)
        return score(plan, obs)

    def test_a_diagnosed_failure_is_distinct_from_silence_and_never_poses(self):
        failed = {"reported": True, "failed_step": "open", "errno": "ENOENT",
                  "errno_number": 2}
        status, record = self.o7(False, failed)
        self.assertEqual(status, INVALID)
        self.assertIn("fixture_descendant_signalled", record["not_posed"])
        self.assertIn("fixture-signal open failed with ENOENT", record["not_posed"])
        self.assertEqual(record["posing_evidence"]["fixture"]["signal_diagnostic"],
                         failed)
        status, record = self.o7(False, {"reported": False})
        self.assertEqual(status, INVALID)
        self.assertIn("wrote no fixture-signal failure line", record["not_posed"])
        self.assertNotIn("failed with", record["not_posed"])
        # The diagnostic never stands in for the signal, in either direction.
        self.assertEqual(self.o7(True, failed)[0], PASS)
        self.assertEqual(driver.POSED_CHECK_READS["fixture_descendant_signalled"],
                         ("fixture_descendant_signalled",))
        published = json.dumps(record["posing_evidence"])
        self.assertNotIn("/tmp", published)

    def test_helper_fork_writes_one_pathless_line_on_failure_only(self):
        source = (EXP / "helper_fork.c").read_text(encoding="utf-8")
        self.assertIn('#define FIXTURE_SIGNAL_FAILED "%s"'
                      % ob.FIXTURE_SIGNAL_FAILED.decode("ascii"), source)
        body = source.split("static void fixture_signal_failed", 1)[1].split("\n}\n", 1)[0]
        self.assertIn('"%s%s:%d\\n", FIXTURE_SIGNAL_FAILED', body)
        self.assertIn("write_all(2, line", body)
        self.assertNotIn("fifo", body)
        signal_block = source.split("if (fifo && rendezvous) {", 1)[1].split(
            "sleep_ms(lifetime_ms)")[0]
        self.assertIn('if (f < 0) {\n                int err = errno;\n'
                      '                fixture_signal_failed("open", err);', signal_block)
        self.assertIn('if (write_all(f, "L", 1) != 0) {\n                    '
                      'int err = errno;\n                    '
                      'fixture_signal_failed("write", err);', signal_block)
        # No descriptor crosses an exec. The opens are /dev/null, the liveness
        # rendezvous and, for P1/P2/P4 only (R-I1), the fixture-health FIFO and
        # the non-blocking liveness precheck; every FIFO open is close-on-exec.
        opens = re.findall(r"\bopen\(([^;]*)\);", source)
        self.assertEqual(len(opens), 4)
        for call in opens:
            if "/dev/null" not in call:
                self.assertIn("O_CLOEXEC", call)
        self.assertIn("pipe2(gate, O_CLOEXEC)", source)
        self.assertNotIn("dup(", source.split("int main", 1)[1].replace("dup2(", ""))


# ======================================= R3: CLD_DUMPED keeps its signal
CLD_EXITED, CLD_KILLED, CLD_DUMPED = 1, 2, 3


def receipt_status_fields(si_code, si_status, reaped=True):
    """launcher_spike.c's three receipt expressions, for a synthetic siginfo.

    ``test_the_launcher_expressions_are_these`` pins this restatement to the C.
    """
    if not reaped:
        si_code = si_status = 0
    return {"exit_code": si_status if si_code == CLD_EXITED else -1,
            "term_signal": si_status if si_code in (CLD_KILLED, CLD_DUMPED) else -1,
            "wait_si_code": {CLD_EXITED: "CLD_EXITED", CLD_KILLED: "CLD_KILLED",
                             CLD_DUMPED: "CLD_DUMPED"}.get(si_code, "")}


class R3SignalPreservation(unittest.TestCase):
    def score_receipt(self, case, disposition, fields, rep=True):
        plan = driver.CASE_PLANS[case]
        spike = D.receipt(process_disposition=disposition, **fields)
        return score(plan, D.obs_for(plan, spike=spike,
                                     rep=full_report() if rep else None))

    def test_the_launcher_expressions_are_these(self):
        source = (EXP / "launcher_spike.c").read_text(encoding="utf-8")
        self.assertIn("int term_signal = (info.si_code == CLD_KILLED || "
                      "info.si_code == CLD_DUMPED)\n                      "
                      "? info.si_status : -1;", source)
        self.assertNotIn("info.si_code == CLD_KILLED ? info.si_status : -1", source)
        self.assertIn("info.si_code == CLD_EXITED ? info.si_status : -1,", source)
        self.assertIn('\\"wait_si_code\\":\\"%s\\"', source)
        self.assertIn("term_signal, wait_si_code_name(reaped, info.si_code),", source)
        mapping = source.split("static const char *wait_si_code_name", 1)[1].split(
            "\n}\n", 1)[0]
        self.assertEqual(re.findall(r'case (CLD_\w+): return "(CLD_\w+)";', mapping),
                         [("CLD_EXITED", "CLD_EXITED"), ("CLD_KILLED", "CLD_KILLED"),
                          ("CLD_DUMPED", "CLD_DUMPED")])
        self.assertIn('if (reaped < 0) { return ""; }', mapping)
        self.assertIn('default: return "";', mapping)

    def test_cld_dumped_and_cld_killed_both_keep_the_signal(self):
        for code in (CLD_DUMPED, CLD_KILLED):
            status, record = self.score_receipt(
                "R3", "Signaled", receipt_status_fields(code, 11))
            self.assertEqual((status, record["outcome"]), (PASS, "Signaled:SIGSEGV"),
                             code)

    def test_cld_exited_keeps_the_exit_status(self):
        for case, code in (("R1", 0), ("R2", 42)):
            status, record = self.score_receipt(
                case, "Exited", receipt_status_fields(CLD_EXITED, code))
            self.assertEqual((status, record["outcome"]), (PASS, "Exited:%d" % code))

    def test_no_signal_is_invented(self):
        # Trial #2's R3: CLD_DUMPED reached the receipt as term_signal -1.
        status, record = self.score_receipt(
            "R3", "Signaled", {"exit_code": -1, "term_signal": -1,
                               "wait_si_code": "CLD_DUMPED"})
        self.assertEqual(status, INVALID)
        self.assertIn("termination signal outside the frozen signal table",
                      record["not_posed"])
        for fields in (receipt_status_fields(0, 0, reaped=False),
                       dict(receipt_status_fields(CLD_EXITED, 0), term_signal=11),
                       {"exit_code": -1, "term_signal": 11, "wait_si_code": ""}):
            self.assertEqual(self.score_receipt("R3", "Signaled", fields)[0], INVALID,
                             fields)
        self.assertEqual(self.score_receipt(
            "R1", "Exited", dict(receipt_status_fields(CLD_DUMPED, 11),
                                 exit_code=0))[0], INVALID)

    def test_the_parser_accepts_exactly_the_closed_classifications(self):
        for code in ("", "CLD_EXITED", "CLD_KILLED", "CLD_DUMPED"):
            self.assertIsNotNone(ob.parse_spike_stdout(json.dumps(
                D.receipt(wait_si_code=code))), code)
        for code in ("CLD_STOPPED", "CLD_CONTINUED", "CLD_TRAPPED", 3, None):
            self.assertIsNone(ob.parse_spike_stdout(json.dumps(
                D.receipt(wait_si_code=code))), code)
        self.assertIsNotNone(ob.parse_spike_stdout(json.dumps(D.receipt())))

    @LINUX
    def test_waitid_reports_si_status_for_an_exit_and_a_kill(self):
        # Isolated fork/waitid probes, not LAUNCH-EXEC cases. No core-dumping
        # signal is raised here, so no crash handler is invoked; CLD_DUMPED's
        # identical si_status rule is waitid(2)'s and the postmortem's probe's.
        self.assertEqual((os.CLD_EXITED, os.CLD_KILLED, os.CLD_DUMPED),
                         (CLD_EXITED, CLD_KILLED, CLD_DUMPED))
        for action, want_code, want_status in (("exit", os.CLD_EXITED, 7),
                                               ("kill", os.CLD_KILLED, 9)):
            pid = os.fork()
            if pid == 0:                                    # pragma: no cover
                try:
                    if action == "exit":
                        os._exit(7)
                    os.kill(os.getpid(), 9)
                finally:
                    os._exit(99)
            info = os.waitid(os.P_PID, pid, os.WEXITED)
            self.assertEqual((info.si_pid, info.si_code, info.si_status),
                             (pid, want_code, want_status), action)
            fields = receipt_status_fields(info.si_code, info.si_status)
            disposition = "Exited" if action == "exit" else "Signaled"
            token = ob.derive("process_disposition", D.obs_for(
                driver.CASE_PLANS["R1"], rep=full_report(),
                spike=D.receipt(process_disposition=disposition, **fields)))[0]
            self.assertEqual(token, "Exited:7" if action == "exit"
                             else "Signaled:SIGKILL")


# ======= R-M1 (correction review): S4 decisive contradictions come first
class S4DecisiveContradictionFirst(unittest.TestCase):
    """Would fail against 7f98d4d, whose report-reading posed check made these
    decisive launcher contradictions INVALID."""

    plan = driver.CASE_PLANS["S4"]
    EXITED_7 = dict(exit_code=fc.S4_EXIT_CODE, wait_si_code="CLD_EXITED")

    def s4(self, spike, rep=None, **over):
        return score(self.plan, D.obs_for(self.plan, spike=spike, rep=rep, **over))

    def test_no_posed_check_reads_the_report(self):
        self.assertIsNone(self.plan.posed_when)
        self.assertNotIn("helper_report_complete", driver.POSED_CHECKS)
        self.assertEqual(ob.ASSERTION_READS["helper_exit_corroborated"],
                         ("report", "report_state", "spike"))

    def test_an_admission_refusal_without_a_report_fails(self):
        status, record = self.s4(D.receipt(admission="refused",
                                           refusal="ElfNotInCohort"))
        self.assertEqual(status, FAIL, record)
        self.assertNotIn("not_posed", record)
        self.assertEqual(record["outcome"], "helper_exit_contradicted")
        self.assertEqual(record["mechanism_outcome"], "refused:ElfNotInCohort")

    def test_an_explicit_pre_exec_failure_without_a_report_fails(self):
        for stage, token in (("EXEC", "ExecFailed:EACCES"),
                             ("CHDIR", "ExecFailed:CHDIR:EACCES")):
            status, record = self.s4(D.receipt(
                process_disposition="ExecFailed", exec_failed_stage=stage,
                exec_failed_errno=13, exit_code=127))
            self.assertEqual(status, FAIL, record)
            self.assertEqual(record["outcome"], "helper_exit_contradicted")
            self.assertEqual(record["mechanism_outcome"], token)

    def test_other_decisive_launcher_contradictions_fail_without_a_report(self):
        for spike in (
                D.receipt(process_disposition="Signaled", term_signal=9,
                          exit_code=-1, wait_si_code="CLD_KILLED"),
                D.receipt(process_disposition="TimedOut",
                          timeout_disposition="KilledByLauncher", term_signal=15,
                          exit_code=-1, wait_si_code="CLD_KILLED"),
                D.receipt(exit_code=3, wait_si_code="CLD_EXITED"),
                D.receipt(process_disposition="ExecStatusIndeterminate",
                          exit_code=-1),
                D.receipt(process_disposition="ExitStatusUnobservable",
                          exit_code=-1, wait_errno=10)):
            status, record = self.s4(spike)
            self.assertEqual(status, FAIL, (spike, record))
            self.assertEqual(record["outcome"], "helper_exit_contradicted")

    def test_a_conservative_receipt_without_positive_evidence_is_invalid(self):
        receipt = D.receipt(**self.EXITED_7)
        for state in (ob.REPORT_ABSENT, ob.REPORT_STREAM_INCOMPLETE,
                      ob.REPORT_TRUNCATED, ob.REPORT_MALFORMED):
            status, record = self.s4(receipt, report_state=state)
            self.assertEqual(status, INVALID, state)
            self.assertIn("an executed-identity assertion could not be observed",
                          record["not_posed"])
        # a report that parses but declares no exit is not positive evidence
        self.assertEqual(self.s4(receipt, rep=D.report())[0], INVALID)
        # clean exec-status EOF alone still proves nothing
        self.assertEqual(ob.derive("launcher_receipt_claim", D.obs_for(
            self.plan, spike=receipt, rep=None))[0], "ExecStatusIndeterminate")

    def test_complete_matching_evidence_passes(self):
        status, record = self.s4(D.receipt(**self.EXITED_7), rep=full_report())
        self.assertEqual((status, record["outcome"]),
                         (PASS, "ExecStatusIndeterminate"), record)

    def test_a_contradiction_outranks_an_uninterpretable_other_side(self):
        rows = ((D.receipt(exit_code=7, wait_si_code="CLD_KILLED"),
                 full_report(declared_exit=3)),
                (D.receipt(exit_code=3, wait_si_code="CLD_EXITED"), None),
                (D.receipt(admission="refused", refusal="ElfNotInCohort"),
                 full_report()))
        for spike, rep in rows:
            status, record = self.s4(spike, rep=rep)
            self.assertEqual((status, record["outcome"]),
                             (FAIL, "helper_exit_contradicted"), record)
        # Nothing decisive on either side: still INVALID, never a guess.
        self.assertEqual(self.s4(D.receipt(exit_code=7, wait_si_code="CLD_KILLED"),
                                 rep=full_report())[0], INVALID)

    def test_s5_is_unchanged(self):
        s5 = driver.CASE_PLANS["S5"]
        self.assertEqual((s5.posed_when, s5.rule, s5.assertions),
                         ("no_helper_report", "process_disposition", ()))
        self.assertEqual(score(s5, D.obs_for(s5, spike=D.receipt()))[0], PASS)
        self.assertEqual(score(s5, D.obs_for(s5, spike=D.receipt(),
                                             rep=full_report()))[0], INVALID)
        self.assertEqual(score(s5, D.obs_for(s5, spike=D.receipt(
            process_disposition="ExecFailed", exec_failed_stage="EXEC",
            exec_failed_errno=13, exit_code=127)))[0], FAIL)


# ===== RR-I1 (re-review db45336): a structured pre-exec failure is decisive
# whether or not its errno has a symbolic name.
REREVIEWED_FIX = "53ad8bfef1b66592f2ae2e4e3df5fc847b526834"
UNNAMED_ERRNO = 7        # E2BIG on Linux x86-64; deliberately absent from ERRNO_NAMES
FROZEN_ERRNO_NUMBERS = {1, 2, 3, 4, 5, 8, 9, 10, 12, 13, 14, 20, 21, 22, 24, 26, 36,
                        38, 40}

OLD_S4_SCORE = r'''
import json, sys
sys.path.insert(0, sys.argv[1])
import checker, driver, observations as ob, oracles
plan = driver.CASE_PLANS["S4"]
block = {"bytes_drained": 0, "drained_sha256": oracles.digest_of(b""),
         "completeness": "CompleteAtEof"}
spike = {"admission": "accepted", "pre_exec_body_sha256": "a" * 64,
         "pre_exec_body_size": 4096, "pre_exec_mode_bits": 493,
         "process_disposition": "ExecFailed", "timeout_disposition": "",
         "exec_failed_stage": "EXEC", "exec_failed_errno": int(sys.argv[2]),
         "exit_code": 127, "term_signal": -1, "wait_si_code": "CLD_EXITED",
         "launcher_signal_issued": False, "group_sweep_issued": True,
         "wait_errno": 0, "stdout": block, "stderr": block}
obs = {"spike": spike, "report": None, "report_state": ob.REPORT_ABSENT,
       "launch_returned": True, "elapsed_ms": 12,
       "declared_pre_exec_stall": plan.pre_exec_stall,
       "declared_body_length_changed": plan.body_length_changed,
       "declared_injection_modes": [], "expected_marker": plan.expected_marker,
       "build_identity_binding": {"bound": True, "classification": "X",
                                  "object_sha256": "a" * 64},
       "cleanup_problems": []}
obs["exec_confirmation"] = ob.exec_confirmation(spike, None, None, ob.REPORT_ABSENT)
obs["repeat_observations"] = [obs]
record = driver.evaluate(plan, obs)
print(json.dumps([checker.score_case("S4", record)[0], record.get("outcome")]))
'''


class S4StructuredPreExecFailure(unittest.TestCase):
    plan = driver.CASE_PLANS["S4"]

    def exec_failed(self, **over):
        fields = dict(process_disposition="ExecFailed", exec_failed_stage="EXEC",
                      exec_failed_errno=13, exit_code=127, wait_si_code="CLD_EXITED")
        fields.update(over)
        spike = D.receipt(**fields)
        for key in [k for k, v in fields.items() if v is D]:
            spike.pop(key)          # D marks a field that must be absent
        return spike

    def s4(self, spike, rep=None, **over):
        return score(self.plan, D.obs_for(self.plan, spike=spike, rep=rep, **over))

    def test_the_errno_table_is_not_widened(self):
        self.assertEqual(set(ob.ERRNO_NAMES), FROZEN_ERRNO_NUMBERS)
        self.assertNotIn(UNNAMED_ERRNO, ob.ERRNO_NAMES)

    def test_a_known_errno_pre_exec_failure_fails(self):                     # A, F
        for stage in ("EXEC", "CHDIR", "DUP2"):
            status, record = self.s4(self.exec_failed(exec_failed_stage=stage))
            self.assertEqual((status, record["outcome"]),
                             (FAIL, "helper_exit_contradicted"), (stage, record))

    def test_an_unnamed_valid_errno_pre_exec_failure_fails(self):            # B
        for stage, number in (("EXEC", UNNAMED_ERRNO), ("SETPGID", 11),
                              ("CLOSE_RANGE", ob.ERRNO_NUMBER_MAX)):
            spike = self.exec_failed(exec_failed_stage=stage, exec_failed_errno=number)
            self.assertIs(ob.explicit_pre_exec_failure(spike), True)
            # the claim rule still renders no symbolic token for it
            self.assertIsNone(ob.derive("launcher_receipt_claim", D.obs_for(
                self.plan, spike=spike))[0])
            status, record = self.s4(spike)
            self.assertEqual((status, record["outcome"]),
                             (FAIL, "helper_exit_contradicted"), (stage, number, record))
            self.assertEqual(record["assertions"]["helper_exit_corroborated"]["result"],
                             ob.ASSERTION_VIOLATED)

    def test_the_rr_i1_regression_against_53ad8bf(self):
        tmp = pathlib.Path(tempfile.mkdtemp(prefix="rr-i1-53ad8bf-"))
        try:
            for name in MANIFEST["sha256"]:
                if name.endswith(".py"):
                    (tmp / name).write_bytes(git_show(
                        REREVIEWED_FIX, "docs/experiments/launch-exec-01/" + name))
            proc = subprocess.run([sys.executable, "-I", "-B", "-c", OLD_S4_SCORE,
                                   str(tmp), str(UNNAMED_ERRNO)],
                                  capture_output=True, timeout=120, cwd=str(tmp))
            self.assertEqual(proc.returncode, 0, proc.stderr.decode("utf-8", "replace"))
            old = json.loads(proc.stdout.decode("utf-8"))
        finally:
            shutil.rmtree(tmp, ignore_errors=True)
        self.assertEqual(old, ["INVALID", None])
        status, record = self.s4(self.exec_failed(exec_failed_errno=UNNAMED_ERRNO))
        self.assertEqual((status, record["outcome"]), (FAIL, "helper_exit_contradicted"))

    def test_a_structurally_invalid_pre_exec_failure_is_invalid(self):       # C, D
        rows = (
            dict(exec_failed_errno="13"), dict(exec_failed_errno="7"),
            dict(exec_failed_errno=None), dict(exec_failed_errno=7.0),
            dict(exec_failed_errno=0),
            dict(exec_failed_errno=-7), dict(exec_failed_errno=ob.ERRNO_NUMBER_MAX + 1),
            dict(exec_failed_errno=D),                       # missing field
            dict(exec_failed_errno=UNNAMED_ERRNO, exec_failed_stage=""),
            dict(exec_failed_errno=UNNAMED_ERRNO, exec_failed_stage="UNKNOWN"),
            dict(exec_failed_errno=UNNAMED_ERRNO, exec_failed_stage="ExecFailed text"),
            dict(exec_failed_errno=UNNAMED_ERRNO, exec_failed_stage=D),
            dict(exec_failed_errno=UNNAMED_ERRNO, timeout_disposition="KilledByLauncher"),
        )
        for over in rows:
            spike = self.exec_failed(**over)
            self.assertIs(ob.explicit_pre_exec_failure(spike), False, over)
            status, record = self.s4(spike)
            self.assertEqual((status, record.get("outcome")), (INVALID, None),
                             (over, record))
        self.assertIs(ob.explicit_pre_exec_failure(None), False)
        # a boolean is not an errno number, whatever Python's int subclassing says
        self.assertIs(ob.explicit_pre_exec_failure(
            self.exec_failed(exec_failed_errno=True)), False)
        self.assertIs(ob.explicit_pre_exec_failure(
            {"admission": "refused", "process_disposition": "ExecFailed",
             "timeout_disposition": "", "exec_failed_stage": "EXEC",
             "exec_failed_errno": 13}), False)

    def test_another_decisive_side_still_decides_a_malformed_record(self):
        spike = self.exec_failed(exec_failed_errno="13")
        status, record = self.s4(spike, rep=full_report(declared_exit=3))
        self.assertEqual((status, record["outcome"]), (FAIL, "helper_exit_contradicted"))
        # a malformed record beside a matching report decides nothing
        self.assertEqual(self.s4(spike, rep=full_report())[0], INVALID)

    def test_the_rest_of_the_s4_precedence_is_unchanged(self):              # E, G-M
        e7 = dict(exit_code=fc.S4_EXIT_CODE, wait_si_code="CLD_EXITED")
        fails = (
            D.receipt(admission="refused", refusal="ElfNotInCohort"),
            D.receipt(process_disposition="TimedOut",
                      timeout_disposition="KilledByLauncher", term_signal=15,
                      exit_code=-1, wait_si_code="CLD_KILLED"),
            D.receipt(process_disposition="Signaled", term_signal=11, exit_code=-1,
                      wait_si_code="CLD_DUMPED"),
            D.receipt(exit_code=3, wait_si_code="CLD_EXITED"),
        )
        for spike in fails:
            status, record = self.s4(spike)
            self.assertEqual((status, record["outcome"]),
                             (FAIL, "helper_exit_contradicted"), spike)
        receipt = D.receipt(**e7)
        for state in (ob.REPORT_ABSENT, ob.REPORT_MALFORMED, ob.REPORT_TRUNCATED):
            self.assertEqual(self.s4(receipt, report_state=state)[0], INVALID, state)
        status, record = self.s4(receipt, rep=full_report())
        self.assertEqual((status, record["outcome"]), (PASS, "ExecStatusIndeterminate"))
        # the accepted conservative exec-status claim is not a contradiction
        claim = ob.derive("launcher_receipt_claim", D.obs_for(self.plan, spike=receipt))
        self.assertEqual(claim[0], fc.BY_NAME["S4"]["predict"])
        self.assertEqual(ob._s4_receipt_exit(receipt)[0], ob.ASSERTION_HOLDS)

    def test_s2_s5_and_s7_are_untouched(self):
        s5 = driver.CASE_PLANS["S5"]
        self.assertNotIn("helper_exit_corroborated", s5.assertions)
        self.assertEqual(score(s5, D.obs_for(s5, spike=self.exec_failed(
            exec_failed_errno=UNNAMED_ERRNO)))[0], INVALID)
        self.assertEqual(score(s5, D.obs_for(s5, spike=self.exec_failed()))[0], FAIL)
        for case in ("S2", "S7"):
            self.assertEqual(driver.CASE_PLANS[case].assertions, ("no_executed_image",))


# ========= R-I1 (correction review): a liveness result needs a proven fixture
H = ob.liveness_fixture_health
PROBE = b"PROBE_REACHED\n"
P_CASES = ("P1", "P2", "P4")
TRIAL3_CANDIDATE = "7f98d4dcdb098abf96fe5e15e314194f5df6cb20"


def p_receipt(case):
    if case != "P4":
        return D.receipt()
    return D.receipt(stdout={"bytes_drained": 0,
                             "drained_sha256": oracles.digest_of(b""),
                             "completeness": "WriterRetainedAfterChildExit"})


def p_score(case, alive, health=None, **over):
    plan = driver.CASE_PLANS[case]
    fields = dict(descendant_alive_after_launch=alive, **over)
    if health is not None:
        fields["liveness_fixture_health"] = health
    return score(plan, D.obs_for(plan, spike=p_receipt(case), **fields))


class LivenessFixtureHealthGrammar(unittest.TestCase):
    def test_the_closed_grammar(self):
        rows = (
            (b"", False, "none", None),
            (PROBE, True, "none", None),
            (PROBE + b"LIVENESS_OPEN_FAILED:2\n", True, "open_failed", 2),
            (PROBE + b"LIVENESS_WRITE_FAILED:32\n", True, "write_failed", 32),
            (b"LIVENESS_OPEN_FAILED:13\n", False, "open_failed", 13),
            (b"LIVENESS_UNEXPECTED_OPEN\n", False, "unexpected_open", None),
            (b"PROBE_REACHED", False, "malformed", None),
            (PROBE + PROBE, False, "malformed", None),
            (b"LIVENESS_WRITE_FAILED:32\n", False, "malformed", None),
            (PROBE + b"LIVENESS_OPEN_FAILED:12345\n", False, "malformed", None),
            (PROBE + b"LIVENESS_OPEN_FAILED:/tmp/O7.fifo\n", False, "malformed", None),
            (PROBE + b"LIVENESS_UNEXPECTED_OPEN\n", False, "malformed", None),
            (b"L", False, "malformed", None),
            (PROBE + b"x" * ob.LIVENESS_HEALTH_MAX_BYTES, False, "malformed", None),
            (None, False, "unreadable", None),
        )
        for raw, probe, failure, number in rows:
            fact = H(raw)
            self.assertEqual((fact["probe_reached"], fact["channel_failure"],
                              fact["errno_number"]), (probe, failure, number), raw)
            self.assertIn(fact["channel_failure"], ob.LIVENESS_CHANNEL_FAILURES)
            self.assertEqual(sorted(fact), ["channel_failure", "errno",
                                            "errno_number", "probe_reached"])
        self.assertEqual(H(PROBE + b"LIVENESS_OPEN_FAILED:2\n")["errno"], "ENOENT")

    def test_the_interpretation_algebra(self):
        invalid = (
            ("A: no probe, no byte", False, H(b"")),
            ("A: no probe, a byte", True, H(b"")),
            ("A: no health observation", False, None),
            ("B: open failed after the probe", False, H(PROBE + b"LIVENESS_OPEN_FAILED:2\n")),
            ("B: write failed after the probe", False,
             H(PROBE + b"LIVENESS_WRITE_FAILED:32\n")),
            ("B: open failed before the probe", False, H(b"LIVENESS_OPEN_FAILED:13\n")),
            ("unexpected open", False, H(b"LIVENESS_UNEXPECTED_OPEN\n")),
            ("malformed", False, H(PROBE + b"ALIVE\n")),
            ("malformed with a byte", True, H(b"garbage")),
            ("unreadable", False, H(None)),
            ("probe but no recorded liveness", None, H(PROBE)),
        )
        for case in P_CASES:
            for label, alive, health in invalid:
                status, record = p_score(case, alive, health)
                self.assertEqual(status, INVALID, (case, label, record))
                self.assertIsNone(record.get("outcome"), (case, label))
            self.assertEqual(p_score(case, True, H(PROBE))[1]["outcome"],
                             "descendant_survived")
            status, record = p_score(case, False, H(PROBE))
            self.assertEqual((status, record["outcome"]), (PASS, "descendant_died"),
                             (case, record))

    def test_the_questions_and_constructions_are_unchanged(self):
        users = sorted(p.case for p in driver._PLAN_LIST
                       if p.rule == driver.LIVENESS_HEALTH_RULE)
        self.assertEqual(users, list(P_CASES))
        life = str(fc.P_DESCENDANT_LIFETIME_MS)
        expected = {
            "P1": ("--release-stdio", "--parent-exit", "0", "--lifetime-ms", life),
            "P2": ("--release-stdio", "--setsid", "--parent-exit", "0",
                   "--lifetime-ms", life),
            "P3": ("--release-stdio", "--parent-exit", "0", "--lifetime-ms", life),
            "P4": ("--retain-stdio", "--setsid", "--parent-exit", "0",
                   "--lifetime-ms", life),
        }
        for case, args in expected.items():
            plan = driver.CASE_PLANS[case]
            self.assertEqual(plan.helper_args, args, case)
            self.assertEqual(plan.setup, "fork_helper", case)
            spec = fc.BY_NAME[case]
            self.assertEqual(spec["cls"], fc.RECORDED, case)
        self.assertEqual(driver.CASE_PLANS["P3"].rule, "sweep")
        self.assertEqual(driver.CASE_PLANS["P4"].posed_when, "retention_observed")
        self.assertIsNone(driver.CASE_PLANS["P1"].posed_when)
        self.assertIsNone(driver.CASE_PLANS["P2"].posed_when)

    def test_p_health_and_the_o6_o7_signal_never_satisfy_each_other(self):
        o7 = driver.CASE_PLANS["O7"]
        spike = D.receipt(exit_code=42, wait_si_code="CLD_EXITED",
                          stderr={"bytes_drained": 0,
                                  "drained_sha256": oracles.digest_of(b""),
                                  "completeness": "WriterRetainedAfterChildExit"})
        obs = D.obs_for(o7, spike=spike, fixture_descendant_signalled=False,
                        descendant_alive_after_launch=True,
                        liveness_fixture_health=H(PROBE))
        self.assertEqual(score(o7, obs)[0], INVALID)
        for case in P_CASES:
            self.assertEqual(p_score(case, False, None,
                                     fixture_descendant_signalled=True)[0], INVALID)
        self.assertNotIn("liveness_fixture_health",
                         driver.POSED_CHECK_READS["fixture_descendant_signalled"])
        self.assertNotEqual(driver._LIVENESS_HEALTH_FD, driver._FIXTURE_FD)

    def test_helper_fork_orders_release_probe_gate_and_rendezvous(self):
        source = (EXP / "helper_fork.c").read_text(encoding="utf-8")
        probe = source.split("static int liveness_probe", 1)[1].split("\n}\n", 1)[0]
        order = [probe.index(s) for s in (
            "*health = open(health_path, O_WRONLY | O_NONBLOCK | O_CLOEXEC);",
            "int pre = open(fifo, O_WRONLY | O_NONBLOCK | O_CLOEXEC);",
            "errno != ENXIO",
            "health_token(*health, LIVENESS_PROBE_REACHED);",
            "close(gate_write);")]
        self.assertEqual(order, sorted(order))
        child = source.split("if (pid == 0) {", 1)[1].split("_exit(0);", 1)[0]
        order = [child.index(s) for s in (
            "close(0);", "close(2);", 'open("/dev/null", O_RDWR)', "setsid();",
            "liveness_probe(health_path, fifo, &health, gate[1])",
            "int f = open(fifo, O_WRONLY | O_CLOEXEC);",
            "health_failure(health, LIVENESS_OPEN_FAILED, err);",
            "health_failure(health, LIVENESS_WRITE_FAILED, err);")]
        self.assertEqual(order, sorted(order))
        parent = source.split("_exit(0);\n    }", 1)[1]
        self.assertLess(parent.index("close(gate[1]);"),
                        parent.index("read(gate[0], &byte, 1)"))
        self.assertLess(parent.index("read(gate[0], &byte, 1)"),
                        parent.index("return parent_exit;"))
        for token in ("PROBE_REACHED\\n", "LIVENESS_OPEN_FAILED",
                      "LIVENESS_WRITE_FAILED", "LIVENESS_UNEXPECTED_OPEN\\n"):
            self.assertIn('"%s"' % token, source)
        self.assertNotIn("--setsid", driver.CASE_PLANS["P1"].helper_args)
        spike = (EXP / "launcher_spike.c").read_text(encoding="utf-8")
        self.assertNotIn("fixture-health", spike)

    def test_the_harness_arms_before_the_spawn_and_reads_after_the_rendezvous(self):
        node = function("_launch_and_observe")
        lines = {}
        for sub in ast.walk(node):
            if isinstance(sub, ast.Call):
                name = ast.unparse(sub.func)
                lines[name] = min(lines.get(name, sub.lineno), sub.lineno)
        self.assertLess(lines["_arm_liveness_health"], lines["subprocess.Popen"])
        self.assertLess(lines["_arm_liveness_health"], lines["_run_with_post_pin"])
        self.assertGreater(lines["_read_liveness_health"], lines["_descendant_alive"])
        self.assertIn("disarm_liveness_health(built)", source_of("_run_once"))
        self.assertIn("disarm_liveness_health(built)",
                      source_of("release_setup_resources"))

    def test_7f98d4d_scored_a_missing_liveness_byte_descendant_died(self):
        """R-I1, the reviewer's false-PASS shape, against the reviewed candidate."""
        tmp = pathlib.Path(tempfile.mkdtemp(prefix="r-i1-7f98d4d-"))
        try:
            for name in MANIFEST["sha256"]:
                if name.endswith(".py"):
                    (tmp / name).write_bytes(git_show(
                        TRIAL3_CANDIDATE, "docs/experiments/launch-exec-01/" + name))
            proc = subprocess.run([sys.executable, "-I", "-B", "-c", OLD_P_SCORE,
                                   str(tmp)], capture_output=True, timeout=120,
                                  cwd=str(tmp))
            self.assertEqual(proc.returncode, 0,
                             proc.stderr.decode("utf-8", "replace"))
            old = json.loads(proc.stdout.decode("utf-8"))
        finally:
            shutil.rmtree(tmp, ignore_errors=True)
        for case in P_CASES:
            self.assertEqual(old[case], ["PASS", "descendant_died"], case)
            # The same observation now: no health channel, or the health a
            # broken liveness open reports, is INVALID and never descendant_died.
            for health in (None, H(b"LIVENESS_OPEN_FAILED:2\n")):
                status, record = p_score(case, False, health)
                self.assertEqual((status, record.get("outcome")), (INVALID, None))


OLD_P_SCORE = r'''
import json, sys
sys.path.insert(0, sys.argv[1])
import checker, driver, observations as ob, oracles
out = {}
for case in ("P1", "P2", "P4"):
    plan = driver.CASE_PLANS[case]
    block = {"bytes_drained": 0, "drained_sha256": oracles.digest_of(b""),
             "completeness": "CompleteAtEof"}
    retained = dict(block, completeness="WriterRetainedAfterChildExit")
    spike = {"admission": "accepted", "pre_exec_body_sha256": "a" * 64,
             "pre_exec_body_size": 4096, "pre_exec_mode_bits": 493,
             "process_disposition": "Exited", "timeout_disposition": "",
             "exec_failed_stage": "", "exec_failed_errno": 0, "exit_code": 0,
             "term_signal": -1, "launcher_signal_issued": False,
             "group_sweep_issued": True, "wait_errno": 0,
             "stdout": retained if case == "P4" else block, "stderr": block}
    obs = {"spike": spike, "report": None, "report_state": ob.REPORT_ABSENT,
           "launch_returned": True, "elapsed_ms": 12,
           "declared_pre_exec_stall": plan.pre_exec_stall,
           "declared_body_length_changed": plan.body_length_changed,
           "declared_injection_modes": [], "expected_marker": plan.expected_marker,
           "build_identity_binding": {"bound": True, "classification": "X",
                                      "object_sha256": "a" * 64},
           "cleanup_problems": [], "descendant_alive_after_launch": False}
    obs["exec_confirmation"] = ob.exec_confirmation(spike, None, None,
                                                    ob.REPORT_ABSENT)
    obs["repeat_observations"] = [obs]
    record = driver.evaluate(plan, obs)
    out[case] = [checker.score_case(case, record)[0], record.get("outcome")]
print(json.dumps(out, sort_keys=True))
'''

# A Python stand-in for helper_fork's P-series descendant, in helper_fork.c's
# order. It is not helper_fork and not a LAUNCH-EXEC case.
P_STANDIN = r'''
import errno, os, sys, time
liveness, health, workdir, mode = sys.argv[1:5]
for fd in (0, 1, 2):                 # P1/P2: the first three actions
    os.close(fd)
null = os.open(os.devnull, os.O_RDWR)
os.dup2(null, 1)
os.dup2(null, 2)
os.chdir(workdir)                    # the launcher's child fchdirs before execveat
if mode == "no_health":
    time.sleep(0.3)
    os._exit(0)
h = os.open(health, os.O_WRONLY | os.O_NONBLOCK | os.O_CLOEXEC)
if mode == "malformed":
    os.write(h, b"PROBE_REACHED\nALIVE\n")
    os._exit(0)
target = liveness + ".missing" if mode == "broken_path" else liveness
try:
    pre = os.open(target, os.O_WRONLY | os.O_NONBLOCK | os.O_CLOEXEC)
except OSError as exc:
    if exc.errno != errno.ENXIO:
        os.write(h, b"LIVENESS_OPEN_FAILED:%d\n" % exc.errno)
        os._exit(0)
else:
    os.close(pre)
    os.write(h, b"LIVENESS_UNEXPECTED_OPEN\n")
    os._exit(0)
os.write(h, b"PROBE_REACHED\n")
if mode == "die_after_probe":
    os._exit(0)
if mode == "late_open_fails":
    os.unlink(target)
try:
    f = os.open(target, os.O_WRONLY | os.O_CLOEXEC)
except OSError as exc:
    os.write(h, b"LIVENESS_OPEN_FAILED:%d\n" % exc.errno)
    os._exit(0)
if mode == "write_fails":
    os.write(h, b"LIVENESS_WRITE_FAILED:%d\n" % errno.EPIPE)
else:
    os.write(f, b"L")
os.close(f)
os._exit(0)
'''


@POSIX
class LivenessFixtureHealthThroughStandIns(unittest.TestCase):
    """Scratch FIFOs and a Python stand-in; helper_fork is never executed."""

    def setUp(self):
        self.root = pathlib.Path(tempfile.mkdtemp(prefix="p-health-"))
        self.cwd = os.getcwd()
        os.chdir(self.root)
        self.build = pathlib.Path("target") / "launch-exec-01"   # relative, as in Trial #2
        self.build.mkdir(parents=True)
        self.work = self.root / "work-dir-capability"
        self.work.mkdir()
        self.ctx = driver.TrialContext(build=str(self.build), work=str(self.build),
                                       preflight={}, freeze={},
                                       sanitiser=evidence.Sanitiser())
        self.built = []

    def tearDown(self):
        for built in self.built:
            driver.disarm_liveness_health(built)
        os.chdir(self.cwd)
        shutil.rmtree(self.root, ignore_errors=True)

    def setup_case(self, case):
        plan = driver.CASE_PLANS[case]
        built = driver.SETUPS[plan.setup](self.ctx, plan)
        self.built.append(built)
        return plan, built

    def observe(self, case, mode):
        """Arm, run the stand-in, wait as the direct child's gate does, rendezvous."""
        plan, built = self.setup_case(case)
        facts, why = driver._arm_liveness_health(built, ())
        self.assertIsNone(why)
        proc = subprocess.Popen(
            [sys.executable, "-I", "-c", P_STANDIN, built["liveness_fifo"],
             built["fixture_health_fifo"], str(self.work), mode],
            stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL)
        # launch() returns only after helper_fork's direct child passed its
        # gate, i.e. after the descendant wrote its health token.
        poller = select.poll()
        poller.register(built[driver._LIVENESS_HEALTH_FD], select.POLLIN)
        poller.poll(1000 if mode == "no_health" else 10000)
        alive = driver._descendant_alive(built, plan, timeout_ms=1500)
        raw = driver._read_liveness_health(built, timeout_ms=500)
        proc.wait(timeout=60)
        health = H(raw)
        obs = D.obs_for(plan, spike=p_receipt(case),
                        descendant_alive_after_launch=alive,
                        liveness_fixture_health=health,
                        liveness_health_arming=facts)
        record = driver.evaluate(plan, obs)
        status = checker.score_case(case, record)[0]
        return status, record, health, alive, obs

    def test_arming_is_fresh_private_non_inheritable_and_absolute(self):
        for case in P_CASES:
            plan, built = self.setup_case(case)
            facts, why = driver._arm_liveness_health(built, ())
            self.assertIsNone(why, case)
            self.assertEqual(facts, {"armed_before_launch": True,
                                     "reader_inheritable": False,
                                     "reader_passed_to_launcher": False,
                                     "fresh_fifo": True, "private_mode": True,
                                     "helper_path_absolute": True,
                                     "helper_path_is_armed_fifo": True}, case)
            fd = built[driver._LIVENESS_HEALTH_FD]
            self.assertFalse(os.get_inheritable(fd), case)
            self.assertTrue(os.path.isabs(built["fixture_health_fifo"]), case)
            self.assertEqual(driver._read_liveness_health(built, timeout_ms=0), b"")

    def test_the_reader_passed_to_the_launcher_is_refused(self):
        plan, built = self.setup_case("P2")
        stale = os.open(os.devnull, os.O_RDONLY)
        os.close(stale)              # the next descriptor the arming will get
        facts, why = driver._arm_liveness_health(built, (stale,))
        self.assertEqual(built[driver._LIVENESS_HEALTH_FD], stale)
        self.assertIsNone(facts)
        self.assertIn("held only by the harness", why)

    def test_a_relative_or_foreign_health_path_is_refused(self):
        plan, built = self.setup_case("P4")
        liveness = built["liveness_fifo"]
        health = built["fixture_health_fifo"]
        built["extra_helper_args"] = ("--liveness-fifo", liveness,
                                      "--fixture-health-fifo",
                                      os.path.relpath(health))
        facts, why = driver._arm_liveness_health(built, ())
        self.assertIsNone(facts)
        self.assertIn("not an absolute path to the armed FIFO", why)
        built["extra_helper_args"] = ("--liveness-fifo", liveness,
                                      "--fixture-health-fifo", liveness)
        self.assertIn("not an absolute path to the armed FIFO",
                      driver._arm_liveness_health(built, ())[1])

    def test_an_unarmable_channel_is_not_posed_and_names_no_path(self):
        plan, built = self.setup_case("P1")
        built["fixture_health_fifo"] = str(self.root / "no-such-dir" / "P1.health")
        facts, why = driver._arm_liveness_health(built, ())
        self.assertIsNone(facts)
        self.assertEqual(why, "the liveness fixture-health channel could not be "
                              "armed: ENOENT")
        record = driver.evaluate(plan, {"not_posed": why, "launch_returned": None,
                                        "repeat_observations": []})
        self.assertEqual(checker.score_case("P1", record)[0], INVALID)

    def test_no_probe_reached_is_invalid(self):
        for case in P_CASES:
            status, record, health, alive, _ = self.observe(case, "no_health")
            self.assertIs(alive, False)
            self.assertEqual((health["probe_reached"], health["channel_failure"]),
                             (False, "none"))
            self.assertEqual((status, record.get("outcome")), (INVALID, None), case)
            self.assertIn("never reported reaching the liveness probe point",
                          record["not_posed"])

    def test_the_reviewers_broken_liveness_open_is_invalid(self):
        for case in P_CASES:
            status, record, health, alive, _ = self.observe(case, "broken_path")
            self.assertIs(alive, False)
            self.assertEqual(health, {"probe_reached": False,
                                      "channel_failure": "open_failed",
                                      "errno": "ENOENT",
                                      "errno_number": errno.ENOENT})
            self.assertEqual((status, record.get("outcome")), (INVALID, None), case)
            self.assertIn("liveness open failed with ENOENT", record["not_posed"])

    def test_probe_then_liveness_open_failure_is_invalid(self):
        for case in P_CASES:
            status, record, health, alive, _ = self.observe(case, "late_open_fails")
            self.assertEqual((health["probe_reached"], health["channel_failure"],
                              health["errno"]), (True, "open_failed", "ENOENT"))
            self.assertEqual((status, record.get("outcome")), (INVALID, None), case)

    def test_probe_then_liveness_write_failure_is_invalid(self):
        for case in P_CASES:
            status, record, health, alive, _ = self.observe(case, "write_fails")
            self.assertIs(alive, False)
            self.assertEqual((health["probe_reached"], health["channel_failure"]),
                             (True, "write_failed"))
            self.assertEqual((status, record.get("outcome")), (INVALID, None), case)

    def test_probe_then_death_before_the_rendezvous_is_descendant_died(self):
        for case in P_CASES:
            status, record, health, alive, _ = self.observe(case, "die_after_probe")
            self.assertIs(alive, False)
            self.assertEqual((health["probe_reached"], health["channel_failure"]),
                             (True, "none"))
            self.assertEqual((status, record["outcome"]), (PASS, "descendant_died"))

    def test_probe_then_a_liveness_byte_is_descendant_survived(self):
        for case in P_CASES:
            status, record, health, alive, _ = self.observe(case, "ok")
            self.assertIs(alive, True)
            self.assertEqual((health["probe_reached"], health["channel_failure"]),
                             (True, "none"))
            self.assertEqual((status, record["outcome"]),
                             (PASS, "descendant_survived"))

    def test_malformed_health_is_invalid(self):
        status, record, health, alive, _ = self.observe("P2", "malformed")
        self.assertEqual(health["channel_failure"], "malformed")
        self.assertEqual((status, record.get("outcome")), (INVALID, None))

    def test_a_stale_token_cannot_satisfy_a_new_invocation(self):
        plan, built = self.setup_case("P1")
        driver._arm_liveness_health(built, ())
        old = os.open(built["fixture_health_fifo"], os.O_WRONLY | os.O_NONBLOCK)
        try:
            os.write(old, PROBE)                    # buffered in the OLD node
            facts, why = driver._arm_liveness_health(built, ())   # next invocation
            self.assertIsNone(why)
            self.assertIs(facts["fresh_fifo"], True)
            # Re-arming closed the old node's only reader: a stale writer now
            # reaches nobody, least of all the new node.
            with self.assertRaises(BrokenPipeError):
                os.write(old, PROBE)
            self.assertEqual(driver._read_liveness_health(built, timeout_ms=200), b"")
        finally:
            os.close(old)
        self.assertEqual(driver.disarm_liveness_health(built), [])
        self.assertFalse(os.path.exists(built["fixture_health_fifo"]))
        self.assertNotIn(driver._LIVENESS_HEALTH_FD, built)
        self.assertIsNone(driver._read_liveness_health(built))

    def test_both_paths_are_absolute_and_survive_a_directory_change(self):
        canonical = pathlib.Path(os.path.realpath(self.build))
        for case in P_CASES:
            plan, built = self.setup_case(case)
            driver._arm_liveness_health(built, ())
            for flag, key in (("--liveness-fifo", "liveness_fifo"),
                              ("--fixture-health-fifo", "fixture_health_fifo")):
                path = driver._helper_argument(built, flag)
                self.assertEqual(path, built[key])
                self.assertTrue(os.path.isabs(path), (case, flag))
                self.assertEqual(pathlib.Path(path).parent, canonical)
                self.assertTrue(pathlib.Path(path).name.startswith(case + "."))
                before = os.stat(path)
                os.chdir(self.work)
                try:
                    after = os.stat(path)
                finally:
                    os.chdir(self.root)
                self.assertTrue(stat.S_ISFIFO(after.st_mode))
                self.assertEqual((after.st_dev, after.st_ino),
                                 (before.st_dev, before.st_ino))
                self.assertEqual(stat.S_IMODE(after.st_mode), 0o600)
        for case in ("O6", "O7", "P3", "T5"):
            plan, built = self.setup_case(case)
            self.assertNotIn("fixture_health_fifo", built, case)
            self.assertNotIn("--fixture-health-fifo", built["extra_helper_args"], case)
            self.assertEqual(driver._arm_liveness_health(built, ()), (None, None))
            driver.disarm_fixture_signal(built)

    def test_published_evidence_carries_no_path(self):
        status, record, health, alive, obs = self.observe("P1", "broken_path")
        plan = driver.CASE_PLANS["P1"]
        published = json.dumps(driver.posing_evidence(plan, obs, [], None),
                               sort_keys=True)
        self.assertEqual(driver.posing_evidence(plan, obs)["fixture"]["liveness_health"],
                         health)
        for private in (str(self.root), os.path.realpath(self.root), "fixture-health",
                        ".liveness", "launch-exec-01"):
            self.assertNotIn(private, published)
            self.assertNotIn(private, record["not_posed"])
        self.assertIn('"liveness_health_arming"', published)


# ============================================ N3 and the global semantics
class UnchangedByTheCorrection(unittest.TestCase):
    def test_n3_stays_conditional_and_manufactures_nothing(self):
        spec = fc.BY_NAME["N3"]
        self.assertEqual((spec["cls"], spec["blocked_if"], spec["predict"]),
                         (fc.CONDITIONAL, "unprivileged_runner",
                          "privilege_transition_suppressed"))
        plan = driver.CASE_PLANS["N3"]
        self.assertIn("no privileged identity",
                      driver.SETUPS[plan.setup](None, plan)["not_posed"])
        record = driver.evaluate(plan, {"blocked": "unprivileged_runner",
                                        "launch_returned": None})
        self.assertEqual(checker.score_case("N3", record)[0], BLOCKED)
        self.assertEqual(checker.score_case("N3", {"outcome": "x",
                                                   "blocked": "euid_zero"})[0],
                         INVALID)
        self.assertNotIn("N3", CANDIDATE["scope"]["corrected_findings"])

    def test_the_classification_and_reduction_are_the_frozen_ones(self):
        self.assertEqual(driver.STATUS_PRECEDENCE, (FAIL, INVALID, PASS))
        self.assertEqual(driver.reduce_repetitions([PASS, INVALID, FAIL, PASS]), FAIL)
        self.assertEqual(driver.reduce_repetitions([PASS, INVALID, PASS]), INVALID)
        self.assertEqual(driver.reduce_repetitions([PASS, PASS]), PASS)
        self.assertEqual(checker.score_case("S4", {"blocked": "euid_zero"})[0], INVALID)
        digest = sha256((EXP / "checker.py").read_bytes())
        historical = json.loads(git_show(TRIAL2_FREEZE, MANIFEST_PATH))
        self.assertEqual(digest, historical["sha256"]["checker.py"])
        self.assertEqual(digest, MANIFEST["sha256"]["checker.py"])


# ============================================ Trial #3 freeze (trial-003)
# Active freeze validation binds the working tree to the Trial #3 manifest
# DIRECTLY. The expected paths and states below are literals of this freeze,
# and every expected hash comes from the manifest, never from the file under test.
TRIAL3_SOURCES = ["README.md", "checker.py", "driver.py", "evidence.py",
                  "frozen_cases.py", "harness.py", "helper_alt.c", "helper_dynamic.c",
                  "helper_fork.c", "helper_report.c", "helper_setid.c", "journal.py",
                  "launcher_spike.c", "make_fixtures.py", "observations.py",
                  "oracles.py", "run_launch_exec_01.py"]
TRIAL3_DEFINITIONS = ["docs/adr/ADR-0024-launch-authority.md",
                      "docs/experiments/LAUNCH-EXEC-01-DEFINITION.md",
                      "docs/research/HELM-LAUNCH-ARCHITECTURE.md"]
TRIAL3_TRACED = ["E1", "E7", "F4", "F7", "M1", "M2", "M3", "M4"]
TRIAL3_C_CHANGED = ["helper_fork.c", "helper_report.c", "launcher_spike.c"]

VERIFY_FREEZE = r'''
import sys
sys.path.insert(0, sys.argv[1])
import run_launch_exec_01 as runner
print("true" if runner.verify_freeze()[0] else "false")
'''


def source_drift(exp_dir, manifest):
    return sorted(name for name, digest in manifest["sha256"].items()
                  if not (exp_dir / name).is_file()
                  or sha256((exp_dir / name).read_bytes()) != digest)


def definition_drift(root, manifest):
    return sorted(path for path, digest in manifest["definition_sha256"].items()
                  if not (root / path).is_file()
                  or sha256((root / path).read_bytes()) != digest)


def frozen_input_names(exp_dir):
    """Every file of a frozen kind in the experiment directory."""
    return sorted(p.name for p in exp_dir.iterdir() if p.is_file()
                  and (p.suffix in (".py", ".c") or p.name == "README.md"))


class Trial3Freeze(unittest.TestCase):
    def test_the_manifest_is_the_trial_3_freeze_and_grants_nothing(self):
        self.assertEqual((MANIFEST["trial"], MANIFEST["status"], MANIFEST["freeze_state"],
                          MANIFEST["valid_trial_count"],
                          MANIFEST["d7_execution_authorised"]),
                         ("trial-003", "NOT_RUN", "FROZEN", 0, False))
        self.assertEqual(MANIFEST["trial_3"], {
            "trial": "trial-003", "freeze_state": "FROZEN", "execution": "NOT_RUN",
            "valid_trial_count": 0, "aggregate": None, "d7": "NOT_AUTHORISED",
            "dispatcher": "NONE", "build": "BUILD_8_REQUIRED",
            "independent_freeze_review": "REQUIRED", "published": False})
        self.assertNotIn("aggregate", MANIFEST)
        text = json.dumps(MANIFEST["trial_3"])
        for verdict in ("MECHANISM_ACCEPTED", "MECHANISM_REJECTED",
                        "MECHANISM_INCONCLUSIVE"):
            self.assertNotIn(verdict, text)
        self.assertEqual(MANIFEST["supersedes"], TRIAL2_FREEZE)
        self.assertEqual(MANIFEST["build"]["status"], "BUILD_8_REQUIRED")
        self.assertIs(MANIFEST["build"]["build_7_binds"], False)
        self.assertIsNone(MANIFEST["build"]["build_8_evidence"])
        self.assertEqual(MANIFEST["build"]["c_sources_changed_since_build_7"],
                         TRIAL3_C_CHANGED)
        self.assertEqual(MANIFEST["correction_candidate_record"]["authority"],
                         "none; this manifest alone is the Trial #3 freeze")
        self.assertEqual(MANIFEST["verify_freeze"]["does_not_check"], "definition_sha256")
        self.assertEqual(MANIFEST["trial_2"]["aggregate"], "MECHANISM_REJECTED")

    def test_the_closed_input_set_is_bound_exactly(self):
        historical = json.loads(git_show(TRIAL2_FREEZE, MANIFEST_PATH))
        self.assertEqual(sorted(MANIFEST["sha256"]), TRIAL3_SOURCES)
        self.assertEqual(sorted(MANIFEST["definition_sha256"]), TRIAL3_DEFINITIONS)
        self.assertEqual(sorted(historical["sha256"]), TRIAL3_SOURCES)
        self.assertEqual(sorted(historical["definition_sha256"]), TRIAL3_DEFINITIONS)
        self.assertEqual(frozen_input_names(EXP), TRIAL3_SOURCES)
        for digest in list(MANIFEST["sha256"].values()) + list(
                MANIFEST["definition_sha256"].values()):
            self.assertRegex(digest, r"\A[0-9a-f]{64}\Z")

    def test_the_exact_frozen_bytes_verify(self):
        self.assertEqual(source_drift(EXP, MANIFEST), [])
        self.assertEqual(definition_drift(ROOT, MANIFEST), [])
        ok, detail = runner.verify_freeze()
        self.assertIs(ok, True, detail)

    def test_drift_fails_on_a_disposable_copy(self):
        tmp = pathlib.Path(tempfile.mkdtemp(prefix="trial3-freeze-"))
        try:
            exp = tmp / "docs" / "experiments" / "launch-exec-01"
            shutil.copytree(EXP, exp, ignore=shutil.ignore_patterns("__pycache__"))
            for path in TRIAL3_DEFINITIONS:
                (tmp / path).parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(ROOT / path, tmp / path)

            def verify():
                proc = subprocess.run([sys.executable, "-I", "-B", "-c", VERIFY_FREEZE,
                                       str(exp)], capture_output=True, timeout=120,
                                      cwd=str(exp))
                self.assertEqual(proc.returncode, 0,
                                 proc.stderr.decode("utf-8", "replace"))
                return proc.stdout.decode("ascii").strip() == "true"

            self.assertIs(verify(), True)
            self.assertEqual(source_drift(exp, MANIFEST), [])
            self.assertEqual(definition_drift(tmp, MANIFEST), [])

            driver_py = exp / "driver.py"
            original = driver_py.read_bytes()
            self.assertTrue(original.endswith(b"\n"))
            # One byte, and still importable: the final newline becomes a space.
            driver_py.write_bytes(original[:-1] + b" ")
            self.assertIs(verify(), False)
            self.assertEqual(source_drift(exp, MANIFEST), ["driver.py"])
            driver_py.write_bytes(original)

            alt = exp / "helper_alt.c"
            kept = alt.read_bytes()
            alt.unlink()
            self.assertIs(verify(), False)
            self.assertEqual(source_drift(exp, MANIFEST), ["helper_alt.c"])
            alt.write_bytes(kept)

            definition = tmp / "docs" / "experiments" / "LAUNCH-EXEC-01-DEFINITION.md"
            text = definition.read_bytes()
            definition.write_bytes(text + b"\n")
            self.assertEqual(definition_drift(tmp, MANIFEST),
                             ["docs/experiments/LAUNCH-EXEC-01-DEFINITION.md"])
            # --verify-freeze checks source hashes only (R-M2); the definition
            # hash contract above is what catches this drift.
            self.assertIs(verify(), True)
            definition.write_bytes(text)

            (exp / "undeclared.py").write_text("X = 1\n", encoding="utf-8")
            self.assertNotEqual(frozen_input_names(exp), TRIAL3_SOURCES)
            self.assertNotIn("undeclared.py", MANIFEST["sha256"])
        finally:
            shutil.rmtree(tmp, ignore_errors=True)

    def test_the_trial_identity_is_trial_003(self):
        self.assertEqual(runner.TRIAL_ID, "trial-003")
        self.assertEqual(MANIFEST["trial"], runner.TRIAL_ID)
        source = (EXP / "run_launch_exec_01.py").read_text(encoding="utf-8")
        self.assertIn('TRIAL_ID = "trial-003"', source)
        self.assertNotIn('"trial-002"', source)
        self.assertIn("trial=TRIAL_ID", source)
        self.assertIn('{"trial": TRIAL_ID,', source)
        # Trial #2's preserved journal keeps its own identity.
        trials = {json.loads(line)["trial"] for line in
                  (TRIAL2_DIR / "journal.jsonl").read_text(encoding="utf-8").splitlines()
                  if line.strip()}
        self.assertEqual(trials, {"trial-002"})

    def test_the_global_shape_is_the_frozen_one(self):
        summary = fc.summary()
        self.assertEqual({k: summary[k] for k in ("total", "mandatory", "conditional",
                                                    "recorded")},
                         {"total": 72, "mandatory": 54, "conditional": 11,
                          "recorded": 7})
        self.assertEqual(sorted(summary["traced"]), TRIAL3_TRACED)
        self.assertEqual(len(fc.MEMBERSHIP), len(set(fc.MEMBERSHIP)))
        completeness = driver.completeness()
        self.assertIs(completeness["complete"], True)
        self.assertEqual((completeness["frozen_total"], completeness["driver_total"]),
                         (72, 72))
        for key in ("missing", "unknown", "duplicates"):
            self.assertEqual(completeness[key], [], key)
        self.assertEqual(driver.unposable_cases(), {})
        self.assertEqual(MANIFEST["case_membership"],
                         {"total": 72, "mandatory": 54, "conditional": 11,
                          "recorded": 7})
        self.assertEqual(MANIFEST["traced_cases"]["cases"], TRIAL3_TRACED)
        self.assertEqual((MANIFEST["driver"]["case_handlers"],
                          MANIFEST["driver"]["posable_against_this_freeze"],
                          MANIFEST["driver"]["unposable_against_this_freeze"]),
                         (72, 72, []))
        self.assertEqual(MANIFEST["vocabulary"]["tokens_protected"],
                         len(evidence.vocabulary()))

    def test_the_frozen_contract_blocks_are_the_code(self):
        pvs = MANIFEST["posing_versus_showing"]
        plans = driver._PLAN_LIST
        self.assertEqual(sorted(pvs["decisive_without_token"]["assertions"]),
                         sorted(ob.DECISIVE_WITHOUT_TOKEN_ASSERTIONS))
        self.assertEqual(pvs["decisive_without_token"]["assertions"],
                         ["helper_exit_corroborated", "no_executed_image"])
        self.assertEqual(pvs["expected_markers"],
                         {p.case: p.expected_marker for p in plans if p.expected_marker})
        self.assertEqual(pvs["liveness_fixture_health"]["cases"], ["P1", "P2", "P4"])
        self.assertEqual(pvs["s4"]["assertions"], list(driver.CASE_PLANS["S4"].assertions))
        for finding in ("R-M2", "RR-M1", "RR-M3"):
            self.assertTrue(MANIFEST["open_findings"][finding].startswith(
                "CORRECTED at this freeze"), finding)
        self.assertTrue(MANIFEST["open_findings"]["RR-M2"].startswith("MINOR, ACCEPTED"))


class Trial3FreezeCapturesReviewedSemantics(unittest.TestCase):
    """Freeze-integrity assertions, not another review: the frozen bytes still
    carry the reviewed S4 and P-series behaviour."""

    def test_s4(self):
        plan = driver.CASE_PLANS["S4"]
        unnamed = D.receipt(process_disposition="ExecFailed", exec_failed_stage="EXEC",
                            exec_failed_errno=UNNAMED_ERRNO, exit_code=127,
                            wait_si_code="CLD_EXITED")
        self.assertEqual(score(plan, D.obs_for(plan, spike=unnamed))[0], FAIL)
        accepted = D.receipt(exit_code=fc.S4_EXIT_CODE, wait_si_code="CLD_EXITED")
        self.assertEqual(score(plan, D.obs_for(plan, spike=accepted))[0], INVALID)
        self.assertEqual(score(plan, D.obs_for(plan, spike=accepted,
                                               rep=full_report()))[0], PASS)

    def test_p1_p2_p4(self):
        for case in P_CASES:
            for broken in (None, H(b""), H(b"LIVENESS_OPEN_FAILED:2\n")):
                self.assertEqual(p_score(case, False, broken)[0], INVALID, case)
            self.assertEqual(p_score(case, False, H(PROBE))[1]["outcome"],
                             "descendant_died", case)
            self.assertEqual(p_score(case, True, H(PROBE))[1]["outcome"],
                             "descendant_survived", case)


if __name__ == "__main__":
    unittest.main()
