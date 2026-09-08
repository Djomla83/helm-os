"""OBS-FS-01 frozen case manifest: recipes, schedules, seeds, expected sets.

Authority: docs/implementation/HELM-OBSERVE-INDEPENDENT-REVIEW.md as stored at
63ac6796296894945dff520423cb1a93351e8524 (Amendment 1).

This file declares expectations ONLY. It never inspects spike output, and the
spike never reads it. Expected regular-file digests are derived from the byte
recipes here by oracles.py using Python hashlib, independently of the spike.
"""

AMENDMENT = "63ac6796296894945dff520423cb1a93351e8524"
ORIGINAL_FREEZE = "b4ed2e108134eb58a2561403f5da3509aed955ee"
PREFLIGHT_HALT = "90e025890a32b36ccc55a4dc223bbb85046ba152"

FILE_CEILING = 512 * 1024 * 1024
TOTAL_CEILING = 1024 * 1024 * 1024
READ_BUF = 64 * 1024

# ---------------------------------------------------------------- byte recipes
# Each recipe is a deterministic function of its declared parameters only.
RECIPES = {
    "plain":      ("bytes", b"obs-fs-01 regular fixture\n"),
    "empty":      ("bytes", b""),
    "wrongsame":  ("bytes", b"different bytes, same name\n"),
    "canary":     ("bytes", b"hardlink canary payload\n"),
    "twofile_a":  ("bytes", b"two-file environment A\n"),
    "twofile_b":  ("bytes", b"two-file environment B\n"),
    "private":    ("bytes", b"PRIVATE-MARKER-must-never-be-read\n"),
    "unlisted":   ("bytes", b"unlisted body must never be read\n"),
    "permleaf":   ("bytes", b"permission denied leaf\n"),
    "permchild":  ("bytes", b"under unsearchable parent\n"),
    # Race fixtures are exact multiples of the read buffer so the interleaving
    # points are enumerable and the allowed digest set is finite.
    "race_old":   ("pattern", ("A", 3 * READ_BUF)),
    "race_new":   ("pattern", ("B", 3 * READ_BUF)),
    "grow_base":  ("pattern", ("G", 2 * READ_BUF)),
    "trunc_base": ("pattern", ("T", 3 * READ_BUF)),
    "alias_old":  ("pattern", ("H", 2 * READ_BUF)),
    "alias_new":  ("pattern", ("K", 2 * READ_BUF)),
    "budget_a":   ("pattern", ("P", 1024)),
    "budget_b":   ("pattern", ("Q", 1024)),
    # Sparse: 1 MiB apparent, one written byte at the end. Digest must include
    # the zero bytes of the hole.
    "sparse":     ("sparse", (1024 * 1024, b"Z")),
}

# Over-limit fixture: sparse so it costs no allocation, apparent size above the
# per-file ceiling. Must never be data-opened.
OVERLIMIT_APPARENT = FILE_CEILING + 4096

