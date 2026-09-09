"""Pre-trial self-tests for the LAUNCH-EXEC-01 frozen definition.

These validate the MANIFEST and the VERDICT LOGIC only. Every record here is
fabricated in-process; nothing in this file builds, executes or otherwise
invokes `launcher_spike`, any helper, or any preregistered case. LAUNCH-EXEC-01
remains NOT_RUN.
"""
import pathlib
import sys
import unittest

EXP = (pathlib.Path(__file__).resolve().parents[2]
       / "docs" / "experiments" / "launch-exec-01")
if str(EXP) not in sys.path:
    sys.path.insert(0, str(EXP))

import checker            # noqa: E402
import frozen_cases as fc  # noqa: E402
import make_fixtures      # noqa: E402
import oracles            # noqa: E402


def passing_record(name):
    """A minimal record that should score PASS for `name`."""
    spec = fc.BY_NAME[name]
    outcome = spec["predict"] if spec["predict"] is not None else spec["safe"][0]
    record = {"outcome": outcome, "launch_returned": True}
    if spec["traced"]:
        record["trace"] = []
    if spec["gates"]:
        record["gates"] = {g: True for g in spec["gates"]}
    if name in fc.DOCUMENTATION_GATES:
        record["documentation_gate"] = True
    return record


def all_passing():
    return {name: passing_record(name) for name in fc.MEMBERSHIP}


class ManifestIntegrity(unittest.TestCase):
    def test_manifest_validates(self):
        self.assertTrue(fc.validate())

    def test_no_duplicate_ids(self):
        self.assertEqual(len(fc.MEMBERSHIP), len(set(fc.MEMBERSHIP)))

    def test_classes_partition_membership(self):
        union = fc.MANDATORY_CASES + fc.CONDITIONAL_CASES + fc.RECORDED_CASES
        self.assertEqual(sorted(union), sorted(fc.MEMBERSHIP))
        self.assertEqual(len(union), len(set(union)), "classes overlap")

    def test_counts_match_summary(self):
        s = fc.summary()
        self.assertEqual(s["total"], len(fc.MEMBERSHIP))
        self.assertEqual(s["mandatory"] + s["conditional"] + s["recorded"],
                         s["total"])

    def test_every_case_has_exactly_one_expectation(self):
        for c in fc.CASES:
            with self.subTest(case=c["case"]):
                self.assertNotEqual(c["predict"] is None, c["safe"] is None)

    def test_recorded_cases_carry_gates(self):
        for name in fc.RECORDED_CASES:
            self.assertTrue(fc.BY_NAME[name]["gates"],
                            f"{name} is recorded but gates nothing")

    def test_conditional_cases_name_a_frozen_block_reason(self):
        for name in fc.CONDITIONAL_CASES:
            reason = fc.BY_NAME[name]["blocked_if"]
            self.assertIn(reason, fc.BLOCK_REASONS)

    def test_wait_classifying_series_are_never_traced(self):
        # ptrace reports a tracee's exit to the tracer before the real parent,
        # and the launcher IS the real parent.
        for c in fc.CASES:
            if c["series"] in {"R", "T", "O", "P", "S"}:
                self.assertFalse(c["traced"], f"{c['case']} must not be traced")

    def test_owner_decisions_are_recorded(self):
        for key in ("D-1", "D-9", "D-10", "D-11"):
            self.assertIn(key, fc.DECISIONS)
        self.assertEqual(fc.DECISIONS["D-9"], "refuse_set_id_objects_at_admission")
        self.assertEqual(fc.DECISIONS["D-10"], "empty_environment_only")
        self.assertEqual(fc.DECISIONS["D-11"], "no_new_privs_required_before_exec")

    def test_no_new_privs_is_a_frozen_stage(self):
        self.assertIn("NO_NEW_PRIVS", fc.STAGES)
        self.assertIn("prctl", fc.CHILD_PERMITTED_SYSCALLS)

    def test_test_only_injections_are_separated_from_the_mechanism_sequence(self):
        # Otherwise M1's minimality claim holds only because the injecting cases
        # happen not to be traced, which is an accident, not a rule.
        for name in ("nanosleep", "clock_nanosleep", "kill", "getpid"):
            self.assertIn(name, fc.CHILD_TEST_INJECTION_SYSCALLS)
            self.assertNotIn(name, fc.CHILD_PERMITTED_SYSCALLS)
        for mode in ("--stall-pre-exec-ms", "--die-before-exec"):
            self.assertIn(mode, fc.CHILD_INJECTION_MODES)
        # The cases that use them must not be traced.
        for name in ("S5", "S6"):
            self.assertFalse(fc.BY_NAME[name]["traced"])

    def test_host_property_block_reasons_exist(self):
        for reason in ("parent_no_new_privs_set", "clone3_unavailable"):
            self.assertIn(reason, fc.BLOCK_REASONS)

    def test_deleted_plan_parse_cases_are_recorded_not_dropped(self):
        for key in ("argv_nul_rejection", "ld_preload_refusal",
                    "explicit_env_entries"):
            self.assertIn(key, fc.OUT_OF_SCOPE)
        self.assertNotIn("A5", fc.MEMBERSHIP)
        self.assertNotIn("V3", fc.MEMBERSHIP)


