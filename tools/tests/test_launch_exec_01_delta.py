"""Trial #2 delta correction: N-1 through N-5 and the dataflow completeness gates.

Every observation here is fabricated in-process. The POSIX-only tests speak the
post-pin barrier protocol to a harmless Python stand-in that plays the launcher's
part -- it holds descriptors, carries a signal state and answers READY -- so the
harness side of each proof runs against a real process. Nothing in this file
builds, executes or otherwise invokes ``launcher_spike``, any helper, any
generated ELF or any preregistered case, and no test constructs a
``driver.Authorisation``. LAUNCH-EXEC-01 remains NOT_RUN.
"""
import ast
import contextlib
import os
import pathlib
import shutil
import signal
import stat
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
import frozen_cases as fc   # noqa: E402
import observations as ob   # noqa: E402
import oracles              # noqa: E402

LINUX_ONLY = unittest.skipUnless(sys.platform.startswith("linux"),
                                 "reads /proc; Linux CI and WSL run it")
SOURCE = (EXP / "driver.py").read_text(encoding="utf-8")
TREE = ast.parse(SOURCE)


def function(name, tree=TREE):
    for node in ast.walk(tree):
        if isinstance(node, (ast.FunctionDef, ast.ClassDef)) and node.name == name:
            return node
    raise KeyError(name)


def source_of(name):
    return ast.unparse(function(name))


# --------------------------------------------------------------- fabrication
def receipt(**over):
    base = {
        "admission": "accepted", "pre_exec_body_sha256": "a" * 64,
        "pre_exec_body_size": 4096, "pre_exec_mode_bits": 0o755,
        "process_disposition": "Exited", "timeout_disposition": "",
        "exec_failed_stage": "", "exec_failed_errno": 0, "exit_code": 0,
        "term_signal": -1, "launcher_signal_issued": False,
        "group_sweep_issued": True, "wait_errno": 0,
        "stdout": {"bytes_drained": 0, "drained_sha256": oracles.digest_of(b""),
                   "completeness": "CompleteAtEof"},
        "stderr": {"bytes_drained": 0, "drained_sha256": oracles.digest_of(b""),
                   "completeness": "CompleteAtEof"},
    }
    base.update(over)
    return base


def report(marker="helper_report"):
    return {"marker": marker, "argv": [], "environ": [],
            "descriptors": [{"fd": n, "cloexec": False} for n in (0, 1, 2)],
            "signals": {"SigBlk": "0" * 16, "SigIgn": "0" * 16,
                        "SigCgt": "0" * 16, "SigPnd": "0" * 16},
            "no_new_privs": "1"}


def obs_for(plan, spike=None, rep=None, **over):
    """A fabricated observation shaped like pose()'s, for one plan."""
    obs = {
        "spike": receipt() if spike is None else spike,
        "report": rep,
        "report_state": ob.REPORT_COMPLETE if rep is not None else ob.REPORT_ABSENT,
        "launch_returned": True, "elapsed_ms": 12,
        "declared_pre_exec_stall": plan.pre_exec_stall,
        "declared_body_length_changed": plan.body_length_changed,
        "declared_injection_modes": [], "expected_marker": plan.expected_marker,
        "build_identity_binding": {"bound": True, "classification": "X",
                                   "object_sha256": "a" * 64},
        "cleanup_problems": [],
    }
    obs.update(over)
    obs["exec_confirmation"] = ob.exec_confirmation(
        obs["spike"], obs["report"], obs.get("payload_is_recipe"),
        obs["report_state"])
    obs.setdefault("repeat_observations", [obs])
    return obs


def score(plan, obs):
    record = driver.evaluate(plan, obs)
    return checker.score_case(plan.case, record), record


class SafetyBarrier(unittest.TestCase):
    def test_this_suite_constructs_no_authorisation(self):
        text = pathlib.Path(__file__).read_text(encoding="utf-8")
        self.assertNotIn("Authorisation" + "(True)", text)
        self.assertNotIn("driver." + "observe(", text)
        self.assertNotIn("driver." + "pose(", text)


