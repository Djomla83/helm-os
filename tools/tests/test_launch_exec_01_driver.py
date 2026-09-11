"""Pre-trial self-tests for the LAUNCH-EXEC-01 case driver, mapping and sanitiser.

These validate the DRIVER TABLE, the P-12 observation-to-token mapping and the
P-14 sanitiser. Every observation here is fabricated in-process. Nothing in this
file builds, executes or otherwise invokes ``launcher_spike``, any helper, any
generated ELF or any preregistered case, and no test constructs a
``driver.Authorisation`` -- which is the object every posing path requires.
LAUNCH-EXEC-01 remains NOT_RUN.
"""
import ast
import contextlib
import io
import json
import os
import pathlib
import shutil
import subprocess
import sys
import tempfile
import unittest

EXP = (pathlib.Path(__file__).resolve().parents[2]
       / "docs" / "experiments" / "launch-exec-01")
if str(EXP) not in sys.path:
    sys.path.insert(0, str(EXP))

import checker              # noqa: E402
import driver               # noqa: E402
import evidence             # noqa: E402
import harness              # noqa: E402
import journal              # noqa: E402
import run_launch_exec_01 as runner   # noqa: E402
import frozen_cases as fc   # noqa: E402
import observations as ob   # noqa: E402
import oracles              # noqa: E402


# --------------------------------------------------------------- fabrication
def receipt(**over):
    """A fabricated ACCEPTED spike receipt with every frozen field present."""
    base = {
        "admission": "accepted",
        "pre_exec_body_sha256": "a" * 64,
        "pre_exec_body_size": 4096,
        "pre_exec_mode_bits": 0o755,
        "process_disposition": "Exited",
        "timeout_disposition": "",
        "exec_failed_stage": "",
        "exec_failed_errno": 0,
        "exit_code": 0,
        "term_signal": -1,
        "launcher_signal_issued": False,
        "group_sweep_issued": True,
        "wait_errno": 0,
        "stdout": {"bytes_drained": 0, "drained_sha256": oracles.digest_of(b""),
                   "completeness": "CompleteAtEof"},
        "stderr": {"bytes_drained": 0, "drained_sha256": oracles.digest_of(b""),
                   "completeness": "CompleteAtEof"},
        "environment_mode": "empty",
        "retained_prefix_not_in_receipt": {
            "stdout_kept": 0, "stdout_truncated": False,
            "stderr_kept": 0, "stderr_truncated": False, "bound": 65536},
        "elapsed_ms_not_in_receipt": 12,
    }
    base.update(over)
    return base


def refusal(token):
    return {"admission": "refused", "refusal": token, "exec_reached": False}


def report(**over):
    base = {
        "marker": "helper_report",
        "argv": [],
        "environ": [],
        "descriptors": [{"fd": 0, "cloexec": False}, {"fd": 1, "cloexec": False},
                        {"fd": 2, "cloexec": False}],
        "uid": 1001, "euid": 1001, "gid": 1001, "egid": 1001,
        "signals": {"SigBlk": "0000000000000000", "SigIgn": "0000000000000000",
                    "SigCgt": "0000000000000000", "SigPnd": "0000000000000000"},
        "no_new_privs": "1",
        "cwd": {"st_dev": 1, "st_ino": 2},
        "requested": {"stdout": 0, "stderr": 0, "exit": 0},
    }
    base.update(over)
    return base


def arm(attempted=True, pidfd=3, pidfd_open_errno=0, waitid_rc=0,
        waitid_errno=0, exit_status_observed=True, exit_status=0):
    """A fabricated M5 rejected-acquisition arm block."""
    return {"attempted": attempted, "pidfd": pidfd,
            "pidfd_open_errno": pidfd_open_errno, "waitid_rc": waitid_rc,
            "waitid_errno": waitid_errno,
            "exit_status_observed": exit_status_observed,
            "exit_status": exit_status}


def argv_of(*values):
    return [{"len": len(v.encode()), "value": v} for v in values]


_DEFAULT = object()


def observation(spike=_DEFAULT, rep=None, **over):
    """A fabricated observation. ``spike=None`` means "no parseable receipt",
    which is distinct from "not specified" and must stay distinguishable."""
    obs = {
        "spike": receipt() if spike is _DEFAULT else spike,
        "report": rep,
        # A report that is present is COMPLETE unless a test says otherwise;
        # absence defaults to the DECISIVE absence, because that is the state
        # S2/S5/S7 rest on and the one worth exercising by default.
        "report_state": (ob.REPORT_COMPLETE if rep is not None
                         else ob.REPORT_ABSENT),
        "launch_returned": True,
        "elapsed_ms": 12,
        "declared_pre_exec_stall": False,
        "declared_body_length_changed": False,
        "declared_injection_modes": [],
    }
    obs.update(over)
    obs["exec_confirmation"] = ob.exec_confirmation(
        obs["spike"], obs["report"], obs.get("payload_is_recipe"),
        obs.get("report_state"))
    return obs


# ============================================================== completeness
class DriverCompleteness(unittest.TestCase):
    def test_exactly_the_frozen_membership(self):
        result = driver.completeness()
        self.assertEqual(result["missing"], [], "cases with no driver handler")
        self.assertEqual(result["unknown"], [], "handlers for unknown cases")
        self.assertEqual(result["duplicates"], [], "duplicate case handlers")
        self.assertEqual(result["frozen_total"], 72)
        self.assertEqual(result["driver_total"], 72)
        self.assertTrue(result["complete"])

    def test_set_equality_is_exact(self):
        self.assertEqual(set(driver.DRIVER_CASE_IDS), set(fc.MEMBERSHIP))

    def test_every_named_hook_resolves(self):
        result = driver.completeness()
        for key in ("unresolved_setups", "unresolved_parents",
                    "unresolved_rules", "unresolved_checks",
                    "unknown_channels"):
            self.assertEqual(result[key], [], key)

    def test_partition_is_still_54_11_7(self):
        summary = fc.summary()
        self.assertEqual(summary["total"], 72)
        self.assertEqual(summary["mandatory"], 54)
        self.assertEqual(summary["conditional"], 11)
        self.assertEqual(summary["recorded"], 7)

    def test_n2_and_m3_remain_conditional_host_properties(self):
        self.assertEqual(fc.BY_NAME["N2"]["cls"], fc.CONDITIONAL)
        self.assertEqual(fc.BY_NAME["N2"]["blocked_if"], "parent_no_new_privs_set")
        self.assertEqual(fc.BY_NAME["M3"]["cls"], fc.CONDITIONAL)
        self.assertEqual(fc.BY_NAME["M3"]["blocked_if"], "clone3_unavailable")

    def test_a_missing_handler_is_detected(self):
        """The completeness check must FAIL when a case has no handler."""
        kept = driver.DRIVER_CASE_IDS
        try:
            driver.DRIVER_CASE_IDS = [c for c in kept if c != "R1"]
            result = driver.completeness()
            self.assertEqual(result["missing"], ["R1"])
            self.assertFalse(result["complete"])
        finally:
            driver.DRIVER_CASE_IDS = kept

    def test_a_duplicate_handler_is_detected(self):
        kept = driver.DRIVER_CASE_IDS
        try:
            driver.DRIVER_CASE_IDS = list(kept) + ["R1"]
            result = driver.completeness()
            self.assertEqual(result["duplicates"], ["R1"])
            self.assertFalse(result["complete"])
        finally:
            driver.DRIVER_CASE_IDS = kept

    def test_an_unknown_case_id_is_detected(self):
        kept = driver.DRIVER_CASE_IDS
        try:
            driver.DRIVER_CASE_IDS = list(kept) + ["Z9"]
            result = driver.completeness()
            self.assertEqual(result["unknown"], ["Z9"])
            self.assertFalse(result["complete"])
        finally:
            driver.DRIVER_CASE_IDS = kept

    def test_every_plan_builds_a_well_formed_invocation(self):
        for plan in driver._PLAN_LIST:
            argv = driver.spike_argv(plan, "/x/exec", "/x/work", "/x/build")
            self.assertTrue(argv[0].endswith("launcher_spike"))
            self.assertIn("--exec-path", argv)
            self.assertEqual(argv.count("--arg"), 1 + len(plan.helper_args))
            for item in argv:
                self.assertIsInstance(item, str)

    def test_traced_minimality_cases_declare_no_injection_mode(self):
        """M1, M2 and M4 trace the PRODUCTION configuration."""
        for name in ("M1", "M2", "M4"):
            self.assertEqual(
                driver.declared_injection_modes(driver.CASE_PLANS[name]), [],
                name + " must not carry a test-only injection mode")

    def test_injection_cases_declare_their_mode(self):
        self.assertIn("--die-before-exec",
                      driver.declared_injection_modes(driver.CASE_PLANS["S5"]))
        self.assertIn("--stall-pre-exec-ms",
                      driver.declared_injection_modes(driver.CASE_PLANS["S6"]))


class EvidenceChannels(unittest.TestCase):
    """The frozen mechanism's ability to deliver each declared observation."""

    def test_unposable_set_is_exactly_reported(self):
        un = driver.unposable_cases()
        # Every unposable case names at least one channel and a reason.
        for name, info in un.items():
            self.assertTrue(info["missing_channels"], name)
            self.assertTrue(info["reasons"], name)
            self.assertIn(name, fc.BY_NAME)

    def test_the_report_channel_is_now_supplied(self):
        """PRE-D7-B1 corrected: the launcher emits the retained prefix."""
        self.assertIn(driver.CH_REPORT, driver.SPIKE_SUPPLIED_CHANNELS)
        un = driver.unposable_cases()
        self.assertEqual(
            [n for n, i in un.items()
             if driver.CH_REPORT in i["missing_channels"]], [])

    def test_every_case_is_now_posable(self):
        """Owner amendment M3-T closed the last gap: M3 is a traced case."""
        self.assertEqual(driver.unposable_cases(), {})

    def test_the_retired_acquisition_channel_is_gone(self):
        """A self-asserted acquisition channel must not exist to be used."""
        self.assertFalse(hasattr(driver, "CH_ACQUISITION"))
        self.assertNotIn("acquisition", driver.ALL_CHANNELS)

    def test_removing_a_channel_makes_its_cases_unposable_again(self):
        """The gate must react to a channel loss, not to a hard-coded list."""
        supplied = set(driver.SPIKE_SUPPLIED_CHANNELS) - {driver.CH_REPORT}
        regressed = driver.unposable_cases(supplied)
        self.assertGreater(len(regressed), 30)
        self.assertIn("A1", regressed)
        self.assertIn("S5", regressed)

    def test_no_case_declares_an_unknown_channel(self):
        for plan in driver._PLAN_LIST:
            for channel in plan.channels:
                self.assertIn(channel, driver.ALL_CHANNELS, plan.case)


# ================================================================ P-12: tokens
class TokenUniverse(unittest.TestCase):
    def test_every_frozen_token_is_reachable_from_a_rule(self):
        """No frozen expectation may be a token no rule can ever produce."""
        universe = set()
        for spec in fc.CASES:
            if spec["predict"]:
                universe.add(spec["predict"])
            for member in spec["safe"] or ():
                universe.add(member)
        produced = set(PRODUCED_TOKENS)
        missing = sorted(universe - produced)
        self.assertEqual(missing, [],
                         "frozen tokens with no demonstrated derivation")

    def test_every_plan_names_a_defined_rule(self):
        for plan in driver._PLAN_LIST:
            self.assertIn(plan.rule, ob.RULES, plan.case)

    def test_unknown_rule_raises_rather_than_returning_a_token(self):
        with self.assertRaises(KeyError):
            ob.derive("no_such_rule", observation())


class SpikeParsing(unittest.TestCase):
    def test_malformed_json_is_not_a_receipt(self):
        self.assertIsNone(ob.parse_spike_stdout(b"{not json"))

    def test_empty_output_is_not_a_receipt(self):
        self.assertIsNone(ob.parse_spike_stdout(b""))
        self.assertIsNone(ob.parse_spike_stdout(None))

    def test_extra_lines_disqualify_the_receipt(self):
        good = json.dumps(receipt())
        self.assertIsNone(ob.parse_spike_stdout(good + "\nsomething else\n"))

    def test_unknown_disposition_is_rejected(self):
        self.assertIsNone(ob.parse_spike_stdout(
            json.dumps(receipt(process_disposition="Fabulous"))))

    def test_unknown_refusal_is_rejected(self):
        self.assertIsNone(ob.parse_spike_stdout(
            json.dumps({"admission": "refused", "refusal": "MadeUp",
                        "exec_reached": False})))

    def test_missing_required_field_is_rejected(self):
        broken = receipt()
        del broken["exit_code"]
        self.assertIsNone(ob.parse_spike_stdout(json.dumps(broken)))

    def test_unknown_stage_is_rejected(self):
        self.assertIsNone(ob.parse_spike_stdout(
            json.dumps(receipt(exec_failed_stage="NOWHERE"))))

    def test_a_valid_receipt_round_trips(self):
        parsed = ob.parse_spike_stdout(json.dumps(receipt()))
        self.assertEqual(parsed["process_disposition"], "Exited")


class MalformedAndUnknownObservations(unittest.TestCase):
    def test_no_receipt_yields_no_token(self):
        token, _ = ob.derive("process_disposition",
                             observation(spike=None, rep=report()))
        self.assertIsNone(token)

    def test_unknown_errno_yields_no_token_not_a_guess(self):
        spike = receipt(process_disposition="ExecFailed",
                        exec_failed_stage="EXEC", exec_failed_errno=4242)
        token, _ = ob.derive("process_disposition", observation(spike=spike))
        self.assertIsNone(token)

    def test_unknown_signal_yields_no_token(self):
        spike = receipt(process_disposition="Signaled", term_signal=250)
        token, _ = ob.derive("process_disposition",
                             observation(spike=spike, rep=report()))
        self.assertIsNone(token)

    def test_out_of_range_exit_code_yields_no_token(self):
        spike = receipt(exit_code=4096)
        token, _ = ob.derive("process_disposition",
                             observation(spike=spike, rep=report()))
        self.assertIsNone(token)

    def test_missing_information_never_becomes_a_pass_token(self):
        """The property that matters most: absence cannot manufacture success."""
        # Definition section 9.6 (F417-I1): a DECISIVE absence may now yield the
        # launcher's lifecycle token, which is contradictory evidence and never
        # a pass token; an absence the stream cannot settle still yields none.
        passing = {"argv_exact", "environ_empty", "fds_exactly_012",
                   "signals_reset", "no_new_privs_1", "no_new_privs_0",
                   "interpreter_ran_with_devfd"}
        for rule in ("argv_exact", "environ_empty", "fds_exactly_012",
                     "signals_reset", "no_new_privs",
                     "interpreter_ran_with_devfd"):
            token, _ = ob.derive(rule, observation(rep=None))
            self.assertNotIn(token, passing, rule + " invented a pass token")
            token, _ = ob.derive(rule, observation(
                rep=None, report_state=ob.REPORT_STREAM_INCOMPLETE))
            self.assertIsNone(token, rule + " invented a token from no report")

    def test_a_no_token_observation_becomes_not_posed_then_invalid(self):
        plan = driver.CASE_PLANS["R1"]
        record = driver.evaluate(plan, observation(spike=None))
        self.assertIn("not_posed", record)
        status, _ = checker.score_case("R1", record)
        self.assertEqual(status, checker.INVALID)


class ExecConfirmation(unittest.TestCase):
    """Section 11: the four states must stay distinguishable."""

    def test_explicit_pre_exec_error(self):
        spike = receipt(process_disposition="ExecFailed",
                        exec_failed_stage="EXEC", exec_failed_errno=13)
        obs = observation(spike=spike)
        self.assertEqual(obs["exec_confirmation"], ob.EXEC_PRE_EXEC_ERROR)
        token, _ = ob.derive("process_disposition", obs)
        self.assertEqual(token, "ExecFailed:EACCES")

    def test_chdir_stage_is_rendered_with_its_stage(self):
        spike = receipt(process_disposition="ExecFailed",
                        exec_failed_stage="CHDIR", exec_failed_errno=13)
        token, _ = ob.derive("process_disposition", observation(spike=spike))
        self.assertEqual(token, "ExecFailed:CHDIR:EACCES")

    def test_exec_reached_when_a_report_exists(self):
        obs = observation(rep=report())
        self.assertEqual(obs["exec_confirmation"], ob.EXEC_REACHED)
        token, _ = ob.derive("process_disposition", obs)
        self.assertEqual(token, "Exited:0")

    def test_status_pipe_eof_alone_is_never_exec(self):
        """S5 is byte-identical to S1 at the parent; only the report separates them."""
        s1 = observation(rep=report())
        s5 = observation(rep=None)
        self.assertEqual(s1["spike"], s5["spike"])
        self.assertEqual(ob.derive("process_disposition", s1)[0], "Exited:0")
        self.assertEqual(ob.derive("process_disposition", s5)[0],
                         "ExecStatusIndeterminate")

    def test_child_death_before_exec_is_indeterminate_not_signaled(self):
        spike = receipt(process_disposition="Signaled", term_signal=9,
                        exit_code=-1)
        token, _ = ob.derive("process_disposition",
                             observation(spike=spike, rep=None))
        self.assertEqual(token, "ExecStatusIndeterminate")

    def test_pre_exec_timeout_is_distinguished_by_the_declared_schedule(self):
        spike = receipt(process_disposition="ExecStatusIndeterminate")
        plain, _ = ob.derive("process_disposition", observation(spike=spike))
        stalled, _ = ob.derive(
            "process_disposition",
            observation(spike=spike, declared_pre_exec_stall=True))
        self.assertEqual(plain, "ExecStatusIndeterminate")
        self.assertEqual(stalled, "ExecStatusIndeterminate:PreExecTimeout")

    def test_a_full_payload_that_is_not_the_recipe_is_a_decisive_mismatch(self):
        # Definition section 9.6 (the F417 correction) replaced this test's
        # former expectation of no token: every declared stream was drained
        # and digested, so a payload that is not the recipe is contradictory
        # evidence, not missing evidence, and renders stream_mismatch.
        obs = observation(rep=None, payload_is_recipe=False)
        self.assertEqual(obs["exec_confirmation"], ob.EXEC_UNINTERPRETABLE)
        token, _ = ob.derive("process_disposition", obs)
        self.assertEqual(token, "stream_mismatch")

    def test_an_uninterpretable_report_still_yields_no_token(self):
        for state in (ob.REPORT_TRUNCATED, ob.REPORT_MALFORMED,
                      ob.REPORT_STREAM_INCOMPLETE):
            obs = observation(rep=None, report_state=state)
            self.assertEqual(obs["exec_confirmation"], ob.EXEC_UNINTERPRETABLE)
            self.assertIsNone(ob.derive("process_disposition", obs)[0], state)

    def test_payload_recipe_is_exec_evidence_for_the_no_report_series(self):
        obs = observation(rep=None, payload_is_recipe=True)
        self.assertEqual(obs["exec_confirmation"], ob.EXEC_REACHED)
        self.assertEqual(ob.derive("process_disposition", obs)[0], "Exited:0")

    def test_exit_status_unobservable_survives_without_a_report(self):
        spike = receipt(process_disposition="ExitStatusUnobservable",
                        wait_errno=10, exit_code=-1)
        token, _ = ob.derive("process_disposition",
                             observation(spike=spike, rep=None))
        self.assertEqual(token, "ExitStatusUnobservable")


class LifecycleTokens(unittest.TestCase):
    def test_exit_codes_render_exactly(self):
        for code in (0, 7, 9, 42, 127):
            spike = receipt(exit_code=code)
            token, _ = ob.derive("process_disposition",
                                 observation(spike=spike, rep=report()))
            self.assertEqual(token, "Exited:" + str(code))

    def test_signal_names_render_from_the_closed_table(self):
        for number, name in ((7, "SIGBUS"), (9, "SIGKILL"), (11, "SIGSEGV")):
            spike = receipt(process_disposition="Signaled", term_signal=number,
                            exit_code=-1)
            token, _ = ob.derive("process_disposition",
                                 observation(spike=spike, rep=report()))
            self.assertEqual(token, "Signaled:" + name)

    def test_exec_failed_errnos_render_from_the_closed_table(self):
        for number, name in ((2, "ENOENT"), (8, "ENOEXEC"), (13, "EACCES"),
                             (26, "ETXTBSY")):
            spike = receipt(process_disposition="ExecFailed",
                            exec_failed_stage="EXEC", exec_failed_errno=number)
            token, _ = ob.derive("process_disposition", observation(spike=spike))
            self.assertEqual(token, "ExecFailed:" + name)

    def test_timeout_renderings(self):
        cases = (
            ({"timeout_disposition": ""}, "TimedOut"),
            ({"timeout_disposition": "KilledByLauncher", "term_signal": 9},
             "TimedOut:KilledByLauncher:SIGKILL"),
            ({"timeout_disposition": "ExitedDuringGrace", "exit_code": 9},
             "TimedOut:ExitedDuringGrace:9"),
        )
        for over, expected in cases:
            spike = receipt(process_disposition="TimedOut", **over)
            token, _ = ob.derive("process_disposition",
                                 observation(spike=spike, rep=report()))
            self.assertEqual(token, expected)

    def test_changed_body_length_renders_as_mutated(self):
        obs = observation(rep=report(marker="\x7fELF-garbage"),
                          declared_body_length_changed=True)
        token, _ = ob.derive("process_disposition", obs)
        self.assertEqual(token, "Exited:mutated")

    def test_admission_refusals_render_exactly(self):
        for token in ("ElfNotInCohort", "NotRegularFile", "SetIdBitsPresent",
                      "DescriptorModeUnsuitable"):
            derived, _ = ob.derive("admission",
                                   observation(spike=refusal(token)))
            self.assertEqual(derived, "refused:" + token)

    def test_set_id_refusal_is_d9(self):
        derived, _ = ob.derive("admission",
                               observation(spike=refusal("SetIdBitsPresent")))
        self.assertEqual(derived, fc.BY_NAME["X7"]["predict"])

    def test_foreign_elf_refusal_is_e8(self):
        derived, _ = ob.derive("admission",
                               observation(spike=refusal("ElfNotInCohort")))
        self.assertEqual(derived, fc.BY_NAME["E8"]["predict"])

    def test_an_admitted_run_cannot_score_a_refusal_case(self):
        # Definition section 9.6: an ADMITTED run is a decisive observation, so
        # the rule now takes its lifecycle token instead of None. It still can
        # never score a refusal case as passing: no lifecycle token is a
        # refusal, and an uninterpretable lifecycle still yields no token.
        token, _ = ob.derive("admission", observation(rep=report()))
        self.assertEqual(token, "Exited:0")
        self.assertFalse(token.startswith("refused:"))
        token, _ = ob.derive("admission", observation(
            rep=None, report_state=ob.REPORT_MALFORMED))
        self.assertIsNone(token)


class ReportDerivedTokens(unittest.TestCase):
    def test_argv_exact_and_mismatch(self):
        good = observation(rep=report(argv=argv_of("helper_report", "a b c")),
                           expected_argv=["helper_report", "a b c"])
        self.assertEqual(ob.derive("argv_exact", good)[0], "argv_exact")
        bad = observation(rep=report(argv=argv_of("helper_report", "a b d")),
                          expected_argv=["helper_report", "a b c"])
        self.assertEqual(ob.derive("argv_exact", bad)[0], "argv_mismatch")

    def test_argv_empty_element_is_preserved(self):
        obs = observation(rep=report(argv=argv_of("helper_report", "")),
                          expected_argv=["helper_report", ""])
        self.assertEqual(ob.derive("argv_exact", obs)[0], "argv_exact")

    def test_argv_length_disagreement_is_uninterpretable(self):
        obs = observation(rep=report(argv=[{"len": 99, "value": "short"}]),
                          expected_argv=["short"])
        self.assertIsNone(ob.derive("argv_exact", obs)[0])

    def test_environ_empty_and_leak(self):
        empty = observation(rep=report(environ=[]))
        self.assertEqual(ob.derive("environ_empty", empty)[0], "environ_empty")
        leaked = observation(rep=report(environ=["HELM_LEAK_CANARY=x"]))
        self.assertEqual(ob.derive("environ_empty", leaked)[0],
                         "environ_inherited")

    def test_fds_exact_and_mismatch(self):
        exact = observation(rep=report())
        self.assertEqual(ob.derive("fds_exactly_012", exact)[0],
                         "fds_exactly_012")
        extra = observation(rep=report(descriptors=[
            {"fd": 0, "cloexec": False}, {"fd": 1, "cloexec": False},
            {"fd": 2, "cloexec": False}, {"fd": 7, "cloexec": False}]))
        self.assertEqual(ob.derive("fds_exactly_012", extra)[0], "fds_0_1_2_7")

    def test_stdio_left_cloexec_is_its_own_failure(self):
        """F6: the dup2(fd,fd) no-op the spike clears explicitly."""
        obs = observation(rep=report(descriptors=[
            {"fd": 0, "cloexec": False}, {"fd": 1, "cloexec": True},
            {"fd": 2, "cloexec": False}]))
        self.assertEqual(ob.derive("fds_exactly_012", obs)[0],
                         "stdio_cloexec_set")

    def test_excused_descriptor_is_removed_by_number(self):
        obs = observation(rep=report(descriptors=[
            {"fd": 0, "cloexec": False}, {"fd": 1, "cloexec": False},
            {"fd": 2, "cloexec": False}, {"fd": 9, "cloexec": True}]),
            excused_descriptors=(9,))
        self.assertEqual(ob.derive("fds_exactly_012", obs)[0], "fds_exactly_012")

    def test_signals_reset_and_inherited(self):
        clean = observation(rep=report())
        self.assertEqual(ob.derive("signals_reset", clean)[0], "signals_reset")
        blocked = observation(rep=report(signals={
            "SigBlk": "0000000000004000", "SigIgn": "0000000000000000",
            "SigCgt": "0", "SigPnd": "0"}))
        self.assertEqual(ob.derive("signals_reset", blocked)[0],
                         "signals_inherited")
        ignored = observation(rep=report(signals={
            "SigBlk": "0000000000000000", "SigIgn": "0000000000001000",
            "SigCgt": "0", "SigPnd": "0"}))
        self.assertEqual(ob.derive("signals_reset", ignored)[0],
                         "signals_inherited")

    def test_malformed_signal_mask_is_uninterpretable(self):
        obs = observation(rep=report(signals={
            "SigBlk": "not-a-mask", "SigIgn": "0", "SigCgt": "0", "SigPnd": "0"}))
        self.assertIsNone(ob.derive("signals_reset", obs)[0])

    def test_no_new_privs_both_arms(self):
        one = observation(rep=report(no_new_privs="1"))
        zero = observation(rep=report(no_new_privs="0"))
        self.assertEqual(ob.derive("no_new_privs", one)[0], "no_new_privs_1")
        self.assertEqual(ob.derive("no_new_privs", zero)[0], "no_new_privs_0")
        odd = observation(rep=report(no_new_privs="7"))
        self.assertIsNone(ob.derive("no_new_privs", odd)[0])

    def test_interpreter_devfd_both_ways(self):
        with_devfd = observation(rep=report(
            argv=argv_of("/bin/sh", "/dev/fd/3")))
        self.assertEqual(ob.derive("interpreter_ran_with_devfd", with_devfd)[0],
                         "interpreter_ran_with_devfd")
        without = observation(rep=report(argv=argv_of("/bin/sh", "script.sh")))
        self.assertEqual(ob.derive("interpreter_ran_with_devfd", without)[0],
                         "interpreter_ran_without_devfd")

    def test_privilege_transition_requires_nnp_and_unchanged_euid(self):
        good = observation(rep=report(uid=1001, euid=1001, no_new_privs="1"))
        self.assertEqual(
            ob.derive("privilege_transition_suppressed", good)[0],
            "privilege_transition_suppressed")
        raised = observation(rep=report(uid=1001, euid=0, no_new_privs="1"))
        self.assertEqual(
            ob.derive("privilege_transition_suppressed", raised)[0],
            "privilege_transition_occurred")
        no_nnp = observation(rep=report(uid=1001, euid=1001, no_new_privs="0"))
        self.assertIsNone(
            ob.derive("privilege_transition_suppressed", no_nnp)[0])