# ------------------------------------------------------------------ case table
# kind: static | direct | race | admission
# safe: allowed result outcomes (the R set)
# trace: named trace invariants enforced by checker.py (the T set)
STATIC_CASES = [
    dict(case="regular_nonempty", target="plain", op="file", recipe="plain",
         safe=["observed_file"], digest="plain",
         trace=["one_pin", "one_reopen", "reads_match_bytes", "no_escape"]),
    dict(case="regular_empty", target="empty", op="file", recipe="empty",
         safe=["observed_file"], digest="empty",
         trace=["one_pin", "one_reopen", "no_escape"]),
    dict(case="missing_leaf", target="missingleaf", op="file",
         safe=["absent"], trace=["no_reopen", "single_resolve", "no_escape"]),
    dict(case="missing_parent", target="missingparent", op="file",
         safe=["absent"], trace=["no_reopen", "single_resolve", "no_escape"]),
    dict(case="wrong_bytes_same_name", target="wrongsame", op="file",
         recipe="wrongsame", safe=["observed_file"], digest="wrongsame",
         trace=["one_pin", "one_reopen", "no_escape"]),
    # Amendment 1: trailing symlinks pin the link and are rejected at
    # classification; a descriptor to the link is permitted.
    dict(case="trailing_symlink_internal", target="linkint", op="file",
         safe=["symlink_forbidden"], stage=["classify"],
         trace=["no_reopen", "no_reads", "no_escape", "no_destination_open"]),
    dict(case="trailing_symlink_escaping", target="linkesc", op="file",
         safe=["symlink_forbidden"], stage=["classify"],
         trace=["no_reopen", "no_reads", "no_escape", "no_destination_open"]),
    dict(case="trailing_symlink_dangling", target="linkdang", op="file",
         safe=["symlink_forbidden"], stage=["classify"],
         trace=["no_reopen", "no_reads", "no_escape", "no_destination_open"]),
    # Non-final symlink: constrained resolution rejection, ELOOP class.
    dict(case="nonfinal_symlink", target="linkdir.child", op="file",
         safe=["symlink_forbidden"], stage=["resolve"], errno=["ELOOP"],
         trace=["no_reopen", "no_reads", "no_escape", "no_descriptor"]),
    dict(case="fifo", target="fifo", op="file",
         safe=["special_file"], stage=["classify"], kind=["fifo"],
         trace=["no_reopen", "no_reads", "no_connect", "no_escape", "metadata_only"]),
    dict(case="unix_socket", target="sock", op="file",
         safe=["special_file"], stage=["classify"], kind=["socket"],
         trace=["no_reopen", "no_reads", "no_connect", "no_escape", "metadata_only"]),
    dict(case="directory_in_file_position", target="adir", op="file",
         safe=["wrong_kind"], stage=["classify"], kind=["directory"],
         trace=["no_reopen", "no_reads", "no_enumeration", "no_escape"]),
    dict(case="regular_in_parent_position", target="plain.child", op="file",
         safe=["wrong_kind"], stage=["resolve"], errno=["ENOTDIR"],
         trace=["no_reopen", "no_reads", "no_escape"]),
    dict(case="permission_denied_leaf", target="permleaf", op="file",
         safe=["permission_denied"],
         trace=["no_reads", "no_escape"]),
    dict(case="permission_denied_parent", target="permdir.child", op="file",
         safe=["permission_denied"], stage=["resolve"], errno=["EACCES"],
         trace=["no_reopen", "no_reads", "no_escape"]),
    dict(case="sparse_file", target="sparse", op="file", recipe="sparse",
         safe=["observed_file"], digest="sparse",
         trace=["one_pin", "one_reopen", "no_seek_hole", "no_escape"]),
    dict(case="metadata_over_limit", target="overlimit", op="file",
         safe=["file_limit"], stage=["budget"],
         trace=["no_reopen", "no_reads", "no_escape"]),
    dict(case="hardlink", target="hardlink", op="file", recipe="canary",
         safe=["observed_file"], digest="canary", min_nlink=2,
         trace=["one_pin", "one_reopen", "no_escape", "no_alias_enumeration"]),
    dict(case="directory_metadata", target="adir", op="dir",
         safe=["observed_directory"], kind=["directory"],
         trace=["no_reopen", "no_reads", "no_enumeration", "no_escape"]),
]

# Multi-target batch cases: exercised through one plan file in declared order.
BATCH_CASES = [
    dict(case="two_file_changing_environment",
         plan=[("filea", "file", "twofile_a"), ("fileb", "file", "twofile_b")],
         safe_per_target={"filea": ["observed_file", "changed_during_read"],
                          "fileb": ["observed_file", "changed_during_read"]},
         trace=["no_escape"]),
    dict(case="budget_exhaustion",
         plan=[("budgeta", "file", "budget_a"), ("budgetb", "file", "budget_b")],
         budget=1200,   # admits budget_a (1024+1) but not budget_b
         safe_per_target={"budgeta": ["observed_file"],
                          "budgetb": ["total_limit"]},
         trace=["no_escape", "every_target_has_outcome"]),
    dict(case="private_and_unlisted",
         plan=[("plain", "file", "plain")],
         unlisted=["private", "unlisted"],
         safe_per_target={"plain": ["observed_file"]},
         trace=["no_escape", "no_unlisted_open", "no_absolute_path_in_output"]),
]

ADMISSION_CASES = [
    # Amendment 1 section 12 requires "plan validation error" with no target I/O.
    # Any code in the plan-validation family satisfies that; the closed vocabulary
    # rejects a descriptor-shaped directive either on arity or on the directive name.
    dict(case="descriptor_shaped_plan_field",
         plan_text="target good file plain\nroot_fd 7 file\n",
         safe=["unknown_directive", "plan_syntax", "id_grammar", "unknown_operation"],
         trace=["no_io_at_all"]),
    dict(case="fake_procfs_tmpfs",
         proc="FAKE", safe=["procfs_rejected", "procfs_unavailable"],
         trace=["no_target_read"]),
    dict(case="procfs_unavailable",
         proc="MISSING", safe=["procfs_unavailable"],
         trace=["no_target_read", "no_pathname_fallback"]),
    dict(case="procfs_foreign_process",
         proc="FOREIGN", safe=["procfs_rejected"],
         trace=["no_target_read"]),
]