# ============================================================ N-1 contract
class ParentStateContract(unittest.TestCase):
    """Every semantic parent-state field has a consumer and a prover."""

    def declared(self, name):
        ctx = driver.TrialContext(build="/x/build", work="/x/work", preflight={},
                                  freeze={}, sanitiser=evidence.Sanitiser())
        plan = next(p for p in driver._PLAN_LIST if p.parent == name) \
            if any(p.parent == name for p in driver._PLAN_LIST) \
            else driver.CASE_PLANS["R1"]
        return driver.PARENT_STATES[name](ctx, plan)

    def test_every_declared_key_is_in_the_closed_schema(self):
        for name in driver.PARENT_STATES:
            extra = set(self.declared(name)) - set(driver.PARENT_STATE_SCHEMA)
            self.assertEqual(extra, set(), name)

    def test_every_schema_key_has_a_consumer_and_a_prover(self):
        consume = source_of("AppliedParentState")
        prove = source_of("prove")
        constant = {value: key for key, value in vars(driver).items()
                    if key.startswith("PS_")}
        for field in driver.PARENT_STATE_SCHEMA:
            name = constant[field]
            if field == driver.PS_THREADS:
                # Created by the plan's own --extra-threads, carried by pose(),
                # proven by the threaded_parent_observed posed check.
                self.assertIn("parent_state.get(PS_THREADS)", source_of("pose"))
                self.assertIn("declared_launcher_threads",
                              driver.POSED_CHECK_READS["threaded_parent_observed"])
                continue
            self.assertIn("state.get(%s)" % name, consume, field + " is not consumed")
            self.assertIn(name, prove, field + " is not proven")
            self.assertIn(field, driver.BARRIER_PROVEN_STATE_KEYS)

    def test_every_semantic_parent_state_is_proven_somewhere(self):
        for plan in driver._PLAN_LIST:
            state = self.declared(plan.parent)
            if not state:
                continue
            barrier = bool(set(state) & driver.BARRIER_PROVEN_STATE_KEYS)
            if not barrier:
                self.assertEqual(plan.posed_when, "threaded_parent_observed",
                                 plan.case)

    def test_the_n1_users_are_exactly_the_frozen_parent_cases(self):
        users = sorted(p.case for p in driver._PLAN_LIST if p.parent != "none")
        self.assertEqual(users, ["F2", "F3", "F5", "F6", "F7", "M2", "M5",
                                 "R4", "T6", "V1"])

    def test_the_declarations_create_the_frozen_conditions(self):
        f2, f3 = self.declared("non_cloexec_fd"), self.declared("cloexec_fd")
        self.assertIs(f2[driver.PS_INHERIT_FD]["cloexec"], False)
        self.assertIs(f3[driver.PS_INHERIT_FD]["cloexec"], True)
        f5 = self.declared("block_and_ignore_signals")
        self.assertEqual(sorted(f5[driver.PS_BLOCK]), [10, 15])
        self.assertEqual(sorted(f5[driver.PS_IGNORE]), [12, 13])
        t6 = self.declared("sigterm_blocked_sigpipe_ignored")
        self.assertEqual((t6[driver.PS_BLOCK], t6[driver.PS_IGNORE]), ([15], [13]))
        self.assertEqual(self.declared("sigchld_ignore")[driver.PS_IGNORE], [17])
        self.assertEqual(self.declared("close_stdio")[driver.PS_CLOSE_LOW], 3)
        self.assertEqual(self.declared("adjacent_fds")[driver.PS_CLOSE_LOW], 1)

    def test_an_undeclared_key_is_refused(self):
        with self.assertRaises(ValueError):
            driver.AppliedParentState("x", {"open_non_cloexec": "/tmp/x"})

    def test_caller_state_flags_are_never_a_plans_own_spike_flags(self):
        for plan in driver._PLAN_LIST:
            for flag in ("--parent-fd-set-cloexec", "--parent-close-low-fds",
                         "--post-pin-control-fd"):
                self.assertNotIn(flag, plan.spike_flags, plan.case)

    def test_the_c_side_implements_both_caller_state_flags(self):
        spike = (EXP / "launcher_spike.c").read_text(encoding="utf-8")
        for flag in ("--parent-fd-set-cloexec", "--parent-close-low-fds"):
            self.assertIn('"%s"' % flag, spike)
        # Established before the M5 arm and the pin, never again.
        self.assertLess(spike.index("parent_close_low_fds > 0"),
                        spike.index("run_rejected_acquisition_arm(&rejected)"))
        self.assertLess(spike.index("parent_close_low_fds > 0"),
                        spike.index("int exec_fd = open(exec_path"))


PEER = r'''
import os, sys
argv = sys.argv[1:]
def opt(name):
    return argv[argv.index(name) + 1] if name in argv else None
ctl = int(opt("--post-pin-control-fd"))
if opt("--parent-fd-set-cloexec") is not None:
    import fcntl
    fcntl.fcntl(int(opt("--parent-fd-set-cloexec")), fcntl.F_SETFD, fcntl.FD_CLOEXEC)
low = opt("--parent-close-low-fds")
if low is not None:
    for fd in range(int(low)):
        os.close(fd)
    os.open(opt("--exec-path"), os.O_RDONLY)
    if int(low) >= 3:
        os.open(opt("--work-dir"), os.O_RDONLY | os.O_DIRECTORY)
held = [os.open(argv[i + 1], os.O_RDONLY)
        for i, a in enumerate(argv) if a == "--hold"]
os.write(ctl, b"R")
os._exit(0 if os.read(ctl, 1) == b"C" else 3)
'''


