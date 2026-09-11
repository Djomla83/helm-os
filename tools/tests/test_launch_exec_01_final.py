"""Trial #2 final classification correction (the F417 review): regressions.

Each class targets behaviour the freeze f417984 got wrong and would fail against
it: the repeated-trial reduction (F417-B1), O8's short repetition (B2), S2/S7's
executed image (B3), F7's circular adjacency proof (B4), report-based rules on
decisive evidence (I1), M2's control arm (I2), durable posing evidence on
BLOCKED records (M1), and the definition section 9.6 audit of every other posed
check and rule that turned a decisive observation into INVALID.

Every observation is fabricated in-process. Nothing builds, executes or
otherwise invokes launcher_spike, a helper, a generated ELF or a preregistered
case, and no test constructs a driver.Authorisation. LAUNCH-EXEC-01 remains
NOT_RUN.
"""
import ast
import json
import pathlib
import shutil
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
import journal              # noqa: E402
import observations as ob   # noqa: E402
import oracles              # noqa: E402
import test_launch_exec_01_delta as D   # noqa: E402  fabrication helpers only

receipt, report, obs_for = D.receipt, D.report, D.obs_for
PASS, FAIL, INVALID, BLOCKED = (checker.PASS, checker.FAIL, checker.INVALID,
                                checker.BLOCKED)
DRIVER_TREE = ast.parse((EXP / "driver.py").read_text(encoding="utf-8"))
OBS_TREE = ast.parse((EXP / "observations.py").read_text(encoding="utf-8"))


def function(tree, name):
    for node in ast.walk(tree):
        if isinstance(node, ast.FunctionDef) and node.name == name:
            return node
    raise KeyError(name)


def calls(node, name):
    return any(isinstance(n, ast.Call) and ast.unparse(n.func).endswith(name)
               for n in ast.walk(node))


def score(plan, obs):
    record = driver.evaluate(plan, obs)
    return checker.score_case(plan.case, record)[0], record


def repeated(plan, trials, **extra):
    """A pose()-shaped observation: case-level facts plus every repetition."""
    obs = dict(obs_for(plan, **extra), **trials[0])
    obs["repeat_observations"] = list(trials)
    return obs


class SafetyBarrier(unittest.TestCase):
    def test_this_suite_constructs_no_authorisation(self):
        text = pathlib.Path(__file__).read_text(encoding="utf-8")
        self.assertNotIn("Authorisation" + "(True)", text)
        self.assertNotIn("driver." + "observe(", text)
        self.assertNotIn("driver." + "pose(", text)


# ======================================================== B1: the reduction
LANDED = {"action": "chmod_work_dir", "landed": True, "detail": "d",
          "capability_open_in_launcher": True, "mode_before": 0o755,
          "mode_after": 0}


def s_trial(**over):
    """One S2/S7 repetition: landed at the barrier, fchdir refused."""
    base = {"spike": receipt(process_disposition="ExecFailed",
                             exec_failed_stage="CHDIR", exec_failed_errno=13),
            "report": None, "report_state": ob.REPORT_ABSENT,
            "launch_returned": True, "elapsed_ms": 20,
            "exec_confirmation": ob.EXEC_PRE_EXEC_ERROR,
            "post_pin_evidence": [dict(LANDED)]}
    base.update(over)
    return base


INDETERMINATE = dict(spike=receipt(process_disposition="ExecStatusIndeterminate"),
                     exec_confirmation=ob.EXEC_DIED_BEFORE_EXEC)
IMAGE_RAN = dict(spike=receipt(), report=report(),
                 report_state=ob.REPORT_COMPLETE, exec_confirmation=ob.EXEC_REACHED)
EXITED_127 = dict(spike=receipt(exit_code=127), report=report(),
                  report_state=ob.REPORT_COMPLETE, exec_confirmation=ob.EXEC_REACHED)
NOT_LANDED = {"not_posed": "post-pin control: the preregistered forced state "
                           "did not land",
              "launch_returned": None,
              "post_pin_evidence": [dict(LANDED, landed=False)]}


def s7(trials):
    return score(driver.CASE_PLANS["S7"], repeated(driver.CASE_PLANS["S7"], trials))


class RepetitionReduction(unittest.TestCase):
    def test_the_reduction_itself(self):
        r = driver.reduce_repetitions
        self.assertEqual(r([PASS] * 200), PASS)
        for where in (0, 99, 199):
            trials = [PASS] * 200
            trials[where] = FAIL
            self.assertEqual(r(trials), FAIL, where)
        self.assertEqual(r([INVALID] + [PASS] * 199), INVALID)
        self.assertEqual(r([INVALID, FAIL] + [PASS] * 198), FAIL)
        self.assertEqual(r([FAIL, INVALID] + [PASS] * 198), FAIL)
        self.assertEqual(r([]), INVALID)

    def test_s7_all_posed_and_correct_passes(self):
        self.assertEqual(s7([s_trial() for _ in range(200)])[0], PASS)

    def test_s7_one_unexpected_repetition_fails_wherever_it_is(self):
        """f417984 PASSed this whenever the odd repetition was the first."""
        for where in (0, 1, 99, 199):
            trials = [s_trial() for _ in range(200)]
            trials[where] = s_trial(**INDETERMINATE)
            status, record = s7(trials)
            self.assertEqual(status, FAIL, where)
            self.assertEqual(record["outcome"], "ExecStatusIndeterminate", where)
            self.assertIn("repeated trial %d" % where, record["reason"])

    def test_first_repetition_unposed_the_rest_correct_is_invalid(self):
        trials = [s_trial(**NOT_LANDED)] + [s_trial() for _ in range(199)]
        status, record = s7(trials)
        self.assertEqual(status, INVALID)
        self.assertIn("repeated trial 0", record["not_posed"])

    def test_a_known_fail_beats_an_unposed_repetition_either_way(self):
        for bad, odd in ((3, 150), (150, 3)):
            trials = [s_trial() for _ in range(200)]
            trials[bad] = s_trial(**NOT_LANDED)
            trials[odd] = s_trial(**INDETERMINATE)
            self.assertEqual(s7(trials)[0], FAIL, (bad, odd))

    def test_every_repetition_is_durable(self):
        trials = [s_trial() for _ in range(200)]
        trials[3] = s_trial(**NOT_LANDED)
        trials[150] = s_trial(**INDETERMINATE)
        _, record = s7(trials)
        evidence_block = record["posing_evidence"]
        self.assertEqual(len(evidence_block["repetitions"]), 200)
        self.assertEqual(evidence_block["reduction"],
                         {FAIL: 1, INVALID: 1, PASS: 198, "decided_by_trial": 150})
        self.assertEqual(evidence_block["repetitions"][3],
                         {"trial": 3, "status": INVALID, "posed": False,
                          "outcome": None})
        self.assertEqual(len([f for f in evidence_block["forced_state"]
                              if f["action"] == "chmod_work_dir"]), 200)