class StreamTokens(unittest.TestCase):
    def _stream_obs(self, length=4096, drained=None, completeness="CompleteAtEof",
                    stream="stdout"):
        drained = length if drained is None else drained
        spike = receipt(**{stream: {
            "bytes_drained": drained,
            "drained_sha256": oracles.stream_digest(stream, drained),
            "completeness": completeness}})
        want = {stream: {"bytes": length,
                         "sha256": oracles.stream_digest(stream, length),
                         "completeness": "CompleteAtEof"}}
        return observation(spike=spike, expected_streams=want)

    def test_stream_exact(self):
        self.assertEqual(ob.derive("stream_exact", self._stream_obs())[0],
                         "stream_exact")

    def test_short_stream_is_a_mismatch_not_a_pass(self):
        token, reason = ob.derive("stream_exact",
                                  self._stream_obs(4096, drained=2048))
        self.assertEqual(token, "stream_mismatch")
        self.assertIn("drained 2048", reason)

    def test_wrong_digest_is_a_mismatch(self):
        spike = receipt(stdout={"bytes_drained": 4096,
                                "drained_sha256": "b" * 64,
                                "completeness": "CompleteAtEof"})
        want = {"stdout": {"bytes": 4096,
                           "sha256": oracles.stream_digest("stdout", 4096),
                           "completeness": "CompleteAtEof"}}
        token, _ = ob.derive("stream_exact",
                             observation(spike=spike, expected_streams=want))
        self.assertEqual(token, "stream_mismatch")

    def test_writer_retained_completeness_is_not_complete_at_eof(self):
        token, reason = ob.derive(
            "stream_exact",
            self._stream_obs(completeness="WriterRetainedAfterChildExit"))
        self.assertEqual(token, "stream_mismatch")
        self.assertIn("WriterRetainedAfterChildExit", reason)

    def test_output_truncation_keeps_the_flag_out_of_the_receipt(self):
        """O4: the truncated flag describes the buffer, never the stream."""
        total = fc.MAX_CAPTURE_BYTES * 2
        spike = receipt(
            stdout={"bytes_drained": total,
                    "drained_sha256": oracles.stream_digest("stdout", total),
                    "completeness": "CompleteAtEof"},
            retained_prefix_not_in_receipt={
                "stdout_kept": fc.MAX_CAPTURE_BYTES, "stdout_truncated": True,
                "stderr_kept": 0, "stderr_truncated": False,
                "bound": fc.MAX_CAPTURE_BYTES})
        published = evidence.receipt_view(spike)
        self.assertNotIn("retained_prefix_not_in_receipt", published)
        self.assertNotIn("elapsed_ms_not_in_receipt", published)
        self.assertEqual(oracles.retained_prefix_length(total),
                         fc.MAX_CAPTURE_BYTES)

    def test_no_declared_streams_yields_no_token(self):
        self.assertIsNone(ob.derive("stream_exact", observation())[0])


class LifecycleAndTraceTokens(unittest.TestCase):
    def test_descendant_lifecycle_both_members(self):
        alive = observation(descendant_alive_after_launch=True)
        dead = observation(descendant_alive_after_launch=False)
        self.assertEqual(ob.derive("descendant_lifecycle", alive)[0],
                         "descendant_survived")
        self.assertEqual(ob.derive("descendant_lifecycle", dead)[0],
                         "descendant_died")
        self.assertIsNone(ob.derive("descendant_lifecycle", observation())[0])

    def test_sweep_both_members(self):
        issued = observation(spike=receipt(group_sweep_issued=True))
        not_issued = observation(spike=receipt(group_sweep_issued=False))
        self.assertEqual(ob.derive("sweep", issued)[0], "sweep_issued")
        self.assertEqual(ob.derive("sweep", not_issued)[0], "sweep_not_issued")

    def test_child_syscalls_within_and_outside_the_frozen_set(self):
        good = observation(trace={"child_syscalls": list(
            fc.CHILD_PERMITTED_SYSCALLS)})
        self.assertEqual(ob.derive("child_syscalls_within_frozen_set", good)[0],
                         "child_syscalls_within_frozen_set")
        bad = observation(trace={"child_syscalls": ["dup2", "mmap"]})
        token, reason = ob.derive("child_syscalls_within_frozen_set", bad)
        self.assertEqual(token, "child_syscalls_outside_frozen_set")
        self.assertIn("mmap", reason)

    def test_an_injection_syscall_in_a_production_trace_is_reported(self):
        obs = observation(trace={"child_syscalls": ["dup2", "nanosleep"]})
        token, reason = ob.derive("child_syscalls_within_frozen_set", obs)
        self.assertEqual(token, "child_syscalls_outside_frozen_set")
        self.assertIn("test-only injections", reason)

    def test_a_declared_injection_run_cannot_evidence_minimality(self):
        obs = observation(trace={"child_syscalls": ["dup2"]},
                          declared_injection_modes=["--die-before-exec"])
        self.assertIsNone(ob.derive("child_syscalls_within_frozen_set", obs)[0])

    def test_missing_trace_yields_no_token(self):
        self.assertIsNone(
            ob.derive("child_syscalls_within_frozen_set", observation())[0])

    def test_single_threaded_comparison(self):
        same = observation(trace={"child_syscalls": ["dup2", "execveat"]},
                           single_threaded_child_syscalls=["dup2", "execveat"])
        self.assertEqual(ob.derive("identical_to_single_threaded_arm", same)[0],
                         "identical_to_single_threaded_arm")
        differs = observation(trace={"child_syscalls": ["dup2"]},
                              single_threaded_child_syscalls=["dup2", "execveat"])
        self.assertEqual(
            ob.derive("identical_to_single_threaded_arm", differs)[0],
            "differs_from_single_threaded_arm")
        self.assertIsNone(ob.derive(
            "identical_to_single_threaded_arm",
            observation(trace={"child_syscalls": ["dup2"]}))[0])

    def test_stage_sequence_order_is_enforced(self):
        good = observation(observed_stage_sequence=list(fc.STAGES))
        self.assertEqual(
            ob.derive("sequence_matches_frozen_stages", good)[0],
            "sequence_matches_frozen_stages")
        swapped = list(fc.STAGES)
        i, j = swapped.index("CHDIR"), swapped.index("CLOSE_RANGE")
        swapped[i], swapped[j] = swapped[j], swapped[i]
        bad = observation(observed_stage_sequence=swapped)
        self.assertEqual(ob.derive("sequence_matches_frozen_stages", bad)[0],
                         "sequence_differs_from_frozen_stages")

    def test_no_new_privs_must_precede_exec(self):
        seq = [s for s in fc.STAGES if s not in ("NO_NEW_PRIVS", "EXEC")]
        seq += ["EXEC", "NO_NEW_PRIVS"]
        obs = observation(observed_stage_sequence=seq)
        self.assertEqual(ob.derive("sequence_matches_frozen_stages", obs)[0],
                         "sequence_differs_from_frozen_stages")

    def test_unknown_stage_is_uninterpretable(self):
        obs = observation(observed_stage_sequence=["DUP2", "TELEPORT"])
        self.assertIsNone(ob.derive("sequence_matches_frozen_stages", obs)[0])

    def test_rejected_acquisition_outcomes(self):
        """M5 derives from the arm's RAW errnos, never from a self-named token."""
        cases = (
            ({"pidfd_open_errno": ob.ESRCH}, "pidfd_open_esrch"),
            ({"pidfd_open_errno": 0, "waitid_errno": ob.ECHILD},
             "waitid_echild"),
            ({"pidfd_open_errno": 0, "waitid_errno": 0},
             "pidfd_open_succeeded"),
        )
        for over, expected in cases:
            obs = observation(spike=receipt(rejected_acquisition_arm=arm(**over)))
            self.assertEqual(ob.derive("rejected_acquisition", obs)[0], expected)

    def test_rejected_arm_not_run_yields_no_token(self):
        obs = observation(spike=receipt(
            rejected_acquisition_arm=arm(attempted=False)))
        self.assertIsNone(ob.derive("rejected_acquisition", obs)[0])
        self.assertIsNone(ob.derive("rejected_acquisition", observation())[0])

    def test_rejected_arm_unexpected_errno_yields_no_token(self):
        obs = observation(spike=receipt(
            rejected_acquisition_arm=arm(pidfd_open_errno=13)))
        token, reason = ob.derive("rejected_acquisition", obs)
        self.assertIsNone(token)
        self.assertIn("EACCES", reason)


# ============================================================ host conditions
class HostConditions(unittest.TestCase):
    def test_n2_blocks_when_the_parent_already_has_no_new_privs(self):
        ctx = _ctx(block_reasons={"parent_no_new_privs_set":
                                  fc.BLOCK_REASONS["parent_no_new_privs_set"]})
        cause = driver.blocked_cause_for(driver.CASE_PLANS["N2"], ctx)
        self.assertEqual(cause, "parent_no_new_privs_set")
        record = driver.evaluate(driver.CASE_PLANS["N2"],
                                 {"blocked": cause, "launch_returned": None})
        status, _ = checker.score_case("N2", record)
        self.assertEqual(status, checker.BLOCKED)

    def test_n2_is_posed_when_the_parent_bit_is_clear(self):
        ctx = _ctx(block_reasons={})
        self.assertIsNone(driver.blocked_cause_for(driver.CASE_PLANS["N2"], ctx))

    def test_m3_blocks_when_clone3_is_unavailable(self):
        ctx = _ctx(block_reasons={"clone3_unavailable":
                                  fc.BLOCK_REASONS["clone3_unavailable"]})
        cause = driver.blocked_cause_for(driver.CASE_PLANS["M3"], ctx)
        self.assertEqual(cause, "clone3_unavailable")
        record = driver.evaluate(driver.CASE_PLANS["M3"],
                                 {"blocked": cause, "launch_returned": None})
        self.assertEqual(checker.score_case("M3", record)[0], checker.BLOCKED)

    def test_a_mandatory_case_never_absorbs_a_block(self):
        """P-3: only a conditional case may absorb an environment block."""
        ctx = _ctx(block_reasons={"euid_zero": fc.BLOCK_REASONS["euid_zero"]})
        self.assertIsNone(driver.blocked_cause_for(driver.CASE_PLANS["R1"], ctx))

    def test_a_conditional_case_only_absorbs_its_own_cause(self):
        ctx = _ctx(block_reasons={"euid_zero": fc.BLOCK_REASONS["euid_zero"]})
        self.assertIsNone(driver.blocked_cause_for(driver.CASE_PLANS["M3"], ctx))


def _ctx(block_reasons):
    return driver.TrialContext(build="/x/build", work="/x/work",
                               preflight={"block_reasons": block_reasons},
                               freeze={}, sanitiser=evidence.Sanitiser())


# ================================================================ aggregation
class RecordsAndAggregate(unittest.TestCase):
    def test_missing_launch_returned_is_invalid(self):
        record = {"outcome": "Exited:0"}
        self.assertEqual(checker.score_case("R1", record)[0], checker.INVALID)

    def test_driver_records_always_carry_launch_returned(self):
        for plan in driver._PLAN_LIST:
            record = driver.evaluate(plan, observation(rep=report()))
            self.assertIn("launch_returned", record, plan.case)

    def test_a_non_return_is_a_fail_not_a_missing_record(self):
        plan = driver.CASE_PLANS["T1"]
        record = driver.evaluate(plan, {"launch_returned": False,
                                        "elapsed_ms": 999999})
        self.assertIs(record["launch_returned"], False)
        self.assertEqual(checker.score_case("T1", record)[0], checker.FAIL)

    def test_a_missing_case_is_invalid_not_dropped(self):
        records = {n: {"outcome": fc.BY_NAME[n]["predict"]
                       or fc.BY_NAME[n]["safe"][0], "launch_returned": True}
                   for n in fc.MEMBERSHIP}
        for name in fc.MEMBERSHIP:
            if fc.BY_NAME[name]["gates"]:
                records[name]["gates"] = {g: True
                                          for g in fc.BY_NAME[name]["gates"]}
            if name in fc.DOCUMENTATION_GATES:
                records[name]["documentation_gate"] = True
            if fc.BY_NAME[name]["traced"]:
                records[name]["trace"] = {"child_syscalls": ["dup2", "execveat"], "integrity_ok": True}
        del records["R1"]
        statuses = checker.score_all(records)
        self.assertEqual(statuses["R1"][0], checker.INVALID)
        aggregate, _ = checker.verdict(statuses)
        self.assertEqual(aggregate, checker.INCONCLUSIVE)

    def test_mandatory_blocked_makes_the_run_inconclusive(self):
        records = _all_passing()
        records["R1"] = {"blocked": "euid_zero", "launch_returned": None}
        statuses = checker.score_all(records)
        self.assertEqual(statuses["R1"][0], checker.INVALID)
        self.assertEqual(checker.verdict(statuses)[0], checker.INCONCLUSIVE)

    def test_conditional_blocked_with_its_cause_still_accepts(self):
        records = _all_passing()
        records["N2"] = {"blocked": "parent_no_new_privs_set",
                         "launch_returned": None}
        statuses = checker.score_all(records)
        self.assertEqual(statuses["N2"][0], checker.BLOCKED)
        aggregate, detail = checker.verdict(statuses)
        self.assertEqual(aggregate, checker.ACCEPTED)
        self.assertIn("N2", detail["conditional_blocked_with_cause"])

    def test_an_unposable_case_would_still_make_a_trial_inconclusive(self):
        """Nothing is unposable now, so the consequence is asserted directly."""
        self.assertEqual(driver.unposable_cases(), {})
        records = _all_passing()
        records["M3"] = {"not_posed": "no evidence channel",
                         "launch_returned": None}
        aggregate, _ = checker.verdict(checker.score_all(records))
        self.assertEqual(aggregate, checker.INCONCLUSIVE)

    def test_documentation_gate_is_computed_from_the_receipt_field_names(self):
        self.assertTrue(driver._documentation_gate(receipt()))
        self.assertFalse(driver._documentation_gate(
            receipt(executed_body_sha256="c" * 64)))
        self.assertFalse(driver._documentation_gate({"admission": "refused"}))

    def test_a_receipt_that_regained_a_duration_is_flagged(self):
        self.assertIsNone(driver._asserted_unobserved_fact(receipt()))
        self.assertIsNotNone(
            driver._asserted_unobserved_fact(receipt(elapsed_ms=5)))


def _all_passing():
    records = {}
    for name in fc.MEMBERSHIP:
        spec = fc.BY_NAME[name]
        record = {"outcome": spec["predict"] or spec["safe"][0],
                  "launch_returned": True}
        if spec["traced"]:
            record["trace"] = {"child_syscalls": ["dup2", "execveat"], "integrity_ok": True}
        if spec["gates"]:
            record["gates"] = {g: True for g in spec["gates"]}
        if name in fc.DOCUMENTATION_GATES:
            record["documentation_gate"] = True
        records[name] = record
    return records


# ============================================================== P-14: privacy
class Sanitisation(unittest.TestCase):
    def setUp(self):
        self.s = evidence.Sanitiser(work="/home/runner/work/helm",
                                    home="/home/runner", user="runner",
                                    build="/home/runner/work/helm/target/x")

    def test_roots_are_replaced_longest_first(self):
        text = "/home/runner/work/helm/target/x/helper_report"
        self.assertEqual(self.s.text(text), "<BUILD>/helper_report")
        self.assertEqual(self.s.text("/home/runner/work/helm/a"), "<WORK>/a")
        self.assertEqual(self.s.text("/home/runner/.cache"), "<HOME>/.cache")

    def test_a_username_in_free_text_is_NOT_substring_redacted(self):
        """V-5. A username is private as a path component or an identity
        field, not as a run of letters inside arbitrary evidence."""
        self.assertEqual(self.s.text("owned by runner"), "owned by runner")
        self.assertEqual(self.s.text("truncated"), "truncated")

    def test_an_explicit_host_identity_field_is_redacted_whole(self):
        out = self.s.record({"user": "runner", "hostname": "vm-7",
                             "owner": "runner"})
        self.assertEqual(out, {"user": evidence.HOST_IDENTITY,
                               "hostname": evidence.HOST_IDENTITY,
                               "owner": evidence.HOST_IDENTITY})

    def test_a_username_path_COMPONENT_is_redacted(self):
        s = evidence.Sanitiser(user="runner")
        self.assertEqual(s.text("/usr/lib/runner/x"), "/usr/lib/<USER>/x")
        self.assertEqual(s.text("/usr/lib/runner-tools/x"),
                         "/usr/lib/runner-tools/x")

    def test_linux_home_of_another_account_is_not_republished(self):
        self.assertEqual(self.s.text("/home/someoneelse/secrets.txt"),
                         "<ABSPATH>")

    def test_windows_paths_are_redacted(self):
        out = self.s.text(r"C:\Users\djoml\AppData\Local\Temp\thing")
        self.assertNotIn("djoml", out)
        self.assertIn("<ABSPATH>", out)

    def test_temp_paths_are_redacted(self):
        self.assertEqual(self.s.text("/tmp/build-1234/helper"), "<TMPPATH>")
        self.assertEqual(self.s.text("/var/tmp/x/y"), "<TMPPATH>")

    def test_system_paths_are_preserved_as_evidence(self):
        for path in ("/lib64/ld-linux-x86-64.so.2", "/proc/self/status",
                     "/usr/bin/strace", "/dev/fd/3"):
            self.assertEqual(self.s.text(path), path)

    def test_credentials_are_redacted(self):
        for token in ("ghp_" + "A" * 36,
                      "github_pat_" + "B" * 30,
                      "xoxb-1234567890-abcdefghij",
                      "AKIAIOSFODNN7EXAMPLE",
                      "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.abcd"):
            out = self.s.text("value=" + token)
            self.assertNotIn(token, out, token)

    def test_bearer_headers_are_redacted(self):
        out = self.s.text("Authorization: Bearer abcdefghijklmnop123456")
        self.assertNotIn("abcdefghijklmnop123456", out)

    def test_long_opaque_runs_are_redacted(self):
        out = self.s.text("ACTIONS_RUNTIME_TOKEN_" + "z" * 40)
        self.assertIn("<OPAQUE:", out)

    def test_digests_survive_because_they_are_the_evidence(self):
        digest = "a" * 64
        self.assertEqual(self.s.text(digest), digest)
        self.assertEqual(self.s.text("b" * 40), "b" * 40)

    def test_declared_env_names_survive_but_values_never_do(self):
        self.assertEqual(self.s.env_name("PATH"), "PATH")
        self.assertTrue(self.s.env_name("ACTIONS_RUNTIME_TOKEN")
                        .startswith("<UNDECLARED:"))
        entry = self.s.env_entry("PATH=/usr/bin:/bin")
        self.assertTrue(entry.startswith("PATH=<VALUE:len=13,"))
        self.assertNotIn("/usr/bin", entry)

    def test_an_undeclared_value_is_never_reproduced(self):
        secret = "ghp_" + "Q" * 36
        entry = self.s.env_entry("ACTIONS_RUNTIME_TOKEN=" + secret)
        self.assertNotIn(secret, entry)
        self.assertTrue(entry.startswith("<UNDECLARED:"))

    def test_malformed_env_entry_is_not_reproduced(self):
        out = self.s.env_entry("no-equals-sign-here")
        self.assertTrue(out.startswith("<MALFORMED_ENV:"))

    def test_environ_arrays_are_sanitised_at_any_depth(self):
        record = {"cases": {"V1": {"report": {"environ": ["HOME=/home/runner"]}}}}
        out = self.s.record(record)
        entry = out["cases"]["V1"]["report"]["environ"][0]
        self.assertTrue(entry.startswith("HOME=<VALUE:"))
        self.assertNotIn("/home/runner", entry)

    def test_keys_are_immutable(self):
        """V-5. Keys are structural identifiers, never rewritten."""
        record = {"stage_sequence": 1, "truncated": 2,
                  "child_syscalls": ["clone3"],
                  "nested": {"specific": {"latest_status": 3}}}

        def keys(value):
            found = set()
            if isinstance(value, dict):
                for k, v in value.items():
                    found.add(k)
                    found |= keys(v)
            elif isinstance(value, list):
                for item in value:
                    found |= keys(item)
            return found

        for user in ("ci", "run", "test", "u", "id", "exec", "clone", "wait"):
            s = evidence.Sanitiser(work="/home/%s/w" % user,
                                   home="/home/" + user, user=user)
            out = s.record(record)
            self.assertEqual(keys(out), keys(record), "username " + user)

    def test_the_only_key_level_operation_is_internal_withholding(self):
        out = self.s.record({"capture_prefix_base64": "AAAA", "keep": 1})
        self.assertEqual(sorted(out), ["capture_prefix_base64", "keep"])
        self.assertEqual(out["capture_prefix_base64"], evidence.WITHHELD)


class FixedVocabularyIsNeverRewritten(unittest.TestCase):
    """V-5 section 6. Adversarial usernames against the frozen schema."""

    USERNAMES = ("ci", "run", "test", "id", "pid", "fd", "exec", "clone",
                 "wait", "user", "root", "u", "a")

    def _vocabulary(self):
        words = {"specific", "truncated", "latest_status", "child_syscalls",
                 "stage_sequence", "clone3", "waitid", "pidfd_open",
                 "ExecStatusIndeterminate", "WriterRetainedAfterChildExit",
                 "DIRECT_CHILD", "DIRECT_CHILD_PIDFD", "execution",
                 "stream_incomplete", "CompleteAtEof", "integrity_ok",
                 "observed_success", "observed_error", "not_observed"}
        # Straight from the frozen schema, so a token added there is covered.
        for case in fc.CASES:
            words.add(case["case"])
            if case["predict"]:
                words.add(case["predict"])
            words.update(case["safe"] or ())
            words.update(case["gates"])
        words.update(fc.STAGES)
        words.update(fc.BLOCK_REASONS)
        words.update(fc.CHILD_PERMITTED_SYSCALLS)
        words.update(fc.CHILD_FORBIDDEN_SYSCALLS)
        return sorted(words)

    def test_every_frozen_token_survives_every_adversarial_username(self):
        vocabulary = self._vocabulary()
        self.assertGreater(len(vocabulary), 60)
        for user in self.USERNAMES:
            s = evidence.Sanitiser(work="/home/%s/w" % user,
                                   home="/home/" + user, user=user)
            for token in vocabulary:
                self.assertEqual(s.text(token), token,
                                 "username %r corrupted %r" % (user, token))

    def test_the_generic_opaque_rule_spares_frozen_vocabulary(self):
        """WriterRetainedAfterChildExit is exactly 28 characters."""
        s = evidence.Sanitiser()
        self.assertEqual(len("WriterRetainedAfterChildExit"), 28)
        self.assertEqual(s.text("WriterRetainedAfterChildExit"),
                         "WriterRetainedAfterChildExit")
        self.assertEqual(s.text("never_reports_unobserved_exit_status"),
                         "never_reports_unobserved_exit_status")

    def test_a_genuinely_opaque_run_is_still_redacted(self):
        s = evidence.Sanitiser()
        self.assertIn("<OPAQUE:", s.text("Zk9" + "q7Lm2Xv" * 5))

    def test_serialised_evidence_is_deterministic_under_any_username(self):
        document = {"b": {"a": "/home/ci/x"}, "a": ["/tmp/ci/y"],
                    "tok": "truncated"}
        for user in self.USERNAMES:
            s = evidence.Sanitiser(work="/home/%s/w" % user,
                                   home="/home/" + user, user=user)
            first = evidence.serialise(s.record(document))
            second = evidence.serialise(s.record(document))
            self.assertEqual(first, second, user)


class HostIdentityStillRedacted(unittest.TestCase):
    """V-5 section 7. The fix must not weaken P-14."""

    def setUp(self):
        self.s = evidence.Sanitiser(work="/home/alice/work/helm",
                                    home="/home/alice",
                                    build="/home/alice/work/helm/target/x",
                                    user="alice")

    def test_host_paths_are_still_redacted(self):
        cases = {
            "/home/alice/.ssh/id_ed25519": "<HOME>",
            "/home/bob/secret.txt": "<ABSPATH>",
            "/tmp/alice-build-123/helper": "<TMPPATH>",
            "/home/alice/work/helm/target/x/h": "<BUILD>",
        }
        for raw, expected in cases.items():
            out = self.s.text(raw)
            self.assertIn(expected, out, raw)
            self.assertNotIn("alice", out, raw)
            self.assertNotIn("bob", out, raw)

    def test_a_windows_profile_path_is_still_redacted(self):
        out = self.s.text(r"C:\Users\alice\AppData\Local\Temp\x")
        self.assertNotIn("alice", out)
        self.assertIn("<ABSPATH>", out)

    def test_credentials_are_still_redacted(self):
        for token in ("ghp_" + "A" * 36, "AKIAIOSFODNN7EXAMPLE",
                      "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJ4In0.sig12345"):
            self.assertNotIn(token, self.s.text("v=" + token))

    def test_environment_values_are_still_never_reproduced(self):
        out = self.s.record({"environ": ["ACTIONS_RUNTIME_TOKEN=ghp_" + "B" * 36,
                                         "HOME=/home/alice"]})
        text = evidence.serialise(out)
        self.assertNotIn("ghp_B", text)
        self.assertNotIn("/home/alice", text)
        self.assertIn("<VALUE:", text)

    def test_child_pid_remains_withheld(self):
        out = self.s.record({"trace": {"child_pid": 31337,
                                       "child_syscalls": ["clone3"]}})
        self.assertEqual(out["trace"]["child_pid"], evidence.WITHHELD)
        self.assertNotIn("31337", evidence.serialise(out))

    def test_raw_internal_fields_remain_withheld(self):
        payload = {k: "sensitive" for k in evidence.INTERNAL_ONLY_KEYS}
        out = self.s.record(payload)
        for key in evidence.INTERNAL_ONLY_KEYS:
            self.assertEqual(out[key], evidence.WITHHELD, key)

    def test_the_fix_did_not_simply_disable_user_redaction(self):
        """A username as a real path component must still go."""
        self.assertNotIn("alice", self.s.text("/usr/lib/alice/plugin.so"))

    def test_non_strings_pass_through_unchanged(self):
        record = {"n": 42, "b": True, "none": None, "f": 1.5}
        self.assertEqual(self.s.record(record), record)

    def test_serialisation_is_deterministic(self):
        doc = {"b": 2, "a": [3, 1], "c": {"z": 1, "y": 2}}
        self.assertEqual(evidence.serialise(doc), evidence.serialise(doc))
        self.assertEqual(json.loads(evidence.serialise(doc)), doc)

    def test_serialisation_adds_no_timestamp_or_hostname(self):
        text = evidence.serialise({"a": 1})
        for banned in ("timestamp", "hostname", "generated_at"):
            self.assertNotIn(banned, text)