@LINUX_ONLY
class ParentStateAgainstALiveStandIn(unittest.TestCase):
    """The barrier proves each state IN the stand-in, and a bypass is caught."""

    def setUp(self):
        self.tmp = pathlib.Path(tempfile.mkdtemp(prefix="delta-parent-"))
        (self.tmp / "helper_report").write_bytes(b"\x7fELF" + b"Z" * 400)
        self.work = self.tmp / "work"
        self.work.mkdir()
        self.ctx = driver.TrialContext(
            build=str(self.tmp), work=str(self.work), preflight={}, freeze={},
            sanitiser=evidence.Sanitiser(),
            build_identity=harness.build_identity(str(self.tmp)))

    def tearDown(self):
        shutil.rmtree(self.tmp, ignore_errors=True)

    def run_case(self, case, bypass=None, hold=()):
        plan = driver.CASE_PLANS[case]
        applied = driver.AppliedParentState(
            plan.parent, driver.PARENT_STATES[plan.parent](self.ctx, plan))
        if bypass:
            bypass(applied)
        built = {"exec_path": str(self.tmp / "helper_report")}
        argv = ([sys.executable, "-c", PEER] + driver.spike_argv(
            plan, built["exec_path"], str(self.work), str(self.tmp),
            parent_flags=applied.spike_flags)[1:])
        for path in hold:
            argv += ["--hold", str(path)]
        try:
            return driver._run_with_post_pin(argv, applied, 20.0, self.ctx,
                                             plan, built)
        finally:
            applied.release()

    def assertPosed(self, out):
        self.assertNotIn("not_posed", out, out)
        self.assertEqual(out["rc"], 0)
        self.assertTrue(out["post_pin"] and all(f["landed"] for f in out["post_pin"]),
                        out["post_pin"])

    def assertBypassCaught(self, out):
        self.assertIn("not_posed", out)
        self.assertIn("did not land", out["not_posed"])

    def test_f2_non_cloexec_descriptor(self):
        out = self.run_case("F2")
        self.assertPosed(out)
        self.assertIs(out["post_pin"][0]["cloexec_in_launcher"], False)
        self.assertBypassCaught(self.run_case(
            "F2", bypass=lambda a: setattr(a, "pass_fds", ())))

    def test_f3_cloexec_descriptor(self):
        out = self.run_case("F3")
        self.assertPosed(out)
        self.assertIs(out["post_pin"][0]["cloexec_in_launcher"], True)
        self.assertBypassCaught(self.run_case(
            "F3", bypass=lambda a: setattr(a, "spike_flags", [])))

    def test_f5_t6_r4_m5_signal_states(self):
        for case in ("F5", "T6", "R4", "M5"):
            with self.subTest(case=case):
                self.assertPosed(self.run_case(case))
                self.assertBypassCaught(self.run_case(
                    case, bypass=lambda a: setattr(a, "spawning",
                                                   contextlib.nullcontext)))

    def test_f6_and_f7_low_descriptors(self):
        # F7 runs under strace in a trial; here the stand-in is spawned
        # directly, so its pid is read as an untraced launcher's would be.
        saved = driver._launcher_pid
        driver._launcher_pid = lambda proc, plan: proc.pid
        try:
            for case in ("F6", "F7"):
                with self.subTest(case=case):
                    self.assertPosed(self.run_case(case))
                    self.assertBypassCaught(self.run_case(
                        case, bypass=lambda a: setattr(a, "spike_flags", [])))
        finally:
            driver._launcher_pid = saved

    def test_v1_environment_names(self):
        out = self.run_case("V1")
        self.assertPosed(out)
        self.assertBypassCaught(self.run_case(
            "V1", bypass=lambda a: setattr(a, "env", {})))

    def test_no_state_outlives_the_spawn(self):
        mask = signal.pthread_sigmask(signal.SIG_BLOCK, [])
        handlers = {s: signal.getsignal(s) for s in (12, 13, 17)}
        for case in ("F2", "F3", "F5", "T6", "R4"):
            self.run_case(case)
        self.assertEqual(signal.pthread_sigmask(signal.SIG_BLOCK, []), mask)
        self.assertEqual({s: signal.getsignal(s) for s in handlers}, handlers)

    def test_the_inherited_descriptor_is_released(self):
        plan = driver.CASE_PLANS["F2"]
        applied = driver.AppliedParentState(
            plan.parent, driver.PARENT_STATES[plan.parent](self.ctx, plan))
        fd = applied.pass_fds[0]
        self.assertEqual(applied.release(), [])
        with self.assertRaises(OSError):
            os.fstat(fd)


# ========================================================= N-3, N-4b stand-ins
@LINUX_ONLY
class LauncherRelationsAgainstALiveStandIn(unittest.TestCase):
    def setUp(self):
        self.tmp = pathlib.Path(tempfile.mkdtemp(prefix="delta-rel-"))
        self.exe = self.tmp / "E5_helper_report"
        self.exe.write_bytes(b"\x7fELF" + b"Z" * 400)
        self.ctx = driver.TrialContext(
            build=str(self.tmp), work=str(self.tmp), preflight={}, freeze={},
            sanitiser=evidence.Sanitiser(),
            build_identity=harness.build_identity(str(self.tmp)))

    def tearDown(self):
        for path in self.tmp.rglob("*"):
            if path.is_dir():
                os.chmod(path, 0o755)
        shutil.rmtree(self.tmp, ignore_errors=True)

    def run_barrier(self, case, built, hold=()):
        argv = [sys.executable, "-c", PEER]
        for path in hold:
            argv += ["--hold", str(path)]
        return driver._run_with_post_pin(argv, None, 20.0, self.ctx,
                                         driver.CASE_PLANS[case], built)

    def test_e5_writer_is_proven_against_the_launchers_pinned_inode(self):
        built = {"exec_path": str(self.exe), "hold_writer": True}
        out = self.run_barrier("E5", built, hold=[self.exe])
        self.assertNotIn("not_posed", out, out)
        fact = out["post_pin"][0]
        self.assertIs(fact["launcher_holds_same_inode"], True)
        self.assertIn("O_RDONLY", fact["launcher_access_modes"])
        self.assertTrue(driver.POSED_CHECKS["same_inode_as_writer"](
            {"post_pin_evidence": out["post_pin"]}))
        driver.release_setup_resources(built)

    def test_e5_is_not_posed_when_the_launcher_does_not_hold_the_object(self):
        other = self.tmp / "other"
        other.write_bytes(b"x")
        built = {"exec_path": str(self.exe), "hold_writer": True}
        out = self.run_barrier("E5", built, hold=[other])
        self.assertIn("not_posed", out)
        self.assertIs(out["post_pin"][0]["launcher_holds_same_inode"], False)
        self.assertFalse(driver.POSED_CHECKS["same_inode_as_writer"](
            {"post_pin_evidence": out["post_pin"]}))

    def test_s2_directory_changes_only_after_the_launcher_holds_it(self):
        work = self.tmp / "S2_workdir"
        work.mkdir(mode=0o755)
        built = {"exec_path": str(self.exe), "work_dir": str(work),
                 "post_pin": ("chmod_work_dir", 0)}
        out = self.run_barrier("S2", built, hold=[work])
        self.assertNotIn("not_posed", out, out)
        fact = out["post_pin"][0]
        self.assertIs(fact["capability_open_in_launcher"], True)
        self.assertEqual(fact["mode_after"], 0)
        self.assertEqual(stat.S_IMODE(work.stat().st_mode), 0)
        self.assertEqual(driver.restore_after_trial(built), [])
        self.assertEqual(stat.S_IMODE(work.stat().st_mode), 0o755)
        self.assertEqual(driver.release_setup_resources(built), [])
        self.assertFalse(work.exists())

    def test_s2_is_not_posed_if_the_launcher_never_opened_the_directory(self):
        work = self.tmp / "S2_workdir"
        work.mkdir(mode=0o755)
        built = {"exec_path": str(self.exe), "work_dir": str(work),
                 "post_pin": ("chmod_work_dir", 0)}
        out = self.run_barrier("S2", built)
        self.assertIn("not_posed", out)
        self.assertEqual(stat.S_IMODE(work.stat().st_mode), 0o755)

    def test_s2_s7_setup_hands_the_launcher_its_case_private_directory(self):
        (self.tmp / "helper_report").write_bytes(b"\x7fELF" + b"Z" * 400)
        self.ctx.build_identity = harness.build_identity(str(self.tmp))
        for case in ("S2", "S7"):
            plan = driver.CASE_PLANS[case]
            built = driver.SETUPS[plan.setup](self.ctx, plan)
            self.assertEqual(built["post_pin"], ("chmod_work_dir", 0))
            argv = driver.spike_argv(plan, built["exec_path"], built["work_dir"],
                                     str(self.tmp))
            self.assertEqual(argv[argv.index("--work-dir") + 1], built["work_dir"])
            driver.release_setup_resources(built)
        self.assertIn("built.get('work_dir') or ctx.work",
                      source_of("_launch_and_observe"))
        # The F6 proof compares the launcher's fd 1 with the SAME capability.
        self.assertIn("built.get('work_dir') or ctx.work",
                      source_of("_run_with_post_pin"))


