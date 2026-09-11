"""Trial #2 AB7 correction (the ab74356 final review): regressions.

AB7-B1: O7's fixture is established by a signal the harness arms BEFORE the
launcher is spawned, because the launcher's own group sweep ends the retaining
descendant before any post-launch rendezvous. AB7-M2: O6 is posed the same way
and its completeness is a result. AB7-I1: S2/S7's explicit CHDIR exec-status
record is authoritative, so no clean end-of-file is required. AB7-M1: a report
sentinel is decisive that an image started even when the report does not parse,
and a status record beside a sentinel is a contradiction.

Observations are fabricated in-process through the real receipt parser. The
POSIX-only tests drive the frozen harness functions against a harmless Python
stand-in that plays the launcher and helper_fork, group sweep included. Nothing
builds, executes or otherwise invokes launcher_spike, a helper, a generated ELF
or a preregistered case, and no test constructs a driver.Authorisation.
LAUNCH-EXEC-01 remains NOT_RUN.
"""
import ast
import base64
import contextlib
import hashlib
import json
import os
import pathlib
import re
import shutil
import stat
import sys
import tempfile
import time
import unittest

HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE.parents[1]
EXP = ROOT / "docs" / "experiments" / "launch-exec-01"
if str(EXP) not in sys.path:
    sys.path.insert(0, str(EXP))

import checker              # noqa: E402
import driver               # noqa: E402
import evidence             # noqa: E402
import frozen_cases as fc   # noqa: E402
import journal              # noqa: E402
import observations as ob   # noqa: E402
import oracles              # noqa: E402

PASS, FAIL, INVALID, BLOCKED = (checker.PASS, checker.FAIL, checker.INVALID,
                                checker.BLOCKED)
POSIX = unittest.skipUnless(sys.platform.startswith("linux"),
                            "FIFOs, pidfds and /proc; Linux CI and WSL run it")
DRIVER_TREE = ast.parse((EXP / "driver.py").read_text(encoding="utf-8"))
OBS_TREE = ast.parse((EXP / "observations.py").read_text(encoding="utf-8"))
RETAINED, EOF = "WriterRetainedAfterChildExit", "CompleteAtEof"
C_SOURCES = ("launcher_spike.c", "helper_report.c", "helper_alt.c",
             "helper_dynamic.c", "helper_fork.c", "helper_setid.c")


def function(name, tree=DRIVER_TREE):
    for node in ast.walk(tree):
        if isinstance(node, ast.FunctionDef) and node.name == name:
            return node
    raise KeyError(name)


def body_without_docstring(name, tree=DRIVER_TREE):
    node = function(name, tree)
    body = node.body[1:] if ast.get_docstring(node) else node.body
    return ast.unparse(ast.Module(body=body, type_ignores=[]))


class SafetyBarrier(unittest.TestCase):
    def test_this_suite_constructs_no_authorisation(self):
        text = pathlib.Path(__file__).read_text(encoding="utf-8")
        self.assertNotIn("Authorisation" + "(True)", text)
        self.assertNotIn("driver." + "observe(", text)
        self.assertNotIn("driver." + "pose(", text)


# ------------------------------------------------------------ fabrication
def stream(data=b"", completeness=EOF, truncated=False, kept=None):
    """A stream block exactly as launcher_spike.c prints it."""
    prefix = data if kept is None else data[:kept]
    return {"bytes_drained": len(data),
            "drained_sha256": hashlib.sha256(data).hexdigest(),
            "completeness": completeness,
            "capture_prefix_length": len(prefix),
            "capture_prefix_truncated": truncated,
            "capture_prefix_base64": base64.b64encode(prefix).decode("ascii")}


def receipt_bytes(**over):
    """One receipt line in the frozen shape, for the real parser."""
    fields = {
        "admission": "accepted", "pre_exec_body_sha256": "c" * 64,
        "pre_exec_body_size": 1, "pre_exec_mode_bits": 493,
        "process_disposition": "Exited", "timeout_disposition": "",
        "exec_failed_stage": "", "exec_failed_errno": 0, "exit_code": 0,
        "term_signal": -1, "launcher_signal_issued": False,
        "group_sweep_issued": True, "wait_errno": 0,
        "stdout": stream(), "stderr": stream(),
        "rejected_acquisition_arm": {"attempted": False, "pidfd": -1,
                                     "pidfd_open_errno": 0, "waitid_rc": 0,
                                     "waitid_errno": 0,
                                     "exit_status_observed": False,
                                     "exit_status": -1},
        "parent_shape": {"extra_threads": 0, "atfork_handler_registered": False,
                         "atfork_prepare_calls": 0},
        "environment_mode": "empty",
        "retained_prefix_not_in_receipt": {"stdout_kept": 0,
                                           "stdout_truncated": False,
                                           "stderr_kept": 0,
                                           "stderr_truncated": False,
                                           "bound": 65536},
        "poll_returns_not_in_receipt": 3, "elapsed_ms_not_in_receipt": 1,
    }
    fields.update(over)
    return (json.dumps(fields) + "\n").encode()


def report_bytes(plan, **over):
    report = {"marker": "helper_report",
              "argv": [{"len": len(a.encode()), "value": a}
                       for a in [plan.argv0] + list(plan.helper_args)],
              "environ": [],
              "descriptors": [{"fd": n, "cloexec": False} for n in (0, 1, 2)],
              "signals": {"SigBlk": "0" * 16, "SigIgn": "0" * 16},
              "no_new_privs": "1"}
    report.update(over)
    return ob.REPORT_SENTINEL + json.dumps(report).encode() + b"\n"


