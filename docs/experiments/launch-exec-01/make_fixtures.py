"""LAUNCH-EXEC-01 deterministic fixture generator.

Emits the non-compiled fixtures byte-for-byte from this source, so they are
reproducible from the repository rather than committed as opaque binaries and
so their SHA-256 is a function of this file alone.

  helper_foreign.elf   E8: a well-formed ELF64 header whose e_machine is
                       EM_AARCH64. Admission must refuse it as ElfNotInCohort,
                       and execveat must never be reached. It is NEVER executed:
                       it could not run on an x86_64 runner anyway, and that is
                       not what the case tests. What it tests is that the cohort
                       is pinned in the HEADER rather than in four magic bytes,
                       because binfmt_misc matches magic with a mask, is
                       consulted ahead of binfmt_elf, and resolves its
                       interpreter by pathname.

  unloadable_in_cohort.elf
                       X4: a 64-byte ELF64 header that passes the cohort rule
                       with e_phnum = 0, so it is admitted and reaches execveat.

  magic_only.bin       Exactly the four bytes 7F 45 4C 46 and nothing else. A
                       negative fixture for the cohort rule; no case executes it.

  script_fixture.sh    X2/X2b/X2c: a #! script. Refused at admission; the
                       counterfactual arms run only in a frozen spike mode.

Each fixture is written with the mode FIXTURE_MODES declares for it. Trial #2
wrote every fixture with write_bytes alone, which creates 0o666 masked by the
umask and so never an execute bit: X2b, X2c and X4 reached execveat and got
EACCES before the behaviour each was written to test. The mode is not part of
the bytes, so every digest below is unchanged by it.
"""
import hashlib
import os
import pathlib
import sys

EM_AARCH64 = 183
EM_X86_64 = 62
ET_EXEC = 2

MAGIC_ONLY = b"\x7fELF"

SCRIPT_FIXTURE = b"""#!/bin/sh
# LAUNCH-EXEC-01 script fixture. Refused at admission by the ELF cohort rule.
# X2b reaches execveat with an O_CLOEXEC descriptor and must fail ENOENT --
# per execveat(2) BUGS, because the program file is inaccessible to the
# interpreter -- NOT because scripts cannot be executed.
# X2c drops O_CLOEXEC and the interpreter then receives /dev/fd/N, which is the
# procfs dependency the ELF rule exists to exclude.
echo "LAUNCH-EXEC-01 script fixture ran with argv: $0 $@"
exit 0
"""


def elf64_header(e_machine, e_type=ET_EXEC):
    """A 64-byte ELF64 little-endian header. Header only, deliberately.

    The object is never executed, so no program headers are needed; admission
    reads the first 64 bytes and must refuse on e_machine alone.
    """
    ident = bytearray(16)
    ident[0:4] = b"\x7fELF"
    ident[4] = 2          # EI_CLASS  = ELFCLASS64
    ident[5] = 1          # EI_DATA   = ELFDATA2LSB
    ident[6] = 1          # EI_VERSION= EV_CURRENT
    ident[7] = 0          # EI_OSABI  = SYSV
    header = bytearray(ident)
    header += e_type.to_bytes(2, "little")        # e_type
    header += e_machine.to_bytes(2, "little")     # e_machine
    header += (1).to_bytes(4, "little")           # e_version
    header += (0).to_bytes(8, "little")           # e_entry
    header += (0).to_bytes(8, "little")           # e_phoff
    header += (0).to_bytes(8, "little")           # e_shoff
    header += (0).to_bytes(4, "little")           # e_flags
    header += (64).to_bytes(2, "little")          # e_ehsize
    header += (56).to_bytes(2, "little")          # e_phentsize
    header += (0).to_bytes(2, "little")           # e_phnum
    header += (64).to_bytes(2, "little")          # e_shentsize
    header += (0).to_bytes(2, "little")           # e_shnum
    header += (0).to_bytes(2, "little")           # e_shstrndx
    assert len(header) == 64, len(header)
    return bytes(header)


FIXTURES = {
    # E8: refused at admission, e_machine is not in the cohort.
    "helper_foreign.elf": elf64_header(EM_AARCH64),
    # X4: PASSES the cohort rule -- ELFCLASS64, ELFDATA2LSB, EM_X86_64, ET_EXEC
    # -- and is therefore admitted and reaches execveat, where the loader finds
    # e_phnum = 0 and nothing to map, giving ENOEXEC. The magic-only fixture
    # below cannot pose X4: once admission became a 64-byte HEADER check it is
    # refused as ElfNotInCohort and never reaches exec at all.
    "unloadable_in_cohort.elf": elf64_header(EM_X86_64),
    # Retained as the negative fixture for the cohort rule itself: four bytes
    # that satisfy a magic check and nothing more.
    "magic_only.bin": MAGIC_ONLY,
    "script_fixture.sh": SCRIPT_FIXTURE,
}

# The permission bits of each fixture, declared per fixture rather than applied
# by a broad chmod: an object a case executes carries an execute bit, and an
# object no case executes carries only what its case needs.
FIXTURE_MODES = {
    # E8 is refused at admission on e_machine, which needs only read access.
    "helper_foreign.elf": 0o644,
    # X4 must reach execveat, and the kernel checks execute permission before
    # any loader looks at e_phnum.
    "unloadable_in_cohort.elf": 0o755,
    # No case executes it.
    "magic_only.bin": 0o644,
    # X2 is refused at admission whatever the mode; X2b and X2c bypass
    # admission and must reach execveat's script handling.
    "script_fixture.sh": 0o755,
}


def digests():
    return {name: hashlib.sha256(data).hexdigest()
            for name, data in sorted(FIXTURES.items())}


def write(out_dir):
    out = pathlib.Path(out_dir)
    out.mkdir(parents=True, exist_ok=True)
    written = {}
    for name, data in sorted(FIXTURES.items()):
        path = out / name
        path.write_bytes(data)
        # Exactly the declared mode, independent of the umask and of any mode
        # an earlier file at this path had.
        os.chmod(path, FIXTURE_MODES[name])
        written[name] = str(path)
    return written


if __name__ == "__main__":
    # P-16. write() returns the absolute path of every fixture it wrote, so this
    # diagnostic emitted host paths raw. It leaves through the one publication
    # boundary now, like everything else this experiment prints.
    import evidence

    if len(sys.argv) > 1:
        evidence.publish(write(sys.argv[1]))
    else:
        evidence.publish(digests())