# ============================================================ safety barriers
class SafetyBarriers(unittest.TestCase):
    def test_authorisation_cannot_be_constructed_without_the_owner_flag(self):
        for value in (False, None, 0, "yes", 1):
            with self.assertRaises(PermissionError):
                driver.Authorisation(value)

    def test_observe_refuses_without_an_authorisation(self):
        ctx = _ctx({})
        for bogus in (None, True, object()):
            with self.assertRaises(PermissionError):
                driver.observe(driver.CASE_PLANS["R1"], ctx, bogus)

    def test_this_suite_constructs_no_authorisation(self):
        """A guard against a future edit quietly enabling execution here.

        The forbidden literal is assembled at runtime so that this test does not
        itself contain the text it is looking for.
        """
        source = pathlib.Path(__file__).read_text(encoding="utf-8")
        forbidden = "Authorisation" + "(True)"
        self.assertNotIn(forbidden, source)
        self.assertNotIn("driver.observe(", source.split("test_observe_refuses")[0])

    def test_no_module_under_test_spawns_at_import(self):
        for module in (ob, evidence, driver):
            self.assertIsNotNone(module.__doc__)
            self.assertIn("NOT_RUN", module.__doc__)


# ================================================= PRE-D7-B1: capture channel
def encode(raw):
    import base64
    return base64.b64encode(raw).decode("ascii")


def stream_with(raw, drained=None, completeness="CompleteAtEof",
                truncated=False, declared_length=None):
    """A stream block carrying a retained capture prefix, as the spike emits it."""
    drained = len(raw) if drained is None else drained
    return {
        "bytes_drained": drained,
        "drained_sha256": oracles.digest_of(raw),
        "completeness": completeness,
        "capture_prefix_length": (len(raw) if declared_length is None
                                  else declared_length),
        "capture_prefix_truncated": truncated,
        "capture_prefix_base64": encode(raw),
    }


def report_bytes(payload=b"", **over):
    return payload + ob.REPORT_SENTINEL + json.dumps(report(**over)).encode()


class CaptureChannel(unittest.TestCase):
    def test_a_valid_capture_decodes(self):
        self.assertEqual(ob.decode_capture(stream_with(b"hello")), b"hello")

    def test_an_absent_field_is_not_an_empty_capture(self):
        self.assertIsNone(ob.decode_capture({"bytes_drained": 0}))
        self.assertEqual(ob.decode_capture(stream_with(b"")), b"")

    def test_invalid_base64_decodes_to_nothing(self):
        block = stream_with(b"x")
        block["capture_prefix_base64"] = "not!valid!base64"
        self.assertIsNone(ob.decode_capture(block))

    def test_a_length_that_disagrees_with_the_bytes_is_rejected(self):
        """The launcher's own count and its own bytes must agree."""
        block = stream_with(b"hello", declared_length=99)
        self.assertIsNone(ob.decode_capture(block))

    def test_binary_bytes_survive_the_encoding(self):
        raw = bytes(range(256)) * 4
        self.assertEqual(ob.decode_capture(stream_with(raw)), raw)


class HelperReportStates(unittest.TestCase):
    """Section 3: the five states must stay distinguishable."""

    def test_complete(self):
        state, rep, payload = ob.helper_report_state(
            stream_with(report_bytes()))
        self.assertEqual(state, ob.REPORT_COMPLETE)
        self.assertEqual(rep["marker"], "helper_report")
        self.assertEqual(payload, b"")

    def test_absent_when_the_stream_is_complete(self):
        state, rep, _ = ob.helper_report_state(stream_with(b"payload only"))
        self.assertEqual(state, ob.REPORT_ABSENT)
        self.assertIsNone(rep)

    def test_truncated_when_the_prefix_hit_the_bound(self):
        raw = report_bytes()[:len(ob.REPORT_SENTINEL) + 20]
        state, rep, _ = ob.helper_report_state(
            stream_with(raw, drained=9999, truncated=True))
        self.assertEqual(state, ob.REPORT_TRUNCATED)
        self.assertIsNone(rep)

    def test_malformed_when_the_stream_was_complete_and_it_did_not_parse(self):
        raw = ob.REPORT_SENTINEL + b"{not json"
        state, rep, _ = ob.helper_report_state(stream_with(raw))
        self.assertEqual(state, ob.REPORT_MALFORMED)
        self.assertIsNone(rep)

    def test_a_report_without_a_marker_is_malformed(self):
        raw = ob.REPORT_SENTINEL + json.dumps({"argv": []}).encode()
        state, _, _ = ob.helper_report_state(stream_with(raw))
        self.assertEqual(state, ob.REPORT_MALFORMED)

    def test_stream_incomplete_when_a_writer_was_retained(self):
        state, _, _ = ob.helper_report_state(
            stream_with(b"payload",
                        completeness="WriterRetainedAfterChildExit"))
        self.assertEqual(state, ob.REPORT_STREAM_INCOMPLETE)

    def test_stream_incomplete_when_bytes_were_dropped_past_the_prefix(self):
        state, _, _ = ob.helper_report_state(
            stream_with(b"payload", drained=100000))
        self.assertEqual(state, ob.REPORT_STREAM_INCOMPLETE)

    def test_payload_before_the_sentinel_is_separated(self):
        _, _, payload = ob.helper_report_state(
            stream_with(report_bytes(payload=b"ABCDEF")))
        self.assertEqual(payload, b"ABCDEF")

    def test_only_a_complete_report_produces_a_token(self):
        """Section 3: incomplete report evidence stays non-success."""
        for state in (ob.REPORT_TRUNCATED, ob.REPORT_MALFORMED,
                      ob.REPORT_STREAM_INCOMPLETE):
            obs = observation(rep=None, report_state=state,
                              expected_argv=["helper_report"])
            for rule in ("argv_exact", "environ_empty", "fds_exactly_012",
                         "signals_reset", "no_new_privs"):
                token, reason = ob.derive(rule, obs)
                self.assertIsNone(token, rule + " from " + state)
                self.assertTrue(reason)

    def test_a_decisive_absence_yields_a_lifecycle_token_never_success(self):
        # Definition section 9.6 (F417-I1) split REPORT_ABSENT out of the test
        # above: a complete stdout holding no report is contradictory evidence
        # for a report-based case, so the rule takes the launcher's lifecycle
        # token -- which is never any report-based expectation -- not None.
        obs = observation(rep=None, report_state=ob.REPORT_ABSENT,
                          expected_argv=["helper_report"])
        success = {"argv_exact", "environ_empty", "fds_exactly_012",
                   "signals_reset", "no_new_privs_1", "no_new_privs_0"}
        for rule in ("argv_exact", "environ_empty", "fds_exactly_012",
                     "signals_reset", "no_new_privs"):
            token, reason = ob.derive(rule, obs)
            self.assertEqual(token, "ExecStatusIndeterminate", rule)
            self.assertNotIn(token, success)
            self.assertTrue(reason)

    def test_an_indecisive_stream_is_not_exec_evidence_either_way(self):
        for state in (ob.REPORT_TRUNCATED, ob.REPORT_MALFORMED,
                      ob.REPORT_STREAM_INCOMPLETE):
            self.assertEqual(
                ob.exec_confirmation(receipt(), None, None, state),
                ob.EXEC_UNINTERPRETABLE, state)

    def test_a_decisive_absence_is_death_before_exec(self):
        self.assertEqual(
            ob.exec_confirmation(receipt(), None, None, ob.REPORT_ABSENT),
            ob.EXEC_DIED_BEFORE_EXEC)

    def test_s5_posed_check_requires_a_decisive_absence(self):
        check = driver.POSED_CHECKS["no_helper_report"]
        self.assertTrue(check({"report_state": ob.REPORT_ABSENT}))
        for state in (ob.REPORT_TRUNCATED, ob.REPORT_MALFORMED,
                      ob.REPORT_STREAM_INCOMPLETE, ob.REPORT_COMPLETE):
            self.assertFalse(check({"report_state": state}), state)


class EncodedCaptureNeverEscapes(unittest.TestCase):
    """Section 2: a secret encoded in base64 is still a secret."""

    SECRETS = (
        "ghp_" + "A" * 36,
        "AKIAIOSFODNN7EXAMPLE",
        "/home/someoneelse/.ssh/id_ed25519",
        "ACTIONS_RUNTIME_TOKEN=eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJ4In0.sig",
        r"C:\Users\djoml\secret.txt",
    )

    def setUp(self):
        self.s = evidence.Sanitiser(work="/home/runner/work/helm",
                                    home="/home/runner", user="runner")
        blob = ("\n".join(self.SECRETS)).encode()
        self.raw = report_bytes(payload=blob,
                                environ=[s for s in self.SECRETS])
        self.spike = receipt(stdout=stream_with(self.raw))

    def _assert_clean(self, text, where):
        for secret in self.SECRETS:
            self.assertNotIn(secret, text, where + " leaked " + secret[:12])
        self.assertNotIn(encode(self.raw), text, where + " leaked the blob")
        self.assertNotIn("capture_prefix_base64\": \"aGVsbG8", text)

    def test_the_receipt_view_drops_the_encoded_capture(self):
        view = evidence.receipt_view(self.spike)
        self.assertNotIn("capture_prefix_base64", view["stdout"])
        self.assertIn("drained_sha256", view["stdout"])
        self._assert_clean(evidence.serialise(view), "receipt_view")

    def test_the_sanitiser_withholds_the_encoded_capture_by_key(self):
        out = self.s.record({"cases": {"V1": {"receipt": self.spike}}})
        block = out["cases"]["V1"]["receipt"]["stdout"]
        self.assertEqual(block["capture_prefix_base64"], evidence.WITHHELD)
        self._assert_clean(evidence.serialise(out), "sanitiser")

    def test_every_internal_key_is_withheld_at_any_depth(self):
        payload = {k: encode(b"secret-bytes") for k in evidence.INTERNAL_ONLY_KEYS}
        out = self.s.record({"a": {"b": [{"c": payload}]}})
        for key in evidence.INTERNAL_ONLY_KEYS:
            self.assertEqual(out["a"]["b"][0]["c"][key], evidence.WITHHELD, key)

    def test_a_decoded_report_still_has_its_values_suppressed(self):
        """Decoding is allowed; republishing what was decoded is not."""
        _, rep, _ = ob.helper_report_state(stream_with(self.raw))
        self.assertIsNotNone(rep)
        out = self.s.record({"report": rep})
        self._assert_clean(evidence.serialise(out), "decoded report")
        for entry in out["report"]["environ"]:
            self.assertIn("<VALUE:", entry + "<VALUE:"
                          if "=" not in entry else entry)

    def test_the_full_evidence_document_is_clean(self):
        doc = evidence.evidence_document({
            "status": "RUN", "aggregate": "MECHANISM_INCONCLUSIVE",
            "detail": {}, "counts": {}, "preflight": {},
            "membership": {}, "freeze": {},
            "cases": {"V1": {"record": {"receipt": self.spike}}},
        })
        self._assert_clean(evidence.serialise(self.s.record(doc)), "document")


class StraceParsing(unittest.TestCase):
    """Fabricated tracer text only; no tracer is ever run here."""

    WINDOW = (
        "111 clone3({flags=CLONE_PIDFD, exit_signal=SIGCHLD}, 88) = 222\n"
        "[pid   222] dup2(5, 0)                  = 0\n"
        "[pid   222] dup2(6, 1)                  = 1\n"
        "[pid   222] dup2(7, 2)                  = 2\n"
        "[pid   222] fcntl(0, F_SETFD, 0)        = 0\n"
        "[pid   222] fchdir(8)                   = 0\n"
        "[pid   222] close_range(3, 7, 0)        = 0\n"
        "[pid   222] setpgid(0, 0)               = 0\n"
        "[pid   222] rt_sigprocmask(SIG_SETMASK, [], NULL, 8) = 0\n"
        "[pid   222] rt_sigaction(SIGHUP, {...}, NULL, 8) = 0\n"
        "[pid   222] prctl(PR_SET_NO_NEW_PRIVS, 1) = 0\n"
        "[pid   222] execveat(3, \"\", NULL, NULL, AT_EMPTY_PATH) = 0\n"
        "111 waitid(P_PIDFD, 4, ...)             = 0\n"
    )

    def test_the_child_window_is_isolated(self):
        out = ob.parse_strace_child_window(self.WINDOW)
        self.assertEqual(out["child_pid"], 222)
        self.assertEqual(out["child_syscalls"][0], "dup2")
        self.assertEqual(out["child_syscalls"][-1], "execveat")
        self.assertNotIn("waitid", out["child_syscalls"])
        self.assertNotIn("clone3", out["child_syscalls"])

    def test_the_stage_sequence_is_the_frozen_order(self):
        out = ob.parse_strace_child_window(self.WINDOW)
        self.assertEqual(out["stage_sequence"],
                         ["DUP2", "CLEAR_CLOEXEC", "CHDIR", "CLOSE_RANGE",
                          "SETPGID", "SIGMASK", "SIGACTION", "NO_NEW_PRIVS",
                          "EXEC"])
        obs = observation(observed_stage_sequence=out["stage_sequence"])
        self.assertEqual(ob.derive("sequence_matches_frozen_stages", obs)[0],
                         "sequence_matches_frozen_stages")

    def test_a_window_within_the_frozen_syscall_set(self):
        out = ob.parse_strace_child_window(self.WINDOW)
        obs = observation(trace=out)
        self.assertEqual(ob.derive("child_syscalls_within_frozen_set", obs)[0],
                         "child_syscalls_within_frozen_set")

    def test_a_stray_syscall_in_the_window_is_caught(self):
        text = self.WINDOW.replace(
            "[pid   222] fchdir(8)                   = 0\n",
            "[pid   222] fchdir(8)                   = 0\n"
            "[pid   222] mmap(NULL, 4096, PROT_READ, MAP_PRIVATE, -1, 0) = 0x7f\n")
        out = ob.parse_strace_child_window(text)
        self.assertIn("mmap", out["child_syscalls"])
        obs = observation(trace=out)
        self.assertEqual(ob.derive("child_syscalls_within_frozen_set", obs)[0],
                         "child_syscalls_outside_frozen_set")

    def test_no_clone3_means_no_window(self):
        self.assertIsNone(ob.parse_strace_child_window("open(...) = 3\n"))
        self.assertIsNone(ob.parse_strace_child_window(""))
        self.assertIsNone(ob.parse_strace_child_window(None))

    def test_a_split_syscall_is_rejoined_in_the_child_window(self):
        """R-1: fragments are paired per task before anything is read."""
        text = (self.WINDOW.replace(
            "[pid   222] execveat(3, \"\", NULL, NULL, AT_EMPTY_PATH) = 0\n",
            "[pid   222] execveat(3, \"\",  <unfinished ...>\n"
            "111 write(2, \"x\", 1)                   = 1\n"
            "[pid   222] <... execveat resumed>NULL, AT_EMPTY_PATH) = 0\n"))
        out = ob.parse_strace_child_window(text)
        self.assertEqual(out["child_syscalls"][-1], "execveat")


class ControlArms(unittest.TestCase):
    """M2 and M5 gained the arms their frozen contracts name."""

    def test_m2_declares_the_threaded_arm_and_a_control(self):
        plan = driver.CASE_PLANS["M2"]
        self.assertIn("--extra-threads", plan.spike_flags)
        self.assertEqual(plan.baseline_flags, ())
        argv = driver.spike_argv(plan, "/x", "/w", "/b")
        control = driver.spike_argv(plan, "/x", "/w", "/b",
                                    flags=plan.baseline_flags)
        self.assertIn("--extra-threads", argv)
        self.assertNotIn("--extra-threads", control)

    def test_m2_is_the_only_case_with_a_control_arm(self):
        with_arm = [p.case for p in driver._PLAN_LIST
                    if p.baseline_flags is not None]
        self.assertEqual(with_arm, ["M2"])

    def test_extra_threads_is_not_a_child_injection_mode(self):
        """It changes the PARENT's shape, so M2 can still trace production."""
        self.assertEqual(
            driver.declared_injection_modes(driver.CASE_PLANS["M2"]), [])
        self.assertNotIn("--extra-threads", fc.CHILD_INJECTION_MODES)

    def test_parent_control_modes_are_declared_and_disjoint(self):
        self.assertEqual(
            set(fc.PARENT_CONTROL_MODES) & set(fc.CHILD_INJECTION_MODES), set())
        # The delta correction declares the barrier flag (A-9 drift) and the
        # two caller-state flags F3, F6 and F7 need. None of them is ever a
        # plan's own spike flag; the driver adds each at spawn time.
        self.assertEqual(set(fc.PARENT_CONTROL_MODES),
                         {"--extra-threads", "--rejected-acquisition-arm",
                          "--post-pin-control-fd", "--parent-fd-set-cloexec",
                          "--parent-close-low-fds"})

    def test_every_control_mode_a_plan_uses_is_declared(self):
        declared = set(fc.PARENT_CONTROL_MODES) | set(fc.CHILD_INJECTION_MODES)
        for plan in driver._PLAN_LIST:
            for flag in plan.spike_flags:
                if flag.startswith("--"):
                    self.assertIn(flag, declared, plan.case + " " + flag)

    def test_no_case_but_m2_and_m5_uses_a_parent_control_mode(self):
        users = sorted(p.case for p in driver._PLAN_LIST
                       if set(p.spike_flags) & set(fc.PARENT_CONTROL_MODES))
        self.assertEqual(users, ["M2", "M5"])

    def test_m5_declares_the_rejected_arm(self):
        plan = driver.CASE_PLANS["M5"]
        self.assertIn("--rejected-acquisition-arm", plan.spike_flags)
        self.assertEqual(fc.BY_NAME["M5"]["cls"], fc.RECORDED)

    def test_m5_gate_forbids_an_unobserved_exit_status(self):
        plan = driver.CASE_PLANS["M5"]
        observed = driver.gates_for(plan, {"spike": receipt(
            rejected_acquisition_arm=arm(exit_status_observed=True,
                                         exit_status=0))})
        self.assertTrue(observed["never_reports_unobserved_exit_status"])

        honest = driver.gates_for(plan, {"spike": receipt(
            rejected_acquisition_arm=arm(exit_status_observed=False,
                                         exit_status=-1))})
        self.assertTrue(honest["never_reports_unobserved_exit_status"])

        lying = driver.gates_for(plan, {"spike": receipt(
            rejected_acquisition_arm=arm(exit_status_observed=False,
                                         exit_status=0))})
        self.assertFalse(lying["never_reports_unobserved_exit_status"])

    def test_m3_gained_no_arm_and_no_receipt_field(self):
        """M3-T uses the external trace; it must NOT gain a spike mode."""
        plan = driver.CASE_PLANS["M3"]
        self.assertEqual(plan.spike_flags, ())
        spike = (EXP / "launcher_spike.c").read_text(encoding="utf-8")
        self.assertNotIn("pidfd_acquisition", spike)

    def test_helper_fd_set_is_unchanged_by_the_correction(self):
        """No case gained an inherited helper descriptor."""
        for plan in driver._PLAN_LIST:
            argv = driver.spike_argv(plan, "/x", "/w", "/b")
            for flag in argv:
                self.assertNotIn("--report-fd", flag)
                self.assertNotIn("--extra-fd", flag)


class PreflightGate(unittest.TestCase):
    """Section 7: D-7 execution halts before the first case unless posable."""

    def _gates(self, supplied):
        import run_launch_exec_01 as runner
        real_channels = driver.SPIKE_SUPPLIED_CHANNELS
        real_gate = runner.harness.static_link_gate
        try:
            driver.SPIKE_SUPPLIED_CHANNELS = frozenset(supplied)
            # Patched so the gate check never invokes a compiler from a test.
            runner.harness.static_link_gate = lambda _: {"ok": True}
            return runner.preflight_gates(
                {"clone3": {"available": True}, "strace": "/usr/bin/strace"},
                "unused")
        finally:
            driver.SPIKE_SUPPLIED_CHANNELS = real_channels
            runner.harness.static_link_gate = real_gate

    def test_a_removed_channel_halts_the_trial(self):
        halts = self._gates(set(driver.SPIKE_SUPPLIED_CHANNELS)
                            - {driver.CH_REPORT})
        gates = [h["gate"] for h in halts]
        self.assertIn("evidence_channels", gates)

    def test_the_current_state_raises_no_channel_halt(self):
        """After M3-T every case is posable, so this gate no longer fires."""
        halts = self._gates(driver.SPIKE_SUPPLIED_CHANNELS)
        self.assertEqual([h for h in halts if h["gate"] == "evidence_channels"],
                         [])

    def test_a_fully_posable_set_raises_no_channel_halt(self):
        halts = self._gates(set(driver.ALL_CHANNELS))
        self.assertEqual([h for h in halts if h["gate"] == "evidence_channels"],
                         [])

    def test_clone3_unavailable_halts_the_whole_trial(self):
        import run_launch_exec_01 as runner
        real_gate = runner.harness.static_link_gate
        try:
            runner.harness.static_link_gate = lambda _: {"ok": True}
            halts = runner.preflight_gates(
                {"clone3": {"available": False}}, "unused")
        finally:
            runner.harness.static_link_gate = real_gate
        self.assertIn("clone3", [h["gate"] for h in halts])


# ============================================ M3-T: external acquisition trace
def acq_trace(flags="CLONE_PIDFD", ptr="0x7ffd0000", out_fd=4, ret=222,
              lifecycle_fd=4, pidfd_open=None, extra="", with_out=True,
              with_waitid=True):
    """A fabricated syscall record. No tracer is run and nothing is executed."""
    out = " => {pidfd=[%d]}" % out_fd if with_out else ""
    flag_field = "flags=%s, " % flags if flags else ""
    lines = [
        "111   clone3({%spidfd=%s, exit_signal=SIGCHLD}%s, 88) = %d"
        % (flag_field, ptr, out, ret) if ptr else
        "111   clone3({%sexit_signal=SIGCHLD}%s, 88) = %d"
        % (flag_field, out, ret),
        "[pid   %d] execveat(3, \"\", NULL, NULL, AT_EMPTY_PATH) = 0" % ret,
        "111   poll([{fd=%d, events=POLLIN}], 1, 5000) = 1" % lifecycle_fd,
    ]
    if pidfd_open is not None:
        lines.append("111   pidfd_open(%d, 0)                = 7" % pidfd_open)
    if with_waitid:
        lines.append(
            "111   waitid(P_PIDFD, %d, {si_pid=%d, si_code=CLD_EXITED, "
            "si_status=0}, WEXITED, NULL) = 0" % (lifecycle_fd, ret))
    if extra:
        lines.append(extra)
    return "\n".join(lines) + "\n"


def acq_obs(text, **over):
    facts = ob.parse_pidfd_acquisition(text)
    return observation(acquisition=facts, **over)