def launch(plan, raw, signalled=None, returned=True, post_pin=None,
           not_posed=None):
    """_launch_and_observe's post-processing of one launcher's stdout.

    The live path is exercised end to end by the POSIX stand-in tests below;
    this is the same derivation for fabricated bytes.
    """
    if not_posed is not None:
        return {"not_posed": not_posed, "post_pin_evidence": post_pin,
                "launch_returned": None, "cleanup_problems": []}
    spike = ob.parse_spike_stdout(raw)
    state, report, payload = ob.REPORT_STREAM_INCOMPLETE, None, b""
    seen = None
    if isinstance(spike, dict) and spike.get("admission") == "accepted":
        state, report, payload = ob.helper_report_state(spike.get("stdout"))
        seen = ob.report_sentinel_seen(spike.get("stdout"))
    recipe = None
    if plan.streams and isinstance(spike, dict) and \
            spike.get("admission") == "accepted":
        recipe = all(isinstance(spike.get(n), dict)
                     and spike[n].get("bytes_drained") == w["bytes"]
                     and spike[n].get("drained_sha256") == w["sha256"]
                     for n, w in plan.streams.items())
    return {
        "spike": spike, "report": report, "report_state": state,
        "payload_len": len(payload), "payload_is_recipe": recipe,
        "exec_confirmation": ob.exec_confirmation(
            spike, report, recipe, state, fixture_signalled=signalled),
        "trace": None, "acquisition": None, "acquisition_normalised": None,
        "trace_sha256": None, "observed_stage_sequence": None,
        "descendant_alive_after_launch": None,
        "fixture_descendant_signalled": signalled,
        "fixture_signal_arming": None, "report_sentinel_seen": seen,
        "launch_returned": returned, "elapsed_ms": 2100 if returned else 19000,
        "spike_exit": 0, "spike_stderr": "", "post_pin_evidence": post_pin,
        "launcher_cpu_ms": None, "poll_returns": 3,
        "exec_status_pair_adjacent": None, "cleanup_problems": [],
    }


def pose_obs(plan, trials, built=None):
    """pose()'s augmentation of its repetitions, statement for statement."""
    obs = dict(trials[0])
    obs["repeat_observations"] = list(trials)
    obs["build_identity_binding"] = {
        "classification": "DIRECT_BASE", "base_artefact": plan.binary,
        "object": plan.binary, "bound": True, "object_sha256": "c" * 64}
    obs["cleanup_problems"] = []
    obs["declared_launcher_threads"] = None
    obs["expected_marker"] = plan.expected_marker
    obs["declared_pre_exec_stall"] = plan.pre_exec_stall
    obs["declared_body_length_changed"] = plan.body_length_changed
    obs["declared_injection_modes"] = driver.declared_injection_modes(plan)
    obs["expected_streams"] = plan.streams or None
    obs["expected_argv"] = ([plan.argv0] + list(plan.helper_args)
                            + list((built or {}).get("extra_helper_args", ())))
    obs["excused_descriptors"] = plan.excused_fds
    obs["traced"] = plan.traced
    return obs


def score(plan, obs):
    record = driver.evaluate(plan, obs)
    return checker.score_case(plan.case, record)[0], record


def roundtrip(case, status, record, pose_started=True):
    tmp = pathlib.Path(tempfile.mkdtemp(prefix="ab7-journal-"))
    try:
        path = tmp / "journal.jsonl"
        sanitiser = evidence.public_sanitiser(build=str(tmp / "build"))
        with journal.Journal(path, sanitiser, trial="trial-002") as j:
            j.case_entered(case, 1)
            if pose_started:
                j.case_pose_started(case, 1)
            j.case_completed(case, status, "r", record)
        raw = path.read_text(encoding="utf-8")
        kept = journal.replay(journal.read(path)[0])["completed"][case]["record"]
        return kept, raw
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