DIRECT_CASES = [
    dict(case="D1_direct_fifo_blocking", target="fifo", nonblock=False,
         safe=["deadline_kill"], trace=["blocking_demonstrated"]),
    dict(case="D2_direct_fifo_nonblock", target="fifo", nonblock=True,
         safe=["direct_data_open_succeeded"], kind=["fifo"],
         trace=["data_open_demonstrated"]),
    dict(case="D3_opath_same_object", target="fifo",
         safe=["special_file"], kind=["fifo"],
         trace=["no_reopen", "no_reads", "metadata_only"]),
]

MOUNT_CASES = [
    dict(case="xdev_fallback_existing_mount", safe=["mount_crossing"],
         errno=["EXDEV"], trace=["no_descriptor", "no_reads"]),
    dict(case="xdev_control_without_flag", safe=["observed_directory"],
         note="control arm proving RESOLVE_NO_XDEV caused the rejection"),
    dict(case="bind_mount_descendant_namespace", conditional=True,
         safe=["mount_crossing", "BLOCKED"],
         residual="RESOLVE_NO_XDEV rejects a bind mount created as a descendant "
                  "of an authorized ext4 root"),
]

# ------------------------------------------------------------------ race cases
# Each schedule fires once, deterministically, at the named syscall stop.
# seed is recorded for provenance; mutations are deterministic, not random.
RACE_CASES = [
    dict(case="leaf_regular_to_symlink_swap", seed=110001,
         stop="openat2_target", mutate="swap_leaf_to_symlink",
         safe=["symlink_forbidden", "observed_file"],
         trace=["injection_landed", "no_escape", "no_destination_open"]),
    dict(case="parent_to_symlink_swap", seed=110002,
         stop="openat2_target", mutate="swap_parent_to_symlink",
         safe=["symlink_forbidden", "observed_file", "absent"],
         trace=["injection_landed", "no_escape", "no_destination_open"]),
    dict(case="replacement_before_pin", seed=110003,
         stop="openat2_target", mutate="replace_regular",
         safe=["observed_file", "absent"], digest_set=["race_old", "race_new"],
         trace=["injection_landed", "no_escape"]),
    dict(case="replacement_after_pin", seed=110004,
         stop="reopen", mutate="replace_regular",
         safe=["observed_file", "changed_during_read", "io_failure"],
         digest_set=["race_old", "race_new", "race_mixed_1"],
         trace=["injection_landed", "no_escape"]),
    dict(case="directory_replacement_after_pin", seed=110005,
         stop="openat2_target", mutate="replace_directory",
         safe=["observed_file", "absent", "symlink_forbidden", "changed_during_read"],
         trace=["injection_landed", "no_escape"]),
    dict(case="growing_file", seed=110006,
         stop="first_read", mutate="grow_file",
         safe=["changed_during_read", "io_failure", "observed_file"],
         trace=["injection_landed", "byte_ceiling_respected", "no_escape"]),
    dict(case="truncating_file", seed=110007,
         stop="first_read", mutate="truncate_file",
         safe=["changed_during_read", "io_failure"],
         trace=["injection_landed", "no_complete_digest_after_short_read", "no_escape"]),
    dict(case="inplace_overwrite_deterministic", seed=110008,
         stop="first_read", mutate="overwrite_tail_same_length",
         safe=["changed_during_read", "observed_file"],
         digest_set=["race_old", "race_new", "race_mixed_1"],
         trace=["injection_landed", "no_escape"]),
    dict(case="inplace_overwrite_undetectable", seed=110009,
         stop="first_read", mutate="overwrite_mmap_same_length",
         safe=["observed_file", "changed_during_read"],
         digest_set=["race_old", "race_new", "race_mixed_1"],
         undetectable_ok=True,
         trace=["injection_landed", "no_escape", "no_snapshot_claim"]),
    dict(case="hardlink_alias_mutation", seed=110010,
         stop="first_read", mutate="mutate_alias",
         safe=["observed_file", "changed_during_read"],
         digest_set=["alias_old", "alias_new", "alias_mixed_1"],
         trace=["injection_landed", "no_escape", "no_alias_enumeration"]),
]

MANDATORY_CASES = (
    [c["case"] for c in STATIC_CASES]
    + [c["case"] for c in BATCH_CASES]
    + [c["case"] for c in ADMISSION_CASES]
    + [c["case"] for c in DIRECT_CASES]
    + [c["case"] for c in RACE_CASES]
    + ["xdev_fallback_existing_mount", "xdev_control_without_flag"]
)
CONDITIONAL_CASES = ["bind_mount_descendant_namespace"]
OPTIONAL_CASES = ["dev_null_zero_classifier"]

MAX_STRESS_REPETITIONS = 20
RACE_REPETITIONS = 1        # one forced run per deterministic schedule