class M3AcquisitionEvidence(unittest.TestCase):
    """Owner amendment M3-T. All evidence is external; none is self-asserted."""

    def _token(self, text, **over):
        return ob.derive("pidfd_acquired_atomically", acq_obs(text, **over))

    # -- the success path, both admissible forms of fact E -------------------
    def test_direct_form_passes(self):
        token, reason = self._token(acq_trace())
        self.assertEqual(token, fc.BY_NAME["M3"]["predict"])
        self.assertIn(ob.EVIDENCE_DIRECT, reason)

    def test_closure_form_is_removed(self):
        """R-2. Absence of pidfd_open no longer proves the descriptor's origin."""
        token, reason = self._token(acq_trace(with_out=False))
        self.assertIsNone(token)
        self.assertIn("did not render", reason)
        self.assertFalse(hasattr(ob, "EVIDENCE_CLOSURE"))

    def test_every_route_the_closure_form_accepted_is_now_refused(self):
        """The six alternate acquisition routes the bounded review found."""
        base = acq_trace(with_out=False)
        routes = {
            "dup2": "111   dup2(4, 7)                      = 7",
            "pidfd_getfd": "111   pidfd_getfd(9, 3, 0)     = 4",
            "procfs": "111   openat(AT_FDCWD, \"/proc/222\", O_DIRECTORY) = 4",
            "legacy_clone": ("111   clone(child_stack=NULL, "
                             "flags=CLONE_PIDFD|SIGCHLD, parent_tid=[4]) = 333"),
            "fork": "111   fork()                          = 333",
            "scm_rights": ("111   recvmsg(8, {msg_control=[{cmsg_type="
                           "SCM_RIGHTS, cmsg_data=[4]}]}, 0) = 1"),
        }
        for label, line in routes.items():
            token, _ = self._token(
                base.replace("111   poll(", line + "\n111   poll("))
            self.assertIsNone(token, label + " still yielded a token")

    def test_the_frozen_facts_are_all_named_in_the_manifest(self):
        self.assertEqual(sorted(fc.M3_EVIDENCE_FACTS), list("ABCDEFGH"))
        self.assertIn("SAME clone3", fc.M3_BOUNDED_CLAIM)
        self.assertTrue(fc.M3_CLAIM_EXCLUSIONS)

    # -- B: CLONE_PIDFD absent ----------------------------------------------
    def test_clone3_without_clone_pidfd_fails(self):
        token, _ = self._token(acq_trace(flags="CLONE_VM"))
        self.assertEqual(token, "pidfd_not_acquired_atomically")
        self.assertNotEqual(token, "pidfd_acquired_atomically")

    def test_clone3_with_no_flags_field_fails(self):
        token, _ = self._token(acq_trace(flags=""))
        self.assertEqual(token, "pidfd_not_acquired_atomically")

    # -- D: clone3 failure ---------------------------------------------------
    def test_clone3_failure_is_a_mechanism_fail(self):
        """A genuine error return, reached AFTER the fragments rejoined."""
        text = ("111   clone3({flags=CLONE_PIDFD, pidfd=0x7ffd0000, "
                "exit_signal=SIGCHLD}, 88) = -1 EPERM (Operation not "
                "permitted)\n")
        token, _ = self._token(text)
        self.assertEqual(token, "clone3_failed")

    # -- C: no output location decoded --------------------------------------
    def test_pidfd_output_pointer_absent_is_invalid(self):
        token, reason = self._token(acq_trace(ptr="", with_out=False))
        self.assertIsNone(token)
        self.assertIn("output location", reason)

    # -- F: pidfd_open acquisition ------------------------------------------
    def test_pidfd_open_on_the_direct_child_is_rejected(self):
        """Exactly the acquisition design the primary mechanism rejects."""
        token, reason = self._token(acq_trace(pidfd_open=222))
        self.assertEqual(token, "pidfd_acquired_by_pidfd_open")
        self.assertIn("numeric pid", reason)

    def test_an_unrelated_pidfd_open_does_not_reject(self):
        """A pidfd_open aimed at another process is not this case's concern."""
        token, _ = self._token(acq_trace(pidfd_open=999))
        self.assertEqual(token, "pidfd_acquired_atomically")

    def test_no_write_back_is_invalid_regardless_of_pidfd_open(self):
        token, reason = self._token(acq_trace(with_out=False, pidfd_open=999))
        self.assertIsNone(token)
        self.assertIn("did not render", reason)

    # -- G: correlation ------------------------------------------------------
    def test_no_waitid_correlation_is_invalid(self):
        token, reason = self._token(acq_trace(with_waitid=False))
        self.assertIsNone(token)
        self.assertIn("waitid", reason)

    def test_two_descriptors_claiming_the_same_child_are_ambiguous(self):
        extra = ("111   waitid(P_PIDFD, 9, {si_pid=222, si_code=CLD_EXITED}, "
                 "WEXITED, NULL) = 0")
        token, reason = self._token(acq_trace(extra=extra))
        self.assertIsNone(token)
        self.assertIn("more than one descriptor", reason)

    def test_unrelated_child_activity_does_not_break_correlation(self):
        """R-3: si_pid filtering means another child's reap is simply ignored."""
        extra = ("111   waitid(P_PIDFD, 9, {si_pid=333, si_code=CLD_EXITED}, "
                 "WEXITED, NULL) = 0")
        token, _ = self._token(acq_trace(extra=extra))
        self.assertEqual(token, "pidfd_acquired_atomically")

    def test_a_descriptor_mismatch_is_invalid(self):
        """clone3 returned one descriptor and the lifecycle used another."""
        token, reason = self._token(acq_trace(out_fd=4, lifecycle_fd=5))
        self.assertIsNone(token)
        self.assertIn("ambiguous", reason)

    # -- A: ambiguity and malformed input ------------------------------------
    def test_two_clone3_calls_are_ambiguous(self):
        text = acq_trace() + acq_trace(ret=444)
        token, reason = self._token(text)
        self.assertIsNone(token)
        self.assertIn("more than one clone3", reason)

    def test_malformed_trace_is_invalid(self):
        for text in ("", "   ", "garbage without any syscall\n",
                     "111 open(\"/etc/passwd\", O_RDONLY) = 3\n"):
            token, _ = self._token(text)
            self.assertIsNone(token, repr(text))

    def test_truncated_trace_is_invalid(self):
        """The record stops before the lifecycle correlates anything."""
        text = "111   clone3({flags=CLONE_PIDFD, pidfd=0x7ffd0000, exit_s"
        token, _ = self._token(text)
        self.assertIsNone(token)

    def test_no_acquisition_facts_at_all_is_invalid(self):
        token, reason = ob.derive("pidfd_acquired_atomically", observation())
        self.assertIsNone(token)
        self.assertIn("no syscall record", reason)

    # -- the rule must never read a launcher self-assertion -------------------
    def test_a_receipt_field_claiming_clone3_is_ignored(self):
        """A launcher describing its own acquisition proves nothing."""
        spike = receipt(pidfd_acquisition="clone3")
        obs = observation(spike=spike, acquisition=None,
                          pidfd_acquisition="clone3_pidfd")
        token, _ = ob.derive("pidfd_acquired_atomically", obs)
        self.assertIsNone(token, "a self-asserted field produced a token")

    def test_the_source_never_reads_an_acquisition_receipt_field(self):
        source = (EXP / "observations.py").read_text(encoding="utf-8")
        rule = source[source.index("def rule_pidfd_acquired_atomically"):]
        rule = rule[:rule.index("\ndef ")]
        self.assertNotIn('"pidfd_acquisition"', rule)
        self.assertNotIn("spike", rule.split("Returns")[0] if "Returns" in rule
                         else rule.split("\n\n")[0])

    # -- host condition ------------------------------------------------------
    def test_clone3_unavailable_remains_the_only_block_cause(self):
        self.assertEqual(fc.BY_NAME["M3"]["blocked_if"], "clone3_unavailable")
        self.assertEqual(fc.BY_NAME["M3"]["cls"], fc.CONDITIONAL)
        ctx = _ctx(block_reasons={"clone3_unavailable":
                                  fc.BLOCK_REASONS["clone3_unavailable"]})
        cause = driver.blocked_cause_for(driver.CASE_PLANS["M3"], ctx)
        self.assertEqual(cause, "clone3_unavailable")
        record = driver.evaluate(driver.CASE_PLANS["M3"],
                                 {"blocked": cause, "launch_returned": None})
        self.assertEqual(checker.score_case("M3", record)[0], checker.BLOCKED)

    def test_a_tracer_failure_is_not_a_legitimate_block(self):
        """If clone3 is supported and the trace fails, that is not BLOCKED."""
        ctx = _ctx(block_reasons={"no_tracer": fc.BLOCK_REASONS["no_tracer"]})
        self.assertIsNone(driver.blocked_cause_for(driver.CASE_PLANS["M3"], ctx))

    def test_the_block_escape_hatch_was_not_broadened(self):
        conditional = {n: fc.BY_NAME[n]["blocked_if"]
                       for n in fc.CONDITIONAL_CASES}
        self.assertEqual(conditional["M3"], "clone3_unavailable")
        for name, cause in conditional.items():
            self.assertIn(cause, fc.BLOCK_REASONS, name)


class M3TracedSetAmendment(unittest.TestCase):
    def test_the_traced_set_is_now_eight(self):
        self.assertEqual(
            sorted(fc.TRACED_CASES),
            ["E1", "E7", "F4", "F7", "M1", "M2", "M3", "M4"])
        self.assertEqual(len(fc.TRACED_CASES), 8)

    def test_the_partition_is_untouched(self):
        s = fc.summary()
        self.assertEqual((s["total"], s["mandatory"], s["conditional"],
                          s["recorded"]), (72, 54, 11, 7))

    def test_m3_is_traced_in_the_plan_too(self):
        self.assertTrue(driver.CASE_PLANS["M3"].traced)
        self.assertIn(driver.CH_TRACE, driver.CASE_PLANS["M3"].channels)

    def test_m3_carries_no_injection_or_control_mode(self):
        """A pidfd_open from the M5 arm would pollute the record M3 reads."""
        plan = driver.CASE_PLANS["M3"]
        self.assertEqual(plan.spike_flags, ())
        self.assertEqual(driver.declared_injection_modes(plan), [])
        self.assertEqual(set(plan.spike_flags) & set(fc.PARENT_CONTROL_MODES),
                         set())

    def test_no_wait_classifying_series_became_traced(self):
        for name in fc.TRACED_CASES:
            self.assertNotIn(fc.BY_NAME[name]["series"], {"R", "T", "O", "P", "S"})

    def test_every_traced_record_carries_its_trace(self):
        """checker scores a traced case INVALID without one.

        Each traced case is given the evidence its own rule needs, so the record
        reaches the trace block instead of short-circuiting at not_posed. That
        short-circuit is itself correct: checker.score_case tests not_posed
        before it tests the trace.
        """
        permitted = list(fc.CHILD_PERMITTED_SYSCALLS)
        for name in fc.TRACED_CASES:
            plan = driver.CASE_PLANS[name]
            obs = observation(
                spike=receipt(parent_shape={"extra_threads": 3,
                                            "atfork_handler_registered": True,
                                            "atfork_prepare_calls": 0}),
                rep=report(), trace={"child_syscalls": permitted},
                trace_sha256="a" * 64,
                single_threaded_child_syscalls=permitted,
                observed_stage_sequence=list(fc.STAGES),
                acquisition=ob.parse_pidfd_acquisition(acq_trace()),
                expected_argv=["helper_report"],
                # Trial #2 delta correction: each traced case also carries the
                # posing proof its plan now requires -- E1's starting identity
                # and marker, F7's observed adjacency, M2's threaded and
                # control arms. Without it the case is, correctly, not posed.
                expected_marker=plan.expected_marker,
                build_identity_binding={"bound": True,
                                        "object_sha256": "a" * 64},
                exec_status_pair_adjacent=True,
                declared_launcher_threads=3,
                # F417-I2: a control arm that ran records whether it returned,
                # as _run_once always does; one that did not is FAIL or INVALID.
                baseline_observation={"launch_returned": True, "spike": receipt(
                    parent_shape={"extra_threads": 0,
                                  "atfork_handler_registered": False,
                                  "atfork_prepare_calls": 0})})
            record = driver.evaluate(plan, obs)
            self.assertNotIn("not_posed", record,
                             name + ": " + str(record.get("not_posed")))
            self.assertIn("trace", record, name)
            self.assertIsNotNone(record["trace"], name)
            self.assertEqual(record["trace_sha256"], "a" * 64, name)
            status, why = checker.score_case(name, record)
            self.assertNotEqual(status, checker.INVALID, name + ": " + why)

    def test_a_traced_case_without_a_trace_is_invalid(self):
        record = driver.evaluate(driver.CASE_PLANS["M3"],
                                 observation(acquisition=None))
        status, _ = checker.score_case("M3", record)
        self.assertEqual(status, checker.INVALID)


class M3Privacy(unittest.TestCase):
    """Section 11: raw trace identifiers must not reach published evidence."""

    def setUp(self):
        self.s = evidence.Sanitiser(work="/home/runner/work/helm",
                                    home="/home/runner", user="runner")
        self.facts = ob.parse_pidfd_acquisition(acq_trace(pidfd_open=999))

    def test_normalisation_replaces_raw_identifiers_with_roles(self):
        norm = ob.normalise_acquisition(self.facts)
        self.assertEqual(norm["direct_child"], "DIRECT_CHILD")
        self.assertEqual(norm["direct_child_pidfd"], "DIRECT_CHILD_PIDFD")
        text = evidence.serialise(norm)
        for raw in ("222", "999", "0x7ffd0000"):
            self.assertNotIn(raw, text, "raw identifier " + raw + " survived")

    def test_normalisation_keeps_only_booleans_flags_and_counts(self):
        norm = ob.normalise_acquisition(self.facts)
        self.assertNotIn("pidfd_open_calls", norm)
        self.assertNotIn("lifecycle_uses", norm)
        self.assertNotIn("clone3_return", norm)
        self.assertNotIn("pidfd_from_clone3", norm)
        self.assertEqual(norm["clone3_flags"], ["CLONE_PIDFD"])
        self.assertEqual(norm["pidfd_open_call_count"], 1)

    def test_raw_acquisition_facts_are_withheld_by_key(self):
        out = self.s.record({"cases": {"M3": {"acquisition": self.facts}}})
        self.assertEqual(out["cases"]["M3"]["acquisition"], evidence.WITHHELD)

    def test_raw_tracer_text_is_withheld_by_key(self):
        blob = acq_trace() + "111 openat(AT_FDCWD, \"/home/runner/.netrc\", 0) = 9\n"
        out = self.s.record({"raw_trace": blob, "trace_text": blob,
                             "strace_output": blob})
        text = evidence.serialise(out)
        self.assertNotIn(".netrc", text)
        self.assertNotIn("clone3", text)

    def test_the_trace_digest_is_publishable(self):
        digest = oracles.digest_of(acq_trace().encode())
        out = self.s.record({"trace_sha256": digest})
        self.assertEqual(out["trace_sha256"], digest)

    def test_a_traced_record_publishes_normalised_facts_only(self):
        plan = driver.CASE_PLANS["M3"]
        record = driver.evaluate(plan, acq_obs(
            acq_trace(), trace={"child_syscalls": ["dup2"]},
            trace_sha256="b" * 64,
            acquisition_normalised=ob.normalise_acquisition(self.facts)))
        self.assertIn("acquisition_normalised", record)
        self.assertNotIn("acquisition", record)
        text = evidence.serialise(self.s.record(record))
        self.assertNotIn("0x7ffd0000", text)


# ================================================ R-1: strace fragment joining
SPLIT_CLONE3 = (
    "111   clone3({flags=CLONE_PIDFD, pidfd=0x7ffd0000, exit_signal=SIGCHLD}"
    " <unfinished ...>\n"
    "[pid   222] execveat(3, \"\", NULL, NULL, AT_EMPTY_PATH) = 0\n"
    "111   <... clone3 resumed> => {pidfd=[4]}, 88) = 222\n"
    "111   poll([{fd=4, events=POLLIN}], 1, 5000) = 1\n"
    "111   waitid(P_PIDFD, 4, {si_pid=222, si_code=CLD_EXITED, si_status=0},"
    " WEXITED, NULL) = 0\n")


class FragmentJoining(unittest.TestCase):
    """R-1. A formatting fragment must never become a mechanism verdict."""

    def _token(self, text):
        return ob.derive("pidfd_acquired_atomically",
                         observation(acquisition=ob.parse_pidfd_acquisition(text)))

    def test_a_normal_split_clone3_rejoins_and_passes(self):
        """The defect the bounded review found: this used to FAIL."""
        token, reason = self._token(SPLIT_CLONE3)
        self.assertEqual(token, "pidfd_acquired_atomically")
        self.assertNotEqual(token, "clone3_failed")
        facts = ob.parse_pidfd_acquisition(SPLIT_CLONE3)
        self.assertTrue(facts["clone3_was_joined"])
        self.assertEqual(facts["clone3_return"], 222)
        self.assertEqual(facts["pidfd_from_clone3"], 4)

    def test_an_interleaved_syscall_from_another_task_does_not_break_the_pair(self):
        text = SPLIT_CLONE3.replace(
            "[pid   222] execveat",
            "[pid   999] write(1, \"noise\", 5)         = 5\n[pid   222] execveat")
        self.assertEqual(self._token(text)[0], "pidfd_acquired_atomically")

    def test_fragments_are_never_spliced_across_tasks(self):
        """The resumed half belongs to a different task than the unfinished."""
        text = SPLIT_CLONE3.replace("111   <... clone3 resumed>",
                                    "777   <... clone3 resumed>")
        token, reason = self._token(text)
        self.assertIsNone(token)
        self.assertIn("resumed", reason)

    def test_two_simultaneous_unfinished_clone3_from_different_tasks(self):
        text = ("111   clone3({flags=CLONE_PIDFD, pidfd=0x1, "
                "exit_signal=SIGCHLD} <unfinished ...>\n"
                "777   clone3({flags=CLONE_PIDFD, pidfd=0x2, "
                "exit_signal=SIGCHLD} <unfinished ...>\n"
                "111   <... clone3 resumed> => {pidfd=[4]}, 88) = 222\n"
                "777   <... clone3 resumed> => {pidfd=[5]}, 88) = 333\n"
                "111   waitid(P_PIDFD, 4, {si_pid=222}, WEXITED, NULL) = 0\n")
        facts = ob.parse_pidfd_acquisition(text)
        # Both pairs rejoin correctly per task, and TWO clone3 calls are then
        # ambiguous for M3 -- which is the honest answer, not a guess.
        self.assertEqual(facts["fragments_unmatched"], 0)
        self.assertEqual(facts["clone3_call_count"], 2)
        token, reason = self._token(text)
        self.assertIsNone(token)
        self.assertIn("more than one clone3", reason)

    def test_two_unfinished_from_the_SAME_task_are_ambiguous(self):
        text = ("111   clone3({flags=CLONE_PIDFD, pidfd=0x1, "
                "exit_signal=SIGCHLD} <unfinished ...>\n"
                "111   clone3({flags=CLONE_PIDFD, pidfd=0x2, "
                "exit_signal=SIGCHLD} <unfinished ...>\n"
                "111   <... clone3 resumed> => {pidfd=[4]}, 88) = 222\n")
        facts = ob.parse_pidfd_acquisition(text)
        self.assertEqual(facts["fragments_ambiguous"], 1)
        token, reason = self._token(text)
        self.assertIsNone(token)
        self.assertIn("unambiguously", reason)

    def test_missing_resumed_half_is_invalid_not_fail(self):
        text = SPLIT_CLONE3.replace(
            "111   <... clone3 resumed> => {pidfd=[4]}, 88) = 222\n", "")
        facts = ob.parse_pidfd_acquisition(text)
        self.assertEqual(facts["fragments_unmatched"], 1)
        token, reason = self._token(text)
        self.assertIsNone(token, "an unfinished fragment produced a verdict")
        self.assertIn("never resumed", reason)

    def test_orphan_resumed_half_is_invalid(self):
        text = ("111   <... clone3 resumed> => {pidfd=[4]}, 88) = 222\n"
                "111   waitid(P_PIDFD, 4, {si_pid=222}, WEXITED, NULL) = 0\n")
        facts = ob.parse_pidfd_acquisition(text)
        self.assertEqual(facts["fragments_orphaned"], 1)
        token, reason = self._token(text)
        self.assertIsNone(token)
        self.assertIn("no matching", reason)

    def test_a_mismatched_resume_is_never_spliced(self):
        """The task resumed a different syscall than it left unfinished."""
        text = SPLIT_CLONE3.replace("<... clone3 resumed>", "<... read resumed>")
        facts = ob.parse_pidfd_acquisition(text)
        self.assertEqual(facts["fragments_orphaned"], 1)
        self.assertEqual(facts["fragments_unmatched"], 1)
        self.assertIsNone(self._token(text)[0])

    def test_resumed_error_return_is_a_real_failure(self):
        text = ("111   clone3({flags=CLONE_PIDFD, pidfd=0x7ffd0000, "
                "exit_signal=SIGCHLD} <unfinished ...>\n"
                "999   write(1, \"x\", 1)                = 1\n"
                "111   <... clone3 resumed>, 88) = -1 EPERM (Operation not "
                "permitted)\n")
        token, _ = self._token(text)
        self.assertEqual(token, "clone3_failed")

    def test_truncated_final_line_is_invalid(self):
        text = SPLIT_CLONE3.replace(
            "111   <... clone3 resumed> => {pidfd=[4]}, 88) = 222\n",
            "111   <... clone3 resumed> => {pidfd=[4]}, 88")
        token, _ = self._token(text)
        self.assertIsNone(token)

    def test_a_malformed_task_prefix_is_invalid(self):
        text = SPLIT_CLONE3.replace("[pid   222]", "[pid   ????]")
        facts = ob.parse_pidfd_acquisition(text)
        self.assertEqual(facts["lines_malformed"], 1)
        token, reason = self._token(text)
        self.assertIsNone(token)
        self.assertIn("task attribution", reason)

    def test_a_well_formed_pid_prefix_is_not_malformed(self):
        """Guards the backtracking bug where [pid   222] read as malformed."""
        for prefix in ("[pid 222]", "[pid   222]", "222", "[pid 7]"):
            joined = ob.join_trace_fragments(prefix + " execveat(3) = 0\n")
            self.assertEqual(joined["malformed"], [], prefix)
            self.assertEqual(len(joined["records"]), 1, prefix)


class SiPidCorrelation(unittest.TestCase):
    """R-3. The reaped process must BE the direct child."""

    def _token(self, text):
        return ob.derive("pidfd_acquired_atomically",
                         observation(acquisition=ob.parse_pidfd_acquisition(text)))

    def test_exact_pid_match_correlates(self):
        self.assertEqual(self._token(acq_trace())[0], "pidfd_acquired_atomically")

    def test_pid_mismatch_is_invalid(self):
        text = acq_trace().replace("si_pid=222", "si_pid=999")
        token, reason = self._token(text)
        self.assertIsNone(token, "a pidfd for another process correlated")
        self.assertIn("si_pid", reason)

    def test_missing_si_pid_is_invalid(self):
        text = acq_trace().replace("{si_pid=222, si_code=CLD_EXITED, "
                                   "si_status=0}", "{...}")
        token, reason = self._token(text)
        self.assertIsNone(token)
        self.assertIn("si_pid", reason)

    def test_repeated_waitid_on_the_same_pidfd_still_correlates(self):
        extra = ("111   waitid(P_PIDFD, 4, {si_pid=222, si_code=CLD_EXITED}, "
                 "WEXITED, NULL) = 0")
        self.assertEqual(self._token(acq_trace(extra=extra))[0],
                         "pidfd_acquired_atomically")

    def test_echild_after_a_correct_correlation_is_harmless(self):
        extra = ("111   waitid(P_PIDFD, 4, {}, WEXITED, NULL) = -1 ECHILD "
                 "(No child processes)")
        self.assertEqual(self._token(acq_trace(extra=extra))[0],
                         "pidfd_acquired_atomically")

    def test_a_descriptor_mismatch_between_clone3_and_waitid_is_invalid(self):
        token, reason = self._token(acq_trace(out_fd=4, lifecycle_fd=5))
        self.assertIsNone(token)
        self.assertIn("lifecycle used", reason)


class TracerPreflightRequirement(unittest.TestCase):
    """R-4. strace is mandatory; its absence HALTS, it never BLOCKS a case."""

    def _gates(self, preflight):
        import run_launch_exec_01 as runner
        real = runner.harness.static_link_gate
        try:
            runner.harness.static_link_gate = lambda _: {"ok": True}
            return runner.preflight_gates(preflight, "unused")
        finally:
            runner.harness.static_link_gate = real

    GOOD = {"clone3": {"available": True}, "strace": "/usr/bin/strace",
            "strace_version": ["strace -- version 6.8"], "ptrace_scope": "1"}

    def test_no_strace_halts(self):
        pf = dict(self.GOOD, strace=None, strace_version=None)
        self.assertIn("tracer", [h["gate"] for h in self._gates(pf)])

    def test_unsupported_strace_halts(self):
        pf = dict(self.GOOD, strace_version=["strace -- version 4.26"])
        self.assertIn("tracer", [h["gate"] for h in self._gates(pf)])

    def test_unreadable_strace_version_halts(self):
        pf = dict(self.GOOD, strace_version=["<unavailable: no such file>"])
        self.assertIn("tracer", [h["gate"] for h in self._gates(pf)])

    def test_restrictive_ptrace_scope_halts(self):
        pf = dict(self.GOOD, ptrace_scope="3")
        self.assertIn("tracer", [h["gate"] for h in self._gates(pf)])

    def test_a_supported_tracer_may_proceed(self):
        self.assertEqual([h["gate"] for h in self._gates(self.GOOD)], [])

    def test_no_tracer_never_becomes_clone3_unavailable(self):
        pf = dict(self.GOOD, strace=None, strace_version=None)
        halts = self._gates(pf)
        self.assertNotIn("clone3", [h["gate"] for h in halts])
        for halt in halts:
            self.assertNotIn("clone3_unavailable", json.dumps(halt))

    def test_the_tracer_condition_is_not_a_conditional_block_cause(self):
        ctx = _ctx(block_reasons={"no_tracer": fc.BLOCK_REASONS["no_tracer"]})
        self.assertIsNone(driver.blocked_cause_for(driver.CASE_PLANS["M3"], ctx))

    def test_no_case_is_posed_while_the_tracer_gate_fails(self):
        """run_trial returns HALT_PREFLIGHT before the case loop is reached.

        Read from the AST rather than by slicing the source text: the previous
        form searched for the first occurrence of the literal HALT_PREFLIGHT
        and broke when a docstring mentioned it, which said nothing about the
        invariant it exists to protect.
        """
        tree = ast.parse((EXP / "run_launch_exec_01.py").read_text(encoding="utf-8"))
        run_trial = [n for n in tree.body
                     if isinstance(n, ast.FunctionDef) and n.name == "run_trial"][0]

        halt_returns = []
        for node in ast.walk(run_trial):
            if isinstance(node, ast.If) and ast.unparse(node.test) == "halts":
                for inner in ast.walk(node):
                    if isinstance(inner, ast.Return) and                             "HALT_PREFLIGHT" in ast.unparse(inner):
                        halt_returns.append(inner.lineno)
        self.assertTrue(halt_returns,
                        "no `if halts:` return carries HALT_PREFLIGHT")

        poses = [n.lineno for n in ast.walk(run_trial)
                 if isinstance(n, ast.Call)
                 and ast.unparse(n.func) in ("driver.observe", "driver.pose")]
        self.assertTrue(poses, "run_trial no longer poses anything")
        self.assertLess(max(halt_returns), min(poses),
                        "the preflight halt must precede any posing")

    def test_the_requirement_is_frozen_in_the_manifest(self):
        self.assertEqual(fc.STRACE_MIN_VERSION, (5, 4))
        self.assertIn("mandatory", fc.TRACER_REQUIREMENT)
        self.assertIn("HALT", fc.TRACER_REQUIREMENT)

    def test_no_fallback_tracer_was_added(self):
        """R-4 forbids a second instrumentation stack, not the word "sudo" in a
        comment saying sudo is never used -- so this greps for CODE."""
        banned = ("PTRACE_ATTACH", "PTRACE_TRACEME", "ptrace(", "bpf(",
                  "import bpf", "perf_event_open", "libbpf", "insmod",
                  "subprocess.run([\"sudo", "\"sudo\"")
        for module in ("driver.py", "observations.py", "harness.py",
                       "run_launch_exec_01.py"):
            text = (EXP / module).read_text(encoding="utf-8")
            code = chr(10).join(line.split("#", 1)[0]
                                for line in text.splitlines())
            for token in banned:
                self.assertNotIn(token, code, module + " gained " + token)

    def test_strace_is_the_only_tracer_the_driver_can_invoke(self):
        source = (EXP / "driver.py").read_text(encoding="utf-8")
        body = source[source.index("def tracer_argv("):]
        body = body[:body.index(chr(10) * 3)]
        self.assertIn("strace", body)
        self.assertIn("return None", body)


# ====================================== V-1: three-state return, bounded record
def clone3_line(flags="CLONE_PIDFD", ptr="0x7ffd0000", out="[4]", ret="222",
                suffix=""):
    o = " => {pidfd=%s}" % out if out else ""
    return ("111   clone3({flags=%s, pidfd=%s, exit_signal=SIGCHLD}%s, 88)"
            " = %s%s" % (flags, ptr, o, ret, suffix))


WAITID = ("111   waitid(P_PIDFD, 4, {si_pid=222, si_code=CLD_EXITED, "
          "si_status=0}, WEXITED, NULL) = 0")


class ReturnStateModel(unittest.TestCase):
    """V-1. "The kernel said no" and "we never saw" are different facts."""

    def _token(self, text):
        return ob.derive("pidfd_acquired_atomically",
                         observation(acquisition=ob.parse_pidfd_acquisition(text)))

    def test_the_three_states_exist_and_are_distinct(self):
        states = {ob.RETURN_OBSERVED_SUCCESS, ob.RETURN_OBSERVED_ERROR,
                  ob.RETURN_NOT_OBSERVED}
        self.assertEqual(len(states), 3)

    def test_parse_return_state_classifies_each(self):
        cases = (
            (" = 222", ob.RETURN_OBSERVED_SUCCESS, 222),
            (" = 0", ob.RETURN_OBSERVED_SUCCESS, 0),
            (" = -1 EPERM (Operation not permitted)",
             ob.RETURN_OBSERVED_ERROR, -1),
            (" = -1 ENOSYS", ob.RETURN_OBSERVED_ERROR, -1),
            (" = ?", ob.RETURN_NOT_OBSERVED, None),
            ("", ob.RETURN_NOT_OBSERVED, None),
            (None, ob.RETURN_NOT_OBSERVED, None),
            ("  nonsense  ", ob.RETURN_NOT_OBSERVED, None),
        )
        for text, state, value in cases:
            got_state, got_value, _ = ob.parse_return_state(text)
            self.assertEqual(got_state, state, repr(text))
            self.assertEqual(got_value, value, repr(text))

    def test_a_missing_return_is_invalid_never_a_failure_token(self):
        text = "111   clone3({flags=CLONE_PIDFD, pidfd=0x1} => {pidfd=[4]}, 88\n"
        token, reason = self._token(text)
        self.assertIsNone(token)
        for forbidden in ("clone3_failed", "pidfd_not_acquired_atomically"):
            self.assertNotEqual(token, forbidden)
        self.assertIn("truncated", reason)

    def test_an_observed_error_is_a_mechanism_failure(self):
        text = clone3_line(out=None, ret="-1 EPERM (Operation not permitted)")
        token, reason = self._token(text + "\n")
        self.assertEqual(token, "clone3_failed")
        self.assertIn("EPERM", reason)

    def test_an_observed_success_proceeds(self):
        self.assertEqual(self._token(clone3_line() + "\n" + WAITID + "\n")[0],
                         "pidfd_acquired_atomically")