# ============================================================ B2: O8 floor
class O8ShortRepetition(unittest.TestCase):
    plan = driver.CASE_PLANS["O8"]
    GOOD = oracles.stream_digest("stdout", 512)

    def trial(self, nbytes=512, digest=None):
        d = digest if digest is not None else oracles.stream_digest("stdout", nbytes)
        return {"spike": receipt(stdout={"bytes_drained": nbytes, "drained_sha256": d,
                                         "completeness": "CompleteAtEof"}),
                "report": None, "report_state": ob.REPORT_ABSENT,
                "launch_returned": True, "elapsed_ms": 5,
                "payload_is_recipe": nbytes == 512 and d == self.GOOD}

    UNAVAILABLE = {"spike": None, "report": None,
                   "report_state": ob.REPORT_STREAM_INCOMPLETE,
                   "launch_returned": True, "elapsed_ms": 5,
                   "payload_is_recipe": None}

    def run_case(self, trials):
        return score(self.plan, repeated(self.plan, trials,
                                         expected_streams=self.plan.streams))

    def test_200_complete_repetitions_pass(self):
        self.assertEqual(self.run_case([self.trial() for _ in range(200)])[0], PASS)

    def test_owner_named_short_repetitions_fail(self):
        for where, nbytes in ((0, 400), (99, 0), (100, 0), (199, 511)):
            trials = [self.trial() for _ in range(200)]
            trials[where] = self.trial(nbytes)
            status, record = self.run_case(trials)
            self.assertEqual(status, FAIL, (where, nbytes))
            self.assertEqual(record["outcome"], "stream_mismatch")

    def test_a_short_posed_repetition_is_never_invalid(self):
        for where in (0, 57, 199):
            for nbytes in (0, 1, 256, 511):
                trials = [self.trial() for _ in range(200)]
                trials[where] = self.trial(nbytes)
                self.assertEqual(self.run_case(trials)[0], FAIL, (where, nbytes))

    def test_a_wrong_drain_in_the_first_repetition_fails(self):
        """f417984 PASSed both of these behind 199 correct repetitions."""
        for first in (self.trial(512, "0" * 64), self.trial(600)):
            trials = [first] + [self.trial() for _ in range(199)]
            self.assertEqual(self.run_case(trials)[0], FAIL)

    def test_an_unavailable_stream_fact_is_invalid_and_a_short_one_still_wins(self):
        trials = [self.trial() for _ in range(200)]
        trials[10] = dict(self.UNAVAILABLE)
        self.assertEqual(self.run_case(trials)[0], INVALID)
        trials[20] = self.trial(100)
        self.assertEqual(self.run_case(trials)[0], FAIL)

    def test_the_byte_floor_posed_check_is_gone_and_the_construction_is_not(self):
        self.assertIsNone(self.plan.posed_when)
        self.assertNotIn("trial_floor_512", driver.POSED_CHECKS)
        # POLLIN and POLLHUP in one return are O8's CONSTRUCTION -- a 512-byte
        # write followed at once by exit, 200 times -- as they always were.
        self.assertEqual(self.plan.helper_args, ("--no-report", "--stdout", "512"))
        self.assertEqual(self.plan.repeat, fc.REPEAT_TRIALS)
        self.assertEqual(fc.REPEAT_TRIALS, 200)
        self.assertEqual(fc.BY_NAME["O8"]["predict"], "stream_exact")