# ================================================== N-3/N-4a/N-2 dataflow
def reads_in(fn_node):
    """Observation keys a function reads: obs.get("k"), obs["k"], _landing_fact."""
    keys = set()
    for node in ast.walk(fn_node):
        if isinstance(node, ast.Call) and isinstance(node.func, ast.Attribute) \
                and node.func.attr == "get" and ast.unparse(node.func.value) == "obs" \
                and node.args and isinstance(node.args[0], ast.Constant):
            keys.add(node.args[0].value)
        if isinstance(node, ast.Subscript) and ast.unparse(node.value) == "obs" \
                and isinstance(node.slice, ast.Constant):
            keys.add(node.slice.value)
        if isinstance(node, ast.Call) and ast.unparse(node.func).endswith(
                "_landing_fact"):
            keys.add("post_pin_evidence")
        if isinstance(node, ast.Call) and ast.unparse(node.func).endswith(
                "_usable_report"):
            keys |= {"report", "report_state"}
    return keys


def produced_keys():
    """Every observation key pose() and _launch_and_observe() can produce."""
    keys = {"not_posed", "launch_returned", "blocked"}
    ret = [n for n in ast.walk(function("_launch_and_observe"))
           if isinstance(n, ast.Return) and isinstance(n.value, ast.Dict)]
    for node in ret:
        keys |= {k.value for k in node.value.keys if isinstance(k, ast.Constant)}
    for node in ast.walk(function("pose")):
        if isinstance(node, ast.Subscript) and ast.unparse(node.value) == "obs" \
                and isinstance(node.slice, ast.Constant):
            keys.add(node.slice.value)
    return keys


def path_produces(plan, key):
    """Keys only some launch paths produce, and the condition for each."""
    ctx = driver.TrialContext(build="/x", work="/x", preflight={}, freeze={},
                              sanitiser=evidence.Sanitiser())
    state = driver.PARENT_STATES[plan.parent](ctx, plan)
    if key == "post_pin_evidence":
        setup = ast.unparse(function(
            next(n.name for n in ast.walk(TREE) if isinstance(n, ast.FunctionDef)
                 and any(isinstance(d, ast.Call) and d.args
                         and getattr(d.args[0], "value", None) == plan.setup
                         and getattr(d.func, "id", "") == "_setup"
                         for d in n.decorator_list))))
        # A setup declares a directive either as a keyword (dict(info,
        # post_pin=...)) or as a literal key ({"post_pin": ...}).
        directive = any(token in setup for token in (
            "post_pin=", "hold_writer=", "'post_pin':", "'hold_writer':"))
        return directive or bool(set(state) & driver.BARRIER_PROVEN_STATE_KEYS)
    if key == "exec_status_pair_adjacent":
        # F417-B4: produced only from a traced launcher that opened its exec
        # descriptor at 0, which --parent-close-low-fds guarantees.
        return plan.traced and bool(state.get(driver.PS_CLOSE_LOW))
    if key == "descendant_alive_after_launch":
        return plan.setup == "fork_helper"
    if key == "baseline_observation":
        return plan.baseline_flags is not None
    if key == "declared_launcher_threads":
        return driver.PS_THREADS in state
    if key == "expected_marker":
        return plan.expected_marker is not None
    return True