class LogicalRecordBoundary(unittest.TestCase):
    """V-1. A record must never absorb the line that follows it."""

    def _facts(self, text):
        return ob.parse_pidfd_acquisition(text)

    def _token(self, text):
        return ob.derive("pidfd_acquired_atomically",
                         observation(acquisition=self._facts(text)))

    # -- the five scenarios the owner instruction names ----------------------
    def test_A_resumed_without_return_then_poll(self):
        """The exact defect: "= 1" from the next line must NOT be adopted."""
        text = ("111   clone3({flags=CLONE_PIDFD, pidfd=0x1, "
                "exit_signal=SIGCHLD} <unfinished ...>\n"
                "111   <... clone3 resumed> => {pidfd=[4]}, 88\n"
                "111   poll([{fd=4, events=POLLIN}], 1, 5000) = 1\n")
        facts = self._facts(text)
        self.assertIsNone(facts["clone3_return"])
        self.assertNotEqual(facts["clone3_return"], 1)
        self.assertEqual(facts["clone3_return_state"], ob.RETURN_NOT_OBSERVED)
        self.assertIsNone(self._token(text)[0])

    def test_B_resumed_truncated_before_the_equals(self):
        text = ("111   clone3({flags=CLONE_PIDFD, pidfd=0x1, "
                "exit_signal=SIGCHLD} <unfinished ...>\n"
                "111   <... clone3 resumed> => {pidfd=[4]}, 88")
        self.assertIsNone(self._facts(text)["clone3_return"])
        self.assertIsNone(self._token(text)[0])

    def test_C_proper_resume_then_poll_keeps_its_own_return(self):
        text = ("111   clone3({flags=CLONE_PIDFD, pidfd=0x1, "
                "exit_signal=SIGCHLD} <unfinished ...>\n"
                "111   <... clone3 resumed> => {pidfd=[4]}, 88) = 222\n"
                "111   poll([{fd=4, events=POLLIN}], 1, 5000) = 1\n"
                + WAITID + "\n")
        facts = self._facts(text)
        self.assertEqual(facts["clone3_return"], 222)
        self.assertEqual(facts["clone3_return_state"],
                         ob.RETURN_OBSERVED_SUCCESS)
        self.assertEqual(self._token(text)[0], "pidfd_acquired_atomically")

    def test_D_genuine_error_ignores_the_unrelated_next_return(self):
        text = ("111   clone3({flags=CLONE_PIDFD, pidfd=0x1, "
                "exit_signal=SIGCHLD} <unfinished ...>\n"
                "111   <... clone3 resumed>, 88) = -1 EPERM (Operation not "
                "permitted)\n"
                "111   read(3, \"abcdefg\", 7)            = 7\n")
        facts = self._facts(text)
        self.assertEqual(facts["clone3_return_state"],
                         ob.RETURN_OBSERVED_ERROR)
        self.assertNotEqual(facts["clone3_return"], 7)
        self.assertEqual(self._token(text)[0], "clone3_failed")

    def test_E_newline_like_noise_inside_a_record(self):
        """A quoted argument containing escapes must not shift the boundary."""
        parts = ob.split_syscall_record(
            'write(2, "line one\\nline two = 99", 21) = 21')
        self.assertIsNotNone(parts)
        self.assertEqual(parts[0], "write")
        state, value, _ = ob.parse_return_state(parts[2])
        self.assertEqual((state, value), (ob.RETURN_OBSERVED_SUCCESS, 21))

    # -- the splitter itself -------------------------------------------------
    def test_parentheses_inside_quoted_arguments_do_not_unbalance(self):
        parts = ob.split_syscall_record('execveat(3, "a(b)c", NULL) = 0')
        self.assertIsNotNone(parts)
        self.assertEqual(parts[0], "execveat")

    def test_an_unclosed_record_is_rejected(self):
        for text in ("clone3({flags=CLONE_PIDFD, pidfd=0x1}, 88",
                     "clone3({flags=CLONE_PIDFD",
                     "clone3("):
            self.assertIsNone(ob.split_syscall_record(text), text)

    def test_trailing_text_that_is_not_a_result_is_rejected(self):
        parts = ob.split_syscall_record("clone3({a}, 88) = 222 then poll(1) = 2")
        self.assertIsNotNone(parts)
        self.assertEqual(ob.parse_return_state(parts[2])[0],
                         ob.RETURN_NOT_OBSERVED)

    def test_the_record_is_bounded_flag_is_recorded(self):
        good = self._facts(clone3_line() + "\n")
        bad = self._facts("111   clone3({flags=CLONE_PIDFD, pidfd=0x1}, 88\n")
        self.assertTrue(good["clone3_record_bounded"])
        self.assertFalse(bad["clone3_record_bounded"])


class ContradictoryWaitidIdentity(unittest.TestCase):
    """V-2. The evidence set must be internally consistent BEFORE filtering."""

    def _token(self, *waitids):
        text = clone3_line() + "\n" + "\n".join(waitids) + "\n"
        return ob.derive("pidfd_acquired_atomically",
                         observation(acquisition=ob.parse_pidfd_acquisition(text)))

    def w(self, fd, si_pid=None, result="0"):
        info = "{si_pid=%d, si_code=CLD_EXITED}" % si_pid if si_pid else "{}"
        return ("111   waitid(P_PIDFD, %d, %s, WEXITED, NULL) = %s"
                % (fd, info, result))

    def test_same_pidfd_same_child_twice_is_valid(self):
        self.assertEqual(self._token(self.w(4, 222), self.w(4, 222))[0],
                         "pidfd_acquired_atomically")

    def test_same_pidfd_then_echild_without_si_pid_stays_valid(self):
        self.assertEqual(
            self._token(self.w(4, 222),
                        self.w(4, None, "-1 ECHILD (No child processes)"))[0],
            "pidfd_acquired_atomically")

    def test_same_pidfd_two_different_children_is_invalid(self):
        token, reason = self._token(self.w(4, 222), self.w(4, 333))
        self.assertIsNone(token)
        self.assertIn("self-contradictory", reason)

    def test_only_a_foreign_child_on_the_candidate_descriptor_is_invalid(self):
        token, reason = self._token(self.w(4, 333))
        self.assertIsNone(token)
        self.assertIn("si_pid", reason)

    def test_two_descriptors_identifying_the_same_child_is_ambiguous(self):
        token, reason = self._token(self.w(4, 222), self.w(9, 222))
        self.assertIsNone(token)
        self.assertIn("more than one descriptor", reason)

    def test_an_unrelated_descriptor_for_another_child_does_not_contaminate(self):
        self.assertEqual(self._token(self.w(4, 222), self.w(9, 333))[0],
                         "pidfd_acquired_atomically")

    def test_the_contradiction_is_found_before_filtering(self):
        """Even when a matching observation exists, the conflict wins."""
        token, _ = self._token(self.w(4, 222), self.w(4, 999))
        self.assertIsNone(token, "the contradictory entry was filtered away")


# ================================== V-3 / V-4: one shared traced-evidence gate
INVALID_TRACES = (
    ("none", None),
    ("empty dict", {}),
    ("empty list", []),
    ("empty string", ""),
    ("zero", 0),
    ("no child_syscalls", {"stage_sequence": ["DUP2"]}),
    ("child_syscalls not a collection", {"child_syscalls": "dup2"}),
    ("child_syscalls empty", {"child_syscalls": []}),
    ("child_syscalls wrong element type", {"child_syscalls": [1, 2]}),
    ("marked integrity-invalid",
     {"child_syscalls": ["dup2"], "integrity_ok": False}),
    ("marked truncated", {"child_syscalls": ["dup2"], "truncated": True}),
)

VALID_TRACE = {"child_syscalls": ["dup2", "execveat"],
               "stage_sequence": ["DUP2", "EXEC"], "integrity_ok": True}


def traced_record(name, trace):
    spec = fc.BY_NAME[name]
    record = {"outcome": spec["predict"] or spec["safe"][0],
              "launch_returned": True, "trace": trace}
    if spec["gates"]:
        record["gates"] = {g: True for g in spec["gates"]}
    if name in fc.DOCUMENTATION_GATES:
        record["documentation_gate"] = True
    return record


class TracedEvidenceGate(unittest.TestCase):
    """V-3 and V-4. Presence alone is not evidence."""

    def test_the_gate_rejects_every_invalid_shape(self):
        for label, trace in INVALID_TRACES:
            ok, why = checker.valid_trace_record(trace)
            self.assertFalse(ok, label + " was accepted")
            self.assertTrue(why)

    def test_the_gate_accepts_a_structurally_valid_record(self):
        ok, _ = checker.valid_trace_record(VALID_TRACE)
        self.assertTrue(ok)

    def test_every_traced_case_rejects_every_invalid_shape(self):
        """All eight, including the four whose rules never read the trace."""
        for name in sorted(fc.TRACED_CASES):
            for label, trace in INVALID_TRACES:
                status, why = checker.score_case(name, traced_record(name, trace))
                self.assertEqual(status, checker.INVALID,
                                 "%s accepted %s: %s" % (name, label, why))

    def test_every_traced_case_still_scores_with_a_valid_record(self):
        for name in sorted(fc.TRACED_CASES):
            status, why = checker.score_case(name, traced_record(name, VALID_TRACE))
            self.assertEqual(status, checker.PASS, name + ": " + why)

    def test_malformed_evidence_is_invalid_and_never_fail(self):
        for name in sorted(fc.TRACED_CASES):
            for _, trace in INVALID_TRACES:
                status, _ = checker.score_case(name, traced_record(name, trace))
                self.assertNotEqual(status, checker.FAIL, name)

    def test_an_untraced_case_is_unaffected_by_the_gate(self):
        for name in ("R1", "T1", "O1", "P1"):
            self.assertFalse(fc.BY_NAME[name]["traced"])
            record = traced_record(name, None)
            record.pop("trace")
            self.assertNotEqual(checker.score_case(name, record)[0],
                                checker.INVALID)

    def test_a_blocked_traced_case_does_not_need_a_trace(self):
        record = {"blocked": "clone3_unavailable", "launch_returned": None}
        self.assertEqual(checker.score_case("M3", record)[0], checker.BLOCKED)

    def test_the_gate_lives_in_one_place(self):
        """No per-case rule may be responsible for remembering the invariant."""
        source = (EXP / "checker.py").read_text(encoding="utf-8")
        self.assertEqual(source.count("def valid_trace_record"), 1)
        self.assertEqual(source.count("valid_trace_record(record.get"), 1)

    def test_the_child_window_does_not_publish_a_raw_pid(self):
        """P-14: child_syscalls and stage_sequence are evidence; the pid is not."""
        nl = chr(10)
        text = (clone3_line(ret="31337") + nl
                + "[pid 31337] execveat(3) = 0" + nl
                + WAITID.replace("222", "31337") + nl)
        window = ob.parse_strace_child_window(text)
        self.assertEqual(window["child_pid"], 31337)
        # A realistic account name. NOTE: a very short one corrupts unrelated
        # text by substring replacement -- recorded as finding V-5 and
        # deliberately NOT fixed in this task.
        s = evidence.Sanitiser(work="/home/runner/w", home="/home/runner",
                               user="runner")
        out = evidence.serialise(s.record({"trace": window}))
        self.assertNotIn("31337", out)
        self.assertIn("child_syscalls", out)
        self.assertIn("stage_sequence", out)
        self.assertEqual(s.record({"trace": window})["trace"]["child_pid"],
                         evidence.WITHHELD)

    def test_the_driver_marks_window_integrity(self):
        good = ob.parse_strace_child_window(
            clone3_line() + "\n[pid   222] execveat(3) = 0\n")
        self.assertIs(good["integrity_ok"], True)
        orphan = ob.parse_strace_child_window(
            "111   <... clone3 resumed> => {pidfd=[4]}, 88) = 222\n"
            "[pid   222] execveat(3) = 0\n")
        self.assertTrue(orphan is None or orphan["integrity_ok"] is False)


class AdverseVerdictPathV(unittest.TestCase):
    """The load-bearing invariant, end to end."""

    def _aggregate(self, text):
        facts = ob.parse_pidfd_acquisition(text)
        obs = observation(acquisition=facts,
                          trace={"child_syscalls": ["dup2"], "integrity_ok": True},
                          trace_sha256="a" * 64)
        record = driver.evaluate(driver.CASE_PLANS["M3"], obs)
        statuses = {n: (checker.PASS, "ok") for n in fc.MEMBERSHIP}
        statuses["M3"] = checker.score_case("M3", record)
        return statuses["M3"][0], checker.verdict(statuses)[0]

    def test_the_exact_v1_example_no_longer_rejects(self):
        """The record that previously produced the strongest wrong verdict."""
        text = ("111   clone3({flags=CLONE_PIDFD, pidfd=0x7ffd0000, "
                "exit_signal=SIGCHLD} <unfinished ...>\n"
                "[pid   222] execveat(3, \"\", NULL, NULL, AT_EMPTY_PATH) = 0\n"
                "111   <... clone3 resumed> => {pidfd=[4]}, 88")
        status, aggregate = self._aggregate(text)
        self.assertEqual(status, checker.INVALID)
        self.assertEqual(aggregate, checker.INCONCLUSIVE)
        self.assertNotEqual(aggregate, checker.REJECTED)

    def test_incomplete_malformed_and_ambiguous_never_reject(self):
        cases = {
            "lost resume": ("111   clone3({flags=CLONE_PIDFD, pidfd=0x1} "
                            "<unfinished ...>\n"),
            "orphan resume": ("111   <... clone3 resumed> => {pidfd=[4]}, 88) "
                              "= 222\n"),
            "malformed prefix": "[pid   ??? ] clone3({flags=CLONE_PIDFD}, 88) = 1\n",
            "truncated single line": ("111   clone3({flags=CLONE_PIDFD, "
                                      "pidfd=0x1}, 88"),
            "two clone3": clone3_line() + "\n" + clone3_line(ret="333") + "\n",
            "empty": "",
            "noise": "some unrelated text\n",
        }
        for label, text in cases.items():
            status, aggregate = self._aggregate(text)
            self.assertEqual(status, checker.INVALID, label)
            self.assertEqual(aggregate, checker.INCONCLUSIVE, label)

    def test_only_reconstructed_facts_reach_a_mechanism_fail(self):
        cases = {
            "observed error": clone3_line(out=None, ret="-1 EPERM (x)") + "\n",
            "no CLONE_PIDFD": clone3_line(flags="CLONE_VM") + "\n" + WAITID + "\n",
            "pidfd_open on the child":
                clone3_line() + "\n111   pidfd_open(222, 0) = 4\n" + WAITID + "\n",
        }
        for label, text in cases.items():
            status, aggregate = self._aggregate(text)
            self.assertEqual(status, checker.FAIL, label)
            self.assertEqual(aggregate, checker.REJECTED, label)


# =========================================== F-1 / F-3: current-vocabulary closure
ADVERSARIAL_USERNAMES = ("ci", "run", "test", "id", "pid", "fd", "exec",
                         "clone", "wait", "u", "a", "user", "root")


def sanitisers():
    for user in ADVERSARIAL_USERNAMES:
        yield user, evidence.Sanitiser(work="/home/%s/w" % user,
                                       home="/home/" + user,
                                       build="/home/%s/b" % user, user=user)


def independent_vocabulary():
    """Enumerate the CURRENT fixed public symbolic vocabulary INDEPENDENTLY.

    Deliberately NOT ``evidence.vocabulary()``. This walks the registries
    itself, so if the sanitiser's own derivation ever drops a source, the two
    disagree and this test fails -- which is the whole point. Deriving both
    sides from one helper would make the test agree with any mistake.
    """
    words = set()
    for case in fc.CASES:
        words.add(case["case"])
        if case["predict"]:
            words.add(case["predict"])
        words.update(case["safe"] or ())
        words.update(case["gates"])
    words.update(fc.STAGES)
    words.update(fc.BLOCK_REASONS)
    words.update(fc.CHILD_PERMITTED_SYSCALLS)
    words.update(fc.CHILD_FORBIDDEN_SYSCALLS)
    words.update(fc.CHILD_TEST_INJECTION_SYSCALLS)
    words.update(fc.CHILD_INJECTION_MODES)
    words.update(fc.PARENT_CONTROL_MODES)
    words.update(fc.DECISIONS)
    words.update(fc.DECISIONS.values())
    words.update(fc.M3_EVIDENCE_FACTS)

    words.update(ob.RULES)
    words.update(ob.SPIKE_DISPOSITIONS)
    words.update(ob.SPIKE_TIMEOUT_DISPOSITIONS)
    words.update(ob.SPIKE_REFUSALS)
    words.update(ob.SIGNAL_NAMES.values())
    words.update(ob.ERRNO_NAMES.values())
    words.update(ob.REPORT_STATES)
    words.update({ob.RETURN_OBSERVED_SUCCESS, ob.RETURN_OBSERVED_ERROR,
                  ob.RETURN_NOT_OBSERVED, ob.EVIDENCE_DIRECT,
                  ob.LIFECYCLE_WAITID, ob.LIFECYCLE_SEND_SIGNAL,
                  ob.LIFECYCLE_POLL, ob.EXEC_PRE_EXEC_ERROR,
                  ob.EXEC_DIED_BEFORE_EXEC, ob.EXEC_REACHED,
                  ob.EXEC_UNINTERPRETABLE})

    words.update({checker.PASS, checker.FAIL, checker.INVALID, checker.BLOCKED,
                  checker.ACCEPTED, checker.REJECTED, checker.INCONCLUSIVE})

    # The driver's fixed symbolic registries -- the class F-1 named.
    words.update(driver.SETUPS)
    words.update(driver.PARENT_STATES)
    words.update(driver.POSED_CHECKS)
    words.update(driver.ALL_CHANNELS)

    # Published observation schema and role names.
    normalised = ob.normalise_acquisition({}) or {}
    words.update(normalised)
    words.update(normalised.get("trace_integrity") or {})
    words.update({"DIRECT_CHILD", "DIRECT_CHILD_PIDFD", "CompleteAtEof",
                  "WriterRetainedAfterChildExit", "child_syscalls",
                  "stage_sequence", "integrity_ok"})
    return sorted(w for w in words if isinstance(w, str) and w)


class CurrentVocabularyClosure(unittest.TestCase):
    """F-1 / F-3. Every reachable fixed symbolic token survives byte-exact."""

    def test_the_enumeration_is_substantial(self):
        vocabulary = independent_vocabulary()
        self.assertGreater(len(vocabulary), 200)
        # The two tokens the final review named must be in the enumeration, or
        # the test would pass by not looking.
        self.assertIn("sigterm_blocked_sigpipe_ignored", vocabulary)
        self.assertIn("returned_before_descendant_lifetime", vocabulary)

    def test_every_current_token_survives_every_adversarial_username(self):
        vocabulary = independent_vocabulary()
        broken = []
        for user, s in sanitisers():
            for token in vocabulary:
                if s.text(token) != token:
                    broken.append((user, token, s.text(token)))
        self.assertEqual(broken, [], "corrupted fixed vocabulary")

    def test_the_sanitiser_derivation_covers_the_independent_enumeration(self):
        """If a registry is dropped from evidence.vocabulary(), this fails."""
        missing = sorted(set(independent_vocabulary()) - evidence.vocabulary())
        self.assertEqual(missing, [])

    def test_driver_registries_are_a_derivation_source(self):
        for name in ("SETUPS", "PARENT_STATES", "POSED_CHECKS"):
            registry = getattr(driver, name)
            self.assertTrue(registry, name + " is empty")
            for token in registry:
                self.assertIn(token, evidence.vocabulary(), name + ":" + token)

    def test_a_newly_registered_symbol_would_be_protected(self):
        """The fix must close the CLASS, not the two named instances."""
        driver.PARENT_STATES["a_freshly_registered_parent_state_name"] = None
        evidence._reset_vocabulary_cache()          # re-derive
        try:
            token = "a_freshly_registered_parent_state_name"
            self.assertGreaterEqual(len(token), 28)
            for _, s in sanitisers():
                self.assertEqual(s.text(token), token)
        finally:
            driver.PARENT_STATES.pop(
                "a_freshly_registered_parent_state_name", None)
            evidence._reset_vocabulary_cache()


class F1RealPlanRegression(unittest.TestCase):
    """F-1, against the REAL T6 plan rather than the token in isolation."""

    def test_t6_parent_is_the_named_token(self):
        self.assertEqual(driver.CASE_PLANS["T6"].as_dict()["parent"],
                         "sigterm_blocked_sigpipe_ignored")

    def test_t6_parent_survives_public_sanitisation(self):
        plan = driver.CASE_PLANS["T6"].as_dict()
        for user, s in sanitisers():
            out = s.record({"cases": {"T6": {"plan": plan}}})
            self.assertEqual(out["cases"]["T6"]["plan"]["parent"],
                             "sigterm_blocked_sigpipe_ignored", user)

    def test_every_plan_symbolic_field_survives(self):
        """The whole published plan table, not just T6."""
        fields = ("case", "binary", "rule", "setup", "parent", "argv0",
                  "posed_when")
        for user, s in sanitisers():
            for plan in driver._PLAN_LIST:
                published = plan.as_dict()
                out = s.record(published)
                for field in fields:
                    self.assertEqual(out[field], published[field],
                                     "%s %s under %r" % (plan.case, field, user))
                self.assertEqual(out["channels"], published["channels"],
                                 plan.case)


class F3UnreachableButRegistered(unittest.TestCase):
    def test_the_unreferenced_check_is_still_protected(self):
        token = "returned_before_descendant_lifetime"
        self.assertIn(token, driver.POSED_CHECKS)
        for user, s in sanitisers():
            self.assertEqual(s.text(token), token, user)

    def test_it_is_still_referenced_by_no_plan(self):
        """Protected because it is registered, not by being made reachable."""
        users = [p.case for p in driver._PLAN_LIST
                 if p.posed_when == "returned_before_descendant_lifetime"]
        self.assertEqual(users, [])


class F2LongDataStaysOpaque(unittest.TestCase):
    """The negative control: vocabulary protection is not a length exemption."""

    def test_a4s_long_argument_may_still_be_opaque(self):
        long_args = [a for a in driver.CASE_PLANS["A4"].as_dict()["helper_args"]
                     if len(a) > 100]
        self.assertTrue(long_args, "A4 no longer carries a long argument")
        s = evidence.Sanitiser(user="ci")
        self.assertIn("<OPAQUE:", s.text(long_args[0]))

    def test_genuinely_opaque_values_are_still_redacted(self):
        s = evidence.Sanitiser(user="ci")
        for probe in ("Zk9q7Lm2XvQp4Rt8Nw1Ys6Bd3Hg5Jc0",
                      "ghp_" + "A" * 36, "AKIAIOSFODNN7EXAMPLE",
                      "x7Yq" * 12):
            self.assertNotEqual(s.text(probe), probe, probe[:20])

    def test_digests_are_still_preserved(self):
        s = evidence.Sanitiser(user="ci")
        self.assertEqual(s.text("a" * 64), "a" * 64)
        self.assertEqual(s.text("b" * 40), "b" * 40)

    def test_argv_and_environment_data_are_not_vocabulary(self):
        s = evidence.Sanitiser(user="ci")
        self.assertNotIn("x" * 4096, evidence.vocabulary())
        out = s.record({"environ": ["SECRET=" + "q" * 40]})
        self.assertNotIn("q" * 40, evidence.serialise(out))


# =========================================================== P-16: host facts
# The independent M-1 check recorded P-16: harness.preflight() collected
# ``uname -a``, whose value embeds the machine's nodename, and one publication
# path -- HALT_PREFLIGHT -- returned its document before the sanitisation
# boundary. Two things are tested here, because the fix has two halves: the
# host name is no longer COLLECTED, and no document reaches stdout without
# passing the same P-14 logic.

HOSTNAME_PROBE = "helm-secret-host-9371"
BROAD_UNAME = ("Linux " + HOSTNAME_PROBE + " 6.5.0-1015-azure #15-Ubuntu SMP "
               "Wed Sep 3 12:00:00 UTC 2025 x86_64 x86_64 x86_64 GNU/Linux")

EXPERIMENT_MODULES = sorted(p for p in EXP.glob("*.py"))


def tainted_preflight():
    """A fabricated preflight carrying every class of private value.

    Nothing here is measured: no command runs, no case is posed, and the
    kernel release and architecture are the harmless facts the experiment does
    need, so a fix that redacts everything fails this as loudly as a fix that
    redacts nothing.
    """
    return {
        "uname": BROAD_UNAME,
        "nodename": HOSTNAME_PROBE,
        "hostname": HOSTNAME_PROBE,
        "kernel_name": "Linux",
        "kernel_release": "6.5.0-1015-azure",
        "arch": "x86_64",
        "user": "helm-secret-user",
        "logname": "helm-secret-user",
        "home": "/home/helm-secret-user/work/helm",
        "strace": "/usr/bin/strace",
        "token": "ghp_" + "Z" * 36,
        "environ": ["ACTIONS_RUNTIME_TOKEN=" + "s" * 44],
        "block_reasons": {},
    }