# ============================================== AB7-B1: O7 fixture signal
class O7PrearmedFixture(unittest.TestCase):
    plan = driver.CASE_PLANS["O7"]

    def o7(self, signalled=True, code=42, err=RETAINED, out=RETAINED, raw=None):
        raw = raw if raw is not None else receipt_bytes(
            exit_code=code, stdout=stream(b"", out), stderr=stream(b"", err))
        return score(self.plan, pose_obs(self.plan, [
            launch(self.plan, raw, signalled=signalled)]))

    def test_the_plan_is_the_owner_fixture_without_setsid(self):
        plan = self.plan
        self.assertEqual(plan.binary, "helper_fork")
        self.assertEqual(plan.setup, "fork_helper_prearmed")
        self.assertEqual(plan.helper_args,
                         ("--retain-stdio", "--parent-exit", "42",
                          "--lifetime-ms", str(fc.P_DESCENDANT_LIFETIME_MS)))
        self.assertNotIn("--setsid", plan.helper_args)
        self.assertEqual(plan.channels, (driver.CH_RECEIPT, driver.CH_LIVENESS))
        self.assertEqual(plan.streams, {})
        self.assertEqual(plan.rule, "process_disposition")
        self.assertEqual(plan.assertions, ("stderr_capture_failure_reported",))
        spec = fc.BY_NAME["O7"]
        self.assertEqual((spec["cls"], spec["predict"]), (fc.MANDATORY, "Exited:42"))

    def test_the_posed_check_reads_only_the_signal(self):
        self.assertEqual(self.plan.posed_when, "fixture_descendant_signalled")
        self.assertEqual(driver.POSED_CHECK_READS["fixture_descendant_signalled"],
                         ("fixture_descendant_signalled",))
        self.assertNotIn("fixture_descendant_alive", driver.POSED_CHECKS)
        body = body_without_docstring("_check_fixture_descendant_signalled")
        for word in ("spike", "completeness", "exit", "outcome", "stdout",
                     "stderr", "descendant_alive_after_launch"):
            self.assertNotIn(word, body, word)
        check = driver.POSED_CHECKS["fixture_descendant_signalled"]
        perfect = ob.parse_spike_stdout(receipt_bytes(
            exit_code=42, stderr=stream(b"", RETAINED)))
        self.assertFalse(check({"fixture_descendant_signalled": False,
                                "spike": perfect}))
        self.assertTrue(check({"fixture_descendant_signalled": True,
                               "spike": None}))

    def test_the_owner_matrix(self):
        rows = (({}, PASS),
                ({"err": EOF}, FAIL),
                ({"code": 0}, FAIL),
                ({"code": 7}, FAIL),
                ({"code": 0, "err": EOF}, FAIL),
                ({"signalled": False}, INVALID),
                ({"signalled": None}, INVALID),
                ({"raw": b"not a receipt\n"}, INVALID))
        for args, want in rows:
            self.assertEqual(self.o7(**args)[0], want, args)

    def test_both_receipt_facts_stay_separate(self):
        status, record = self.o7(err=EOF)
        self.assertEqual(status, FAIL)
        self.assertEqual(record["outcome"], "capture_failure_not_reported")
        self.assertEqual(record["mechanism_outcome"], "Exited:42")
        status, record = self.o7(code=0)
        self.assertEqual(record["outcome"], "Exited:0")
        self.assertEqual(record["assertions"]["stderr_capture_failure_reported"]
                         ["result"], ob.ASSERTION_HOLDS)

    def test_a_missing_required_receipt_fact(self):
        def without_completeness(code):
            obs = pose_obs(self.plan, [launch(self.plan, receipt_bytes(
                exit_code=code, stdout=stream(b"", RETAINED),
                stderr=stream(b"", RETAINED)), signalled=True)])
            spike = dict(obs["spike"], stderr={"bytes_drained": 0})
            obs["spike"] = obs["repeat_observations"][0]["spike"] = spike
            return score(self.plan, obs)[0]
        self.assertEqual(without_completeness(42), INVALID)
        # ... unless a decisive contradiction already makes it a FAIL.
        self.assertEqual(without_completeness(0), FAIL)

    def test_stdout_retention_is_not_scored(self):
        self.assertEqual(self.o7(out=EOF)[0], PASS)

    def test_the_signal_is_exec_evidence_and_nothing_older_is(self):
        spike = ob.parse_spike_stdout(receipt_bytes(
            exit_code=42, stderr=stream(b"", RETAINED),
            stdout=stream(b"", RETAINED)))
        incomplete = ob.REPORT_STREAM_INCOMPLETE
        self.assertEqual(ob.exec_confirmation(spike, None, None, incomplete,
                                              fixture_signalled=True),
                         ob.EXEC_REACHED)
        self.assertEqual(ob.exec_confirmation(spike, None, None, incomplete,
                                              fixture_signalled=False),
                         ob.EXEC_UNINTERPRETABLE)
        self.assertEqual(ob.exec_confirmation(spike, None, False, incomplete,
                                              fixture_signalled=True),
                         ob.EXEC_UNINTERPRETABLE)
        with self.assertRaises(TypeError):
            ob.exec_confirmation(spike, None, None, incomplete,
                                 descendant_alive=True)
        launch_src = ast.unparse(function("_launch_and_observe"))
        self.assertIn("fixture_signalled=signalled", launch_src)
        self.assertIn("report_sentinel_seen(spike.get('stdout'))", launch_src)

    def test_the_durable_record(self):
        plan = self.plan
        obs = pose_obs(plan, [dict(
            launch(plan, receipt_bytes(exit_code=42,
                                       stdout=stream(b"", RETAINED),
                                       stderr=stream(b"", RETAINED)),
                   signalled=True),
            fixture_signal_arming={"armed_before_launch": True,
                                   "reader_inheritable": False,
                                   "reader_passed_to_launcher": False,
                                   "fresh_fifo": True})])
        status, record = score(plan, obs)
        self.assertEqual(status, PASS)
        block = record["posing_evidence"]
        self.assertEqual(block["posed_check"],
                         {"name": "fixture_descendant_signalled", "held": True})
        self.assertIs(block["measured"]["fixture_descendant_signalled"], True)
        self.assertEqual(block["fixture"]["helper_args"], list(plan.helper_args))
        self.assertEqual(block["fixture"]["signal_arming"],
                         {"armed_before_launch": True, "reader_inheritable": False,
                          "reader_passed_to_launcher": False, "fresh_fifo": True})
        kept, raw = roundtrip("O7", status, record)
        self.assertEqual(kept, json.loads(json.dumps(record)))
        self.assertIsNone(re.search(r"fixture-signal|/tmp/|/home/|\"fd\"", raw))