# ======================================================== B3: S2 and S7
class S2S7LandingVersusResult(unittest.TestCase):
    def s2(self, trial):
        plan = driver.CASE_PLANS["S2"]
        return score(plan, repeated(plan, [trial]))

    def test_s2_correct_passes_and_absent_state_is_invalid(self):
        self.assertEqual(self.s2(s_trial())[0], PASS)
        self.assertEqual(self.s2(s_trial(**NOT_LANDED))[0], INVALID)

    def test_s2_landed_and_the_image_ran_fails(self):
        for result in (IMAGE_RAN, EXITED_127, INDETERMINATE):
            self.assertEqual(self.s2(s_trial(**result))[0], FAIL, result["spike"])

    def test_a_report_sentinel_beside_the_expected_token_fails(self):
        for state in (ob.REPORT_COMPLETE, ob.REPORT_MALFORMED, ob.REPORT_TRUNCATED):
            status, record = self.s2(s_trial(report_state=state,
                                             report=report() if state == ob.REPORT_COMPLETE
                                             else None))
            self.assertEqual(status, FAIL, state)
            self.assertEqual(record["outcome"], "executed_image_observed")
            self.assertEqual(record["mechanism_outcome"], "ExecFailed:CHDIR:EACCES")

    def test_an_explicit_chdir_status_needs_no_decidable_stream(self):
        # AB7-I1, owner decision: the child's explicit ExecFailed:CHDIR:EACCES
        # record is authoritative, so stdout need not reach a clean
        # end-of-file. ab74356 scored this repetition INVALID.
        self.assertEqual(self.s2(s_trial(report_state=ob.REPORT_STREAM_INCOMPLETE))[0],
                         PASS)
        # Without an explicit status, an undecidable stream stays INVALID.
        undecided = s_trial(spike=receipt(), exec_confirmation=ob.EXEC_UNINTERPRETABLE,
                            report_state=ob.REPORT_STREAM_INCOMPLETE)
        self.assertEqual(self.s2(undecided)[0], INVALID)

    def test_s7_later_and_first_unexpected_results_fail(self):
        for where in (0, 42, 199):
            for result in (EXITED_127, IMAGE_RAN):
                trials = [s_trial() for _ in range(200)]
                trials[where] = s_trial(**result)
                self.assertEqual(s7(trials)[0], FAIL, (where, result["spike"]))

    def test_s7_forced_state_absent_is_invalid(self):
        trials = [s_trial() for _ in range(200)]
        trials[77] = s_trial(**NOT_LANDED)
        self.assertEqual(s7(trials)[0], INVALID)

    def test_the_report_is_a_result_not_a_posing_check_and_blocking_is_unchanged(self):
        for case in ("S2", "S7"):
            plan = driver.CASE_PLANS[case]
            self.assertIsNone(plan.posed_when, case)
            self.assertEqual(plan.assertions, ("no_executed_image",), case)
            self.assertEqual(fc.BY_NAME[case]["blocked_if"], "euid_zero")
            record = driver.evaluate(plan, {"blocked": "euid_zero",
                                            "launch_returned": None})
            self.assertEqual(checker.score_case(case, record)[0], BLOCKED)
        users = sorted(p.case for p in driver._PLAN_LIST
                       if p.posed_when == "no_helper_report")
        self.assertEqual(users, ["S5"])


# ================================================ B4: F7, non-circular proof
CLONE = ("111 clone3({flags=CLONE_PIDFD, pidfd=0x7ffd0000, "
         "exit_signal=SIGCHLD}, 88) = 222")
PARENT = ["111 pipe2([4, 5], O_CLOEXEC) = 0", "111 pipe2([6, 7], O_CLOEXEC) = 0",
          "111 pipe2([8, 9], O_CLOEXEC) = 0", "111 pipe2([10, 11], O_CLOEXEC) = 0",
          "111 fcntl(0, F_DUPFD_CLOEXEC, 3) = 12", "111 close(0) = 0"]
PARENT_GAP = PARENT[:4] + ["111 fcntl(0, F_DUPFD_CLOEXEC, 3) = 13",
                           "111 close(0) = 0"]
GOOD_CHILD = ["dup2(4, 0) = 0", "close_range(3, 10, 0) = 0",
              "close_range(13, 4294967295, 0) = 0",
              'execveat(12, "", [], [], AT_EMPTY_PATH) = 0']
EINVAL_CHILD = ["close_range(3, 10, 0) = 0",
                "close_range(12, 11, 0) = -1 EINVAL (Invalid argument)",
                'write(11, "\\5\\0\\0\\0\\26\\0\\0\\0", 8) = 8', "exit_group(127) = ?"]


def f7_trace(parent, child):
    return "\n".join(list(parent) + [CLONE] + ["222 " + line for line in child]) + "\n"


def adjacent(text):
    return ob.parent_pair_is_adjacent(ob.parse_parent_descriptor_pair(text, 0))