class P16HostCollectionIsMinimal(unittest.TestCase):
    """P-16 part one: the nodename is never gathered in the first place."""

    def _uname_flags(self):
        """Every uname invocation in harness.py, read from its source."""
        tree = ast.parse((EXP / "harness.py").read_text(encoding="utf-8"))
        flags = []
        for node in ast.walk(tree):
            if not isinstance(node, ast.Call) or not node.args:
                continue
            arg = node.args[0]
            if not isinstance(arg, (ast.List, ast.Tuple)):
                continue
            parts = [e.value for e in arg.elts
                     if isinstance(e, ast.Constant) and isinstance(e.value, str)]
            if parts and parts[0] == "uname":
                flags.extend(parts[1:])
        return flags

    def test_uname_is_only_ever_called_with_narrow_flags(self):
        flags = self._uname_flags()
        self.assertTrue(flags, "harness no longer calls uname at all")
        self.assertEqual(sorted(set(flags)), ["-m", "-r", "-s"])

    def test_the_broad_and_nodename_forms_are_gone(self):
        for forbidden in ("-a", "-n", "--all", "--nodename"):
            self.assertNotIn(forbidden, self._uname_flags(), forbidden)

    def test_preflight_declares_no_broad_host_field(self):
        source = (EXP / "harness.py").read_text(encoding="utf-8")
        tree = ast.parse(source)
        keys = set()
        for node in ast.walk(tree):
            if isinstance(node, ast.Dict):
                keys.update(k.value for k in node.keys
                            if isinstance(k, ast.Constant)
                            and isinstance(k.value, str))
        self.assertNotIn("uname", keys)
        self.assertIn("kernel_name", keys)
        # The narrow facts the experiment actually needs are still collected.
        for needed in ("kernel_release", "arch"):
            self.assertIn(needed, keys, needed)

    def _commands_invoked(self):
        """Every external command harness.py runs, read from its source.

        Deliberately AST-derived rather than a substring search: the word
        "hostname" appears in the comments that explain why it is not
        collected, and a test that cannot tell a comment from a command would
        fail on its own documentation.
        """
        tree = ast.parse((EXP / "harness.py").read_text(encoding="utf-8"))
        commands = set()
        for node in ast.walk(tree):
            if not isinstance(node, ast.Call):
                continue
            name = ast.unparse(node.func)
            if name.endswith("which") and node.args:
                if isinstance(node.args[0], ast.Constant):
                    commands.add(node.args[0].value)
                continue
            for arg in node.args:
                if isinstance(arg, (ast.List, ast.Tuple)) and arg.elts:
                    first = arg.elts[0]
                    if isinstance(first, ast.Constant) and                             isinstance(first.value, str):
                        commands.add(first.value)
        return commands

    def test_no_host_identity_command_is_invoked(self):
        invoked = self._commands_invoked()
        self.assertTrue(invoked, "no commands found; the walk is broken")
        for forbidden in ("hostname", "hostnamectl", "dnsdomainname",
                          "domainname", "whoami", "id", "getent", "host"):
            self.assertNotIn(forbidden, invoked, forbidden)


class P16PublicationBoundary(unittest.TestCase):
    """P-16 part two: one boundary, and every path goes through it."""

    def test_no_module_serialises_json_of_its_own(self):
        """A whole-tree property, not one line: only evidence may serialise."""
        offenders = []
        for path in EXPERIMENT_MODULES:
            if path.name == "evidence.py":
                continue
            tree = ast.parse(path.read_text(encoding="utf-8"))
            for node in ast.walk(tree):
                if (isinstance(node, ast.Call)
                        and isinstance(node.func, ast.Attribute)
                        and node.func.attr in ("dumps", "dump")
                        and getattr(node.func.value, "id", "") == "json"):
                    offenders.append("%s:%d" % (path.name, node.lineno))
        self.assertEqual(offenders, [])

    def test_no_module_writes_to_stdout_of_its_own(self):
        offenders = []
        for path in EXPERIMENT_MODULES:
            if path.name == "evidence.py":
                continue
            tree = ast.parse(path.read_text(encoding="utf-8"))
            for node in ast.walk(tree):
                if isinstance(node, ast.Call):
                    name = (getattr(node.func, "id", None)
                            or ast.unparse(node.func))
                    if name == "print" or name.endswith("stdout.write"):
                        offenders.append("%s:%d %s"
                                         % (path.name, node.lineno, name))
        self.assertEqual(offenders, [])

    def test_every_main_block_publishes_through_the_boundary(self):
        """Every module that can be run directly emits via evidence.publish."""
        checked = 0
        for path in EXPERIMENT_MODULES:
            source = path.read_text(encoding="utf-8")
            if '__name__ == "__main__"' not in source:
                continue
            tree = ast.parse(source)
            for node in tree.body:
                if not isinstance(node, ast.If):
                    continue
                if '__name__' not in ast.unparse(node.test):
                    continue
                body = ast.unparse(node)
                if "publish" in body or "sys.exit(main())" in body:
                    checked += 1
                else:
                    self.fail("%s emits without the boundary:\n%s"
                              % (path.name, body[:200]))
        self.assertGreaterEqual(checked, 5)

    def test_run_trial_returns_a_raw_document_and_its_sanitiser(self):
        """The boundary must be what removes the taint, not luck upstream."""
        code, document, sanitiser = self._halt_trial()
        self.assertEqual(code, 6)
        self.assertIsInstance(sanitiser, evidence.Sanitiser)
        # RAW on the way out of run_trial -- this is deliberate.
        self.assertIn(HOSTNAME_PROBE, json.dumps(document))

    # ------------------------------------------------------------- helpers
    def _halt_trial(self):
        """Reach the HALT_PREFLIGHT return with nothing built and nothing posed.

        ``preflight_gates`` is replaced so no static-link probe is compiled, and
        ``auth`` is None because the halt returns before any case is posed --
        which is itself the property being relied on.
        """
        halt = [{"gate": "clone3", "detail": "clone3 is unavailable",
                 "evidence": {"available": False}}]
        real_preflight, real_gates = harness.preflight, runner.preflight_gates
        harness.preflight = tainted_preflight
        runner.preflight_gates = lambda preflight, build_dir: halt
        out = tempfile.mkdtemp(prefix="launch-exec-01-halt-")
        try:
            return runner.run_trial("target/launch-exec-01-not-a-real-dir",
                                    None, out)
        finally:
            harness.preflight, runner.preflight_gates = real_preflight, real_gates
            shutil.rmtree(out, ignore_errors=True)

    def _published(self, argv):
        buffer = io.StringIO()
        real = harness.preflight
        harness.preflight = tainted_preflight
        try:
            with contextlib.redirect_stdout(buffer):
                code = runner.main(argv)
        finally:
            harness.preflight = real
        return code, buffer.getvalue()

    # ------------------------------------------------ behavioural coverage
    def test_the_not_run_document_goes_through_the_boundary(self):
        code, text = self._published([])
        self.assertEqual(code, 3)
        self.assertEqual(json.loads(text)["status"], "NOT_RUN")
        self.assertNotIn(HOSTNAME_PROBE, text)

    def test_the_preflight_only_document_is_sanitised(self):
        code, text = self._published(["--preflight-only"])
        self.assertEqual(code, 0)
        published = json.loads(text)["preflight"]
        self.assertNotIn(HOSTNAME_PROBE, text)
        self.assertEqual(published["uname"], evidence.HOST_DESCRIPTOR)
        self.assertEqual(published["nodename"], evidence.HOST_DESCRIPTOR)
        self.assertEqual(published["hostname"], evidence.HOST_IDENTITY)

    def test_the_driver_completeness_document_goes_through_the_boundary(self):
        code, text = self._published(["--driver-completeness"])
        self.assertEqual(code, 0)
        self.assertEqual(json.loads(text)["completeness"]["driver_total"], 72)

    def test_the_freeze_verification_document_goes_through_the_boundary(self):
        _, text = self._published(["--verify-freeze"])
        self.assertIn("freeze_verified", json.loads(text))
        self.assertNotIn(HOSTNAME_PROBE, text)


class P16OriginalLeakCannotRecur(unittest.TestCase):
    """P-16 sections 3 and 5: the exact leak, through the exact paths."""

    def _publish(self, document, sanitiser=None):
        return evidence.publish(document, sanitiser, stream=io.StringIO())

    def test_the_broad_uname_value_never_reaches_public_evidence(self):
        for key in ("uname", "uname_all", "nodename", "node", "fqdn"):
            text = self._publish({"preflight": {key: BROAD_UNAME}})
            self.assertNotIn(HOSTNAME_PROBE, text, key)
            self.assertIn(evidence.HOST_DESCRIPTOR, text, key)

    def test_the_bare_hostname_never_reaches_public_evidence(self):
        for key in ("hostname", "host", "runner_name"):
            text = self._publish({"preflight": {key: HOSTNAME_PROBE}})
            self.assertNotIn(HOSTNAME_PROBE, text, key)

    def test_the_halt_preflight_document_is_sanitised(self):
        """Section 5's negative control, through the real halt path."""
        boundary = P16PublicationBoundary("test_the_not_run_document_"
                                          "goes_through_the_boundary")
        code, document, sanitiser = boundary._halt_trial()
        text = self._publish(document, sanitiser)
        published = json.loads(text)

        self.assertEqual(code, 6)
        # Private values gone.
        self.assertNotIn(HOSTNAME_PROBE, text)
        self.assertNotIn("helm-secret-user", text)
        self.assertNotIn("Z" * 36, text)
        self.assertNotIn("s" * 44, text)
        # Required platform facts kept.
        self.assertEqual(published["preflight"]["kernel_release"],
                         "6.5.0-1015-azure")
        self.assertEqual(published["preflight"]["arch"], "x86_64")
        self.assertEqual(published["preflight"]["kernel_name"], "Linux")
        # The halt itself still says what happened.
        self.assertEqual(published["status"], "HALT_PREFLIGHT")
        self.assertIn("no case was posed", published["reason"])
        self.assertEqual(published["halts"][0]["gate"], "clone3")

    def test_the_fix_is_not_blanket_redaction(self):
        text = self._publish({"preflight": tainted_preflight()})
        published = json.loads(text)["preflight"]
        self.assertEqual(published["kernel_name"], "Linux")
        self.assertEqual(published["kernel_release"], "6.5.0-1015-azure")
        self.assertEqual(published["arch"], "x86_64")
        self.assertEqual(published["strace"], "/usr/bin/strace")

    def test_the_boundary_inherits_the_m1_fail_closed_rule(self):
        """A document cannot be published while the vocabulary is unavailable."""
        import types
        real = sys.modules.get("driver")
        evidence._reset_vocabulary_cache()
        sys.modules["driver"] = types.ModuleType("driver")
        try:
            with self.assertRaises(evidence.VocabularyUnavailable):
                self._publish({"status": "NOT_RUN"})
        finally:
            if real is not None:
                sys.modules["driver"] = real
            else:
                sys.modules.pop("driver", None)
            evidence._reset_vocabulary_cache()
        self.assertIn("NOT_RUN", self._publish({"status": "NOT_RUN"}))


# ======================================================= Trial #2 corrections
# Trial #1 crossed the immutability boundary and aborted in E5b's fixture setup
# with EBADF, taking E1-E5's already-scored records with it because they lived
# only in memory. Two defects, both fixed here and both tested here: the access
# mode, and the preservation architecture.
#
# Nothing in this section poses a preregistered case, builds a HELM ELF or runs
# launcher_spike. Where a file is needed it is a throwaway byte pattern in a
# temporary directory.

# subprocess pass_fds, os.pread/pwrite, POSIX mode bits and unprivileged
# symlinks are all Unix-only. Their absence on Windows is exactly why the
# Trial #1 defect survived to the trial, so these run on Linux CI and are
# skipped here rather than quietly weakened.
POSIX_ONLY = unittest.skipUnless(
    os.name == "posix", "POSIX-only; Linux CI is the authority for this")

TRIAL2_FD_READS = {"read", "pread", "readv", "preadv"}
TRIAL2_FD_WRITES = {"write", "pwrite", "writev", "pwritev"}


def descriptor_map(function_node):
    """Every ``os.open`` in a function, mapped to the ops on THAT variable.

    Function-level "does this function read anywhere" is what makes an audit
    miss a correct two-descriptor sequence, so this follows the variable.
    """
    opened = {}
    for node in ast.walk(function_node):
        if isinstance(node, ast.Assign) and isinstance(node.value, ast.Call) \
                and ast.unparse(node.value.func) == "os.open":
            name = getattr(node.targets[0], "id", None)
            if name:
                opened[name] = {"flags": ast.unparse(node.value.args[1])
                                if len(node.value.args) > 1 else "",
                                "ops": []}
    for node in ast.walk(function_node):
        if not isinstance(node, ast.Call) or not node.args:
            continue
        name = ast.unparse(node.func)
        if not name.startswith("os."):
            continue
        target = ast.unparse(node.args[0])
        if target in opened:
            opened[target]["ops"].append(name.split(".")[-1])
    return opened


def driver_function(name):
    tree = ast.parse((EXP / "driver.py").read_text(encoding="utf-8"))
    return [n for n in ast.walk(tree)
            if isinstance(n, ast.FunctionDef) and n.name == name][0]


def fake_build(tmp, names=("helper_report",), payload=b"\x7fELF" + b"Z" * 512):
    """A throwaway build directory and its identity map. Not a HELM binary."""
    build = pathlib.Path(tmp)
    build.mkdir(parents=True, exist_ok=True)
    for name in names:
        (build / name).write_bytes(payload)
    return build, harness.build_identity(str(build))


class Trial2E5bAccessModes(unittest.TestCase):
    """The defect that aborted Trial #1, and the fix that keeps E5b's claim."""

    def setUp(self):
        self.fn = driver_function("_setup_writer_open_closed")
        self.fds = descriptor_map(self.fn)

    def test_the_writer_is_still_opened_write_only(self):
        """Widening to O_RDWR would silently change what E5b tests."""
        wronly = [v for v in self.fds.values() if "O_WRONLY" in v["flags"]]
        self.assertEqual(len(wronly), 1, "E5b must have exactly one writer")
        for var, info in self.fds.items():
            self.assertNotIn("O_RDWR", info["flags"], var)

    def test_nothing_reads_through_the_write_only_descriptor(self):
        """The exact Trial #1 defect: pread on an O_WRONLY fd is EBADF."""
        for var, info in self.fds.items():
            if "O_WRONLY" in info["flags"]:
                reads = [o for o in info["ops"] if o in TRIAL2_FD_READS]
                self.assertEqual(reads, [], "%s is read from" % var)

    def test_the_byte_is_read_through_a_separate_read_only_descriptor(self):
        readers = [v for v in self.fds.values() if "O_RDONLY" in v["flags"]]
        self.assertEqual(len(readers), 1)
        self.assertTrue(any(o in TRIAL2_FD_READS for o in readers[0]["ops"]))
        for info in readers:
            self.assertEqual([o for o in info["ops"] if o in TRIAL2_FD_WRITES],
                             [])

    def test_the_reader_is_closed_before_the_writer_opens(self):
        """Source order: read, close, then open the writer."""
        body = ast.unparse(self.fn)
        close_reader = body.index("os.close(read_fd)")
        open_writer = body.index("os.open(path, os.O_WRONLY)")
        self.assertLess(close_reader, open_writer)

    def test_every_descriptor_is_closed_on_the_failure_path(self):
        """Both opens sit in try/finally, so a raise cannot leak a descriptor."""
        tries = [n for n in ast.walk(self.fn) if isinstance(n, ast.Try)]
        self.assertEqual(len(tries), 2)
        for node in tries:
            self.assertTrue(node.finalbody)
            self.assertIn("os.close", ast.unparse(node.finalbody[0]))

    # ------------------------------------------------- behavioural, on a mock
    def _inject(self, pread, pwrite):
        """os.pread/os.pwrite are Unix-only, so they are injected rather than
        merely replaced. Windows has neither, which is precisely why the
        Trial #1 defect could not surface until the trial itself ran."""
        self._saved = {n: getattr(os, n, None) for n in ("pread", "pwrite")}
        os.pread, os.pwrite = pread, pwrite

    def _restore(self):
        for name, value in getattr(self, "_saved", {}).items():
            if value is None:
                if hasattr(os, name):
                    delattr(os, name)
            else:
                setattr(os, name, value)

    def _ctx(self, tmp):
        build, identity = fake_build(tmp)
        return driver.TrialContext(build=str(build), work=str(build),
                                   preflight={}, freeze={},
                                   sanitiser=evidence.Sanitiser(),
                                   build_identity=identity), build

    @unittest.skipUnless(hasattr(os, "pread") and hasattr(os, "pwrite"),
                         "os.pread/os.pwrite are Unix-only; Linux CI runs "
                         "this, and their absence here is exactly why the "
                         "Trial #1 defect survived to the trial")
    def test_the_executable_is_byte_identical_afterwards(self):
        tmp = tempfile.mkdtemp(prefix="e5b-")
        try:
            ctx, build = self._ctx(tmp)
            plan = driver.CASE_PLANS["E5b"]
            before = (build / plan.binary).read_bytes()
            info = driver.SETUPS["writer_open_closed"](ctx, plan)
            self.assertNotIn("not_posed", info, info)
            after = pathlib.Path(info["exec_path"]).read_bytes()
            self.assertEqual(after, before, "E5b changed the executable")
        finally:
            shutil.rmtree(tmp, ignore_errors=True)

    def test_a_short_read_does_not_pose_the_case(self):
        tmp = tempfile.mkdtemp(prefix="e5b-short-read-")
        try:
            ctx, _ = self._ctx(tmp)
            self._inject(lambda fd, n, off: b"", lambda fd, d, o: len(d))
            info = driver.SETUPS["writer_open_closed"](ctx,
                                                       driver.CASE_PLANS["E5b"])
            self.assertIn("not_posed", info)
            self.assertIn("0 bytes read", info["not_posed"])
            status, _ = checker.score_case("E5b", info)
            self.assertEqual(status, checker.INVALID)
        finally:
            self._restore()
            shutil.rmtree(tmp, ignore_errors=True)

    def test_a_short_write_does_not_pose_the_case(self):
        tmp = tempfile.mkdtemp(prefix="e5b-short-write-")
        try:
            ctx, _ = self._ctx(tmp)
            self._inject(lambda fd, n, off: b"A", lambda fd, d, o: 0)
            info = driver.SETUPS["writer_open_closed"](ctx,
                                                       driver.CASE_PLANS["E5b"])
            self.assertIn("not_posed", info)
            self.assertIn("wrote 0 bytes", info["not_posed"])
            status, _ = checker.score_case("E5b", info)
            self.assertEqual(status, checker.INVALID)
        finally:
            self._restore()
            shutil.rmtree(tmp, ignore_errors=True)


class Trial2DefectClassAudit(unittest.TestCase):
    """The class, not the instance."""

    def test_no_python_descriptor_has_an_access_mode_mismatch(self):
        offenders = []
        for path in sorted(EXP.glob("*.py")):
            tree = ast.parse(path.read_text(encoding="utf-8"))
            for fn in [n for n in ast.walk(tree)
                       if isinstance(n, ast.FunctionDef)]:
                for var, info in descriptor_map(fn).items():
                    reads = [o for o in info["ops"] if o in TRIAL2_FD_READS]
                    writes = [o for o in info["ops"] if o in TRIAL2_FD_WRITES]
                    if "O_WRONLY" in info["flags"] and reads:
                        offenders.append((path.name, fn.name, var, "read"))
                    if "O_RDONLY" in info["flags"] and writes:
                        offenders.append((path.name, fn.name, var, "write"))
        self.assertEqual(offenders, [])

    def test_no_descriptor_is_used_after_close(self):
        """Within one function, source order: close must not precede a use."""
        offenders = []
        for path in sorted(EXP.glob("*.py")):
            tree = ast.parse(path.read_text(encoding="utf-8"))
            for fn in [n for n in ast.walk(tree)
                       if isinstance(n, ast.FunctionDef)]:
                for var in descriptor_map(fn):
                    closed_at = uses_after = None
                    for node in ast.walk(fn):
                        if not isinstance(node, ast.Call) or not node.args:
                            continue
                        name = ast.unparse(node.func)
                        if not name.startswith("os.") or \
                                ast.unparse(node.args[0]) != var:
                            continue
                        if name == "os.close":
                            closed_at = node.lineno
                        elif closed_at is not None and node.lineno > closed_at:
                            uses_after = (name, node.lineno)
                    if uses_after:
                        offenders.append((path.name, fn.name, var, uses_after))
        self.assertEqual(offenders, [])


class Trial2BuildIdentity(unittest.TestCase):
    """The trial must be able to say which bytes it ran, even if it aborts."""

    def test_identity_covers_every_generated_file(self):
        tmp = tempfile.mkdtemp(prefix="identity-")
        try:
            build, identity = fake_build(
                tmp, names=("helper_report", "helper_alt", "fixture.bin"))
            self.assertEqual(sorted(identity),
                             ["fixture.bin", "helper_alt", "helper_report"])
            for name, entry in identity.items():
                self.assertEqual(sorted(entry), ["kind", "sha256", "size"])
                self.assertEqual(len(entry["sha256"]), 64)
                self.assertEqual(entry["size"],
                                 (build / name).stat().st_size)
        finally:
            shutil.rmtree(tmp, ignore_errors=True)

    def test_elf_classification_reads_the_headers(self):
        static = bytearray(b"\x7fELF\x02\x01" + b"\x00" * 118)
        static[32:40] = (64).to_bytes(8, "little")      # e_phoff
        static[54:56] = (56).to_bytes(2, "little")      # e_phentsize
        static[56:58] = (1).to_bytes(2, "little")       # e_phnum
        static[64:68] = (1).to_bytes(4, "little")       # PT_LOAD
        self.assertEqual(harness._elf_kind(bytes(static)), "elf_static")
        self.assertEqual(harness._elf_kind(b"#!/bin/sh\n"), "not_elf")
        self.assertEqual(harness._elf_kind(b""), "not_elf")
        # One PT_INTERP program header at offset 64.
        dynamic = bytearray(b"\x7fELF\x02\x01" + b"\x00" * 58 + b"\x00" * 56)
        dynamic[32:40] = (64).to_bytes(8, "little")     # e_phoff
        dynamic[54:56] = (56).to_bytes(2, "little")     # e_phentsize
        dynamic[56:58] = (1).to_bytes(2, "little")      # e_phnum
        dynamic[64:68] = (harness.PT_INTERP).to_bytes(4, "little")
        self.assertEqual(harness._elf_kind(bytes(dynamic)), "elf_dynamic")

    def test_a_setup_refuses_an_artefact_that_is_not_the_hashed_one(self):
        tmp = tempfile.mkdtemp(prefix="identity-drift-")
        try:
            build, identity = fake_build(tmp)
            ctx = driver.TrialContext(build=str(build), work=str(build),
                                      preflight={}, freeze={},
                                      sanitiser=evidence.Sanitiser(),
                                      build_identity=identity)
            plan = driver.CASE_PLANS["E1"]
            (build / plan.binary).write_bytes(b"\x7fELF" + b"Q" * 512)
            info = driver.SETUPS[plan.setup](ctx, plan)
            self.assertIn("not_posed", info)
            self.assertIn("not the one this trial hashed", info["not_posed"])
            self.assertEqual(checker.score_case("E1", info)[0], checker.INVALID)
        finally:
            shutil.rmtree(tmp, ignore_errors=True)

    def test_a_setup_refuses_when_no_identity_was_recorded(self):
        tmp = tempfile.mkdtemp(prefix="identity-missing-")
        try:
            build, _ = fake_build(tmp)
            ctx = driver.TrialContext(build=str(build), work=str(build),
                                      preflight={}, freeze={},
                                      sanitiser=evidence.Sanitiser())
            info = driver.SETUPS["copy"](ctx, driver.CASE_PLANS["E1"])
            self.assertIn("not_posed", info)
            self.assertIn("no build identity", info["not_posed"])
        finally:
            shutil.rmtree(tmp, ignore_errors=True)

    def test_verify_build_identity_reports_every_deviation(self):
        tmp = tempfile.mkdtemp(prefix="identity-verify-")
        try:
            build, identity = fake_build(tmp, names=("a.bin", "b.bin"))
            ctx = driver.TrialContext(build=str(build), work=str(build),
                                      preflight={}, freeze={},
                                      sanitiser=evidence.Sanitiser(),
                                      build_identity=identity)
            self.assertEqual(driver.verify_build_identity(ctx), [])
            (build / "a.bin").write_bytes(b"changed")
            deviations = driver.verify_build_identity(ctx)
            self.assertEqual([d["artefact"] for d in deviations], ["a.bin"])
        finally:
            shutil.rmtree(tmp, ignore_errors=True)

    def test_the_driver_never_builds_anything(self):
        """It opens what the harness produced; it must not produce its own."""
        source = (EXP / "driver.py").read_text(encoding="utf-8")
        tree = ast.parse(source)
        for node in ast.walk(tree):
            if isinstance(node, ast.Call):
                name = ast.unparse(node.func)
                self.assertNotIn(name, ("harness.build", "harness.build_identity",
                                        "make_fixtures.write"))
        for banned in ('"cc"', "'cc'", "-static"):
            self.assertNotIn(banned, source, banned)

    def test_identity_is_hashed_before_the_first_case_is_entered(self):
        """Source order inside run_trial, read from the AST."""
        tree = ast.parse((EXP / "run_launch_exec_01.py").read_text(encoding="utf-8"))
        fn = [n for n in tree.body if isinstance(n, ast.FunctionDef)
              and n.name == "run_trial"][0]
        def line_of(fragment):
            for node in ast.walk(fn):
                if isinstance(node, ast.Call) and fragment in ast.unparse(node):
                    return node.lineno
            return None
        hashed = line_of("harness.build_identity")
        journalled = line_of("jrnl.build_identity")
        entered = line_of("jrnl.case_entered")
        posed = line_of("driver.pose")
        for name, value in (("build_identity", hashed),
                            ("journal build_identity", journalled),
                            ("case_entered", entered), ("observe", posed)):
            self.assertIsNotNone(value, name + " is missing from run_trial")
        self.assertLess(hashed, journalled)
        self.assertLess(journalled, entered)
        self.assertLess(entered, posed)


