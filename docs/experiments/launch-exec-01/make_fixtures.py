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

  magic_only.bin       X4: exactly the four bytes 7F 45 4C 46 and nothing else.
                       It passes a magic check, reaches execveat, and must fail
                       ENOEXEC. A non-ELF file cannot pose this case at all,
                       because the cohort rule refuses it at admission.

  script_fixture.sh    X2/X2b/X2c: a #! script. Refused at admission; the
                       counterfactual arms run only in a frozen spike mode.
"""
import hashlib
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
    "helper_foreign.elf": elf64_header(EM_AARCH64),
    "magic_only.bin": MAGIC_ONLY,
    "script_fixture.sh": SCRIPT_FIXTURE,
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
        written[name] = str(path)
    return written


if __name__ == "__main__":
    import json
    if len(sys.argv) > 1:
        print(json.dumps(write(sys.argv[1]), indent=2, sort_keys=True))
    else:
        print(json.dumps(digests(), indent=2, sort_keys=True))