class ObservationDependencies(unittest.TestCase):
    """Every posed-check and assertion input has a producer on the case's path."""

    def test_each_posed_check_declares_exactly_what_it_reads(self):
        for name, fn in driver.POSED_CHECKS.items():
            node = function(fn.__name__)
            self.assertEqual(reads_in(node), set(driver.POSED_CHECK_READS[name]),
                             name)

    def test_each_assertion_declares_exactly_what_it_reads(self):
        tree = ast.parse((EXP / "observations.py").read_text(encoding="utf-8"))
        for name, fn in ob.ASSERTIONS.items():
            self.assertEqual(reads_in(function(fn.__name__, tree)),
                             set(ob.ASSERTION_READS[name]), name)

    def test_every_dependency_has_a_producer_on_the_cases_path(self):
        produced = produced_keys()
        missing = []
        for plan in driver._PLAN_LIST:
            wants = set(driver.POSED_CHECK_READS.get(plan.posed_when, ()))
            for name in plan.assertions:
                wants |= set(ob.ASSERTION_READS[name])
            for key in sorted(wants):
                if key not in produced or not path_produces(plan, key):
                    missing.append((plan.case, key))
        self.assertEqual(missing, [])

    def test_the_detector_would_have_caught_e5_and_o5(self):
        """The Trial #2 defect, replayed against the detector."""
        produced = produced_keys()
        for key in ("writer_inode", "pinned_inode", "spike_cpu_ms"):
            self.assertNotIn(key, produced)
        old = ast.parse('def c(obs):\n    return obs.get("spike_cpu_ms") < 200\n')
        self.assertEqual(reads_in(old.body[0]), {"spike_cpu_ms"})

    def test_poll_returns_is_emitted_by_the_launcher(self):
        spike = (EXP / "launcher_spike.c").read_text(encoding="utf-8")
        self.assertIn("poll_returns_not_in_receipt", spike)
        self.assertIn("poll_returns++;", spike)
        self.assertIn("poll_returns_not_in_receipt", source_of("_launch_and_observe"))
        # Kept out of the published receipt, as the elapsed time is.
        self.assertNotIn("poll_returns_not_in_receipt", evidence.receipt_view(
            receipt(poll_returns_not_in_receipt=3)))


class CasePlanFieldsAreNeverInert(unittest.TestCase):
    def test_every_slot_has_a_declared_consumer_that_reads_it(self):
        self.assertEqual(set(driver.CasePlan.__slots__),
                         set(driver.CASEPLAN_FIELD_CONSUMERS))
        for field, consumer in driver.CASEPLAN_FIELD_CONSUMERS.items():
            if consumer is None:
                self.assertEqual(field, "note")
                continue
            body = source_of(consumer)
            self.assertTrue("plan.%s" % field in body or "self.%s" % field in body,
                            "%s is not read by %s" % (field, consumer))

    def test_the_inert_s2_s7_field_is_gone(self):
        self.assertNotIn("work_dir_kind", driver.CasePlan.__slots__)
        used = [n for n in ast.walk(TREE)
                if (isinstance(n, ast.keyword) and n.arg == "work_dir_kind")
                or (isinstance(n, ast.Attribute) and n.attr == "work_dir_kind")]
        self.assertEqual(used, [])


# ==================================================== N-2 executed identity
class ExecutedIdentity(unittest.TestCase):
    def test_substituted_body_is_a_fail_not_invalid_or_pass(self):
        for case, marker in (("E2", "helper_alt"), ("E4", "helper_alt"),
                             ("E3", "helper_alt"), ("E1", "helper_alt"),
                             ("E6", "helper_report")):
            plan = driver.CASE_PLANS[case]
            (status, why), record = score(plan, obs_for(
                plan, rep=report(marker), trace={"child_syscalls": ["execveat"]}))
            self.assertEqual(status, checker.FAIL, (case, why, record))
            self.assertEqual(record["outcome"], "executed_body_mismatch")
            self.assertEqual(record["mechanism_outcome"], "Exited:0")

    def test_the_declared_body_passes(self):
        for case in ("E1", "E2", "E3", "E4", "E6"):
            plan = driver.CASE_PLANS[case]
            obs = obs_for(plan, rep=report(plan.expected_marker),
                          trace={"child_syscalls": ["execveat"]})
            (status, why), _ = score(plan, obs)
            self.assertEqual(status, checker.PASS, (case, why))

    def test_e6_declares_the_mutated_marker_and_writes_exactly_it(self):
        plan = driver.CASE_PLANS["E6"]
        self.assertEqual(plan.expected_marker, ob.E6_MUTATED_MARKER)
        self.assertEqual(driver._marker_bytes(plan.expected_marker),
                         b"MUTATED" + b"\0" * 9)
        self.assertIn("_marker_bytes(plan.expected_marker)",
                      source_of("_setup_mutate_marker"))
        self.assertNotIn("+ 1", source_of("_pp_pwrite_marker"))

    def test_a_measurement_other_than_the_starting_identity_fails(self):
        for case in ("E1", "E2", "E4", "E6"):
            plan = driver.CASE_PLANS[case]
            obs = obs_for(plan, rep=report(plan.expected_marker),
                          trace={"child_syscalls": ["execveat"]},
                          build_identity_binding={"bound": True,
                                                  "object_sha256": "b" * 64})
            (status, _), record = score(plan, obs)
            self.assertEqual(status, checker.FAIL, case)
            self.assertEqual(record["outcome"], "measurement_mismatch")

    def test_missing_starting_identity_is_never_a_pass(self):
        plan = driver.CASE_PLANS["E2"]
        obs = obs_for(plan, rep=report("helper_report"),
                      build_identity_binding=None)
        (status, _), record = score(plan, obs)
        self.assertEqual(status, checker.INVALID, record)

    def test_mode_bits_must_be_the_pre_change_value(self):
        for case in ("E6d", "X1"):
            plan = driver.CASE_PLANS[case]
            failed = receipt(process_disposition="ExecFailed",
                             exec_failed_stage="EXEC", exec_failed_errno=13,
                             pre_exec_mode_bits=0o755)
            fact = {"action": "fchmod", "landed": True, "detail": "d",
                    "mode_before": 0o755, "mode_after": 0}
            (status, why), _ = score(plan, obs_for(plan, spike=failed,
                                                   post_pin_evidence=[fact]))
            self.assertEqual(status, checker.PASS, (case, why))
            lied = dict(failed, pre_exec_mode_bits=0)
            (status, _), record = score(plan, obs_for(plan, spike=lied,
                                                      post_pin_evidence=[fact]))
            self.assertEqual(status, checker.FAIL, case)
            self.assertEqual(record["outcome"], "mode_measurement_mismatch")
            (status, _), _ = score(plan, obs_for(plan, spike=failed))
            self.assertEqual(status, checker.INVALID, case)

    def test_no_violation_token_is_any_cases_expectation(self):
        for token in ob.ASSERTION_VIOLATION_TOKENS.values():
            for spec in fc.CASES:
                self.assertNotEqual(token, spec["predict"])
                self.assertNotIn(token, spec["safe"] or ())