class Trial2Journal(unittest.TestCase):
    """The preservation architecture Trial #1 did not have."""

    def setUp(self):
        self.tmp = tempfile.mkdtemp(prefix="journal-")
        self.path = pathlib.Path(self.tmp) / "journal.jsonl"
        self.s = evidence.Sanitiser(work="/w", home="/h", build="/b",
                                    user="runner")

    def tearDown(self):
        shutil.rmtree(self.tmp, ignore_errors=True)

    def _journal(self):
        return journal.Journal(self.path, self.s, trial="trial-002")

    def test_case_entered_is_distinguishable_from_case_completed(self):
        with self._journal() as j:
            j.case_entered("E1", 0)
            j.case_completed("E1", checker.PASS, "ok", {"launch_returned": True})
            j.case_entered("E2", 1)          # entered, then the process dies
        records, truncated = journal.read(self.path)
        self.assertFalse(truncated)
        state = journal.replay(records)
        self.assertEqual(state["entered"], ["E1", "E2"])
        self.assertEqual(sorted(state["completed"]), ["E1"])
        self.assertEqual(state["entered_not_completed"], ["E2"])
        self.assertEqual(state["last_entered"], "E2")
        self.assertEqual(state["last_completed"], "E1")

    def test_an_entered_case_is_never_given_a_frozen_status(self):
        self.assertNotIn(journal.ENTERED_NOT_COMPLETED,
                         (checker.PASS, checker.FAIL, checker.INVALID,
                          checker.BLOCKED))

    def test_completed_records_survive_the_next_case_exception(self):
        """Exactly Trial #1's shape: five completed, the sixth aborts in setup."""
        done = ["E1", "E2", "E3", "E4", "E5"]
        with self.assertRaises(OSError):
            with self._journal() as j:
                for i, name in enumerate(done):
                    j.case_entered(name, i)
                    j.case_completed(name, checker.PASS, "ok", {"case": name})
                j.case_entered("E5b", 5)
                raise OSError(9, "Bad file descriptor")
        records, truncated = journal.read(self.path)
        self.assertFalse(truncated)
        state = journal.replay(records)
        self.assertEqual(sorted(state["completed"]), done)
        self.assertEqual(state["entered_not_completed"], ["E5b"])
        self.assertEqual(state["last_entered"], "E5b")
        for name in done:
            self.assertEqual(state["completed"][name]["status"], checker.PASS)

    def test_a_completed_status_cannot_be_revised(self):
        with self._journal() as j:
            j.case_entered("E1", 0)
            j.case_completed("E1", checker.PASS, "ok", {})
            with self.assertRaises(journal.JournalError):
                j.case_completed("E1", checker.FAIL, "second thoughts", {})
            with self.assertRaises(journal.JournalError):
                j.case_entered("E1", 0)

    def test_completing_a_case_that_was_never_entered_is_refused(self):
        with self._journal() as j:
            with self.assertRaises(journal.JournalError):
                j.case_completed("E9", checker.PASS, "ok", {})

    def test_the_journal_has_no_update_or_delete(self):
        for banned in ("def update", "def delete", "def revise", "def retry"):
            self.assertNotIn(banned, (EXP / "journal.py").read_text(encoding="utf-8"))

    def test_a_torn_final_line_is_dropped_and_reported(self):
        with self._journal() as j:
            j.case_entered("E1", 0)
            j.case_completed("E1", checker.PASS, "ok", {})
        # Simulate a crash mid-write: a partial line with no newline.
        with open(self.path, "a", encoding="utf-8") as handle:
            handle.write('{"kind":"case_completed","case":"E2","stat')
        records, truncated = journal.read(self.path)
        self.assertTrue(truncated)
        state = journal.replay(records)
        self.assertEqual(sorted(state["completed"]), ["E1"])

    def test_every_record_is_exactly_one_line(self):
        with self._journal() as j:
            j.trial_begin(freeze={"a": 1}, membership={"total": 72})
            j.case_entered("E1", 0)
            j.case_completed("E1", checker.PASS, "reason with\nnewline", {})
        lines = self.path.read_text(encoding="utf-8").splitlines()
        self.assertEqual(len(lines), 3)
        for line in lines:
            self.assertIsNotNone(evidence.parse_line(line))

    def test_sequence_numbers_are_monotonic(self):
        with self._journal() as j:
            j.trial_begin(freeze={}, membership={})
            j.case_entered("E1", 0)
            j.case_completed("E1", checker.PASS, "ok", {})
        records, _ = journal.read(self.path)
        self.assertEqual([r["n"] for r in records], [0, 1, 2])

    def test_the_final_document_is_reconstructable_without_inventing_status(self):
        with self._journal() as j:
            j.case_entered("E1", 0)
            j.case_completed("E1", checker.PASS, "ok", {"launch_returned": True})
            j.case_entered("E2", 1)
        state = journal.replay(journal.read(self.path)[0])
        recovered = journal.recovered_records(state)
        self.assertEqual(sorted(recovered), ["E1"])
        # E2 was entered and never completed: the journal says so and stops
        # there. It does NOT hand the reader a status.
        self.assertNotIn("E2", recovered)
        self.assertIn("E2", state["entered_not_completed"])

    def test_the_journal_is_p14_sanitised(self):
        with self._journal() as j:
            j.preflight({"uname": "Linux secret-host-1 6.1 x86_64",
                         "hostname": "secret-host-1",
                         "user": "secret-user",
                         "kernel_release": "6.1",
                         "environ": ["TOKEN=" + "q" * 44]})
            j.case_entered("E1", 0)
            j.case_completed("E1", checker.PASS, "ok",
                             {"child_pid": 31337, "raw_trace": "clone3() = 31337",
                              "capture_prefix_base64": "c2VjcmV0",
                              "path": "/w/build/x"})
        text = self.path.read_text(encoding="utf-8")
        for leak in ("secret-host-1", "secret-user", "q" * 44, "31337",
                     "c2VjcmV0", "clone3()"):
            self.assertNotIn(leak, text, leak)
        self.assertIn(evidence.HOST_DESCRIPTOR, text)
        self.assertIn(evidence.WITHHELD, text)
        self.assertIn("6.1", text)                  # the useful fact survives
        self.assertIn(checker.PASS, text)           # so does the frozen status

    def test_the_journal_inherits_the_m1_fail_closed_rule(self):
        import types
        real = sys.modules.get("driver")
        evidence._reset_vocabulary_cache()
        sys.modules["driver"] = types.ModuleType("driver")
        try:
            with self.assertRaises(evidence.VocabularyUnavailable):
                with self._journal() as j:
                    j.case_entered("E1", 0)
        finally:
            if real is not None:
                sys.modules["driver"] = real
            else:
                sys.modules.pop("driver", None)
            evidence._reset_vocabulary_cache()


class Trial2RunnerCrashSimulation(unittest.TestCase):
    """Trial #1's exact shape, at the runner level, with nothing posed.

    Every experiment call the loop makes is replaced by a fabricated one: no
    fixture is built, no ELF exists, no launcher_spike runs and no
    ``driver.Authorisation`` is constructed. What is exercised is the
    preservation architecture -- the thing Trial #1 did not have.
    """

    def setUp(self):
        self.tmp = tempfile.mkdtemp(prefix="trial2-runner-")
        self.out = pathlib.Path(self.tmp) / "out"
        self.saved = {
            "preflight": harness.preflight,
            "gates": runner.preflight_gates,
            "build": harness.build,
            "identity": harness.build_identity,
            "verify": driver.verify_build_identity,
            "prepare": driver.prepare,
            "pose": driver.pose,
            "evaluate": driver.evaluate,
        }
        harness.preflight = lambda: {"kernel_name": "Linux", "block_reasons": {}}
        runner.preflight_gates = lambda preflight, build_dir: []
        harness.build = lambda build_dir: {"helper_report": "a" * 64}
        harness.build_identity = lambda build_dir: {
            "helper_report": {"size": 4, "sha256": "a" * 64, "kind": "elf_static"}}
        driver.verify_build_identity = lambda ctx: []

    def tearDown(self):
        harness.preflight = self.saved["preflight"]
        runner.preflight_gates = self.saved["gates"]
        harness.build = self.saved["build"]
        harness.build_identity = self.saved["identity"]
        driver.verify_build_identity = self.saved["verify"]
        driver.prepare = self.saved["prepare"]
        driver.pose = self.saved["pose"]
        driver.evaluate = self.saved["evaluate"]
        shutil.rmtree(self.tmp, ignore_errors=True)

    def _ready(self, plan, ctx, auth):
        return {"ready": True, "built": {"exec_path": "x"}, "binding": {}}

    def _abort_at(self, target):
        """Abort inside pose(), i.e. AFTER case_pose_started is durable."""
        def pose(plan, ctx, auth, prepared):
            if plan.case == target:
                raise OSError(9, "Bad file descriptor")
            return {"case": plan.case}
        driver.prepare = self._ready
        driver.pose = pose
        driver.evaluate = lambda plan, obs: {"launch_returned": True,
                                             "fabricated": True}

    def test_the_journal_survives_an_abort_in_the_sixth_case(self):
        self._abort_at("E5b")
        with self.assertRaises(OSError):
            runner.run_trial(self.tmp, None, str(self.out))

        records, truncated = journal.read(self.out / runner.JOURNAL_FILE)
        self.assertFalse(truncated)
        state = journal.replay(records)

        # Exactly Trial #1's shape, now preserved instead of lost.
        self.assertEqual(state["entered"][:6],
                         ["E1", "E2", "E3", "E4", "E5", "E5b"])
        self.assertEqual(sorted(state["completed"]),
                         ["E1", "E2", "E3", "E4", "E5"])
        self.assertEqual(state["entered_not_completed"], ["E5b"])
        self.assertEqual(state["last_entered"], "E5b")
        self.assertEqual(state["last_completed"], "E5")
        self.assertIsNone(state["trial_end"])

        # The facts a crash must not destroy were already durable.
        self.assertIsNotNone(state["preflight"])
        self.assertIsNotNone(state["build_identity"])
        self.assertIn("helper_report", state["build_identity"])

    def test_preflight_and_build_identity_are_on_disk_before_the_abort(self):
        self._abort_at("E1")
        with self.assertRaises(OSError):
            runner.run_trial(self.tmp, None, str(self.out))
        for name in (runner.PREFLIGHT_FILE, runner.BUILD_IDENTITY_FILE,
                     runner.JOURNAL_FILE):
            path = self.out / name
            self.assertTrue(path.exists(), name)
            self.assertGreater(path.stat().st_size, 0, name)
        # Not even the first case completed, and the journal says exactly that.
        state = journal.replay(journal.read(self.out / runner.JOURNAL_FILE)[0])
        self.assertEqual(state["entered"], ["E1"])
        self.assertEqual(state["completed"], {})

    def test_a_completed_run_writes_every_declared_artifact_file(self):
        driver.prepare = self._ready
        driver.pose = lambda plan, ctx, auth, prepared: {"case": plan.case}
        driver.evaluate = lambda plan, obs: {"launch_returned": True}
        code, document, _ = runner.run_trial(self.tmp, None, str(self.out))
        self.assertEqual(code, 0)
        for name in (runner.PREFLIGHT_FILE, runner.BUILD_IDENTITY_FILE,
                     runner.JOURNAL_FILE, runner.EVIDENCE_FILE):
            self.assertTrue((self.out / name).exists(), name)
        state = journal.replay(journal.read(self.out / runner.JOURNAL_FILE)[0])
        self.assertEqual(len(state["completed"]), 72)
        self.assertEqual(state["entered_not_completed"], [])
        self.assertEqual(state["trial_end"]["status"], "RUN")
        # The document is a summary of durable facts, not their only copy.
        self.assertEqual(sorted(state["completed"]), sorted(document["cases"]))
        for case, entry in state["completed"].items():
            self.assertEqual(entry["status"], document["cases"][case]["status"])

    def test_the_halt_path_journals_its_end_and_poses_nothing(self):
        runner.preflight_gates = lambda preflight, build_dir: [
            {"gate": "clone3", "detail": "clone3 is unavailable"}]
        driver.pose = self._never
        code, document, _ = runner.run_trial(self.tmp, None, str(self.out))
        self.assertEqual(code, 6)
        self.assertEqual(document["status"], "HALT_PREFLIGHT")
        state = journal.replay(journal.read(self.out / runner.JOURNAL_FILE)[0])
        self.assertEqual(state["entered"], [])
        self.assertEqual(state["trial_end"]["status"], "HALT_PREFLIGHT")
        self.assertIsNone(state["build_identity"])

    def test_a_build_identity_deviation_halts_before_the_first_case(self):
        driver.verify_build_identity = lambda ctx: [
            {"artefact": "helper_report", "detail": "changed"}]
        driver.pose = self._never
        code, document, _ = runner.run_trial(self.tmp, None, str(self.out))
        self.assertEqual(code, 6)
        self.assertEqual(document["status"], "HALT_BUILD_IDENTITY")
        state = journal.replay(journal.read(self.out / runner.JOURNAL_FILE)[0])
        self.assertEqual(state["entered"], [])

    @staticmethod
    def _never(*args, **kwargs):
        raise AssertionError("a case was posed after a halt")

    def test_every_written_file_is_sanitised(self):
        harness.preflight = lambda: {"kernel_name": "Linux",
                                     "uname": "Linux secret-host-2 6.1",
                                     "user": "secret-user-2",
                                     "block_reasons": {}}
        driver.prepare = self._ready
        driver.pose = lambda plan, ctx, auth, prepared: {"case": plan.case}
        driver.evaluate = lambda plan, obs: {"launch_returned": True,
                                             "child_pid": 4242}
        runner.run_trial(self.tmp, None, str(self.out))
        for name in (runner.PREFLIGHT_FILE, runner.BUILD_IDENTITY_FILE,
                     runner.JOURNAL_FILE, runner.EVIDENCE_FILE):
            text = (self.out / name).read_text(encoding="utf-8")
            for leak in ("secret-host-2", "secret-user-2", "4242"):
                self.assertNotIn(leak, text, "%s leaked into %s" % (leak, name))


class Trial2SetupContract(unittest.TestCase):
    """T2-R1. No setup-produced semantic directive may be producer-only.

    Trial #1's setups declared post_pin, hold_writer, mutated_marker and
    cleanup and nothing read any of them, so ten cases ran without the
    adversarial condition that defines them. These tests are derived from the
    real plans and the real source, not from a hand-maintained list of the keys
    that happened to be broken.
    """

    def setUp(self):
        self.tree = ast.parse((EXP / "driver.py").read_text(encoding="utf-8"))

    def _setup_functions(self):
        for fn in [n for n in ast.walk(self.tree)
                   if isinstance(n, ast.FunctionDef)]:
            names = [dec.args[0].value for dec in fn.decorator_list
                     if isinstance(dec, ast.Call)
                     and getattr(dec.func, "id", "") == "_setup"]
            if names:
                yield names[0], fn

    def produced_keys(self):
        """Every key any setup can return, derived from the source."""
        produced = {}
        for name, fn in self._setup_functions():
            keys, body = set(), ast.unparse(fn)
            for node in ast.walk(fn):
                if isinstance(node, ast.Dict):
                    keys.update(k.value for k in node.keys
                                if isinstance(k, ast.Constant)
                                and isinstance(k.value, str)
                                and not k.value.startswith("_"))
                if isinstance(node, ast.Call) and \
                        getattr(node.func, "id", "") == "dict":
                    keys.update(kw.arg for kw in node.keywords if kw.arg)
            if "_setup_copy(ctx, plan)" in body or "_setup_none(ctx, plan)" in body:
                keys |= {"exec_path", "not_posed"}
            produced[name] = keys
        return produced

    def test_every_produced_key_is_in_the_closed_schema(self):
        undeclared = {}
        for name, keys in self.produced_keys().items():
            extra = sorted(k for k in keys
                           if k not in driver.SETUP_RESULT_SCHEMA)
            if extra:
                undeclared[name] = extra
        self.assertEqual(undeclared, {},
                         "a setup returns a key the schema does not name")

    def test_every_semantic_key_has_an_execution_consumer(self):
        """A producer-only semantic key is the Trial #1 defect, and fails here."""
        source = (EXP / "driver.py").read_text(encoding="utf-8")
        consumers = {
            "post_pin": 'built.get("post_pin")',
            "hold_writer": 'built.get("hold_writer")',
            "mutated_marker": 'built.get("mutated_marker")',
            "cleanup": 'built.get("cleanup")',
            "liveness_fifo": 'built.get("liveness_fifo")',
            "extra_helper_args": 'built.get("extra_helper_args"',
            "work_dir": 'built.get("work_dir")',
        }
        for key in sorted(driver.SEMANTIC_SETUP_KEYS):
            self.assertIn(key, consumers, key + " has no declared consumer")
            self.assertIn(consumers[key], source,
                          key + " is produced but never read")

    def test_the_schema_covers_exactly_what_setups_produce(self):
        produced = set()
        for keys in self.produced_keys().values():
            produced |= keys
        self.assertEqual(produced - set(driver.SETUP_RESULT_SCHEMA), set())

    def test_every_post_pin_directive_has_an_implementation(self):
        directives = set()
        for _, fn in self._setup_functions():
            for node in ast.walk(fn):
                if isinstance(node, ast.Tuple) and node.elts and \
                        isinstance(node.elts[0], ast.Constant) and \
                        isinstance(node.elts[0].value, str):
                    parent = ast.unparse(node)
                    if parent.startswith("("):
                        directives.add(node.elts[0].value)
        named = {d for d in directives if d in driver.POST_PIN_ACTIONS}
        self.assertTrue(named, "no post-pin directives were found at all")
        for action in ("rename_over", "rename_away_and_unlink",
                       "retarget_symlink", "pwrite_marker", "truncate_rewrite",
                       "mmap_write", "fchmod"):
            self.assertIn(action, driver.POST_PIN_ACTIONS, action)

    def test_the_affected_case_set_is_derived_and_includes_x1_and_x8(self):
        """The independent review named eight cases; T2-R1's real set was ten.

        The delta correction adds S2 and S7, whose inert work_dir_kind became
        the work_dir_fchmod_zero setup. Still derived from the plans.
        """
        produced = self.produced_keys()
        affected = sorted(
            p.case for p in driver._PLAN_LIST
            if produced.get(p.setup, set()) & driver.SEMANTIC_SETUP_KEYS
            - {"extra_helper_args", "liveness_fifo"})
        # T2-R1's ten -- X1 shares E6d's setup and X8 owns `cleanup` -- plus
        # S2 and S7 from the delta correction.
        self.assertEqual(affected, ["E2", "E3", "E4", "E5", "E6", "E6b",
                                    "E6c", "E6d", "S2", "S7", "X1", "X8"])

    def test_every_post_pin_action_reports_whether_it_landed(self):
        for name, action in driver.POST_PIN_ACTIONS.items():
            source = ast.unparse(ast.parse(
                (EXP / "driver.py").read_text(encoding="utf-8")))
            self.assertIn("_landed", source, name)
        self.assertTrue(all(callable(a) for a in driver.POST_PIN_ACTIONS.values()))


class Trial2PostPinActions(unittest.TestCase):
    """The actions themselves, on inert files. No launcher is involved."""

    def setUp(self):
        self.tmp = pathlib.Path(tempfile.mkdtemp(prefix="postpin-"))
        self.body = b"\x7fELF" + bytes(range(256)) * 4
        (self.tmp / "helper_report").write_bytes(self.body)
        (self.tmp / "helper_alt").write_bytes(b"\x7fELF" + b"ALT!" * 200)
        self.identity = harness.build_identity(str(self.tmp))
        self.ctx = driver.TrialContext(
            build=str(self.tmp), work=str(self.tmp), preflight={}, freeze={},
            sanitiser=evidence.Sanitiser(), build_identity=self.identity)

    def tearDown(self):
        shutil.rmtree(self.tmp, ignore_errors=True)

    def _built(self, name="obj.bin", data=None):
        path = self.tmp / name
        path.write_bytes(self.body if data is None else data)
        return {"exec_path": str(path)}

    def test_rename_over_replaces_the_body_and_proves_it(self):
        built = self._built()
        replacement = self.tmp / "replacement"
        replacement.write_bytes(b"\x7fELFDIFFERENT" * 9)
        fact = driver.POST_PIN_ACTIONS["rename_over"](
            self.ctx, driver.CASE_PLANS["E2"], built, str(replacement))
        self.assertTrue(fact["landed"], fact)
        self.assertEqual(pathlib.Path(built["exec_path"]).read_bytes(),
                         b"\x7fELFDIFFERENT" * 9)
        self.assertNotEqual(fact["pinned_body_sha256"],
                            fact["path_body_sha256_after"])

    def test_rename_away_and_unlink_removes_the_pathname(self):
        built = self._built()
        fact = driver.POST_PIN_ACTIONS["rename_away_and_unlink"](
            self.ctx, driver.CASE_PLANS["E3"], built, None)
        self.assertTrue(fact["landed"], fact)
        self.assertFalse(pathlib.Path(built["exec_path"]).exists())

    @POSIX_ONLY
    def test_retarget_symlink_moves_the_link(self):
        link = self.tmp / "link"
        link.symlink_to(self.tmp / "helper_report")
        built = {"exec_path": str(link)}
        fact = driver.POST_PIN_ACTIONS["retarget_symlink"](
            self.ctx, driver.CASE_PLANS["E4"], built, str(self.tmp / "helper_alt"))
        self.assertTrue(fact["landed"], fact)
        self.assertNotEqual(fact["target_before"], fact["target_after"])

    @POSIX_ONLY
    def test_pwrite_marker_is_length_preserving(self):
        built = self._built()
        built["mutated_marker"] = b"ABCDEFGH"
        before = pathlib.Path(built["exec_path"]).stat().st_size
        fact = driver.POST_PIN_ACTIONS["pwrite_marker"](
            self.ctx, driver.CASE_PLANS["E6"], built, 16)
        self.assertTrue(fact["landed"], fact)
        self.assertTrue(fact["length_preserved"])
        self.assertEqual(pathlib.Path(built["exec_path"]).stat().st_size, before)
        self.assertNotEqual(fact["body_sha256_before"], fact["body_sha256_after"])

    def test_pwrite_marker_refuses_without_recorded_marker_bytes(self):
        fact = driver.POST_PIN_ACTIONS["pwrite_marker"](
            self.ctx, driver.CASE_PLANS["E6"], self._built(), 16)
        self.assertFalse(fact["landed"])
        self.assertIn("no marker bytes", fact["detail"])

    def test_truncate_rewrite_changes_the_length(self):
        built = self._built()
        fact = driver.POST_PIN_ACTIONS["truncate_rewrite"](
            self.ctx, driver.CASE_PLANS["E6b"], built, None)
        self.assertTrue(fact["landed"], fact)
        self.assertNotEqual(fact["size_before"], fact["size_after"])

    @POSIX_ONLY
    def test_mmap_write_mutates_and_keeps_the_mapping_after_the_fd_closes(self):
        """Both halves are now PROVEN from /proc, so this runs on Linux only."""
        built = self._built()
        built["mutated_marker"] = b"MUTATED".ljust(16, b"\0")
        fact = driver.POST_PIN_ACTIONS["mmap_write"](
            self.ctx, driver.CASE_PLANS["E6c"], built, 16)
        self.assertTrue(fact["landed"], fact)
        self.assertTrue(fact["descriptor_closed"])
        self.assertTrue(fact["mapping_retained"])
        self.assertEqual(len(built["_open_mappings"]), 1)
        self.assertEqual(driver.release_setup_resources(built), [])
        self.assertEqual(built.get("_open_mappings", []), [])

    @POSIX_ONLY
    def test_fchmod_clears_the_mode(self):
        built = self._built()
        fact = driver.POST_PIN_ACTIONS["fchmod"](
            self.ctx, driver.CASE_PLANS["E6d"], built, 0)
        self.assertTrue(fact["landed"], fact)
        self.assertNotEqual(fact["mode_before"], fact["mode_after"])

    def test_a_failed_action_reports_landed_false_rather_than_raising(self):
        built = {"exec_path": str(self.tmp / "does-not-exist")}
        fact = driver.POST_PIN_ACTIONS["retarget_symlink"](
            self.ctx, driver.CASE_PLANS["E4"], built, str(self.tmp / "helper_alt"))
        self.assertFalse(fact["landed"])

    @unittest.skipUnless(hasattr(os, "pwrite"), "Unix-only; Linux CI runs it")
    def test_hold_writer_proves_the_inode_rather_than_the_pathname(self):
        """N-3: without a launcher to compare against, the writer is NOT held.

        The relation E5 needs is between the writer and the object the launcher
        pinned; test_launch_exec_01_delta proves it against a live stand-in.
        The raw (st_dev, st_ino) pair is no longer a published fact.
        """
        built = self._built()
        landed, facts = driver.perform_post_pin(
            self.ctx, driver.CASE_PLANS["E5"],
            dict(built, hold_writer=True))
        self.assertFalse(landed, facts)
        fact = facts[0]
        self.assertEqual(fact["access_mode"], "O_WRONLY")
        self.assertIs(fact["launcher_holds_same_inode"], False)
        self.assertNotIn("st_ino", fact)
        self.assertNotIn("st_dev", fact)

    def test_cleanup_removes_a_case_private_copy(self):
        target = self.tmp / "x8_copy"
        target.write_bytes(b"x")
        built = {"exec_path": str(target), "cleanup": str(target)}
        self.assertEqual(driver.release_setup_resources(built), [])
        self.assertFalse(target.exists())

    def test_cleanup_of_a_missing_file_is_not_a_problem(self):
        built = {"cleanup": str(self.tmp / "never-existed")}
        self.assertEqual(driver.release_setup_resources(built), [])


@POSIX_ONLY
class Trial2MockBarrier(unittest.TestCase):
    """The harness side of the post-pin protocol, against a harmless mock peer.

    launcher_spike is never executed. The peer is a plain Python process that
    speaks the same three bytes.
    """

    def setUp(self):
        self.tmp = pathlib.Path(tempfile.mkdtemp(prefix="barrier-"))
        (self.tmp / "helper_report").write_bytes(b"\x7fELF" + b"Z" * 400)
        self.identity = harness.build_identity(str(self.tmp))
        self.ctx = driver.TrialContext(
            build=str(self.tmp), work=str(self.tmp), preflight={}, freeze={},
            sanitiser=evidence.Sanitiser(), build_identity=self.identity)
        self.plan = driver.CASE_PLANS["E6d"]

    def tearDown(self):
        shutil.rmtree(self.tmp, ignore_errors=True)

    def _peer(self, behaviour):
        """argv for a mock peer that reads --post-pin-control-fd like the spike."""
        code = (
            "import os,sys\n"
            "fd=int(sys.argv[sys.argv.index('--post-pin-control-fd')+1])\n"
            + behaviour)
        return [sys.executable, "-c", code]

    def _built(self):
        path = self.tmp / "target.bin"
        path.write_bytes(b"\x7fELF" + b"Z" * 400)
        return {"exec_path": str(path), "post_pin": ("fchmod", 0)}

    def _run(self, behaviour, built=None):
        return driver._run_with_post_pin(
            self._peer(behaviour), None, 20.0, self.ctx, self.plan,
            built if built is not None else self._built())

    def test_ready_action_continue(self):
        out = self._run("os.write(fd,b'R')\n"
                        "assert os.read(fd,1)==b'C'\n"
                        "sys.exit(0)\n")
        self.assertNotIn("not_posed", out)
        self.assertEqual(out["rc"], 0)
        self.assertTrue(out["post_pin"][0]["landed"], out["post_pin"])

    def test_missing_ready_is_not_a_mechanism_result(self):
        out = self._run("import time\ntime.sleep(0.2)\nsys.exit(3)\n")
        self.assertIn("not_posed", out)
        self.assertIn("post-pin control", out["not_posed"])

    def test_a_wrong_message_is_refused(self):
        out = self._run("os.write(fd,b'X')\nsys.exit(0)\n")
        self.assertIn("not_posed", out)
        self.assertIn("instead of READY", out["not_posed"])

    def test_peer_death_before_ready_is_refused(self):
        out = self._run("os._exit(9)\n")
        self.assertIn("not_posed", out)

    def test_an_action_that_cannot_land_stops_the_case(self):
        built = {"exec_path": str(self.tmp / "absent.bin"),
                 "post_pin": ("fchmod", 0)}
        out = self._run("os.write(fd,b'R')\n"
                        "os.read(fd,1)\n"
                        "sys.exit(0)\n", built=built)
        self.assertIn("not_posed", out)
        self.assertIn("did not land", out["not_posed"])
        self.assertFalse(out["post_pin"][0]["landed"])

    def test_an_unknown_directive_stops_the_case(self):
        built = {"exec_path": str(self.tmp / "helper_report"),
                 "post_pin": ("no_such_action", None)}
        out = self._run("os.write(fd,b'R')\nos.read(fd,1)\nsys.exit(0)\n",
                        built=built)
        self.assertIn("not_posed", out)
        self.assertIn("no implementation", out["post_pin"][0]["detail"])

    def test_the_control_descriptor_does_not_outlive_the_call(self):
        built = self._built()
        out = self._run("os.write(fd,b'R')\nassert os.read(fd,1)==b'C'\n"
                        "sys.exit(0)\n", built=built)
        self.assertNotIn("not_posed", out)
        # Both socket ends are closed by _run_with_post_pin; nothing leaks into
        # the observation it returns.
        self.assertNotIn("control", out)

    def test_a_case_without_a_directive_never_opens_a_control_socket(self):
        self.assertFalse(driver.needs_post_pin({"exec_path": "x"}))
        self.assertTrue(driver.needs_post_pin({"post_pin": ("fchmod", 0)}))
        self.assertTrue(driver.needs_post_pin({"hold_writer": True}))