class F7IndependentAdjacency(unittest.TestCase):
    plan = driver.CASE_PLANS["F7"]
    TRACE = {"child_syscalls": ["close_range", "execveat"]}

    def test_the_pair_is_established_from_the_parent_before_clone(self):
        self.assertIs(adjacent(f7_trace(PARENT, GOOD_CHILD)), True)
        # The EINVAL itself does not unmake a pair established before it.
        self.assertIs(adjacent(f7_trace(PARENT, EINVAL_CHILD)), True)

    def test_adjacency_absent_or_unestablished(self):
        self.assertIs(adjacent(f7_trace(PARENT_GAP, [])), False)
        self.assertIsNone(adjacent(f7_trace(PARENT[:3], GOOD_CHILD)))
        unobserved = PARENT[:4] + ["111 fcntl(0, F_DUPFD_CLOEXEC, 3) = ?"]
        self.assertIsNone(adjacent(f7_trace(unobserved, GOOD_CHILD)))
        other = GOOD_CHILD[:3] + ['execveat(9, "", [], [], AT_EMPTY_PATH) = 0']
        self.assertIsNone(adjacent(f7_trace(PARENT, other)))

    def test_adjacency_cannot_be_derived_from_close_range(self):
        # Child spans that LOOK adjacent prove nothing without the parent.
        self.assertIsNone(adjacent(f7_trace([], GOOD_CHILD)))
        # And child spans cannot overrule a parent that was not adjacent.
        looks_adjacent = ["close_range(3, 11, 0) = 0",
                          "close_range(14, 4294967295, 0) = 0"]
        self.assertIs(adjacent(f7_trace(PARENT_GAP, looks_adjacent)), False)
        body = ast.unparse(function(OBS_TREE, "parse_parent_descriptor_pair"))
        self.assertNotIn("close_range", body)
        self.assertFalse(hasattr(ob, "layout_is_adjacent"))
        self.assertFalse(hasattr(ob, "parse_child_descriptor_layout"))
        launch = ast.unparse(function(DRIVER_TREE, "_launch_and_observe"))
        self.assertIn("parse_parent_descriptor_pair", launch)
        self.assertEqual(driver.POSED_CHECK_READS[self.plan.posed_when],
                         ("exec_status_pair_adjacent",))

    def test_absent_adjacency_is_invalid(self):
        for value in (None, False):
            status, record = score(self.plan, obs_for(
                self.plan, rep=report(), exec_status_pair_adjacent=value,
                trace=self.TRACE))
            self.assertEqual(status, INVALID, value)
            self.assertIn("exec_status_pair_adjacent", record["not_posed"])

    def test_established_adjacency_then_einval_fails(self):
        failed = receipt(process_disposition="ExecFailed",
                         exec_failed_stage="CLOSE_RANGE", exec_failed_errno=22)
        status, record = score(self.plan, obs_for(
            self.plan, spike=failed, rep=None,
            exec_status_pair_adjacent=adjacent(f7_trace(PARENT, EINVAL_CHILD)),
            trace={"child_syscalls": ["close_range", "write", "exit_group"]}))
        self.assertEqual(status, FAIL)
        self.assertEqual(record["outcome"], "ExecFailed:CLOSE_RANGE:EINVAL")
        self.assertIs(record["posing_evidence"]["posed_check"]["held"], True)

    def test_established_adjacency_and_a_clean_image_passes(self):
        status, _ = score(self.plan, obs_for(
            self.plan, rep=report(), exec_status_pair_adjacent=True,
            trace=self.TRACE))
        self.assertEqual(status, PASS)


# ================================================== I1: report-based rules
REPORT_RULES = {"argv_exact", "environ_empty", "fds_exactly_012",
                "signals_reset", "no_new_privs", "interpreter_ran_with_devfd",
                "privilege_transition_suppressed"}


def report_based_cases():
    return sorted(p.case for p in driver._PLAN_LIST if p.rule in REPORT_RULES)


def case_obs(case, **over):
    plan = driver.CASE_PLANS[case]
    extra = {}
    if plan.traced:
        extra["trace"] = {"child_syscalls": ["execveat"]}
    if plan.posed_when == "exec_status_pair_adjacent":
        extra["exec_status_pair_adjacent"] = True
    extra.update(over)
    return plan, obs_for(plan, **extra)


class ReportBasedRules(unittest.TestCase):
    def test_the_affected_rule_set_is_derived_mechanically(self):
        derived = {name for name, fn in ob.RULES.items()
                   if calls(function(OBS_TREE, fn.__name__), "_report_or_decisive")}
        self.assertEqual(derived, REPORT_RULES)
        direct = {name for name, fn in ob.RULES.items()
                  if calls(function(OBS_TREE, fn.__name__), "_usable_report")}
        self.assertEqual(direct, set(), "a rule reads the report past the door")
        self.assertEqual(report_based_cases(), sorted(
            ["A1", "A2", "A3", "A4", "A6", "V1", "F1", "F2", "F3", "F4", "F5",
             "F6", "F7", "N1", "N2", "N3", "X2c"]))

    def test_genuinely_missing_evidence_stays_invalid(self):
        for case in report_based_cases():
            for state in (ob.REPORT_TRUNCATED, ob.REPORT_MALFORMED,
                          ob.REPORT_STREAM_INCOMPLETE):
                plan, obs = case_obs(case, rep=None, report_state=state)
                self.assertEqual(score(plan, obs)[0], INVALID, (case, state))
            plan, obs = case_obs(case, rep=None)
            obs["spike"], obs["exec_confirmation"] = None, None
            self.assertEqual(score(plan, obs)[0], INVALID, (case, "no receipt"))

    def test_a_decisive_pre_exec_failure_fails(self):
        for case in report_based_cases():
            for stage, errno in (("CLOSE_RANGE", 22), ("DUP2", 9), ("EXEC", 2)):
                failed = receipt(process_disposition="ExecFailed",
                                 exec_failed_stage=stage, exec_failed_errno=errno)
                plan, obs = case_obs(case, spike=failed, rep=None)
                status, record = score(plan, obs)
                self.assertEqual(status, FAIL, (case, stage))
                self.assertTrue(record["outcome"].startswith("ExecFailed:"))

    def test_a_decisive_absence_and_a_refusal_fail(self):
        for case in report_based_cases():
            plan, obs = case_obs(case, rep=None, report_state=ob.REPORT_ABSENT)
            self.assertEqual(score(plan, obs)[0], FAIL, (case, "absent"))
            refused = {"admission": "refused", "refusal": "ElfNotInCohort",
                       "exec_reached": False}
            plan, obs = case_obs(case, spike=refused, rep=None,
                                 report_state=ob.REPORT_STREAM_INCOMPLETE)
            self.assertEqual(score(plan, obs)[0], FAIL, (case, "refused"))

    def test_f6_stdout_lost_through_cloexec_fails(self):
        lost = receipt(stdout={"bytes_drained": 0,
                               "drained_sha256": oracles.digest_of(b""),
                               "completeness": "CompleteAtEof",
                               "capture_prefix_length": 0,
                               "capture_prefix_truncated": False,
                               "capture_prefix_base64": ""})
        state, rep, _ = ob.helper_report_state(lost["stdout"])
        self.assertEqual((state, rep), (ob.REPORT_ABSENT, None))
        plan, obs = case_obs("F6", spike=lost, rep=None, report_state=state)
        status, record = score(plan, obs)
        self.assertEqual(status, FAIL)
        self.assertEqual(record["outcome"], "ExecStatusIndeterminate")

    def test_wrong_report_content_fails_and_correct_content_passes(self):
        wrong = {
            "F2": dict(report(), descriptors=[{"fd": n, "cloexec": False}
                                              for n in (0, 1, 2, 7)]),
            "V1": dict(report(), environ=["PATH=/usr/bin"]),
            "A1": dict(report(), argv=[{"len": 1, "value": "x"}]),
            "F5": dict(report(), signals=dict(report()["signals"],
                                              SigIgn="0000000000001000")),
            "N1": dict(report(), no_new_privs="0"),
        }
        for case, rep in wrong.items():
            plan = driver.CASE_PLANS[case]
            plan, obs = case_obs(case, rep=rep, expected_argv=(
                [plan.argv0] + list(plan.helper_args)))
            self.assertEqual(score(plan, obs)[0], FAIL, case)
        for case in ("F1", "F2", "F3", "F6", "V1", "N1", "F5"):
            plan, obs = case_obs(case, rep=report())
            self.assertEqual(score(plan, obs)[0], PASS, case)

    def test_no_decisive_token_is_a_report_based_expectation(self):
        lifecycle = ("ExecFailed", "Exited", "Signaled", "TimedOut", "refused:",
                     "ExecStatus", "ExitStatus", "stream_mismatch")
        for case in report_based_cases():
            spec = fc.BY_NAME[case]
            for token in [spec["predict"]] + list(spec["safe"] or ()):
                if token is not None:
                    self.assertFalse(token.startswith(lifecycle), (case, token))


