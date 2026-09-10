"""LAUNCH-EXEC-01 publication sanitiser and evidence serialization (P-14).

The independent pre-trial review recorded **P-14**: ``DECLARED_ENV_NAMES``,
``sanitise_env_name`` and ``sanitise_path`` existed and nothing called them,
because the driver was absent. They were also not sufficient on their own --
reviewed here rather than assumed:

* ``sanitise_path`` replaced the work and home directories and nothing else. It
  left every other absolute path, the account name, and every temporary
  directory intact, and its two replacements were order-dependent when one root
  was a prefix of the other.
* ``sanitise_env_name`` sanitised NAMES. Nothing ever sanitised VALUES, and
  ``helper_report.c`` reports the **complete** environment by design -- that is
  what makes a V1 leak detectable at all. On a hosted runner the inherited
  environment contains ``ACTIONS_RUNTIME_TOKEN`` and
  ``ACTIONS_ID_TOKEN_REQUEST_TOKEN``.

So the rule this module enforces is: **an environment value is never
reproduced**, in any case, declared or not. A leak stays detectable through the
name and the value's length and digest prefix, which is enough to prove a leak
occurred without republishing what leaked.

Serialization is **deterministic**: sorted keys, fixed separators, no wall-clock
timestamp and no hostname. The same record always serialises to the same bytes,
so an evidence document can be diffed and re-verified.

Nothing here executes anything. Importing this module poses no case.

NOT_RUN: no trial has been executed and no case has been posed.
"""
import hashlib
import json
import re

# Names the report may reproduce verbatim. Everything else is reported as
# <UNDECLARED:xxxxxxxx>. This is about NAMES only -- see the module docstring for
# why no value is ever reproduced regardless of its name.
DECLARED_ENV_NAMES = frozenset({
    "HELM_LEAK_CANARY", "LD_LIBRARY_PATH", "PATH", "HOME",
})

# Fields that are INTERNAL observation input and never appear in published
# evidence, at any depth, under any encoding.
#
# The launcher now emits the retained capture prefix base64-encoded so the
# helper's report can reach the driver at all (PRE-D7-B1). Those bytes are
# whatever the executed image wrote: on a hosted runner that can include
# environment values, absolute paths and credentials. Base64 is an encoding, not
# a protection -- a secret in base64 is still a secret -- and the generic
# redaction rules below cannot see inside an encoded blob. So these keys are
# withheld wholesale rather than scanned, which is the only rule that stays
# correct when the encoding changes.
INTERNAL_ONLY_KEYS = frozenset({
    "capture_prefix_base64",
    "capture_prefix_hex",
    "capture_prefix",
    "raw_capture",
    "decoded_capture",
    # M3-T. The raw acquisition facts carry the direct child's pid, the
    # descriptor numbers and the pidfd_open target pids. They are
    # experiment-local identifiers used for correlation, never part of a
    # receipt's identity, so only observations.normalise_acquisition() output
    # is publishable -- under the DIFFERENT key acquisition_normalised.
    "acquisition",
    "lifecycle_uses",
    "pidfd_open_calls",
    # The parsed child window carries the direct child's raw pid for
    # correlation. child_syscalls and stage_sequence are legitimate published
    # evidence; the pid is an experiment-local identifier and is not.
    "child_pid",
    # The tracer's own text. Host paths, unrelated processes and environment
    # values pass through a syscall record; its SHA-256 is publishable, it is
    # not.
    "raw_trace",
    "trace_text",
    "strace_output",
})

WITHHELD = "<WITHHELD:internal-observation-input>"

# Absolute paths that are legitimate experiment evidence and must survive
# sanitisation: the loader E7 requires, the procfs the helper reads, the tracer
# preflight records, /dev/fd/N which is the whole of X2c's expectation. These are
# properties of the platform, not of the account running the trial.
SYSTEM_PATH_PREFIXES = (
    "/proc/", "/sys/", "/dev/", "/usr/", "/bin/", "/sbin/", "/lib/", "/lib64/",
    "/etc/os-release", "/etc/ld.so", "/run/systemd/",
)