class Trial2BuildIdentityCoverage(unittest.TestCase):
    """T2-R3. One central binding, and every case classified."""

    def setUp(self):
        self.tmp = pathlib.Path(tempfile.mkdtemp(prefix="bind-"))
        for name in ("helper_report", "helper_alt", "helper_fork",
                     "helper_dynamic", "helper_setid", "helper_foreign.elf",
                     "unloadable_in_cohort.elf", "script_fixture.sh"):
            (self.tmp / name).write_bytes(b"\x7fELF" + name.encode() * 8)
        self.identity = harness.build_identity(str(self.tmp))
        self.ctx = driver.TrialContext(
            build=str(self.tmp), work=str(self.tmp), preflight={}, freeze={},
            sanitiser=evidence.Sanitiser(), build_identity=self.identity)

    def tearDown(self):
        shutil.rmtree(self.tmp, ignore_errors=True)

    def test_every_plan_classifies_its_object(self):
        """No case may be skipped by accident: each is classified explicitly."""
        unclassified = []
        for plan in driver._PLAN_LIST:
            declared = driver.SETUP_OBJECT_CLASS.get(plan.setup)
            if declared is not None:
                continue
            base = plan.binary
            if base in self.identity:
                continue
            unclassified.append((plan.case, plan.setup, base))
        self.assertEqual(unclassified, [],
                         "a plan's object is neither declared nor derivable")

    def test_direct_base_is_verified_exactly(self):
        built = {"exec_path": str(self.tmp / "helper_dynamic")}
        fact = driver.bind_build_identity(self.ctx, driver.CASE_PLANS["E7"], built)
        self.assertTrue(fact["bound"], fact)
        self.assertEqual(fact["classification"], driver.DIRECT_BASE)
        self.assertEqual(fact["object_sha256"], fact["base_sha256"])

    def test_a_drifted_direct_base_is_not_bound(self):
        (self.tmp / "helper_dynamic").write_bytes(b"changed")
        built = {"exec_path": str(self.tmp / "helper_dynamic")}
        fact = driver.bind_build_identity(self.ctx, driver.CASE_PLANS["E7"], built)
        self.assertFalse(fact["bound"])

    def test_a_case_copy_must_start_equal_to_its_base(self):
        copy = self.tmp / "E1_helper_report"
        shutil.copy2(self.tmp / "helper_report", copy)
        fact = driver.bind_build_identity(self.ctx, driver.CASE_PLANS["E1"],
                                          {"exec_path": str(copy)})
        self.assertTrue(fact["bound"], fact)
        self.assertEqual(fact["base_artefact"], "helper_report")

    def test_a_mutation_target_is_bound_by_its_STARTING_bytes(self):
        copy = self.tmp / "E6_helper_report"
        shutil.copy2(self.tmp / "helper_report", copy)
        fact = driver.bind_build_identity(self.ctx, driver.CASE_PLANS["E6"],
                                          {"exec_path": str(copy)})
        self.assertTrue(fact["bound"], fact)
        self.assertEqual(fact["classification"],
                         driver.INTENTIONAL_MUTATION_TARGET)
        # After the frozen intentional mutation the base digest is NOT required.
        copy.write_bytes(b"mutated by the case")
        self.assertNotEqual(driver._digest(copy), fact["base_sha256"])

    def test_a_non_build_object_is_classified_not_skipped(self):
        target = self.tmp / "X5_dir"
        target.mkdir()
        fact = driver.bind_build_identity(self.ctx, driver.CASE_PLANS["X5"],
                                          {"exec_path": str(target)})
        self.assertTrue(fact["bound"])
        self.assertEqual(fact["classification"], driver.NON_BUILD_OBJECT)

    def test_a_symlink_is_bound_through_its_target(self):
        link = self.tmp / "E4_link"
        link.symlink_to(self.tmp / "helper_report")
        fact = driver.bind_build_identity(self.ctx, driver.CASE_PLANS["E4"],
                                          {"exec_path": str(link)})
        self.assertTrue(fact["bound"], fact)
        self.assertEqual(fact["classification"], driver.SYMLINK_TO_BASE)

    def test_an_unclassifiable_object_is_not_bound(self):
        stray = self.tmp / "stray.bin"
        stray.write_bytes(b"x")
        fact = driver.bind_build_identity(self.ctx, driver.CASE_PLANS["E1"],
                                          {"exec_path": str(stray)})
        self.assertFalse(fact["bound"])
        self.assertIn("not classified", fact["detail"])

    def test_the_binding_runs_for_every_case_from_one_place(self):
        source = (EXP / "driver.py").read_text(encoding="utf-8")
        # Defined once, called once: one central binding, not ten ad-hoc
        # checks bolted onto whichever setups happened to have one.
        tree = ast.parse(source)
        defs = [n for n in ast.walk(tree) if isinstance(n, ast.FunctionDef)
                and n.name == "bind_build_identity"]
        calls = [n for n in ast.walk(tree) if isinstance(n, ast.Call)
                 and ast.unparse(n.func) == "bind_build_identity"]
        self.assertEqual(len(defs), 1)
        self.assertEqual(len(calls), 1)


class Trial2FrozenAbortSemantics(unittest.TestCase):
    """T2-R2. What a partial journal MEANS is frozen before the trial."""

    def setUp(self):
        self.tmp = pathlib.Path(tempfile.mkdtemp(prefix="abort-"))
        self.path = self.tmp / "journal.jsonl"
        self.s = evidence.Sanitiser()

    def tearDown(self):
        shutil.rmtree(self.tmp, ignore_errors=True)

    def _replay(self):
        return journal.replay(journal.read(self.path)[0])

    def test_no_pose_started_means_the_boundary_was_not_crossed(self):
        with journal.Journal(self.path, self.s) as j:
            j.case_entered("E1", 0)
            j.case_completed("E1", checker.BLOCKED, "environment", {})
        status = journal.project_status(self._replay())
        self.assertEqual(status["trial_status"], journal.TRIAL_NOT_STARTED)
        self.assertFalse(status["d7_consumed"])
        self.assertIsNone(status["aggregate"])

    def test_one_pose_started_consumes_d7_and_crosses_the_boundary(self):
        with journal.Journal(self.path, self.s) as j:
            j.case_entered("E1", 0)
            j.case_pose_started("E1", 0)
        state = self._replay()
        self.assertTrue(state["boundary_crossed"])
        status = journal.project_status(state)
        self.assertEqual(status["trial_status"],
                         journal.TRIAL_ABORTED_AFTER_BOUNDARY)
        self.assertTrue(status["d7_consumed"])
        self.assertEqual(status["aggregate"], journal.AGGREGATE_NOT_DERIVABLE)

    def test_an_incomplete_case_has_a_frozen_review_status_not_a_case_status(self):
        with journal.Journal(self.path, self.s) as j:
            j.case_entered("E1", 0)
            j.case_pose_started("E1", 0)
        state = self._replay()
        self.assertEqual(state["incomplete_case_status"]["E1"],
                         journal.UNKNOWN_FROM_PRESERVED_EVIDENCE)
        self.assertNotIn(journal.UNKNOWN_FROM_PRESERVED_EVIDENCE,
                         (checker.PASS, checker.FAIL, checker.INVALID,
                          checker.BLOCKED))
        self.assertNotIn("E1", journal.recovered_records(state))

    def test_completed_cases_keep_exactly_their_recorded_status(self):
        with journal.Journal(self.path, self.s) as j:
            j.case_entered("E1", 0)
            j.case_pose_started("E1", 0)
            j.case_completed("E1", checker.FAIL, "the real reason", {"r": 1})
            j.case_entered("E2", 1)
            j.case_pose_started("E2", 1)
        state = self._replay()
        self.assertEqual(state["completed"]["E1"]["status"], checker.FAIL)
        self.assertEqual(state["completed"]["E1"]["reason"], "the real reason")
        self.assertEqual(state["entered_not_completed"], ["E2"])

    def test_a_trial_end_with_an_aggregate_needs_every_case_completed(self):
        with journal.Journal(self.path, self.s) as j:
            j.case_entered("E1", 0)
            j.case_completed("E1", checker.PASS, "ok", {})
            with self.assertRaises(journal.JournalError):
                j.trial_end(status="RUN", aggregate=checker.ACCEPTED,
                            membership=["E1", "E2"])

    def test_a_trial_end_without_an_aggregate_is_allowed_for_a_halt(self):
        with journal.Journal(self.path, self.s) as j:
            j.trial_end(status="HALT_PREFLIGHT", halts=[{"gate": "clone3"}])
        status = journal.project_status(self._replay())
        self.assertEqual(status["trial_status"], journal.TRIAL_NOT_STARTED)

    def test_replay_never_synthesises_a_trial_end(self):
        with journal.Journal(self.path, self.s) as j:
            j.case_entered("E1", 0)
            j.case_pose_started("E1", 0)
            j.case_completed("E1", checker.PASS, "ok", {})
        self.assertIsNone(self._replay()["trial_end"])


class Trial2ReplayIntegrity(unittest.TestCase):
    """T2-R4. The reader refuses impossible histories instead of repairing them."""

    def setUp(self):
        self.tmp = pathlib.Path(tempfile.mkdtemp(prefix="replay-"))
        self.path = self.tmp / "j.jsonl"

    def tearDown(self):
        shutil.rmtree(self.tmp, ignore_errors=True)

    def _write(self, *records):
        text = "".join(evidence.serialise_line(dict(r, n=i))
                       for i, r in enumerate(records))
        self.path.write_bytes(text.encode("utf-8"))
        return journal.read(self.path)[0]

    def _refuses(self, *records):
        with self.assertRaises(journal.JournalError):
            journal.replay(self._write(*records))

    def test_completion_before_entry_is_refused(self):
        self._refuses({"kind": journal.CASE_COMPLETED, "case": "E1",
                       "status": "PASS", "reason": "", "record": {}})

    def test_pose_start_before_entry_is_refused(self):
        self._refuses({"kind": journal.CASE_POSE_STARTED, "case": "E1", "index": 0})

    def test_duplicate_pose_start_is_refused(self):
        self._refuses({"kind": journal.CASE_ENTERED, "case": "E1", "index": 0},
                      {"kind": journal.CASE_POSE_STARTED, "case": "E1", "index": 0},
                      {"kind": journal.CASE_POSE_STARTED, "case": "E1", "index": 0})

    def test_duplicate_entry_is_refused(self):
        self._refuses({"kind": journal.CASE_ENTERED, "case": "E1", "index": 0},
                      {"kind": journal.CASE_ENTERED, "case": "E1", "index": 0})

    def test_duplicate_completion_is_refused(self):
        self._refuses({"kind": journal.CASE_ENTERED, "case": "E1", "index": 0},
                      {"kind": journal.CASE_COMPLETED, "case": "E1",
                       "status": "PASS", "reason": "", "record": {}},
                      {"kind": journal.CASE_COMPLETED, "case": "E1",
                       "status": "FAIL", "reason": "", "record": {}})

    def test_pose_start_after_completion_is_refused(self):
        self._refuses({"kind": journal.CASE_ENTERED, "case": "E1", "index": 0},
                      {"kind": journal.CASE_COMPLETED, "case": "E1",
                       "status": "PASS", "reason": "", "record": {}},
                      {"kind": journal.CASE_POSE_STARTED, "case": "E1", "index": 0})

    def test_events_after_trial_end_are_refused(self):
        self._refuses({"kind": journal.TRIAL_END, "status": "RUN"},
                      {"kind": journal.CASE_ENTERED, "case": "E1", "index": 0})

    def test_two_trial_ends_are_refused(self):
        self._refuses({"kind": journal.TRIAL_END, "status": "RUN"},
                      {"kind": journal.TRIAL_END, "status": "RUN"})

    def test_a_valid_history_is_accepted(self):
        state = journal.replay(self._write(
            {"kind": journal.CASE_ENTERED, "case": "E1", "index": 0},
            {"kind": journal.CASE_POSE_STARTED, "case": "E1", "index": 0},
            {"kind": journal.CASE_COMPLETED, "case": "E1", "status": "PASS",
             "reason": "ok", "record": {}}))
        self.assertEqual(state["pose_started"], ["E1"])
        self.assertEqual(state["completed"]["E1"]["status"], "PASS")


class Trial2Membership(unittest.TestCase):
    """The corrections must not have moved the experiment."""

    def test_the_partition_is_unchanged(self):
        self.assertEqual(len(fc.MEMBERSHIP), 72)
        self.assertEqual(len(fc.MANDATORY_CASES), 54)
        self.assertEqual(len(fc.CONDITIONAL_CASES), 11)
        self.assertEqual(len(fc.RECORDED_CASES), 7)

    def test_all_72_handlers_remain_and_every_case_is_posable(self):
        complete = driver.completeness()
        self.assertTrue(complete["complete"])
        self.assertEqual(complete["driver_total"], 72)
        self.assertEqual(complete["frozen_total"], 72)
        self.assertEqual(driver.unposable_cases(), {})

    def test_the_traced_set_is_unchanged(self):
        self.assertEqual([c["case"] for c in fc.CASES if c.get("traced")],
                         ["E1", "E7", "F4", "F7", "M1", "M2", "M3", "M4"])

    def test_e5b_keeps_its_expectation_and_class(self):
        spec = fc.BY_NAME["E5b"]
        self.assertEqual(spec["cls"], fc.MANDATORY)
        self.assertEqual(spec["predict"], "Exited:0")
        self.assertEqual(driver.CASE_PLANS["E5b"].setup, "writer_open_closed")


class Trial2PersistentFilePrivacy(unittest.TestCase):
    """Every file the trial writes for upload is already sanitised."""

    def test_every_persistent_write_goes_through_the_boundary(self):
        tree = ast.parse((EXP / "run_launch_exec_01.py").read_text(encoding="utf-8"))
        fn = [n for n in tree.body if isinstance(n, ast.FunctionDef)
              and n.name == "_write"][0]
        self.assertIn("evidence.publish", ast.unparse(fn))

    def test_the_journal_serialises_only_through_evidence(self):
        source = (EXP / "journal.py").read_text(encoding="utf-8")
        tree = ast.parse(source)
        for node in ast.walk(tree):
            if isinstance(node, ast.Call):
                name = ast.unparse(node.func)
                self.assertNotIn(name, ("json.dumps", "json.dump"))
        self.assertIn("evidence.serialise_line", source)

    def test_the_declared_artifact_files_are_the_sanitised_ones(self):
        self.assertEqual(
            sorted([runner.PREFLIGHT_FILE, runner.BUILD_IDENTITY_FILE,
                    runner.JOURNAL_FILE, runner.EVIDENCE_FILE]),
            ["build-identity.json", "evidence.json", "journal.jsonl",
             "preflight.json"])


# ============================================ M-1: fail-closed lazy vocabulary
# ``evidence`` reads the driver's registries through a lazy import, because
# ``driver`` imports ``evidence``. The hazard the micro-review recorded as M-1
# is that ``import driver`` succeeds against a module whose body is STILL
# RUNNING -- Python publishes the module object first -- so the registries can
# be read while empty, and the empty answer was then cached for the life of the
# process. Published symbolic evidence would silently become <OPAQUE:...>.
#
# Import order is a property of a whole interpreter, so these scenarios run in
# FRESH interpreters rather than by poking sys.modules inside this one, where
# every module is already imported and the interesting states are unreachable.
# Nothing here builds, executes or poses anything: each child imports Python
# modules and asks for a vocabulary.

_PROBE = "sigterm_blocked_sigpipe_ignored"


def fresh(body):
    """Run body in a fresh interpreter rooted at the experiment directory."""
    code = "import sys\nsys.path.insert(0, %r)\n%s" % (str(EXP), body)
    proc = subprocess.run([sys.executable, "-c", code],
                          capture_output=True, text=True)
    return proc.returncode, (proc.stdout or "").strip(), (proc.stderr or "").strip()


# Each scenario prints one line the test parses. Keeping the reporting identical
# across scenarios is what makes "the same complete vocabulary" checkable.
_TAIL = ('v = evidence.vocabulary()\n'
         'print(len(v), "%s" in v, isinstance(v, frozenset))\n' % _PROBE)

# A halt is only correct if it publishes NOTHING, through every public entry
# point, and caches NOTHING.
_HALT_PROBE = '''
halted, published = 0, []
for call in (lambda: evidence.vocabulary(),
             lambda: evidence.Sanitiser(user="ci").text("Zz" * 20),
             lambda: evidence.Sanitiser(user="ci").record({"p": "Zz" * 20}),
             lambda: evidence.serialise({"p": "Zz" * 20})):
    try:
        published.append(call())
    except evidence.VocabularyUnavailable:
        halted += 1
clean = evidence._DRIVER_VOCABULARY is None and evidence._VOCABULARY is None
print(halted, len(published), "not_cached" if clean else "CACHED")
'''

SCENARIOS = {
    "A": "import evidence, driver\n" + _TAIL,
    "B": "import driver, evidence\n" + _TAIL,
    "C": "import checker, observations, evidence\n" + _TAIL,
    "C2": "import evidence\n" + _TAIL,
    # Deliberate import failure. A None entry in sys.modules is exactly how the
    # interpreter reports "this import is not available".
    "D": ("import evidence\n"
          "sys.modules['driver'] = None\n" + _HALT_PROBE),
    "E": ("import evidence\n"
          "sys.modules['driver'] = None\n"
          "try:\n"
          "    evidence.vocabulary()\n"
          "    print('NO_HALT')\n"
          "except evidence.VocabularyUnavailable:\n"
          "    pass\n"
          "del sys.modules['driver']\n" + _TAIL),
    # F: partial init with the registries ABSENT -- a bare module object, which
    # is precisely what sys.modules holds while driver's body is executing.
    "F": ("import types, evidence\n"
          "sys.modules['driver'] = types.ModuleType('driver')\n"
          + _HALT_PROBE +
          "del sys.modules['driver']\n" + _TAIL),
    # G: partial init with the registries PRESENT but empty, and a finalised
    # accessor, so only the validation can catch it.
    "G": ("import types, evidence\n"
          "stub = types.ModuleType('driver')\n"
          "stub.SETUPS, stub.PARENT_STATES, stub.POSED_CHECKS = {}, {}, {}\n"
          "stub.ALL_CHANNELS = ()\n"
          "stub.registry_vocabulary = lambda: {'SETUPS': (),"
          " 'PARENT_STATES': (), 'POSED_CHECKS': (), 'ALL_CHANNELS': ()}\n"
          "sys.modules['driver'] = stub\n"
          + _HALT_PROBE +
          "del sys.modules['driver']\n" + _TAIL),
    # G2: a registry missing from the snapshot entirely.
    "G2": ("import types, evidence\n"
           "stub = types.ModuleType('driver')\n"
           "stub.registry_vocabulary = lambda: {'SETUPS': ('a',),"
           " 'PARENT_STATES': ('b',), 'POSED_CHECKS': ('c',)}\n"
           "sys.modules['driver'] = stub\n"
           + _HALT_PROBE +
           "del sys.modules['driver']\n" + _TAIL),
    # G3: a registry of the wrong type.
    "G3": ("import types, evidence\n"
           "stub = types.ModuleType('driver')\n"
           "stub.registry_vocabulary = lambda: {'SETUPS': 'not-a-sequence',"
           " 'PARENT_STATES': ('b',), 'POSED_CHECKS': ('c',),"
           " 'ALL_CHANNELS': ('d',)}\n"
           "sys.modules['driver'] = stub\n"
           + _HALT_PROBE +
           "del sys.modules['driver']\n" + _TAIL),
    # G4: the accessor itself raises.
    "G4": ("import types, evidence\n"
           "def boom():\n"
           "    raise RuntimeError('half-built')\n"
           "stub = types.ModuleType('driver')\n"
           "stub.registry_vocabulary = boom\n"
           "sys.modules['driver'] = stub\n"
           + _HALT_PROBE +
           "del sys.modules['driver']\n" + _TAIL),
    "I": ("import evidence, driver\n"
          "sizes = {len(evidence.vocabulary()) for _ in range(50)}\n"
          "ids = {id(evidence.vocabulary()) for _ in range(50)}\n"
          "assert len(sizes) == 1 and len(ids) == 1, (sizes, ids)\n"
          + _TAIL),
}


class M1FailClosedVocabulary(unittest.TestCase):
    """M-1. A partial or failed driver load halts, publishes nothing, caches
    nothing, and a later complete load recovers."""

    results = {}

    @classmethod
    def setUpClass(cls):
        for name, body in SCENARIOS.items():
            cls.results[name] = fresh(body)

    def lines(self, name):
        rc, out, err = self.results[name]
        self.assertEqual(rc, 0, "%s: %s" % (name, err[-500:]))
        return out.splitlines()

    def complete(self, name, index=-1):
        """Parse a trailing '<size> True True' report as a complete load."""
        size, probe, frozen = self.lines(name)[index].split()
        self.assertEqual(probe, "True", name + ": F-1 token missing")
        self.assertEqual(frozen, "True", name + ": vocabulary is not immutable")
        return int(size)

    def halted(self, name, index=0):
        """Parse a '<halts> <published> <cache>' report as a clean refusal."""
        halts, published, cache = self.lines(name)[index].split()
        self.assertEqual(halts, "4", name + ": something did not halt")
        self.assertEqual(published, "0", name + ": evidence was published")
        self.assertEqual(cache, "not_cached", name + ": cache was poisoned")

    # ------------------------------------------------ A/B/C: import order
    def test_import_order_does_not_change_the_vocabulary(self):
        sizes = {name: self.complete(name) for name in ("A", "B", "C", "C2")}
        self.assertEqual(len(set(sizes.values())), 1, sizes)
        self.assertGreater(min(sizes.values()), 200)

    def test_the_in_process_vocabulary_agrees_with_a_fresh_interpreter(self):
        self.assertEqual(len(evidence.vocabulary()), self.complete("A"))

    # ------------------------------------------------ D/E: import failure
    def test_import_failure_halts_and_publishes_nothing(self):
        self.halted("D")

    def test_import_failure_is_not_cached_and_recovers(self):
        self.assertEqual(self.complete("E"), self.complete("A"))

    # ------------------------------------- F/G: partial initialisation
    def test_registries_absent_halts_without_caching(self):
        self.halted("F")

    def test_registries_present_but_empty_halts_without_caching(self):
        self.halted("G")

    def test_a_missing_registry_halts_without_caching(self):
        self.halted("G2")

    def test_a_mistyped_registry_halts_without_caching(self):
        self.halted("G3")

    def test_an_accessor_that_raises_halts_without_caching(self):
        self.halted("G4")

    # ------------------------------------- H: recovery after F/G
    def test_every_partial_state_recovers_to_the_complete_vocabulary(self):
        baseline = self.complete("A")
        for name in ("F", "G", "G2", "G3", "G4"):
            self.assertEqual(self.complete(name), baseline, name)

    # ------------------------------------------------ I: determinism
    def test_fifty_queries_return_one_identical_object(self):
        self.assertEqual(self.complete("I"), self.complete("A"))

    # -------------------------------- the readiness signal is structural
    def test_the_accessor_is_the_last_statement_of_drivers_module_body(self):
        """The whole design rests on this: a driver that has not finished
        executing cannot have the attribute, so its presence is a fact rather
        than a guess. Anything appended after it would silently break that."""
        tree = ast.parse((EXP / "driver.py").read_text(encoding="utf-8"))
        last = tree.body[-1]
        self.assertIsInstance(last, ast.FunctionDef)
        self.assertEqual(last.name, "registry_vocabulary")

    def test_the_snapshot_is_the_registries_and_not_a_second_list(self):
        snapshot = driver.registry_vocabulary()
        self.assertEqual(set(snapshot["SETUPS"]), set(driver.SETUPS))
        self.assertEqual(set(snapshot["PARENT_STATES"]),
                         set(driver.PARENT_STATES))
        self.assertEqual(set(snapshot["POSED_CHECKS"]),
                         set(driver.POSED_CHECKS))
        self.assertEqual(tuple(snapshot["ALL_CHANNELS"]),
                         tuple(driver.ALL_CHANNELS))

    def test_the_snapshot_cannot_mutate_the_registries(self):
        snapshot = driver.registry_vocabulary()
        for name, value in snapshot.items():
            self.assertIsInstance(value, tuple, name)
        snapshot["SETUPS"] = ()
        self.assertTrue(driver.SETUPS, "the live registry was reachable")

    def test_the_accessor_poses_nothing(self):
        """It is called during sanitisation, so it must never act."""
        source = ast.parse((EXP / "driver.py").read_text(encoding="utf-8"))
        fn = [n for n in source.body
              if isinstance(n, ast.FunctionDef)
              and n.name == "registry_vocabulary"][0]
        called = {n.func.id for n in ast.walk(fn)
                  if isinstance(n, ast.Call) and isinstance(n.func, ast.Name)}
        self.assertEqual(called - {"tuple", "sorted"}, set())

    def test_the_cached_vocabulary_is_immutable(self):
        self.assertIsInstance(evidence.vocabulary(), frozenset)


# The tokens this suite has demonstrated a derivation for. The universe test
# above compares the frozen expectations against exactly this list, so a frozen
# token nobody can produce is a test failure rather than a discovery at trial
# time.
PRODUCED_TOKENS = {
    "Exited:0", "Exited:7", "Exited:9", "Exited:42", "Exited:127",
    "Exited:mutated",
    "Signaled:SIGBUS", "Signaled:SIGKILL", "Signaled:SIGSEGV",
    "ExecFailed:EACCES", "ExecFailed:ENOENT", "ExecFailed:ENOEXEC",
    "ExecFailed:ETXTBSY", "ExecFailed:CHDIR:EACCES",
    "ExecStatusIndeterminate", "ExecStatusIndeterminate:PreExecTimeout",
    "ExitStatusUnobservable",
    "TimedOut", "TimedOut:KilledByLauncher:SIGKILL",
    "TimedOut:ExitedDuringGrace:9",
    "refused:ElfNotInCohort", "refused:NotRegularFile",
    "refused:SetIdBitsPresent", "refused:DescriptorModeUnsuitable",
    "argv_exact", "environ_empty", "fds_exactly_012", "signals_reset",
    "no_new_privs_0", "no_new_privs_1", "stream_exact",
    "interpreter_ran_with_devfd", "privilege_transition_suppressed",
    "descendant_survived", "descendant_died", "sweep_issued",
    "sweep_not_issued", "child_syscalls_within_frozen_set",
    "identical_to_single_threaded_arm", "sequence_matches_frozen_stages",
    "pidfd_acquired_atomically", "pidfd_open_esrch", "waitid_echild",
    "pidfd_open_succeeded",
}


if __name__ == "__main__":
    unittest.main()
