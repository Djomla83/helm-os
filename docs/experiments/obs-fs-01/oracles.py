"""OBS-FS-01 independent expectation oracle.

Materialises the frozen byte recipes and computes every expected SHA-256 with
Python hashlib. It never reads spike output and the spike never reads it. The
allowed interleaving digest sets for mutation cases are enumerated here from the
scheduled read-buffer boundary, not observed after the fact.
"""
import hashlib

from frozen_cases import RECIPES, READ_BUF, OVERLIMIT_APPARENT


def materialise(name):
    """Return the exact bytes a recipe denotes."""
    kind, arg = RECIPES[name]
    if kind == "bytes":
        return arg
    if kind == "pattern":
        char, length = arg
        return char.encode("ascii") * length
    if kind == "sparse":
        length, tail = arg
        return b"\0" * (length - len(tail)) + tail
    raise ValueError("unknown recipe kind " + kind)


def digest(name):
    return hashlib.sha256(materialise(name)).hexdigest()


def digest_of(data):
    return hashlib.sha256(data).hexdigest()


def interleavings():
    """Allowed digests for scheduled same-length mutation cases.

    The race fixtures are exact multiples of the 64 KiB read buffer and the
    mutation is injected after the first read returns, so the only byte
    sequences a correct implementation can deliver are enumerable in advance:
    entirely old, entirely new, or old-first-buffer followed by new remainder.
    """
    old, new = materialise("race_old"), materialise("race_new")
    mixed = old[:READ_BUF] + new[READ_BUF:]
    aold, anew = materialise("alias_old"), materialise("alias_new")
    amixed = aold[:READ_BUF] + anew[READ_BUF:]
    return {
        "race_old": digest_of(old),
        "race_new": digest_of(new),
        "race_mixed_1": digest_of(mixed),
        "alias_old": digest_of(aold),
        "alias_new": digest_of(anew),
        "alias_mixed_1": digest_of(amixed),
    }


def expectations():
    table = {name: digest(name) for name in RECIPES}
    table.update(interleavings())
    return table


if __name__ == "__main__":
    import json
    out = {
        "expected_digests": expectations(),
        "recipe_sizes": {n: len(materialise(n)) for n in RECIPES},
        "overlimit_apparent_bytes": OVERLIMIT_APPARENT,
        "read_buffer": READ_BUF,
        "oracle": "Python hashlib over frozen byte recipes; independent of the spike",
    }
    print(json.dumps(out, indent=2, sort_keys=True))