class AggregatePrecedence(unittest.TestCase):
    def test_all_passing_is_accepted(self):
        report = checker.report(all_passing())
        self.assertEqual(report["aggregate"], checker.ACCEPTED,
                         report["detail"])

    def test_precedence_is_total(self):
        # Every single-case perturbation must still reach exactly one verdict.
        for name in fc.MEMBERSHIP:
            records = all_passing()
            records[name] = {"outcome": "something_else", "launch_returned": True}
            report = checker.report(records)
            self.assertIn(report["aggregate"],
                          {checker.ACCEPTED, checker.REJECTED,
                           checker.INCONCLUSIVE})

    def test_any_mandatory_fail_rejects(self):
        # Change ONLY the outcome: a traced case whose record carries no syscall
        # trace is INVALID rather than FAIL, which is correct and is asserted
        # separately below, but it is not what this test is about.
        for name in fc.MANDATORY_CASES:
            records = all_passing()
            records[name] = dict(passing_record(name), outcome="wrong")
            with self.subTest(case=name):
                self.assertEqual(checker.report(records)["aggregate"],
                                 checker.REJECTED)

    def test_traced_case_without_a_trace_is_invalid_not_fail(self):
        # An unprovable minimality claim must not be silently accepted, and it
        # must not be scored as a falsification either.
        for name in fc.TRACED_CASES:
            records = all_passing()
            records[name] = {"outcome": fc.BY_NAME[name]["predict"],
                             "launch_returned": True}
            with self.subTest(case=name):
                report = checker.report(records)
                self.assertEqual(report["statuses"][name]["status"],
                                 checker.INVALID)
                self.assertEqual(report["aggregate"], checker.INCONCLUSIVE)

    def test_missing_status_is_never_silently_ignored(self):
        records = all_passing()
        del records["E1"]
        report = checker.report(records)
        self.assertEqual(report["statuses"]["E1"]["status"], checker.INVALID)
        self.assertEqual(report["aggregate"], checker.INCONCLUSIVE)

    def test_mandatory_blocked_is_inconclusive(self):
        records = all_passing()
        records["E1"] = {"blocked": "euid_zero"}
        self.assertEqual(checker.report(records)["aggregate"],
                         checker.INCONCLUSIVE)

    def test_conditional_blocked_with_cause_still_accepts(self):
        # This is what "conditional" means. A rule that treated an expected,
        # caused block as inconclusive would make the class meaningless -- and
        # N3 is BLOCKED by construction on any unprivileged runner.
        records = all_passing()
        records["N3"] = {"blocked": "unprivileged_runner"}
        records["X8"] = {"blocked": "no_noexec_mount"}
        report = checker.report(records)
        self.assertEqual(report["aggregate"], checker.ACCEPTED, report["detail"])
        self.assertIn("N3", report["detail"]["conditional_blocked_with_cause"])

    def test_only_a_conditional_case_may_absorb_a_block(self):
        # Letting any class absorb a block would retire a load-bearing gate by
        # reclassification: a recorded case scored BLOCKED would skip its gated
        # sub-assertions while the run still reached ACCEPTED. For P1-P4 those
        # gates are what close the descendant-held-pipe hole.
        for name in fc.MANDATORY_CASES[:3] + fc.RECORDED_CASES:
            records = all_passing()
            records[name] = {"blocked": "euid_zero"}
            with self.subTest(case=name):
                report = checker.report(records)
                self.assertEqual(report["statuses"][name]["status"],
                                 checker.INVALID)
                self.assertEqual(report["aggregate"], checker.INCONCLUSIVE)

    def test_conditional_blocked_with_the_wrong_frozen_cause_is_invalid(self):
        records = all_passing()
        records["X8"] = {"blocked": "no_tracer"}   # X8's cause is no_noexec_mount
        report = checker.report(records)
        self.assertEqual(report["statuses"]["X8"]["status"], checker.INVALID)

    def test_missing_launch_returned_is_invalid(self):
        # A hang is exactly the case where a record could be absent, so the
        # harness must always record whether launch() returned.
        records = all_passing()
        records["E2"] = {"outcome": fc.BY_NAME["E2"]["predict"]}
        report = checker.report(records)
        self.assertEqual(report["statuses"]["E2"]["status"], checker.INVALID)
        self.assertIn("launch()", report["statuses"]["E2"]["reason"])

    def test_n2_and_m3_are_blockable_host_properties(self):
        # A host property must never be scored as a falsified mechanism claim.
        records = all_passing()
        records["N2"] = {"blocked": "parent_no_new_privs_set"}
        records["M3"] = {"blocked": "clone3_unavailable"}
        report = checker.report(records)
        self.assertEqual(report["aggregate"], checker.ACCEPTED, report["detail"])
        for n in ("N2", "M3"):
            self.assertEqual(report["statuses"][n]["status"], checker.BLOCKED)

    def test_blocked_with_unfrozen_cause_is_invalid(self):
        records = all_passing()
        records["X8"] = {"blocked": "because_we_felt_like_it"}
        report = checker.report(records)
        self.assertEqual(report["statuses"]["X8"]["status"], checker.INVALID)
        self.assertEqual(report["aggregate"], checker.INCONCLUSIVE)

    def test_launcher_non_return_rejects_even_in_a_negative_control(self):
        # A launcher-side non-return is never a recordable outcome. This is the
        # single rule that closes the acceptance path a descendant-held pipe
        # opened in the original 43-case definition.
        for name in ("P1", "P2", "P4"):
            records = all_passing()
            records[name] = dict(passing_record(name), launch_returned=False)
            with self.subTest(case=name):
                self.assertEqual(checker.report(records)["aggregate"],
                                 checker.REJECTED)

    def test_exceeding_the_declared_bound_rejects(self):
        records = all_passing()
        records["O6"] = dict(passing_record("O6"),
                             total_bound_ms=9000, elapsed_ms=30000)
        self.assertEqual(checker.report(records)["aggregate"], checker.REJECTED)

    def test_recorded_case_gate_failure_rejects(self):
        records = all_passing()
        records["R4"] = {"outcome": "Exited:42", "launch_returned": True,
                         "gates": {"never_reports_exited_zero": False,
                                   "never_hangs": True}}
        self.assertEqual(checker.report(records)["aggregate"], checker.REJECTED)

    def test_recorded_case_second_safe_member_still_passes(self):
        records = all_passing()
        spec = fc.BY_NAME["E6c"]
        records["E6c"] = {"outcome": spec["safe"][1], "launch_returned": True,
                          "gates": {g: True for g in spec["gates"]}}
        self.assertEqual(checker.report(records)["aggregate"], checker.ACCEPTED)

    def test_documentation_gate_blocks_a_kernel_only_pass(self):
        # E6 cannot be PASSed by observing the expected kernel behaviour while a
        # document still presents the pre-execution measurement as the body that
        # ran.
        records = all_passing()
        records["E6"] = dict(passing_record("E6"), documentation_gate=False)
        report = checker.report(records)
        self.assertEqual(report["statuses"]["E6"]["status"], checker.FAIL)
        self.assertEqual(report["aggregate"], checker.REJECTED)

    def test_unobserved_fact_assertion_rejects(self):
        records = all_passing()
        records["T4"] = dict(passing_record("T4"),
                             asserted_unobserved_fact="TimedOut for an observed "
                                                      "early exit")
        self.assertEqual(checker.report(records)["aggregate"], checker.REJECTED)

    def test_tracing_a_wait_classifying_case_is_invalid(self):
        records = all_passing()
        records["T4"] = dict(passing_record("T4"), traced=True)
        report = checker.report(records)
        self.assertEqual(report["statuses"]["T4"]["status"], checker.INVALID)