# Recognisable credential shapes, redacted whole. Cheap, and it costs nothing to
# be certain: no test in this repository needs a real secret to exercise them.
_CREDENTIAL_PATTERNS = (
    re.compile(r"gh[pousr]_[A-Za-z0-9]{16,}"),
    re.compile(r"github_pat_[A-Za-z0-9_]{20,}"),
    re.compile(r"xox[baprs]-[A-Za-z0-9-]{10,}"),
    re.compile(r"AKIA[0-9A-Z]{16}"),
    re.compile(r"eyJ[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]{4,}"),
    re.compile(r"(?i)\bBearer\s+[A-Za-z0-9._~+/=-]{12,}"),
)

# A SHA-256 or SHA-1 digest is evidence this experiment exists to publish, so the
# generic high-entropy rule must not eat it.
_DIGEST_RE = re.compile(r"\A[0-9a-f]{40}\Z|\A[0-9a-f]{64}\Z")

# A generic opaque run: long, no spaces, mixed alphabet. Deliberately conservative
# -- it fires only on tokens at least this long, and never on a bare digest.
_OPAQUE_RE = re.compile(r"[A-Za-z0-9_\-+/=]{28,}")

# Absolute paths in either platform's spelling. The trial runs on Linux; the
# Windows form is covered because a driver defect could copy a developer path
# into a record, and an evidence document is published either way.
#
# The lookbehind matters: the root replacements run FIRST, so by the time this
# fires a path may already read "<WORK>/helper_report". Without it the residual
# "/helper_report" would be swallowed as an unknown absolute path and the label
# would lose the very detail it exists to keep.
_POSIX_PATH_RE = re.compile(r"(?<![>\w])/(?:[A-Za-z0-9._+-]+/)*[A-Za-z0-9._+-]*")
_WINDOWS_PATH_RE = re.compile(r"(?<![>\w])[A-Za-z]:\\(?:[^\\/:*?\"<>|\r\n]+\\?)*")

# Temporary-directory roots that carry no experiment meaning and often carry an
# account name.
_TEMP_ROOTS = ("/tmp/", "/var/tmp/", "/private/var/folders/")


def _digest8(text):
    return hashlib.sha256(text.encode("utf-8", "surrogateescape")).hexdigest()[:8]


class Sanitiser:
    """Deterministic redaction for everything published from a trial.

    Constructed once per trial from the roots preflight recorded, then applied to
    every record. It holds no state that changes between calls, so the same input
    always produces the same output.
    """

    def __init__(self, work=None, home=None, user=None, build=None,
                 extra_roots=()):
        # Longest first, so /home/u/work is replaced before /home/u and a nested
        # root can never be half-substituted. This is the order dependence the
        # original two-line sanitise_path had.
        roots = []
        if build:
            roots.append((str(build), "<BUILD>"))
        if work:
            roots.append((str(work), "<WORK>"))
        if home:
            roots.append((str(home), "<HOME>"))
        for root in extra_roots:
            roots.append((str(root), "<ROOT>"))
        self._roots = sorted((r for r in roots if r[0]),
                             key=lambda pair: len(pair[0]), reverse=True)
        self._user = str(user) if user else None

    # -------------------------------------------------------------- primitives
    def text(self, value):
        """Redact one string. Non-strings are returned unchanged."""
        if not isinstance(value, str):
            return value
        out = value
        for root, label in self._roots:
            out = out.replace(root, label)
        if self._user:
            # After the roots, so <HOME> has already absorbed /home/<user>.
            out = out.replace(self._user, "<USER>")
        for pattern in _CREDENTIAL_PATTERNS:
            out = pattern.sub("<CREDENTIAL>", out)
        out = _WINDOWS_PATH_RE.sub(self._path_token, out)
        out = _POSIX_PATH_RE.sub(self._path_token, out)
        out = _OPAQUE_RE.sub(self._opaque_token, out)
        return out

    def _path_token(self, match):
        path = match.group(0)
        if path.startswith(SYSTEM_PATH_PREFIXES):
            return path
        if path in ("/", "/proc", "/sys", "/dev", "/tmp"):
            return path
        for root in _TEMP_ROOTS:
            if path.startswith(root):
                return "<TMPPATH>"
        if path.startswith("/") or _WINDOWS_PATH_RE.fullmatch(path):
            return "<ABSPATH>"
        return path

    def _opaque_token(self, match):
        token = match.group(0)
        if _DIGEST_RE.match(token):
            return token
        if token.startswith(("<UNDECLARED:", "<VALUE:")):
            return token
        return "<OPAQUE:" + _digest8(token) + ">"

    # ------------------------------------------------------------ environment
    def env_name(self, name):
        """A declared name survives; anything else becomes its digest prefix."""
        if not isinstance(name, str):
            return name
        if name in DECLARED_ENV_NAMES:
            return name
        return "<UNDECLARED:" + _digest8(name) + ">"

    def env_entry(self, entry):
        """One ``NAME=VALUE`` line from the child's environ.

        The NAME is sanitised and the VALUE is **never** reproduced -- only its
        length and digest prefix, which is what makes a leak provable without
        republishing it. Under D-10 the only correct observation is that there
        are no entries at all, so anything reaching here is already a finding.
        """
        if not isinstance(entry, str):
            return entry
        name, sep, value = entry.partition("=")
        if not sep:
            return "<MALFORMED_ENV:" + _digest8(entry) + ">"
        return (self.env_name(name) + "=<VALUE:len=" +
                str(len(value.encode("utf-8", "surrogateescape"))) + "," +
                _digest8(value) + ">")

    # ---------------------------------------------------------------- records
    def record(self, value, _key=None):
        """Recursively sanitise a record. Structure is preserved exactly.

        Keys are sanitised too: a dict keyed by pathname would otherwise leak
        through its keys. ``environ`` arrays get the value-suppressing treatment
        wherever they appear, at any depth.
        """
        if isinstance(value, dict):
            out = {}
            for k, v in value.items():
                if isinstance(k, str) and k in INTERNAL_ONLY_KEYS:
                    # Withheld by KEY, before any content is examined. Scanning
                    # an encoded blob for secrets is a game the scanner loses.
                    out[k] = WITHHELD
                    continue
                out[self.text(k) if isinstance(k, str) else k] = self.record(
                    v, _key=k)
            return out
        if isinstance(value, list):
            if _key == "environ":
                return [self.env_entry(item) for item in value]
            return [self.record(item, _key=_key) for item in value]
        if isinstance(value, str):
            if _key in ("env_name", "name") and value in DECLARED_ENV_NAMES:
                return value
            return self.text(value)
        return value