# ====================================== section 9.6 audit: other decisive forms
class OtherDecisiveObservations(unittest.TestCase):
    def test_an_admitted_run_where_a_refusal_is_frozen_fails(self):
        admitted = receipt(process_disposition="ExecFailed",
                           exec_failed_stage="EXEC", exec_failed_errno=8)
        for case in ("E8", "X2", "X5", "X6", "X7"):
            plan = driver.CASE_PLANS[case]
            status, record = score(plan, obs_for(plan, spike=admitted, rep=None))
            self.assertEqual(status, FAIL, case)
            self.assertEqual(record["outcome"], "ExecFailed:ENOEXEC")
            status, _ = score(plan, obs_for(plan, rep=None,
                                            report_state=ob.REPORT_MALFORMED))
            self.assertEqual(status, INVALID, case)

    def test_a_refused_helper_in_a_stream_case_fails(self):
        plan = driver.CASE_PLANS["O1"]
        refused = {"admission": "refused", "refusal": "ElfNotInCohort",
                   "exec_reached": False}
        status, record = score(plan, obs_for(plan, spike=refused,
                                             expected_streams=plan.streams))
        self.assertEqual(status, FAIL)
        self.assertEqual(record["outcome"], "refused:ElfNotInCohort")

    def test_a_full_payload_that_is_not_the_recipe_fails(self):
        plan = driver.CASE_PLANS["T5"]
        status, record = score(plan, obs_for(plan, rep=None,
                                             payload_is_recipe=False,
                                             spike=receipt(exit_code=7)))
        self.assertEqual(status, FAIL)
        self.assertEqual(record["outcome"], "stream_mismatch")


class O3O6P4(unittest.TestCase):
    def o3(self, elapsed):
        plan = driver.CASE_PLANS["O3"]
        spike = receipt(**{name: {"bytes_drained": want["bytes"],
                                  "drained_sha256": want["sha256"],
                                  "completeness": "CompleteAtEof"}
                           for name, want in plan.streams.items()})
        return score(plan, obs_for(plan, spike=spike, payload_is_recipe=True,
                                   expected_streams=plan.streams,
                                   elapsed_ms=elapsed))

    def test_o3_duration_is_a_result(self):
        self.assertIsNone(driver.CASE_PLANS["O3"].posed_when)
        self.assertEqual(self.o3(9999)[0], PASS)
        status, record = self.o3(10000)
        self.assertEqual(status, FAIL)
        self.assertEqual(record["outcome"], "completion_over_bound")
        self.assertEqual(self.o3(None)[0], INVALID)

    def retained(self, case, alive, completeness):
        plan = driver.CASE_PLANS[case]
        want = oracles.stream_digest("stdout", 512)
        spike = receipt(stdout={"bytes_drained": 512, "drained_sha256": want,
                                "completeness": completeness})
        # P4 is posed by its post-launch rendezvous, O6 by the pre-armed fixture
        # signal (AB7-M2); each case reads only its own fact.
        return score(plan, obs_for(plan, spike=spike, payload_is_recipe=True,
                                   expected_streams=plan.streams or None,
                                   descendant_alive_after_launch=alive,
                                   fixture_descendant_signalled=alive))

    def test_o6_completeness_is_a_result_once_the_fixture_signalled(self):
        retained, eof = "WriterRetainedAfterChildExit", "CompleteAtEof"
        self.assertEqual(self.retained("O6", True, retained)[0], PASS)
        status, record = self.retained("O6", True, eof)
        self.assertEqual(status, FAIL)
        self.assertEqual(record["outcome"], "completeness_mismatch")
        self.assertEqual(self.retained("O6", False, eof)[0], INVALID)
        # ab74356 posed O6 from the receipt's own retained completeness; with no
        # fixture signal the fixture is not established (AB7-M2).
        self.assertEqual(self.retained("O6", False, retained)[0], INVALID)

    def test_p4_completeness_is_its_gate_once_retention_is_proven(self):
        retained, eof = "WriterRetainedAfterChildExit", "CompleteAtEof"
        self.assertEqual(self.retained("P4", True, retained)[0], PASS)
        self.assertEqual(self.retained("P4", True, eof)[0], FAIL)
        self.assertEqual(self.retained("P4", False, eof)[0], INVALID)