class IndependentOracles(unittest.TestCase):
    def test_recipe_is_deterministic_and_independent(self):
        a = oracles.stream_bytes("stdout", 4096)
        b = bytes((i * 251 + 1) % 256 for i in range(4096))
        self.assertEqual(a, b)

    def test_streams_differ_by_tag(self):
        self.assertNotEqual(oracles.stream_digest("stdout", 4096),
                            oracles.stream_digest("stderr", 4096))

    def test_known_empty_digest(self):
        self.assertEqual(
            oracles.digest_of(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")

    def test_sentinel_split_separates_payload_from_report(self):
        payload = oracles.stream_bytes("stdout", 32)
        raw = payload + fc.REPORT_SENTINEL + b'{"marker":"helper_report"}'
        got_payload, report = oracles.split_report(raw)
        self.assertEqual(got_payload, payload)
        self.assertIn(b"helper_report", report)

    def test_absent_report_is_distinguishable(self):
        # S2 and S5 rest on the ABSENCE of a report being evidence that the
        # image never ran.
        payload, report = oracles.split_report(b"no sentinel here")
        self.assertIsNone(report)
        self.assertEqual(payload, b"no sentinel here")

    def test_cohort_predicate_admits_x86_64_and_refuses_foreign(self):
        x86 = make_fixtures.elf64_header(make_fixtures.EM_X86_64)
        arm = make_fixtures.elf64_header(make_fixtures.EM_AARCH64)
        self.assertTrue(oracles.elf64_header_is_in_cohort(x86))
        self.assertFalse(oracles.elf64_header_is_in_cohort(arm))

    def test_cohort_predicate_refuses_magic_only(self):
        # Magic alone is not enough: binfmt_misc matches magic with a mask and
        # resolves its interpreter by pathname.
        self.assertFalse(oracles.elf64_header_is_in_cohort(b"\x7fELF"))

    def test_total_bound_is_the_sum_of_the_frozen_phases(self):
        self.assertEqual(
            oracles.total_bound_ms(5000, 2000, fc.SPAWN_CONFIRM_TIMEOUT_MS,
                                   fc.POST_EXIT_DRAIN_MS),
            5000 + 2000 + fc.SPAWN_CONFIRM_TIMEOUT_MS + fc.POST_EXIT_DRAIN_MS)


class FixtureDeterminism(unittest.TestCase):
    def test_fixture_digests_are_stable(self):
        self.assertEqual(make_fixtures.digests(), make_fixtures.digests())

    def test_foreign_fixture_is_a_well_formed_elf64_header(self):
        data = make_fixtures.FIXTURES["helper_foreign.elf"]
        self.assertEqual(len(data), 64)
        self.assertEqual(data[:4], b"\x7fELF")

    def test_magic_only_fixture_is_exactly_four_bytes(self):
        self.assertEqual(make_fixtures.FIXTURES["magic_only.bin"], b"\x7fELF")


class ProseMatchesManifest(unittest.TestCase):
    """The definition's prose is DERIVED from the manifest; drift is a defect."""

    DEF = (pathlib.Path(__file__).resolve().parents[2] / "docs" / "experiments"
           / "LAUNCH-EXEC-01-DEFINITION.md")

    def setUp(self):
        self.text = self.DEF.read_text(encoding="utf-8")

    def test_every_frozen_case_appears_in_the_definition(self):
        import re
        rows = set(re.findall(r"^\| \*\*([EAVFXORTNPSM][0-9]+[a-d]?)\*\* \|",
                              self.text, re.MULTILINE))
        self.assertEqual(rows, set(fc.MEMBERSHIP),
                         "the prose case table and frozen_cases.py disagree")

    def test_stated_counts_match_derived_counts(self):
        s = fc.summary()
        self.assertIn(f"| **Mandatory** — must PASS | **{s['mandatory']}** |",
                      self.text)
        self.assertIn(f"| **Total** | **{s['total']}** | |", self.text)
        self.assertIn(f"## 3. Preregistered cases — {s['total']}", self.text)

    def test_instant_rejection_list_matches(self):
        listed = ", ".join(sorted(fc.INSTANT_REJECT))
        self.assertIn(listed, self.text,
                      "the instant-rejection list in the prose is not the "
                      "manifest's instant_reject set")

    def test_definition_still_says_not_run(self):
        self.assertIn("NOT_RUN", self.text)
        self.assertIn("D-7", self.text)


class NotRun(unittest.TestCase):
    def test_runner_refuses_without_owner_authorisation(self):
        # Importing the runner must not pose a case, and its default path must
        # refuse: D-7 is not granted.
        import run_launch_exec_01 as runner
        self.assertEqual(runner.main([]), 3)


if __name__ == "__main__":
    unittest.main()