# ------------------------------------------------------------- serialization
def serialise(document):
    """Deterministic JSON for an evidence document.

    Sorted keys and fixed separators, so the same record always produces the same
    bytes. No timestamp and no hostname are added here or anywhere else: D-8
    keeps duration and time out of the receipt, and a published document that
    differs run to run cannot be diffed against its own re-verification.
    """
    return json.dumps(document, indent=2, sort_keys=True,
                      separators=(",", ": "), ensure_ascii=False,
                      default=str) + "\n"


def receipt_view(spike_receipt):
    """The receipt fields only, with the fields D-8 and the freeze keep out.

    ``launcher_spike.c`` already names its two excluded fields
    ``retained_prefix_not_in_receipt`` and ``elapsed_ms_not_in_receipt``; this
    function is the parent-side counterpart, so a published receipt cannot
    quietly regain a duration or the in-memory truncation flag. Both facts stay
    in the evidence document -- outside the receipt, where the spike put them.
    """
    if not isinstance(spike_receipt, dict):
        return None
    out = {}
    for key, value in spike_receipt.items():
        if key.endswith("_not_in_receipt"):
            continue
        if isinstance(value, dict):
            # Stream blocks now carry the retained capture prefix. It is
            # observation input, not receipt content, and it is dropped here
            # rather than redacted -- there is nothing in it a receipt needs.
            value = {k: v for k, v in value.items()
                     if k not in INTERNAL_ONLY_KEYS}
        out[key] = value
    return out


def evidence_document(trial):
    """Assemble the whole published document from a completed trial.

    Deliberately plain: the interesting decisions are all upstream, in the
    per-case records and in the checker. This function must add no judgement of
    its own, so it computes nothing and only arranges what it is given.
    """
    return {
        "experiment": "LAUNCH-EXEC-01",
        "status": trial["status"],
        "aggregate": trial["aggregate"],
        "detail": trial["detail"],
        "counts": trial["counts"],
        "preflight": trial["preflight"],
        "membership": trial["membership"],
        "cases": trial["cases"],
        "build": trial.get("build"),
        "freeze": trial["freeze"],
        "uncontrolled": trial.get("uncontrolled", []),
        "notes": trial.get("notes", []),
    }