# ========================================================== N-4a O5 drain
class BoundedDrain(unittest.TestCase):
    plan = driver.CASE_PLANS["O5"]

    def o5(self, **measured):
        spike = receipt(stdout={"bytes_drained": 4096,
                                "drained_sha256": oracles.stream_digest("stdout", 4096),
                                "completeness": "CompleteAtEof"},
                        stderr={"bytes_drained": 4096,
                                "drained_sha256": oracles.stream_digest("stderr", 4096),
                                "completeness": "CompleteAtEof"})
        return score(self.plan, obs_for(self.plan, spike=spike,
                                        expected_streams=self.plan.streams,
                                        payload_is_recipe=True, **measured))

    def test_o5_no_longer_has_an_unsatisfiable_posed_check(self):
        self.assertIsNone(self.plan.posed_when)
        self.assertNotIn("bounded_poll_and_cpu", driver.POSED_CHECKS)
        self.assertEqual(self.plan.assertions, ("bounded_drain",))

    def test_absent_cpu_or_poll_count_cannot_pass(self):
        for measured in ({}, {"launcher_cpu_ms": 10}, {"poll_returns": 5},
                         {"launcher_cpu_ms": None, "poll_returns": 5},
                         {"launcher_cpu_ms": True, "poll_returns": 5}):
            (status, _), record = self.o5(**measured)
            self.assertEqual(status, checker.INVALID, (measured, record))

    def test_an_exceeded_bound_is_a_fail(self):
        for measured in ({"launcher_cpu_ms": 200, "poll_returns": 5},
                         {"launcher_cpu_ms": 10, "poll_returns": 10001}):
            (status, _), record = self.o5(**measured)
            self.assertEqual(status, checker.FAIL, measured)
            self.assertEqual(record["outcome"], "drain_unbounded")

    def test_within_both_bounds_passes(self):
        (status, why), record = self.o5(launcher_cpu_ms=199, poll_returns=10000)
        self.assertEqual(status, checker.PASS, why)
        self.assertEqual(record["posing_evidence"]["measured"],
                         {"launcher_cpu_ms": 199, "poll_returns": 10000})

    def test_cpu_is_measured_outside_the_launcher(self):
        body = source_of("_launch_and_observe")
        self.assertIn("_children_cpu_ms()", body)
        self.assertIn("RUSAGE_CHILDREN", source_of("_children_cpu_ms"))


# =========================================================== F7 and R4, M2
def window(*child_lines):
    head = ["111 clone3({flags=CLONE_PIDFD, pidfd=0x7ffd0000, "
            "exit_signal=SIGCHLD}, 88) = 222"]
    return "\n".join(head + ["222 " + line for line in child_lines]) + "\n"


class DescriptorLayout(unittest.TestCase):
    """F7's pair, from the PARENT's pre-clone records (F417-B4 replaced the
    child close_range reading these tests first covered)."""

    PIPES = ["pipe2([4, 5], O_CLOEXEC) = 0", "pipe2([6, 7], O_CLOEXEC) = 0",
             "pipe2([8, 9], O_CLOEXEC) = 0"]

    def adjacent(self, *parent_lines):
        text = "\n".join(["111 " + line for line in parent_lines] + [
            "111 clone3({flags=CLONE_PIDFD, pidfd=0x7ffd0000, "
            "exit_signal=SIGCHLD}, 88) = 222"]) + "\n"
        return ob.parent_pair_is_adjacent(ob.parse_parent_descriptor_pair(text, 0))

    def test_adjacent_pairs_are_recognised_in_both_orders(self):
        self.assertIs(self.adjacent(*self.PIPES, "pipe2([10, 11], O_CLOEXEC) = 0",
                                    "fcntl(0, F_DUPFD_CLOEXEC, 3) = 12"), True)
        self.assertIs(self.adjacent(*self.PIPES, "pipe2([10, 13], O_CLOEXEC) = 0",
                                    "fcntl(0, F_DUPFD_CLOEXEC, 3) = 12"), True)

    def test_a_gap_between_them_is_not_adjacent(self):
        self.assertIs(self.adjacent(*self.PIPES, "pipe2([10, 11], O_CLOEXEC) = 0",
                                    "fcntl(0, F_DUPFD_CLOEXEC, 3) = 14"), False)

    def test_missing_evidence_is_never_a_layout(self):
        self.assertIsNone(self.adjacent(*self.PIPES, "pipe2([10, 11], O_CLOEXEC) = ?",
                                        "fcntl(0, F_DUPFD_CLOEXEC, 3) = 12"))
        self.assertIsNone(self.adjacent(*self.PIPES,
                                        "fcntl(0, F_DUPFD_CLOEXEC, 3) = 12"))
        self.assertIsNone(ob.parent_pair_is_adjacent(None))
        self.assertIsNone(ob.parse_parent_descriptor_pair("", 0))

    def test_f7_is_posed_only_when_adjacency_was_observed(self):
        plan = driver.CASE_PLANS["F7"]
        for value, posed in ((True, True), (False, False), (None, False)):
            obs = obs_for(plan, rep=report(), exec_status_pair_adjacent=value,
                          trace={"child_syscalls": ["execveat"]})
            record = driver.evaluate(plan, obs)
            self.assertEqual("not_posed" not in record, posed, value)
            self.assertEqual(record["posing_evidence"]["posed_check"]["held"], posed)