# ================================================== AB7-M2: O6 as a result
class O6PrearmedFixture(unittest.TestCase):
    plan = driver.CASE_PLANS["O6"]
    PREWRITE = bytes((i * 251 + 1) % 256 for i in range(512))

    def o6(self, signalled=True, completeness=RETAINED, returned=True, **over):
        raw = receipt_bytes(**dict({"stdout": stream(self.PREWRITE, completeness),
                                    "stderr": stream(b"", completeness)}, **over))
        return score(self.plan, pose_obs(self.plan, [
            launch(self.plan, raw, signalled=signalled, returned=returned)]))

    def test_the_construction_is_unchanged_and_posed_by_the_signal(self):
        plan = self.plan
        self.assertEqual(plan.setup, "fork_helper_prearmed")
        self.assertEqual(plan.helper_args,
                         ("--prewrite", "512", "--retain-stdio", "--parent-exit",
                          "0", "--lifetime-ms", str(fc.P_DESCENDANT_LIFETIME_MS)))
        self.assertNotIn("--setsid", plan.helper_args)
        self.assertEqual(plan.posed_when, "fixture_descendant_signalled")
        self.assertEqual(plan.assertions, ("stream_completeness_as_declared",))
        self.assertEqual(plan.streams["stdout"]["completeness"], RETAINED)
        self.assertEqual(fc.BY_NAME["O6"]["predict"], "Exited:0")
        users = sorted(p.case for p in driver._PLAN_LIST
                       if p.posed_when == "retention_observed")
        self.assertEqual(users, ["P4"])

    def test_the_owner_matrix(self):
        self.assertEqual(self.o6(signalled=False)[0], INVALID)
        self.assertEqual(self.o6(signalled=None)[0], INVALID)
        self.assertEqual(self.o6()[0], PASS)
        status, record = self.o6(completeness=EOF)
        self.assertEqual(status, FAIL)
        self.assertEqual(record["outcome"], "completeness_mismatch")
        self.assertEqual(self.o6(exit_code=3)[0], FAIL)
        self.assertEqual(self.o6(returned=False)[0], FAIL)
        timed_out = self.o6(process_disposition="TimedOut",
                            timeout_disposition="KilledByLauncher",
                            exit_code=-1, term_signal=9)
        self.assertEqual(timed_out[0], FAIL)
        self.assertTrue(timed_out[1]["outcome"].startswith("TimedOut"))

    def test_retained_completeness_is_never_posing_evidence(self):
        # ab74356 posed O6 from the receipt's own completeness, so CompleteAtEof
        # -- the frozen FAIL -- could only ever be INVALID.
        status, record = self.o6(signalled=False, completeness=RETAINED)
        self.assertEqual(status, INVALID)
        self.assertIn("fixture_descendant_signalled", record["not_posed"])


# ================================== AB7-I1 / AB7-M1: S2 and S7 exec status
LANDED = [{"action": "chmod_work_dir", "landed": True, "detail": "d",
           "capability_open_in_launcher": True, "mode_before": 0o755,
           "mode_after": 0}]
NOT_LANDED = [dict(LANDED[0], landed=False)]
EACCES = dict(process_disposition="ExecFailed", exec_failed_stage="CHDIR",
              exec_failed_errno=13, exit_code=127)


def s_rep(plan, raw=None, landed=True):
    if not landed:
        return launch(plan, None, post_pin=NOT_LANDED,
                      not_posed="post-pin control: the preregistered forced "
                                "state did not land")
    return launch(plan, raw if raw is not None else receipt_bytes(**EACCES),
                  post_pin=LANDED)


def s_case(case, odd=None, where=0):
    plan = driver.CASE_PLANS[case]
    trials = [s_rep(plan) for _ in range(plan.repeat)]
    if odd is not None:
        trials[where] = odd(plan)
    return score(plan, pose_obs(plan, trials))


def undrained_eacces(plan):
    return s_rep(plan, receipt_bytes(**dict(
        EACCES, stdout=stream(b"", RETAINED), stderr=stream(b"", RETAINED))))


def image_ran(plan):
    return s_rep(plan, receipt_bytes(stdout=stream(report_bytes(plan))))


def malformed_after_sentinel(plan):
    return s_rep(plan, receipt_bytes(
        stdout=stream(ob.REPORT_SENTINEL + b"{not json\n")))


def truncated_after_sentinel(plan):
    return s_rep(plan, receipt_bytes(stdout=stream(
        ob.REPORT_SENTINEL + b'{"marker": "helper_rep', RETAINED)))


def undetermined(plan):
    return s_rep(plan, receipt_bytes(stdout=stream(b"", RETAINED),
                                     stderr=stream(b"", RETAINED)))


def contradiction(plan):
    return s_rep(plan, receipt_bytes(**dict(
        EACCES, stdout=stream(report_bytes(plan)))))


def indeterminate(plan):
    return s_rep(plan, receipt_bytes(
        process_disposition="ExecStatusIndeterminate", exit_code=-1))


