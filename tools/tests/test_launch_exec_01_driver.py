"""Pre-trial self-tests for the LAUNCH-EXEC-01 case driver, mapping and sanitiser.

These validate the DRIVER TABLE, the P-12 observation-to-token mapping and the
P-14 sanitiser. Every observation here is fabricated in-process. Nothing in this
file builds, executes or otherwise invokes ``launcher_spike``, any helper, any
generated ELF or any preregistered case, and no test constructs a
``driver.Authorisation`` -- which is the object every posing path requires.
LAUNCH-EXEC-01 remains NOT_RUN.
"""
import json
import pathlib
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
                records[name]["trace"] = []
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
            record["trace"] = []
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

    def test_username_is_redacted_even_outside_a_path(self):
        self.assertEqual(self.s.text("owned by runner"), "owned by <USER>")

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

    def test_dict_keys_are_sanitised_too(self):
        out = self.s.record({"/home/runner/work/helm/a": 1})
        self.assertIn("<WORK>/a", out)

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

    def test_a_resumed_line_is_still_a_syscall(self):
        text = (self.WINDOW.replace(
            "[pid   222] execveat(3, \"\", NULL, NULL, AT_EMPTY_PATH) = 0\n",
            "[pid   222] <... execveat resumed>) = 0\n"))
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

    def test_closure_form_passes_when_no_pidfd_open_exists(self):
        """The tracer did not print the write-back, but the record is closed."""
        token, reason = self._token(acq_trace(with_out=False))
        self.assertEqual(token, "pidfd_acquired_atomically")
        self.assertIn(ob.EVIDENCE_CLOSURE, reason)

    def test_the_frozen_facts_are_all_named_in_the_manifest(self):
        self.assertEqual(sorted(fc.M3_EVIDENCE_FACTS), list("ABCDEFG"))
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
    def test_clone3_failure_is_not_success(self):
        token, _ = self._token(acq_trace(ret=-1, with_out=False,
                                         with_waitid=False))
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

    def test_closure_form_refuses_when_any_pidfd_open_exists(self):
        """Without the write-back, the record must be complete about pidfd_open."""
        token, reason = self._token(acq_trace(with_out=False, pidfd_open=999))
        self.assertIsNone(token)
        self.assertIn("not closed", reason)

    # -- G: correlation ------------------------------------------------------
    def test_no_waitid_correlation_is_invalid(self):
        token, reason = self._token(acq_trace(with_waitid=False))
        self.assertIsNone(token)
        self.assertIn("correlated", reason)

    def test_ambiguous_correlation_is_invalid(self):
        extra = ("111   waitid(P_PIDFD, 9, {si_pid=333, si_code=CLD_EXITED}, "
                 "WEXITED, NULL) = 0")
        token, reason = self._token(acq_trace(extra=extra))
        self.assertIsNone(token)
        self.assertIn("correlated", reason)

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
