"""LAUNCH-EXEC-01 independent expectation oracle.

Materialises the frozen byte recipe and computes every expected count and
SHA-256 with Python ``hashlib``. It never reads spike output and the spike never
reads it. Nothing here imports, links or reuses implementation logic from
``launcher_spike.c``; a byte volume alone is not a recipe, which is why the
recipe lives in ``frozen_cases.py`` and is reproduced here from first
principles rather than observed after the fact.
"""
import hashlib

from frozen_cases import (
    MAX_CAPTURE_BYTES,
    REPORT_SENTINEL,
    STREAM_TAG,
    stream_byte,
)


def stream_bytes(stream, length):
    """The exact bytes a helper must emit on ``stream`` for ``length``.

    Independent restatement of the frozen recipe byte[i] = (i*251 + tag) % 256.
    """
    tag = STREAM_TAG[stream]
    return bytes(stream_byte(i, tag) for i in range(length))


def stream_digest(stream, length):
    return hashlib.sha256(stream_bytes(stream, length)).hexdigest()


def digest_of(data):
    return hashlib.sha256(data).hexdigest()


def file_digest(path):
    """SHA-256 of a file, computed independently of anything the spike measured."""
    h = hashlib.sha256()
    with open(path, "rb") as handle:
        for chunk in iter(lambda: handle.read(65536), b""):
            h.update(chunk)
    return h.hexdigest()


def retained_prefix_length(total):
    """Bytes a capture_prefix run may retain in memory, never in the receipt."""
    return min(total, MAX_CAPTURE_BYTES)


def split_report(raw):
    """Split a helper's fd-1 bytes into (payload, report_json_bytes).

    The frozen sentinel exists because the report and the measured payload share
    descriptor 1: no descriptor above 2 is ever passed to a helper, so O-series
    counts and digests are computed over the pre-sentinel segment only.
    Returns (payload, None) when no report was emitted, which is itself
    load-bearing evidence in S2 and S5.
    """
    index = raw.find(REPORT_SENTINEL)
    if index < 0:
        return raw, None
    return raw[:index], raw[index + len(REPORT_SENTINEL):]


def payload_matches(stream, raw, expected_length):
    """Independent verdict on one stream's payload."""
    payload, _ = split_report(raw)
    return {
        "expected_length": expected_length,
        "observed_length": len(payload),
        "expected_sha256": stream_digest(stream, expected_length),
        "observed_sha256": digest_of(payload),
        "match": (len(payload) == expected_length
                  and digest_of(payload) == stream_digest(stream, expected_length)),
    }


def elf64_header_is_in_cohort(first64):
    """Admission cohort predicate, restated independently of the spike.

    Magic alone is not enough: binfmt_misc matches bytes at the start of a file
    with a mask, is consulted ahead of binfmt_elf, and resolves its interpreter
    by pathname. Pinning the cohort in the header is what closes that.
    """
    if len(first64) < 20:
        return False
    if first64[:4] != b"\x7fELF":
        return False
    if first64[4] != 2:            # EI_CLASS = ELFCLASS64
        return False
    if first64[5] != 1:            # EI_DATA  = ELFDATA2LSB
        return False
    e_type = int.from_bytes(first64[16:18], "little")
    e_machine = int.from_bytes(first64[18:20], "little")
    if e_machine != 62:            # EM_X86_64
        return False
    return e_type in (2, 3)        # ET_EXEC, ET_DYN


def total_bound_ms(timeout_ms, grace_ms, spawn_confirm_ms, post_exit_drain_ms):
    """The declared upper bound on launch(); exceeding it is a rejection."""
    return spawn_confirm_ms + timeout_ms + grace_ms + post_exit_drain_ms


if __name__ == "__main__":
    import json
    print(json.dumps({
        "stdout_4k_sha256": stream_digest("stdout", 4096),
        "stderr_4k_sha256": stream_digest("stderr", 4096),
        "stdout_512_sha256": stream_digest("stdout", 512),
        "stdout_8m_sha256": stream_digest("stdout", 8 * 1024 * 1024),
        "empty_sha256": digest_of(b""),
    }, indent=2, sort_keys=True))
