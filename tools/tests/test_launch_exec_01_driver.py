"""Pre-trial self-tests for the LAUNCH-EXEC-01 case driver, mapping and sanitiser.

These validate the DRIVER TABLE, the P-12 observation-to-token mapping and the
P-14 sanitiser. Every observation here is fabricated in-process. Nothing in this
file builds, executes or otherwise invokes ``launcher_spike``, any helper, any
generated ELF or any preregistered case, and no test constructs a
``driver.Authorisation`` -- which is the object every posing path requires.
LAUNCH-EXEC-01 remains NOT_RUN.
"""
import ast
import json
import pathlib
import subprocess
import sys
import unittest

EXP = (pathlib.Path(__file__).resolve().parents[2]
       / "docs" / "experiments" / "launch-exec-01")
if str(EXP) not in sys.path:
    sys.path.insert(0, str(EXP))

import checker              # noqa: E402
import driver               # noqa: E402
import evidence             # noqa: E402
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
        for rule in ("argv_exact", "environ_empty", "fds_exactly_012",
                     "signals_reset", "no_new_privs",
                     "interpreter_ran_with_devfd"):
            token, _ = ob.derive(rule, observation(rep=None))
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

    def test_uninterpretable_payload_yields_no_token(self):
        obs = observation(rep=None, payload_is_recipe=False)
        self.assertEqual(obs["exec_confirmation"], ob.EXEC_UNINTERPRETABLE)
        token, _ = ob.derive("process_disposition", obs)
        self.assertIsNone(token)

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
        token, _ = ob.derive("admission", observation())
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
                      ob.REPORT_ABSENT, ob.REPORT_STREAM_INCOMPLETE):
            obs = observation(rep=None, report_state=state,
                              expected_argv=["helper_report"])
            for rule in ("argv_exact", "environ_empty", "fds_exactly_012",
                         "signals_reset", "no_new_privs"):
                token, reason = ob.derive(rule, obs)
                self.assertIsNone(token, rule + " from " + state)
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
        self.assertEqual(set(fc.PARENT_CONTROL_MODES),
                         {"--extra-threads", "--rejected-acquisition-arm"})

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
                rep=report(), trace={"child_syscalls": permitted},
                trace_sha256="a" * 64,
                single_threaded_child_syscalls=permitted,
                observed_stage_sequence=list(fc.STAGES),
                acquisition=ob.parse_pidfd_acquisition(acq_trace()),
                expected_argv=["helper_report"])
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
        """run_trial returns HALT_PREFLIGHT before the case loop is reached."""
        import run_launch_exec_01 as runner
        source = (EXP / "run_launch_exec_01.py").read_text(encoding="utf-8")
        body = source[source.index("def run_trial("):]
        halt_at = body.index("HALT_PREFLIGHT")
        pose_at = body.index("driver.observe(")
        self.assertLess(halt_at, pose_at,
                        "the preflight halt must precede any posing")
        self.assertIn("if halts:", body[:halt_at])

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