class S2S7ExecStatusIsAuthoritative(unittest.TestCase):
    def test_an_explicit_chdir_status_needs_no_clean_eof(self):
        for case in ("S2", "S7"):
            self.assertEqual(s_case(case)[0], PASS, case)
            plan = driver.CASE_PLANS[case]
            trials = [undrained_eacces(plan) for _ in range(plan.repeat)]
            self.assertEqual(trials[0]["report_state"], ob.REPORT_STREAM_INCOMPLETE)
            self.assertIs(trials[0]["report_sentinel_seen"], False)
            status, record = score(plan, pose_obs(plan, trials))
            self.assertEqual(status, PASS, case)
            self.assertEqual(record["outcome"], "ExecFailed:CHDIR:EACCES")

    def test_a_sentinel_is_an_image_that_started_whatever_the_parse(self):
        for odd in (image_ran, malformed_after_sentinel, truncated_after_sentinel):
            status, record = s_case("S2", odd)
            self.assertEqual(status, FAIL, odd.__name__)
            self.assertEqual(record["outcome"], "executed_image_observed",
                             odd.__name__)

    def test_a_status_beside_a_sentinel_is_a_contradiction(self):
        status, record = s_case("S2", contradiction)
        self.assertEqual(status, FAIL)
        self.assertEqual(record["outcome"], "executed_image_observed")
        self.assertEqual(record["mechanism_outcome"], "ExecFailed:CHDIR:EACCES")
        self.assertIn("contradictory evidence", record["reason"])

    def test_undetermined_and_unlanded_are_invalid(self):
        self.assertEqual(s_case("S2", undetermined)[0], INVALID)
        self.assertEqual(s_case("S2", lambda plan: s_rep(plan, landed=False))[0],
                         INVALID)

    def test_any_other_decisive_disposition_fails(self):
        self.assertEqual(s_case("S2", indeterminate)[0], FAIL)

    def test_s7_one_wrong_repetition_anywhere_fails_the_case(self):
        for odd in (image_ran, malformed_after_sentinel, contradiction,
                    indeterminate):
            for where in (0, 100, 199):
                status, record = s_case("S7", odd, where)
                self.assertEqual(status, FAIL, (odd.__name__, where))
                self.assertEqual(record["posing_evidence"]["reduction"]
                                 ["decided_by_trial"], where)
        status, record = s_case("S7", undetermined, 57)
        self.assertEqual(status, INVALID)
        self.assertEqual(record["posing_evidence"]["reduction"][INVALID], 1)

    def test_the_sentinel_fact_is_derived_from_the_retained_prefix(self):
        plan = driver.CASE_PLANS["S2"]
        self.assertIs(ob.report_sentinel_seen(stream(report_bytes(plan))), True)
        self.assertIs(ob.report_sentinel_seen(
            stream(ob.REPORT_SENTINEL + b"{broken")), True)
        self.assertIs(ob.report_sentinel_seen(stream(b"no sentinel here")), False)
        self.assertIs(ob.report_sentinel_seen({"completeness": EOF}), None)
        self.assertIs(ob.report_sentinel_seen(None), None)

    def test_only_s2_and_s7_may_fail_without_a_rule_token(self):
        self.assertEqual(ob.DECISIVE_WITHOUT_TOKEN_ASSERTIONS,
                         frozenset({"no_executed_image"}))
        users = sorted(p.case for p in driver._PLAN_LIST
                       if set(p.assertions) & ob.DECISIVE_WITHOUT_TOKEN_ASSERTIONS)
        self.assertEqual(users, ["S2", "S7"])
        for case in ("S2", "S7"):
            self.assertIsNone(driver.CASE_PLANS[case].posed_when, case)

    def test_the_sentinel_is_durable_as_a_normalised_count(self):
        status, record = s_case("S7", image_ran, 5)
        self.assertEqual(status, FAIL)
        measured = record["posing_evidence"]["measured"]
        self.assertEqual(measured["report_sentinel_seen"],
                         {"False": 199, "True": 1})
        kept, raw = roundtrip("S7", status, record)
        self.assertEqual(kept, json.loads(json.dumps(record)))
        self.assertNotIn(base64.b64encode(ob.REPORT_SENTINEL).decode(), raw)
        blocked = driver.evaluate(driver.CASE_PLANS["S7"],
                                  {"blocked": "euid_zero", "launch_returned": None})
        self.assertNotIn("report_sentinel_seen",
                         blocked["posing_evidence"].get("measured", {}))


