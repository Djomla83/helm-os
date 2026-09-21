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

Vocabulary acquisition is **fail-closed** (M-1). The driver's fixed symbolic
registries are read through an accessor it defines last, and if they cannot be
read completely -- import failure, a partially initialised module, a missing,
mistyped or empty registry -- sanitisation and serialization raise
``VocabularyUnavailable`` and publish nothing. Nothing incomplete is ever cached,
so a later call recovers once the driver is initialised.

Nothing here executes anything. Importing this module poses no case.

NOT_RUN: no trial has been executed and no case has been posed.
"""
import hashlib
import json
import pathlib
import re
import sys

import checker
import frozen_cases as fc
import observations as ob

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


# --------------------------------------------------- V-5: what is structural
# The sanitiser used to redact the account name with a bare
# ``text.replace(username, "<USER>")``. That is unbounded substring
# replacement, so a short or common account name corrupted unrelated evidence:
# "ci" turned ``specific`` into ``spe<USER>fic``, "run" turned ``truncated``
# into ``t<USER>cated``. It over-redacted rather than leaking, but it damaged
# field names, syscall names and frozen tokens alike.
#
# Sanitisation is now STRUCTURE-AWARE. Redaction happens to VALUES, according
# to what a value is, and never by rewriting identifiers that happen to contain
# a username-shaped substring.

# The frozen vocabulary, DERIVED FROM THE REGISTRIES rather than restated, so a
# symbol added to any of them cannot silently start being redacted here. This is
# finding F-1: `sigterm_blocked_sigpipe_ignored` is a driver PARENT_STATES key,
# 31 characters, reachable in published evidence as T6's plan["parent"], and a
# hand-written allowlist had simply missed it. Adding that one string would have
# fixed the instance and left the class open, so every fixed symbolic registry
# is enumerated instead.
#
# What is NOT vocabulary, and must stay redactable: argv contents, environment
# values, host paths, captured bytes, tracer text and anything high-entropy.
# A4's 4096-byte argument is data and still becomes an <OPAQUE:...> token.

# Receipt vocabulary that launcher_spike.c prints and no Python registry owns.
_SPIKE_RECEIPT_VOCABULARY = frozenset({
    "accepted", "refused", "empty",
    "CompleteAtEof", "WriterRetainedAfterChildExit",
})

# Role names normalise_acquisition() substitutes for raw identifiers.
_NORMALISED_ROLES = frozenset({"DIRECT_CHILD", "DIRECT_CHILD_PIDFD"})


def _manifest_vocabulary():
    words = set()
    for case in fc.CASES:
        words.add(case["case"])
        if case["predict"]:
            words.add(case["predict"])
        words.update(case["safe"] or ())
        words.update(case["gates"])
    words |= set(fc.STAGES)
    words |= set(fc.BLOCK_REASONS)
    words |= set(fc.CHILD_PERMITTED_SYSCALLS)
    words |= set(fc.CHILD_FORBIDDEN_SYSCALLS)
    words |= set(fc.CHILD_TEST_INJECTION_SYSCALLS)
    words |= set(fc.CHILD_INJECTION_MODES)
    words |= set(fc.PARENT_CONTROL_MODES)
    words |= set(fc.DECISIONS) | set(fc.DECISIONS.values())
    words |= set(fc.M3_EVIDENCE_FACTS)
    return words


def _observation_vocabulary():
    words = set(ob.RULES)
    words |= set(ob.SPIKE_DISPOSITIONS)
    words |= set(ob.SPIKE_TIMEOUT_DISPOSITIONS)
    words |= set(ob.SPIKE_REFUSALS)
    words |= set(ob.SIGNAL_NAMES.values())
    words |= set(ob.ERRNO_NAMES.values())
    words |= set(ob.REPORT_STATES)
    words |= {ob.RETURN_OBSERVED_SUCCESS, ob.RETURN_OBSERVED_ERROR,
              ob.RETURN_NOT_OBSERVED, ob.EVIDENCE_DIRECT,
              ob.LIFECYCLE_WAITID, ob.LIFECYCLE_SEND_SIGNAL, ob.LIFECYCLE_POLL,
              ob.EXEC_PRE_EXEC_ERROR, ob.EXEC_DIED_BEFORE_EXEC,
              ob.EXEC_REACHED, ob.EXEC_UNINTERPRETABLE}
    # Trial #2 delta correction: executed-identity and O5 assertions, their
    # results and violation tokens, and the closed set of publishable markers.
    words |= set(ob.ASSERTIONS)
    words |= set(ob.ASSERTION_VIOLATION_TOKENS.values())
    words |= {ob.ASSERTION_HOLDS, ob.ASSERTION_VIOLATED,
              ob.ASSERTION_UNOBSERVABLE}
    words |= set(ob.PUBLISHABLE_MARKERS)
    return words


def _checker_vocabulary():
    return {checker.PASS, checker.FAIL, checker.INVALID, checker.BLOCKED,
            checker.ACCEPTED, checker.REJECTED, checker.INCONCLUSIVE}


def _schema_vocabulary():
    """Field names of the published observation objects.

    Keys are immutable, so these are already safe in key position. They are
    registered anyway because the SCHEMA is fixed symbolic vocabulary of the
    same class, and a future record that carried one as a VALUE would otherwise
    reopen exactly the omission F-1 named.
    """
    words = set()
    normalised = ob.normalise_acquisition({}) or {}
    words |= set(normalised)
    words |= set(normalised.get("trace_integrity") or {})
    words |= {"child_syscalls", "stage_sequence", "integrity_ok", "child_pid",
              "trace_sha256", "acquisition_normalised"}
    return words


_STATIC_VOCABULARY = None
_DRIVER_VOCABULARY = None
_VOCABULARY = None


def _expand(words):
    """Composite tokens are published in their parts too."""
    out = set()
    for word in words:
        if not isinstance(word, str) or not word:
            continue
        out.add(word)
        for part in word.replace(":", " ").split():
            if part:
                out.add(part)
    return frozenset(out)


class VocabularyUnavailable(RuntimeError):
    """The driver's fixed symbolic registries could not be read completely.

    **M-1.** Raised instead of returning an empty set. An incomplete vocabulary
    is not a degraded mode: every unrecognised token at least 28 characters long
    is rewritten into an ``<OPAQUE:...>`` digest, so a missing registry does not
    produce a visible failure, it produces published evidence that is quietly
    wrong. Halting is the only honest answer, and because nothing is cached on
    the way out, a later call made once the driver is fully initialised recovers
    the complete set.
    """


# Every registry the driver's snapshot must supply. A snapshot missing one of
# these, or carrying one that is empty or is not a sequence of names, is an
# incomplete snapshot and is refused -- that is precisely the partially
# initialised state this check exists to catch.
_REQUIRED_DRIVER_REGISTRIES = ("SETUPS", "PARENT_STATES", "POSED_CHECKS",
                               "ALL_CHANNELS")


def _driver_registry_snapshot():
    """Read the driver's registries through its finalised accessor, or refuse.

    ``driver`` imports this module, so a module-level import here would be
    circular and the import has to stay lazy. What that creates is the risk of
    importing a module whose body is still running, because Python publishes the
    module object before executing it.

    ``driver.registry_vocabulary`` is defined as the LAST statement of that
    module's body, so asking for it is a positive readiness test rather than a
    guess about whether some global has been filled in yet.
    """
    try:
        import driver
    except Exception as exc:                               # noqa: BLE001
        raise VocabularyUnavailable(
            "the driver module could not be imported: %r" % (exc,))
    accessor = getattr(driver, "registry_vocabulary", None)
    if accessor is None:
        raise VocabularyUnavailable(
            "driver is only partially initialised: registry_vocabulary() is "
            "defined after every registry and is not present yet")
    if not callable(accessor):
        raise VocabularyUnavailable(
            "driver.registry_vocabulary is not callable")
    try:
        snapshot = accessor()
    except Exception as exc:                               # noqa: BLE001
        raise VocabularyUnavailable(
            "driver.registry_vocabulary() failed: %r" % (exc,))
    if not isinstance(snapshot, dict):
        raise VocabularyUnavailable(
            "driver.registry_vocabulary() returned %s, not a mapping"
            % type(snapshot).__name__)
    return snapshot


def _driver_vocabulary():
    """The driver's fixed symbolic registries, validated before being cached.

    The cache rule is the whole of M-1: a failed load is not cached, a partial
    load is not cached, and only a complete validated snapshot is stored, as an
    immutable frozenset. So a partially initialised import cannot poison the
    cache, and the first call after the driver finishes initialising returns the
    complete set.
    """
    global _DRIVER_VOCABULARY
    if _DRIVER_VOCABULARY is not None:
        return _DRIVER_VOCABULARY
    snapshot = _driver_registry_snapshot()
    words = set()
    for name in _REQUIRED_DRIVER_REGISTRIES:
        if name not in snapshot:
            raise VocabularyUnavailable(
                "the driver registry snapshot is missing " + name)
        entries = snapshot[name]
        if not isinstance(entries, (tuple, list, set, frozenset)):
            raise VocabularyUnavailable(
                "driver registry %s is %s, not a sequence of names"
                % (name, type(entries).__name__))
        if not entries:
            raise VocabularyUnavailable(
                "driver registry %s is empty, which is either a partially "
                "initialised import or a broken freeze" % name)
        for entry in entries:
            if not isinstance(entry, str) or not entry:
                raise VocabularyUnavailable(
                    "driver registry %s holds %r, which is not a name"
                    % (name, entry))
            words.add(entry)
    _DRIVER_VOCABULARY = _expand(words)
    return _DRIVER_VOCABULARY


def vocabulary():
    """Every fixed symbolic value that may reach published evidence.

    Raises ``VocabularyUnavailable`` when the driver's registries cannot be read
    completely -- see ``_driver_vocabulary``. The union is cached only once it
    is complete, so a cached value is always the whole vocabulary and never a
    partial one.
    """
    global _STATIC_VOCABULARY, _VOCABULARY
    if _VOCABULARY is not None:
        return _VOCABULARY
    if _STATIC_VOCABULARY is None:
        _STATIC_VOCABULARY = _expand(
            _manifest_vocabulary() | _observation_vocabulary()
            | _checker_vocabulary() | _schema_vocabulary()
            | _SPIKE_RECEIPT_VOCABULARY | _NORMALISED_ROLES)
    _VOCABULARY = _STATIC_VOCABULARY | _driver_vocabulary()
    return _VOCABULARY


def _reset_vocabulary_cache():
    """Drop the cached vocabulary so the next call re-derives it.

    Only tests use this, to register a symbol at runtime and show the CLASS of
    omission F-1 named is covered rather than the two instances it named.
    Nothing on the trial path calls it: the registries are frozen.
    """
    global _DRIVER_VOCABULARY, _VOCABULARY
    _DRIVER_VOCABULARY = None
    _VOCABULARY = None


# Fields whose VALUE is an explicit host identity. The whole value is redacted,
# because that is what the field means -- never an arbitrary substring of some
# other field that happens to contain the same letters.
HOST_IDENTITY_KEYS = frozenset({
    "user", "username", "login", "account", "owner", "hostname", "host",
    "runner_name", "logname",
})

HOST_IDENTITY = "<USER>"

# Fields whose VALUE is a BROAD HOST DESCRIPTOR. P-16.
#
# ``uname -a`` buries the machine's nodename in the middle of an otherwise
# useful string -- "Linux <nodename> <release> #<build> <arch> GNU/Linux" -- and
# there is no reliable way to find a hostname inside free text: it is a short
# arbitrary token, so no length, entropy or shape rule can separate it from the
# kernel build string beside it. The value therefore goes WHOLE.
#
# The first line of defence is that the harness no longer collects it: kernel
# name, release and architecture are recorded as three narrow fields, none of
# which can carry a nodename. This set is the second line, so a field of that
# shape reappearing cannot quietly publish the host's name.
HOST_DESCRIPTOR_KEYS = frozenset({
    "uname", "uname_all", "uname_a", "nodename", "node", "domainname",
    "fqdn", "hostname_fqdn",
})

HOST_DESCRIPTOR = "<WITHHELD:host-descriptor>"

_PATH_SPLIT = re.compile(r"([\\/])")


def _digest8(text):
    return hashlib.sha256(text.encode("utf-8", "surrogateescape")).hexdigest()[:8]


class Sanitiser:
    """Deterministic, structure-aware redaction for everything published.

    Constructed once per trial from the roots preflight recorded, then applied
    to every record. It holds no state that changes between calls, and no rule
    depends on dictionary iteration order or on the order substrings happen to
    appear, so the same input always serialises to the same bytes.
    """

    def __init__(self, work=None, home=None, user=None, build=None,
                 extra_roots=()):
        # Longest first, so /home/u/work is replaced before /home/u and a
        # nested root can never be half-substituted.
        roots = []
        if build:
            roots.append((str(build), "<BUILD>"))
        if work:
            roots.append((str(work), "<WORK>"))
        if home:
            roots.append((str(home), "<HOME>"))
        for root in extra_roots:
            roots.append((str(root), "<ROOT>"))
        ordered = sorted((r for r in roots if r[0]),
                         key=lambda pair: len(pair[0]), reverse=True)
        # A root only matches at a PATH BOUNDARY. Without this, a short root
        # such as "/w" would rewrite "/work" and "/warm" alike -- the same
        # substring defect V-5 names, one level up.
        self._roots = [(re.compile(re.escape(root) + r"(?![A-Za-z0-9._+-])"),
                        label) for root, label in ordered]
        self._user = str(user) if user else None

    # -------------------------------------------------------------- primitives
    def text(self, value):
        """Redact one free-text VALUE. Non-strings are returned unchanged.

        The account name is deliberately NOT replaced here. A username is only
        private as a path component or as an explicit host-identity value, and
        both are handled where that structure is known. Redacting it in
        arbitrary text is what corrupted ``truncated`` and ``specific``.
        """
        if not isinstance(value, str):
            return value
        # M-1. Resolve the vocabulary BEFORE any redaction. Only _opaque_token
        # consults it, and only for runs of at least 28 characters, so a value
        # that happens to contain no long token would otherwise be published
        # having never established that the vocabulary was available at all.
        # Failing closed here makes the halt depend on the driver's state rather
        # than on which data the case happened to produce.
        vocabulary()
        out = value
        for pattern, label in self._roots:
            out = pattern.sub(label, out)
        for pattern in _CREDENTIAL_PATTERNS:
            out = pattern.sub("<CREDENTIAL>", out)
        out = _WINDOWS_PATH_RE.sub(self._path_token, out)
        out = _POSIX_PATH_RE.sub(self._path_token, out)
        out = _OPAQUE_RE.sub(self._opaque_token, out)
        return out

    def host_identity(self, value):
        """A field that IS a host identity: the whole value goes."""
        if not isinstance(value, str) or not value:
            return value
        return HOST_IDENTITY

    def _redact_user_components(self, path):
        """Replace path COMPONENTS equal to the account name, nothing less.

        ``/opt/runtime`` with user "run" is untouched, because ``runtime`` is
        not ``run``.
        """
        if not self._user:
            return path
        return "".join(HOST_IDENTITY if part == self._user else part
                       for part in _PATH_SPLIT.split(path))

    def _path_token(self, match):
        path = match.group(0)
        if path.startswith(SYSTEM_PATH_PREFIXES):
            return self._redact_user_components(path)
        if path in ("/", "/proc", "/sys", "/dev", "/tmp"):
            return path
        for root in _TEMP_ROOTS:
            if path.startswith(root):
                return "<TMPPATH>"
        if path.startswith("/") or _WINDOWS_PATH_RE.fullmatch(path):
            return "<ABSPATH>"
        return self._redact_user_components(path)

    def _opaque_token(self, match):
        token = match.group(0)
        if _DIGEST_RE.match(token):
            return token
        if token in vocabulary():
            # Frozen vocabulary is published evidence, not an opaque blob.
            # WriterRetainedAfterChildExit is exactly 28 characters and used to
            # be redacted by the generic high-entropy rule.
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

        **V-5: KEYS ARE IMMUTABLE.** They are structural identifiers -- field
        names, case ids, stage names -- not data, and rewriting them is how
        ``stage_sequence`` became ``stage_seq<USER>ence``. The only key-level
        operation is the INTERNAL_ONLY withholding below, which replaces a
        VALUE and leaves its key byte-exact so the reader can see what was
        withheld.

        ``environ`` arrays get the value-suppressing treatment wherever they
        appear, at any depth, and a field that IS a host identity is redacted
        whole rather than by substring.
        """
        vocabulary()      # M-1: halt before emitting anything, not part-way
        if isinstance(value, dict):
            out = {}
            for k, v in value.items():
                if isinstance(k, str) and k in INTERNAL_ONLY_KEYS:
                    # Withheld by KEY, before any content is examined. Scanning
                    # an encoded blob for secrets is a game the scanner loses.
                    out[k] = WITHHELD
                    continue
                if isinstance(k, str) and k in HOST_DESCRIPTOR_KEYS:
                    # P-16. Withheld by KEY, whole, before the content is read.
                    # A hostname cannot be found reliably inside free text, so
                    # the field goes and its key stays byte-exact.
                    out[k] = HOST_DESCRIPTOR if v else v
                    continue
                if isinstance(k, str) and k in HOST_IDENTITY_KEYS:
                    out[k] = self.host_identity(v) if isinstance(v, str) \
                        else self.record(v, _key=k)
                    continue
                out[k] = self.record(v, _key=k)
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

    M-1: this is the publication boundary, so it refuses to emit bytes while the
    fixed vocabulary is unavailable, even if a caller assembled the document by
    some route that did not go through ``Sanitiser.record``.
    """
    vocabulary()
    return json.dumps(document, indent=2, sort_keys=True,
                      separators=(",", ": "), ensure_ascii=False,
                      default=str) + "\n"


def public_sanitiser(build=None, work=None, home=None, user=None):
    """The redaction roots for anything this process publishes.

    Derived from the running account rather than passed from each call site, so
    a diagnostic path cannot publish with weaker roots than the trial does
    simply by forgetting an argument. Every root is optional: a missing home
    directory leaves absolute paths to the generic ``<ABSPATH>`` rule rather
    than failing.
    """
    if home is None:
        try:
            home = str(pathlib.Path.home())
        except Exception:                                  # noqa: BLE001
            home = None
    if build is not None:
        try:
            build = str(pathlib.Path(build).resolve())
        except Exception:                                  # noqa: BLE001
            build = str(build)
    if work is None:
        work = build
    if user is None and home:
        user = pathlib.PurePath(home).name or None
    return Sanitiser(work=work, home=home, build=build, user=user)


def publish(document, sanitiser=None, stream=None):
    """THE publication boundary. Everything this experiment emits goes here.

    **P-16.** Sanitisation used to happen at each return site, which meant every
    new return site had to remember -- and ``HALT_PREFLIGHT`` did not. A failed
    mandatory preflight is not more trustworthy than a completed trial: it
    carries the same host-derived environment inventory, and it was published
    raw. Sanitising HERE, once, removes the thing that has to be remembered.

    Returns the serialised text as well as writing it, so a test can assert on
    what the boundary actually produced rather than on how a caller reached it.

    ``record`` and ``serialise`` each establish the fixed vocabulary first, so
    the M-1 fail-closed rule covers this boundary too: an unavailable driver
    vocabulary halts publication instead of emitting corrupted evidence.
    """
    if sanitiser is None:
        sanitiser = public_sanitiser()
    text = serialise(sanitiser.record(document))
    (stream if stream is not None else sys.stdout).write(text)
    return text


def serialise_line(document):
    """One deterministic JSON line, for the append-only progress journal.

    The same determinism rules as :func:`serialise` -- sorted keys, fixed
    separators, no timestamp -- but compact, because a journal record has to be
    exactly one line. That is what makes a torn write from a crash detectable
    instead of silently corrupting the record before it.
    """
    vocabulary()
    return json.dumps(document, sort_keys=True, separators=(",", ":"),
                      ensure_ascii=False, default=str) + "\n"


def parse_line(text):
    """One journal line back, or ``None`` when it does not parse.

    ``None`` means a torn write and the caller drops the line. Nothing here
    repairs a partial record: half a record is not evidence.
    """
    try:
        value = json.loads(text)
    except ValueError:
        return None
    return value if isinstance(value, dict) else None


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