class RecordedGatesAndControlArms(unittest.TestCase):
    def test_r4_gate_is_computed_from_the_real_token(self):
        plan = driver.CASE_PLANS["R4"]
        self.assertFalse(driver.gates_for(plan, {"launch_returned": True},
                                          "Exited:0")["never_reports_exited_zero"])
        self.assertTrue(driver.gates_for(plan, {"launch_returned": True},
                                         "Exited:42")["never_reports_exited_zero"])
        self.assertFalse(driver.gates_for(plan, {})["never_reports_exited_zero"])
        self.assertNotIn('get("outcome_token")', SOURCE)

    def test_r4_reporting_exited_zero_fails(self):
        plan = driver.CASE_PLANS["R4"]
        (status, _), _ = score(plan, obs_for(plan, rep=report(),
                                             spike=receipt(exit_code=0)))
        self.assertEqual(status, checker.FAIL)

    def test_m2_needs_its_threads_and_its_control(self):
        plan = driver.CASE_PLANS["M2"]
        check = driver.POSED_CHECKS[plan.posed_when]

        def shape(n):
            return receipt(parent_shape={"extra_threads": n,
                                         "atfork_handler_registered": n > 0})
        good = {"spike": shape(3), "baseline_observation": {"spike": shape(0)},
                "declared_launcher_threads": 3}
        self.assertTrue(check(good))
        self.assertFalse(check(dict(good, spike=shape(0))))
        self.assertFalse(check(dict(good, baseline_observation={"spike": shape(3)})))
        self.assertFalse(check(dict(good, declared_launcher_threads=None)))


# ================================================= repeated trials (S7, O8)
class RepeatedTrials(unittest.TestCase):
    plan = driver.CASE_PLANS["S7"]

    def trial(self, **over):
        spike = receipt(process_disposition="ExecFailed",
                        exec_failed_stage="CHDIR", exec_failed_errno=13)
        base = {"spike": spike, "report": None,
                "report_state": ob.REPORT_ABSENT, "launch_returned": True,
                "exec_confirmation": ob.EXEC_PRE_EXEC_ERROR}
        base.update(over)
        return base

    def obs(self, trials):
        obs = dict(obs_for(self.plan, spike=trials[0]["spike"]), **trials[0])
        obs["repeat_observations"] = trials
        return obs

    def test_every_trial_posed_and_agreeing_passes(self):
        (status, why), _ = score(self.plan, self.obs([self.trial()] * 5))
        self.assertEqual(status, checker.PASS, why)

    def test_one_unposed_trial_makes_the_case_invalid(self):
        trials = [self.trial(), self.trial(not_posed="post-pin control: x")]
        (status, why), _ = score(self.plan, self.obs(trials))
        self.assertEqual(status, checker.INVALID)
        self.assertIn("repeated trial 1", why)

    def test_a_reported_image_in_any_trial_is_a_fail(self):
        # F417-B3 corrected this test's former expectation of INVALID: the
        # directory's landing is proven at the barrier, so an image that ran
        # is the repetition's RESULT and fails S7 (definition section 9.6).
        trials = [self.trial(), self.trial(report=report(),
                                           report_state=ob.REPORT_COMPLETE)]
        (status, _), record = score(self.plan, self.obs(trials))
        self.assertEqual(status, checker.FAIL)
        self.assertEqual(record["outcome"], "executed_image_observed")

    def test_a_disagreeing_trial_is_a_fail(self):
        other = receipt(process_disposition="ExecFailed", exec_failed_stage="EXEC",
                        exec_failed_errno=13)
        trials = [self.trial(), self.trial(spike=other)]
        (status, _), record = score(self.plan, self.obs(trials))
        self.assertEqual(status, checker.FAIL)
        self.assertEqual(record["outcome"], "ExecFailed:EACCES")


# ======================================================= N-5 durable records
def barrier_cases():
    ctx = driver.TrialContext(build="/x", work="/x", preflight={}, freeze={},
                              sanitiser=evidence.Sanitiser())
    out = set()
    for plan in driver._PLAN_LIST:
        state = driver.PARENT_STATES[plan.parent](ctx, plan)
        if set(state) & driver.BARRIER_PROVEN_STATE_KEYS:
            out.add(plan.case)
        if path_produces(plan, "post_pin_evidence") and not state:
            out.add(plan.case)
    return sorted(out)