# ============================================ AB7-B1: FIFO hygiene, live
@POSIX
class FixtureSignalHygiene(unittest.TestCase):
    def setUp(self):
        self.tmp = pathlib.Path(tempfile.mkdtemp(prefix="ab7-fifo-"))
        self.ctx = driver.TrialContext(build=str(self.tmp), work=str(self.tmp),
                                       preflight={}, freeze={},
                                       sanitiser=evidence.Sanitiser())

    def tearDown(self):
        shutil.rmtree(self.tmp, ignore_errors=True)

    def built(self, case="O7"):
        plan = driver.CASE_PLANS[case]
        return driver.SETUPS[plan.setup](self.ctx, plan)

    @staticmethod
    def write(path, data):
        fd = os.open(path, os.O_WRONLY | os.O_NONBLOCK)
        try:
            os.write(fd, data)
        finally:
            os.close(fd)

    def test_setup_arms_nothing_and_removes_a_stale_node(self):
        stale = self.tmp / "O7.fixture-signal"
        os.mkfifo(stale)
        built = self.built()
        self.assertFalse(stale.exists())
        self.assertEqual(built["fixture_signal_fifo"], str(stale))
        self.assertEqual(built["extra_helper_args"], ("--liveness-fifo", str(stale)))
        self.assertNotIn(driver._FIXTURE_FD, built)

    def test_the_reader_is_fresh_non_inheritable_and_harness_only(self):
        built = self.built()
        facts, why = driver._arm_fixture_signal(built, ())
        self.assertIsNone(why)
        fd = built[driver._FIXTURE_FD]
        self.assertIs(os.get_inheritable(fd), False)
        self.assertTrue(stat.S_ISFIFO(os.fstat(fd).st_mode))
        self.assertEqual(facts, {"armed_before_launch": True,
                                 "reader_inheritable": False,
                                 "reader_passed_to_launcher": False,
                                 "fresh_fifo": True})
        path = built["fixture_signal_fifo"]
        self.assertEqual(driver.disarm_fixture_signal(built), [])
        self.assertFalse(os.path.exists(path))
        with self.assertRaises(OSError):
            os.fstat(fd)

    def test_a_reader_among_the_launchers_descriptors_is_refused(self):
        built = self.built()
        probe = os.open(os.devnull, os.O_RDONLY)
        os.close(probe)                      # the next descriptor will be this one
        facts, why = driver._arm_fixture_signal(built, (probe,))
        self.assertIsNone(facts)
        self.assertIn("held only by the harness", why)
        self.assertEqual(driver.disarm_fixture_signal(built), [])

    def test_only_the_exact_frozen_byte_counts(self):
        for data, want in ((b"L", True), (b"", False), (b"X", False),
                           (b"LL", False)):
            built = self.built()
            driver._arm_fixture_signal(built, ())
            if data:
                self.write(built["fixture_signal_fifo"], data)
            got = driver._read_fixture_signal(built, timeout_ms=200)
            driver.disarm_fixture_signal(built)
            self.assertIs(got, want, data)

    def test_the_harness_never_writes_the_signal(self):
        built = self.built()
        driver._arm_fixture_signal(built, ())
        self.assertIs(driver._read_fixture_signal(built, timeout_ms=100), False)
        driver.disarm_fixture_signal(built)
        for name in ("_setup_fork_helper_prearmed", "_arm_fixture_signal",
                     "_read_fixture_signal", "disarm_fixture_signal"):
            body = ast.unparse(function(name))
            for word in ("os.write", "O_WRONLY", "O_RDWR", "O_APPEND"):
                self.assertNotIn(word, body, (name, word))
        self.assertIn("os.O_RDONLY", ast.unparse(function("_arm_fixture_signal")))
        for node in ast.walk(DRIVER_TREE):
            if isinstance(node, ast.Call) and "write" in ast.unparse(node.func):
                self.assertNotIn("FIXTURE_SIGNAL_BYTE", ast.unparse(node))

    def test_a_previous_signal_cannot_be_read_again(self):
        built = self.built()
        driver._arm_fixture_signal(built, ())
        self.write(built["fixture_signal_fifo"], b"L")     # buffered, never read
        driver.disarm_fixture_signal(built)
        driver._arm_fixture_signal(built, ())               # the next invocation
        self.assertIs(driver._read_fixture_signal(built, timeout_ms=200), False)
        driver.disarm_fixture_signal(built)

    def test_a_writer_of_an_old_node_cannot_reach_the_new_one(self):
        built = self.built()
        driver._arm_fixture_signal(built, ())
        old = os.open(built["fixture_signal_fifo"], os.O_WRONLY | os.O_NONBLOCK)
        try:
            driver._arm_fixture_signal(built, ())           # re-armed: a new node
            with contextlib.suppress(BrokenPipeError):
                os.write(old, b"L")
            self.assertIs(driver._read_fixture_signal(built, timeout_ms=200), False)
        finally:
            os.close(old)
            driver.disarm_fixture_signal(built)

    def test_another_case_cannot_read_this_cases_signal(self):
        o6, o7 = self.built("O6"), self.built("O7")
        self.assertNotEqual(o6["fixture_signal_fifo"], o7["fixture_signal_fifo"])
        driver._arm_fixture_signal(o6, ())
        driver._arm_fixture_signal(o7, ())
        try:
            self.write(o6["fixture_signal_fifo"], b"L")
            self.assertIs(driver._read_fixture_signal(o7, timeout_ms=200), False)
            self.assertIs(driver._read_fixture_signal(o6, timeout_ms=200), True)
        finally:
            driver.disarm_fixture_signal(o6)
            driver.disarm_fixture_signal(o7)

    def test_the_bounded_read_cannot_hang(self):
        built = self.built()
        driver._arm_fixture_signal(built, ())
        silent = os.open(built["fixture_signal_fifo"], os.O_WRONLY | os.O_NONBLOCK)
        try:
            started = time.monotonic()
            self.assertIs(driver._read_fixture_signal(built, timeout_ms=300), False)
            self.assertLess(time.monotonic() - started, 2.0)
        finally:
            os.close(silent)
            driver.disarm_fixture_signal(built)

    def test_cleanup_removes_the_fifo_and_the_reader(self):
        built = self.built()
        driver._arm_fixture_signal(built, ())
        path = built["fixture_signal_fifo"]
        self.assertTrue(os.path.exists(path))
        self.assertEqual(driver.release_setup_resources(built), [])
        self.assertFalse(os.path.exists(path))
        self.assertNotIn(driver._FIXTURE_FD, built)
        # _run_once disarms after every invocation, whatever happened.
        run_once = ast.unparse(function("_run_once"))
        self.assertIn("disarm_fixture_signal(built)", run_once)
        self.assertIn("disarm_fixture_signal(built)",
                      ast.unparse(function("release_setup_resources")))

    def test_the_reader_is_armed_before_the_spawn_and_read_after_it(self):
        node = function("_launch_and_observe")
        lines = {}
        for sub in ast.walk(node):
            if isinstance(sub, ast.Call):
                name = ast.unparse(sub.func)
                lines[name] = min(lines.get(name, sub.lineno), sub.lineno)
        self.assertLess(lines["_arm_fixture_signal"], lines["subprocess.Popen"])
        self.assertLess(lines["_arm_fixture_signal"], lines["_run_with_post_pin"])
        self.assertGreater(lines["_read_fixture_signal"], lines["proc.communicate"])