# ======================================================= I2: M2 control arm
class M2ControlArm(unittest.TestCase):
    plan = driver.CASE_PLANS["M2"]
    CALLS = ["dup2", "close_range", "execveat"]

    def m2(self, control, primary=None):
        def shape(n):
            return receipt(parent_shape={"extra_threads": n,
                                         "atfork_handler_registered": n > 0})
        trace = (control or {}).get("trace") if isinstance(control, dict) else None
        return score(self.plan, obs_for(
            self.plan, spike=shape(3), rep=None,
            trace={"child_syscalls": list(primary or self.CALLS)},
            baseline_observation=control, declared_launcher_threads=3,
            single_threaded_child_syscalls=(trace or {}).get("child_syscalls")))

    def control(self, **over):
        base = {"spike": receipt(parent_shape={"extra_threads": 0,
                                               "atfork_handler_registered": False}),
                "launch_returned": True, "elapsed_ms": 30,
                "trace": {"child_syscalls": list(self.CALLS)}}
        base.update(over)
        return base

    def test_both_arms_return_and_agree(self):
        self.assertEqual(self.m2(self.control())[0], PASS)

    def test_both_arms_return_and_differ(self):
        status, record = self.m2(self.control(), primary=["dup2", "execveat"])
        self.assertEqual(status, FAIL)
        self.assertEqual(record["outcome"], "differs_from_single_threaded_arm")

    def test_an_invoked_control_arm_that_does_not_return_fails(self):
        status, record = self.m2(self.control(launch_returned=False, spike=None,
                                              trace=None))
        self.assertEqual(status, FAIL)
        self.assertIs(record["launch_returned"], False)
        self.assertIn("control arm", record["reason"])
        self.assertIs(record["posing_evidence"]["control_arm"]["launch_returned"],
                      False)

    def test_a_control_arm_that_returns_late_fails(self):
        late = self.plan.total_bound_ms() + 1
        self.assertEqual(self.m2(self.control(elapsed_ms=late))[0], FAIL)

    def test_a_control_arm_never_posed_is_invalid(self):
        self.assertEqual(self.m2(self.control(not_posed="no tracer",
                                              launch_returned=None))[0], INVALID)
        self.assertEqual(self.m2(None)[0], INVALID)


# ============================================= M1: BLOCKED posing evidence
def roundtrip(case, status, reason, record, pose_started):
    tmp = pathlib.Path(tempfile.mkdtemp(prefix="final-journal-"))
    try:
        path = tmp / "journal.jsonl"
        sanitiser = evidence.public_sanitiser(build=str(tmp / "build"))
        with journal.Journal(path, sanitiser, trial="trial-002") as j:
            j.case_entered(case, 1)
            if pose_started:
                j.case_pose_started(case, 1)
            j.case_completed(case, status, reason, record)
        records = journal.read(path)[0]
        return journal.replay(records)["completed"][case]["record"]
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


class DurablePosingEvidenceEverywhere(unittest.TestCase):
    def test_every_blocked_record_carries_posing_evidence(self):
        for spec in fc.CASES:
            if spec["cls"] != fc.CONDITIONAL:
                continue
            case, cause = spec["case"], spec["blocked_if"]
            plan = driver.CASE_PLANS[case]
            record = driver.evaluate(plan, {"blocked": cause, "launch_returned": None})
            status, reason = checker.score_case(case, record)
            self.assertEqual(status, BLOCKED, case)
            self.assertEqual(record["posing_evidence"]["invocation"],
                             {"pose_started": False, "mechanism_invoked": False,
                              "blocked_cause": cause}, case)
            self.assertNotIn("forced_state", record["posing_evidence"], case)

    def test_blocked_posing_evidence_survives_the_journal(self):
        plan = driver.CASE_PLANS["X8"]
        record = driver.evaluate(plan, {"blocked": "no_noexec_mount",
                                        "launch_returned": None})
        status, reason = checker.score_case("X8", record)
        kept = roundtrip("X8", status, reason, record, pose_started=False)
        self.assertEqual(kept["posing_evidence"],
                         json.loads(json.dumps(record["posing_evidence"])))

    def test_pre_setup_and_posed_records_say_what_happened(self):
        plan = driver.CASE_PLANS["F1"]
        record = driver.evaluate(plan, {"not_posed": "the mechanism supplies no "
                                                     "channel for report",
                                        "launch_returned": None})
        self.assertEqual(record["posing_evidence"]["invocation"],
                         {"pose_started": False, "mechanism_invoked": False})
        record = driver.evaluate(plan, obs_for(plan, rep=report()))
        self.assertEqual(record["posing_evidence"]["invocation"],
                         {"pose_started": True, "mechanism_invoked": True})

    def test_all_200_repetitions_survive_the_journal(self):
        trials = [s_trial() for _ in range(200)]
        trials[5] = s_trial(**IMAGE_RAN)
        status, record = s7(trials)
        self.assertEqual(status, FAIL)
        kept = roundtrip("S7", status, "r", record, pose_started=True)
        self.assertEqual(len(kept["posing_evidence"]["repetitions"]), 200)
        self.assertEqual(kept["posing_evidence"]["repetitions"][5]["status"], FAIL)
        self.assertEqual(kept["posing_evidence"],
                         json.loads(json.dumps(record["posing_evidence"])))