class DurablePosingEvidence(unittest.TestCase):
    FORBIDDEN = {"st_dev", "st_ino", "pid", "fd", "path", "exec_path",
                 "work_dir", "writer_inode", "pinned_inode"}

    def test_the_barrier_set_is_derived_from_the_plans(self):
        self.assertEqual(barrier_cases(), [
            "E2", "E3", "E4", "E5", "E6", "E6b", "E6c", "E6d", "F2", "F3",
            "F5", "F6", "F7", "M5", "R4", "S2", "S7", "T6", "V1", "X1"])

    def test_every_forced_state_fact_survives_into_the_durable_record(self):
        fact = {"action": "anything", "landed": True, "detail": "d",
                "st_ino": 99, "st_dev": 1, "path": "/home/secret/x"}
        for case in barrier_cases():
            plan = driver.CASE_PLANS[case]
            record = driver.evaluate(plan, obs_for(plan, post_pin_evidence=[fact]))
            forced = record["posing_evidence"]["forced_state"]
            self.assertEqual(forced[0]["action"], "anything", case)
            self.assertTrue(set(forced[0]).isdisjoint(self.FORBIDDEN), case)

    def test_evidence_survives_when_the_case_is_not_posed(self):
        plan = driver.CASE_PLANS["E2"]
        fact = {"action": "rename_over", "landed": False, "detail": "failed"}
        record = driver.evaluate(plan, obs_for(plan, not_posed="post-pin control",
                                               post_pin_evidence=[fact]))
        self.assertIn("not_posed", record)
        self.assertEqual(record["posing_evidence"]["forced_state"][0]["landed"],
                         False)
        self.assertIn("build_identity_binding", record["posing_evidence"])

    def test_every_fact_field_the_driver_emits_is_public_and_normalised(self):
        emitted = set()
        for node in ast.walk(TREE):
            if isinstance(node, ast.Call) and getattr(node.func, "id", "") == "_landed":
                emitted |= {kw.arg for kw in node.keywords if kw.arg}
        self.assertEqual(emitted - driver.FACT_PUBLIC_KEYS, set())
        self.assertTrue(emitted.isdisjoint(self.FORBIDDEN), emitted & self.FORBIDDEN)

    def test_cleanup_problems_carry_no_private_path(self):
        problem = driver._problem("writer close failed",
                                  OSError(9, "Bad file descriptor", "/home/u/x"))
        self.assertEqual(problem, "writer close failed: EBADF")
        record = driver.evaluate(driver.CASE_PLANS["E5"], obs_for(
            driver.CASE_PLANS["E5"], cleanup_problems=[problem]))
        self.assertEqual(record["posing_evidence"]["cleanup_problems"], [problem])

    def test_the_proof_is_readable_from_the_journal_after_the_process(self):
        tmp = pathlib.Path(tempfile.mkdtemp(prefix="delta-journal-"))
        try:
            plan = driver.CASE_PLANS["E6d"]
            fact = {"action": "fchmod", "landed": True, "detail": "d",
                    "mode_before": 0o755, "mode_after": 0}
            record = driver.evaluate(plan, obs_for(plan, post_pin_evidence=[fact]))
            path = tmp / "journal.jsonl"
            with journal.Journal(path, evidence.Sanitiser()) as j:
                j.case_entered("E6d", 9)
                j.case_completed("E6d", "FAIL", "r", record)
            state = journal.replay(journal.read(path)[0])
            kept = state["completed"]["E6d"]["record"]["posing_evidence"]
            self.assertEqual(kept["forced_state"][0]["mode_before"], 0o755)
            self.assertEqual(kept["build_identity_binding"]["bound"], True)
        finally:
            shutil.rmtree(tmp, ignore_errors=True)


# ==================================================== E6c and the manifest
class E6cMapping(unittest.TestCase):
    def test_the_mapping_holds_no_descriptor_by_construction(self):
        body = source_of("_pp_mmap_write")
        self.assertNotIn("import mmap", SOURCE)
        calls = [n for n in ast.walk(TREE) if isinstance(n, ast.Call)
                 and ast.unparse(n.func) == "mmap.mmap"]
        self.assertEqual(calls, [])
        self.assertIn("descriptor_closed=closed", body)
        self.assertNotIn("descriptor_closed=True", body)

    def test_e6c_mutates_the_marker_region_not_the_elf_header(self):
        self.assertIn("find_marker_region", source_of(
            "_setup_shared_writable_mapping"))


class ManifestAgreesWithTheCorrection(unittest.TestCase):
    def setUp(self):
        import json
        self.manifest = json.loads((EXP / "SOURCE-HASHES.json").read_text(
            encoding="utf-8"))

    def test_the_barrier_set_in_the_manifest_is_the_derived_one(self):
        self.assertEqual(self.manifest["post_pin_barrier"]["barrier_cases"],
                         barrier_cases())

    def test_the_t2_r1_set_is_kept_as_history_not_as_the_barrier_set(self):
        self.assertEqual(
            self.manifest["post_pin_barrier"]["t2_r1_producer_only_set"],
            ["E2", "E3", "E4", "E5", "E6", "E6b", "E6c", "E6d", "X1", "X8"])
        self.assertNotIn("affected_cases", self.manifest["post_pin_barrier"])

    def test_the_stale_t2_r3_wording_is_gone(self):
        text = (EXP / "SOURCE-HASHES.json").read_text(encoding="utf-8")
        self.assertNotIn("the two setups", text)


if __name__ == "__main__":
    unittest.main()