# ===================== AB7-B1: the real harness against a live stand-in
STANDIN = r'''
import base64, hashlib, json, os, select, signal, sys, time
argv = sys.argv[1:]
inherited = {}
for name in os.listdir("/proc/self/fd"):
    try:
        inherited[name] = os.readlink("/proc/self/fd/" + name)
    except OSError:
        pass
def opt(name):
    return argv[argv.index(name) + 1] if name in argv else None
args = [argv[i + 1] for i, a in enumerate(argv) if a == "--arg"][1:]
drain_ms = int(opt("--post-exit-drain-ms"))
silent = "--standin-no-signal" in argv
out_r, out_w = os.pipe()
err_r, err_w = os.pipe()
child = os.fork()
if child == 0:
    os.setpgid(0, 0)
    os.dup2(out_w, 1)
    os.dup2(err_w, 2)
    os.closerange(3, 4096)
    retain, parent_exit, lifetime_ms, fifo, i = 0, 0, 20000, None, 0
    while i < len(args):
        a = args[i]
        v = args[i + 1] if i + 1 < len(args) else None
        if a == "--retain-stdio":
            retain = 1
        elif a == "--parent-exit" and v is not None:
            parent_exit, i = int(v), i + 1
        elif a == "--lifetime-ms" and v is not None:
            lifetime_ms, i = int(v), i + 1
        elif a == "--prewrite" and v is not None:
            for k in range(int(v)):
                os.write(1, bytes([(k * 251 + 1) % 256]))
            i += 1
        elif a == "--liveness-fifo" and v is not None:
            fifo, i = v, i + 1
        i += 1
    if os.fork() == 0:
        if fifo and not silent:
            f = os.open(fifo, os.O_WRONLY | os.O_CLOEXEC)
            os.write(f, b"L")
            os.close(f)
        time.sleep(lifetime_ms / 1000.0)
        os._exit(0)
    os._exit(parent_exit)
try:
    os.setpgid(child, child)
except OSError:
    pass
os.close(out_w)
os.close(err_w)
pidfd = os.pidfd_open(child)
bufs, live, child_end = {out_r: b"", err_r: b""}, {out_r: True, err_r: True}, None
poller = select.poll()
for fd in (out_r, err_r, pidfd):
    poller.register(fd, select.POLLIN)
while True:
    if child_end is not None:
        left = child_end + drain_ms / 1000.0 - time.monotonic()
        if left <= 0:
            break
        timeout = int(left * 1000) + 1
    else:
        timeout = 10000
    for fd, _ in poller.poll(timeout):
        if fd == pidfd:
            child_end = time.monotonic()
            poller.unregister(pidfd)
            continue
        data = os.read(fd, 65536)
        if data:
            bufs[fd] += data
        else:
            live[fd] = False
            poller.unregister(fd)
    if child_end is not None and not live[out_r] and not live[err_r]:
        break
os.killpg(child, signal.SIGKILL)
_, status = os.waitpid(child, 0)
def block(data, retained):
    return {"bytes_drained": len(data),
            "drained_sha256": hashlib.sha256(data).hexdigest(),
            "completeness": "WriterRetainedAfterChildExit" if retained else "CompleteAtEof",
            "capture_prefix_length": len(data), "capture_prefix_truncated": False,
            "capture_prefix_base64": base64.b64encode(data).decode("ascii")}
receipt = {
    "admission": "accepted", "pre_exec_body_sha256": "c" * 64,
    "pre_exec_body_size": 1, "pre_exec_mode_bits": 493,
    "process_disposition": "Exited", "timeout_disposition": "",
    "exec_failed_stage": "", "exec_failed_errno": 0,
    "exit_code": os.WEXITSTATUS(status), "term_signal": -1,
    "launcher_signal_issued": False, "group_sweep_issued": True, "wait_errno": 0,
    "stdout": block(bufs[out_r], live[out_r]), "stderr": block(bufs[err_r], live[err_r]),
    "poll_returns_not_in_receipt": 3, "elapsed_ms_not_in_receipt": 1,
    "standin_inherited": inherited,
}
sys.stdout.write(json.dumps(receipt) + "\n")
sys.stdout.flush()
os._exit(0)
'''


@POSIX
class PrearmedSignalThroughTheRealHarness(unittest.TestCase):
    """_launch_and_observe's own spawn, against a stand-in launcher."""

    def setUp(self):
        self.tmp = pathlib.Path(tempfile.mkdtemp(prefix="ab7-live-"))
        self.ctx = driver.TrialContext(build=str(self.tmp), work=str(self.tmp),
                                       preflight={}, freeze={},
                                       sanitiser=evidence.Sanitiser())
        self.saved = (driver.spike_argv, driver._arm_fixture_signal)

    def tearDown(self):
        driver.spike_argv, driver._arm_fixture_signal = self.saved
        shutil.rmtree(self.tmp, ignore_errors=True)

    def run_case(self, case, *extra):
        plan = driver.CASE_PLANS[case]
        built = driver.SETUPS[plan.setup](self.ctx, plan)
        applied = driver.AppliedParentState(
            plan.parent, driver.PARENT_STATES[plan.parent](self.ctx, plan))
        original_argv, original_arm = self.saved
        armed = {}

        def standin_argv(*a, **k):
            return ([sys.executable, "-c", STANDIN]
                    + original_argv(*a, **k)[1:] + list(extra))

        def spy(b, pass_fds=()):
            facts, why = original_arm(b, pass_fds)
            fd = b.get(driver._FIXTURE_FD)
            armed.update(fd=fd, pass_fds=tuple(pass_fds),
                         inheritable=os.get_inheritable(fd) if fd is not None
                         else None)
            return facts, why

        driver.spike_argv, driver._arm_fixture_signal = standin_argv, spy
        try:
            result = driver._launch_and_observe(plan, self.ctx, built, applied,
                                                None)
        finally:
            problems = driver.disarm_fixture_signal(built)
            applied.release()
        return plan, built, result, armed, problems

    def assertNeverReachedTheLauncher(self, built, result, armed):
        self.assertIs(armed["inheritable"], False)
        self.assertNotIn(armed["fd"], armed["pass_fds"])
        inherited = result["spike"]["standin_inherited"]
        self.assertNotIn(built["fixture_signal_fifo"], inherited.values())
        self.assertFalse(os.path.exists(built["fixture_signal_fifo"]))

    def test_o7_signal_survives_the_group_sweep_and_passes(self):
        plan, built, result, armed, problems = self.run_case("O7")
        self.assertEqual(problems, [])
        self.assertIs(result["spike"]["group_sweep_issued"], True)
        self.assertEqual(result["spike"]["exit_code"], 42)
        self.assertEqual(result["spike"]["stderr"]["completeness"], RETAINED)
        self.assertIs(result["fixture_descendant_signalled"], True)
        self.assertIsNone(result["descendant_alive_after_launch"])
        self.assertEqual(result["exec_confirmation"], ob.EXEC_REACHED)
        self.assertEqual(result["fixture_signal_arming"],
                         {"armed_before_launch": True, "reader_inheritable": False,
                          "reader_passed_to_launcher": False, "fresh_fifo": True})
        self.assertNeverReachedTheLauncher(built, result, armed)
        self.assertEqual(score(plan, pose_obs(plan, [result], built))[0], PASS)

    def test_o7_without_a_signal_is_invalid_beside_a_perfect_receipt(self):
        plan, built, result, armed, _ = self.run_case("O7", "--standin-no-signal")
        self.assertEqual(result["spike"]["exit_code"], 42)
        self.assertEqual(result["spike"]["stderr"]["completeness"], RETAINED)
        self.assertIs(result["fixture_descendant_signalled"], False)
        self.assertNeverReachedTheLauncher(built, result, armed)
        status, record = score(plan, pose_obs(plan, [result], built))
        self.assertEqual(status, INVALID)
        self.assertIn("fixture_descendant_signalled", record["not_posed"])

    def test_o6_signal_and_retained_writer_pass(self):
        plan, built, result, armed, problems = self.run_case("O6")
        self.assertEqual(problems, [])
        self.assertIs(result["fixture_descendant_signalled"], True)
        self.assertIs(result["payload_is_recipe"], True)
        self.assertEqual(result["spike"]["stdout"]["completeness"], RETAINED)
        self.assertNeverReachedTheLauncher(built, result, armed)
        self.assertEqual(score(plan, pose_obs(plan, [result], built))[0], PASS)