# ============================= O7: the owner's retained-writer fixture decision
RETAINED, EOF = "WriterRetainedAfterChildExit", "CompleteAtEof"


def o7_obs(alive=True, exit_code=42, stderr=RETAINED, stdout=RETAINED,
           parseable=True):
    """A pose()-shaped O7 observation: helper_fork, no report, no payload."""
    plan = driver.CASE_PLANS["O7"]

    def block(completeness):
        out = {"bytes_drained": 0, "drained_sha256": oracles.digest_of(b"")}
        if completeness is not None:
            out["completeness"] = completeness
        return out
    obs = obs_for(plan, spike=receipt(exit_code=exit_code, stdout=block(stdout),
                                      stderr=block(stderr)),
                  rep=None, report_state=ob.REPORT_STREAM_INCOMPLETE,
                  fixture_descendant_signalled=alive,
                  build_identity_binding={"classification": "DIRECT_BASE",
                                          "base_artefact": "helper_fork",
                                          "object": "helper_fork", "bound": True,
                                          "object_sha256": "c" * 64})
    if not parseable:
        obs["spike"] = None
    # The live path hands the pre-armed fixture signal (AB7-B1) to
    # exec_confirmation; obs_for does not.
    obs["exec_confirmation"] = ob.exec_confirmation(
        obs["spike"], None, None, obs["report_state"], fixture_signalled=alive)
    return plan, obs


class O7RetainedWriterFixture(unittest.TestCase):
    def test_a_both_facts_on_an_established_fixture_pass(self):
        plan, obs = o7_obs()
        status, record = score(plan, obs)
        self.assertEqual(status, PASS)
        self.assertEqual(record["outcome"], "Exited:42")
        self.assertEqual(record["assertions"]["stderr_capture_failure_reported"]
                         ["result"], ob.ASSERTION_HOLDS)

    def test_b_exit_42_without_the_capture_failure_fails(self):
        status, record = score(*o7_obs(stderr=EOF))
        self.assertEqual(status, FAIL)
        self.assertEqual(record["outcome"], "capture_failure_not_reported")
        # The two receipt facts stay separate in the durable record.
        self.assertEqual(record["mechanism_outcome"], "Exited:42")

    def test_c_the_capture_failure_with_the_wrong_exit_fails(self):
        for code in (0, 7, 43):
            status, record = score(*o7_obs(exit_code=code))
            self.assertEqual(status, FAIL, code)
            self.assertEqual(record["outcome"], "Exited:%d" % code)
        self.assertEqual(score(*o7_obs(exit_code=0, stderr=EOF))[0], FAIL)

    def test_d_missing_or_uninterpretable_evidence_is_invalid(self):
        self.assertEqual(score(*o7_obs(stderr=None))[0], INVALID)
        self.assertEqual(score(*o7_obs(parseable=False))[0], INVALID)

    def test_e_an_unestablished_fixture_is_invalid_whatever_the_receipt_says(self):
        for alive in (False, None):
            status, record = score(*o7_obs(alive=alive))
            self.assertEqual(status, INVALID, alive)
            self.assertIn("fixture_descendant_signalled", record["not_posed"])

    def test_f_capture_completeness_is_not_a_posing_check(self):
        plan = driver.CASE_PLANS["O7"]
        # AB7-B1: the pre-armed fixture signal replaced fixture_descendant_alive.
        self.assertEqual(plan.posed_when, "fixture_descendant_signalled")
        self.assertEqual(driver.POSED_CHECK_READS[plan.posed_when],
                         ("fixture_descendant_signalled",))
        self.assertNotIn("stderr_capture_failed", driver.POSED_CHECKS)
        self.assertNotIn("fixture_descendant_alive", driver.POSED_CHECKS)
        node = function(DRIVER_TREE, "_check_fixture_descendant_signalled")
        # The executable statements only; the docstring may say what is NOT read.
        body = ast.unparse(ast.Module(body=node.body[1:], type_ignores=[]))
        self.assertNotIn("spike", body)
        self.assertNotIn("completeness", body)
        self.assertIn("fixture_descendant_signalled", body)
        self.assertEqual(plan.assertions, ("stderr_capture_failure_reported",))

    def test_g_the_plan_is_exactly_the_owner_fixture(self):
        plan = driver.CASE_PLANS["O7"]
        self.assertEqual(plan.binary, "helper_fork")
        self.assertEqual(plan.setup, "fork_helper_prearmed")
        self.assertEqual(plan.rule, "process_disposition")
        self.assertEqual(plan.helper_args,
                         ("--retain-stdio", "--parent-exit", "42",
                          "--lifetime-ms", str(fc.P_DESCENDANT_LIFETIME_MS)))
        self.assertEqual(plan.channels, (driver.CH_RECEIPT, driver.CH_LIVENESS))
        spec = fc.BY_NAME["O7"]
        self.assertEqual((spec["cls"], spec["predict"]), (fc.MANDATORY, "Exited:42"))

    def test_h_no_requirement_rests_on_a_4096_byte_stderr_payload(self):
        plan = driver.CASE_PLANS["O7"]
        self.assertEqual(plan.streams, {})
        self.assertNotIn(driver.CH_PAYLOAD, plan.channels)
        self.assertNotIn("4096", plan.helper_args)
        self.assertNotIn("--stderr", plan.helper_args)
        self.assertEqual(ob.ASSERTION_READS["stderr_capture_failure_reported"],
                         ("spike",))
        # Any drained byte count passes: O7 claims no recipe.
        plan, obs = o7_obs()
        obs["spike"]["stderr"]["bytes_drained"] = 12345
        self.assertEqual(score(plan, obs)[0], PASS)

    def test_stdout_retention_is_a_side_effect_not_a_verdict(self):
        self.assertEqual(score(*o7_obs(stdout=EOF))[0], PASS)

    def test_the_fixture_signal_is_exec_evidence_and_a_bad_payload_still_wins(self):
        spike = receipt(exit_code=42)
        incomplete = ob.REPORT_STREAM_INCOMPLETE
        self.assertEqual(ob.exec_confirmation(spike, None, None, incomplete,
                                              fixture_signalled=True),
                         ob.EXEC_REACHED)
        self.assertEqual(ob.exec_confirmation(spike, None, None, incomplete),
                         ob.EXEC_UNINTERPRETABLE)
        self.assertEqual(ob.exec_confirmation(spike, None, False, incomplete,
                                              fixture_signalled=True),
                         ob.EXEC_UNINTERPRETABLE)
        launch = ast.unparse(function(DRIVER_TREE, "_launch_and_observe"))
        self.assertIn("fixture_signalled=signalled", launch)

    def test_the_durable_record_shows_the_fixture_was_posed(self):
        plan, obs = o7_obs()
        record = driver.evaluate(plan, obs)
        evidence_block = record["posing_evidence"]
        self.assertEqual(evidence_block["fixture"],
                         {"binary": "helper_fork",
                          "helper_args": list(plan.helper_args)})
        self.assertEqual(evidence_block["build_identity_binding"]["base_artefact"],
                         "helper_fork")
        self.assertIs(evidence_block["measured"]["fixture_descendant_signalled"],
                      True)
        self.assertEqual(evidence_block["posed_check"],
                         {"name": "fixture_descendant_signalled", "held": True})
        kept = roundtrip("O7", PASS, "r", record, pose_started=True)
        self.assertEqual(kept["posing_evidence"],
                         json.loads(json.dumps(evidence_block)))


# ================================================= I3: preregistration
class PreregistrationAgrees(unittest.TestCase):
    POSING = {"same_inode_as_writer", "exec_status_pair_adjacent",
              "threaded_parent_observed", "retention_observed",
              "no_helper_report", "fixture_descendant_signalled"}

    def setUp(self):
        self.definition = (ROOT / "docs" / "experiments"
                           / "LAUNCH-EXEC-01-DEFINITION.md").read_text(encoding="utf-8")
        self.manifest = json.loads((EXP / "SOURCE-HASHES.json").read_text(
            encoding="utf-8"))

    def test_section_9_6_freezes_the_classification(self):
        self.assertIn("### 9.6 Final classification semantics", self.definition)
        for phrase in ("any FAIL makes the case FAIL", "exec_status_pair_adjacent",
                       "no_executed_image", "completed_under_ten_seconds",
                       "stream_completeness_as_declared", "retention_observed",
                       "O7 — frozen construction and interpretation"):
            self.assertIn(phrase, self.definition, phrase)

    def test_the_posing_checks_in_use_are_exactly_the_preregistered_ones(self):
        used = {p.posed_when for p in driver._PLAN_LIST if p.posed_when}
        self.assertEqual(used, self.POSING)
        inputs = self.manifest["posing_versus_showing"]["posed_check_inputs"]
        for name in used:
            self.assertEqual(tuple(inputs[name]), driver.POSED_CHECK_READS[name], name)
        for name in used:
            self.assertFalse(set(driver.POSED_CHECK_READS[name])
                             & {"elapsed_ms", "repeat_observations",
                                "descriptor_layout_adjacent"}, name)

    def test_the_manifest_assertions_are_the_implemented_ones(self):
        declared = self.manifest["posing_versus_showing"]["assertions"]
        self.assertEqual(set(declared), set(ob.ASSERTIONS))
        for name, entry in declared.items():
            users = sorted(p.case for p in driver._PLAN_LIST if name in p.assertions)
            self.assertEqual(sorted(entry["cases"]), users, name)
            self.assertEqual(tuple(entry["reads"]), ob.ASSERTION_READS[name], name)
            self.assertEqual(entry["violation_token"],
                             ob.ASSERTION_VIOLATION_TOKENS[name], name)

    def test_no_violation_token_is_any_cases_expectation(self):
        for token in ob.ASSERTION_VIOLATION_TOKENS.values():
            for spec in fc.CASES:
                self.assertNotEqual(token, spec["predict"])
                self.assertNotIn(token, spec["safe"] or ())

    def test_the_o7_owner_decision_is_frozen(self):
        finding = self.manifest["open_findings"]["O7-CONSTRUCTION"]
        self.assertTrue(finding.startswith("RESOLVED"), finding)
        self.assertIn("owner decision", finding)
        o7 = self.manifest["posing_versus_showing"]["o7"]
        self.assertIn("WriterRetainedAfterChildExit", o7["capture_failure_fact"])
        self.assertIn("never posing evidence", o7["posing_evidence"])
        self.assertEqual(o7["scoring"]["established, disposition other than "
                                       "Exited:42"], "FAIL")
        self.assertEqual(fc.BY_NAME["O7"]["predict"], "Exited:42")
        for phrase in ("helper_fork --retain-stdio --parent-exit 42",
                       "fixture_descendant_signalled",
                       "stderr_capture_failure_reported",
                       "fixture side-effect, not an O7 requirement"):
            self.assertIn(phrase, self.definition, phrase)


if __name__ == "__main__":
    unittest.main()