# ==================================================== preregistration
class AB7Preregistration(unittest.TestCase):
    def setUp(self):
        text = (ROOT / "docs" / "experiments"
                / "LAUNCH-EXEC-01-DEFINITION.md").read_text(encoding="utf-8")
        # Markdown wraps prose anywhere; compare phrases on normalised spacing.
        self.definition = " ".join(text.split())
        self.manifest = json.loads((EXP / "SOURCE-HASHES.json").read_text(
            encoding="utf-8"))

    def test_section_9_7_freezes_the_owner_decisions(self):
        self.assertIn("### 9.7 AB7 correction", self.definition)
        for phrase in ("fork_helper_prearmed", "fixture_descendant_signalled",
                       "O_RDONLY | O_NONBLOCK | O_CLOEXEC", "pass_fds",
                       "FIXTURE_SIGNAL_READ_TIMEOUT_MS", "kill(-child, SIGKILL)",
                       "report_sentinel_seen", "DECISIVE_WITHOUT_TOKEN_ASSERTIONS",
                       "contradictory evidence cannot pass",
                       "whether or not stdout reached a clean end-of-file",
                       "does not mean the descendant outlived `launch()`",
                       "P4 keeps `retention_observed`"):
            self.assertIn(phrase, self.definition, phrase)

    def test_the_manifest_records_the_same_contract(self):
        pvs = self.manifest["posing_versus_showing"]
        self.assertNotIn("fixture_descendant_alive", pvs["posed_check_inputs"])
        self.assertEqual(pvs["posed_check_inputs"]["fixture_descendant_signalled"],
                         ["fixture_descendant_signalled"])
        self.assertEqual(pvs["assertions"]["no_executed_image"]["reads"],
                         list(ob.ASSERTION_READS["no_executed_image"]))
        self.assertEqual(sorted(pvs["decisive_without_token"]["assertions"]),
                         sorted(ob.DECISIVE_WITHOUT_TOKEN_ASSERTIONS))
        for block in ("o6", "o7", "s2_s7", "fixture_signal"):
            self.assertIn(block, pvs, block)
        self.assertIn("fixture_descendant_signalled", pvs["o7"]["posing_evidence"])
        self.assertIn("never posing evidence", pvs["o7"]["posing_evidence"])
        for finding in ("AB7-B1", "AB7-I1", "AB7-M1", "AB7-M2"):
            self.assertRegex(self.manifest["open_findings"][finding],
                             r"^(CORRECTED|FIXED)", finding)
        for finding in ("AB7-N1", "AB7-N2", "AB7-N3"):
            self.assertTrue(self.manifest["open_findings"][finding]
                            .startswith("BACKLOG"), finding)

    def test_no_c_or_helper_source_changed(self):
        for name in C_SOURCES:
            actual = hashlib.sha256((EXP / name).read_bytes()).hexdigest()
            self.assertEqual(actual, self.manifest["sha256"][name], name)

    def test_the_case_table_is_untouched(self):
        self.assertEqual(fc.summary()["total"], 72)
        counts = {cls: sum(1 for spec in fc.CASES if spec["cls"] == cls)
                  for cls in (fc.MANDATORY, fc.CONDITIONAL, fc.RECORDED)}
        self.assertEqual(counts, {fc.MANDATORY: 54, fc.CONDITIONAL: 11,
                                  fc.RECORDED: 7})
        self.assertEqual(sorted(s["case"] for s in fc.CASES if s["traced"]),
                         ["E1", "E7", "F4", "F7", "M1", "M2", "M3", "M4"])
        for case in ("O6", "O7", "S2", "S7"):
            self.assertEqual(self.manifest["case_membership"]["total"], 72, case)


if __name__ == "__main__":
    unittest.main()
