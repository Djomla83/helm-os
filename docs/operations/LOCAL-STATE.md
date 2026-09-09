# Local state layout — repository versus machine-local state

This is a **policy** document. It defines where HELM state belongs on a development machine,
symbolically. It deliberately contains no machine-specific paths, no host names and no operator
identities, so that it stays valid on every machine and can be published.

## 1. The convention

Two siblings under one root. `<HELM_ROOT>` is whatever directory an operator chooses.

```text
<HELM_ROOT>\
├─ helm-os\                 authoritative Git working tree
└─ local\                   non-Git machine-local state
   ├─ vm\
   ├─ private\
   ├─ downloads\
   └─ scratch\
```

Throughout this document `<HELM_LOCAL_ROOT>` means `<HELM_ROOT>\local`.

The Git working tree and machine-local state are **physically separate directories**. Neither is
nested inside the other. There is no extra `repo\` level.

## 2. What each location is for

### `helm-os\` — the authoritative Git working tree

- source code and tooling;
- tracked documentation;
- experiment definitions and frozen experiment sources;
- sanitised, publishable evidence.

Nothing in this tree is machine-specific by intent. Anything here is expected to be committed,
reviewed and published.

### `<HELM_LOCAL_ROOT>\vm\` — virtual machine state

Hyper-V and WSL virtual disk state retained outside Git: registered distribution disks, registered
virtual machine configuration and storage, and any retained unregistered baseline disk image.

These are gigabyte-scale binaries. They are never a Git concern.

### `<HELM_LOCAL_ROOT>\private\` — unsanitised and host-identifying material

- raw, unsanitised provenance captured during experiments, from which published evidence was
  derived by redaction;
- evidence that identifies the host, the operator or the local environment;
- local-only inventories describing where retained assets live on this particular machine.

Published evidence may refer to this material abstractly — for example, as a private archive held
outside Git — while the material itself stays here.

### `<HELM_LOCAL_ROOT>\downloads\` — large reproducible inputs

Installers, OS images and runtime archives that are large and reproducible from an upstream source.
They are pinned by checksum in evidence rather than committed. Keeping the bytes here, and the pin
in the repository, is what makes the input both verifiable and out of Git.

### `<HELM_LOCAL_ROOT>\scratch\` — transient working material

Experiment transfer payloads, build by-products, migration records and other material that is
working state rather than a result.

## 3. Rules

These are the parts that are easy to get wrong.

**`.gitignore` is not the boundary.** Ignore rules decide what Git *notices*; they do not decide
what *belongs* in the working tree. A file being ignorable is not a reason to place it inside
`helm-os\`.

**Do not put VM disks or private provenance inside `helm-os\`.** Not even under an ignored path.
Virtual disk images and unsanitised or host-identifying provenance live under `<HELM_LOCAL_ROOT>`,
because the separation is physical, not declarative. This keeps clones small, keeps private
material out of a tree that is routinely published, and makes accidental commitment structurally
unlikely rather than merely forbidden.

**Do not commit machine-specific absolute paths** — with one exception. Absolute host paths may
appear in the repository only where they are *historical evidence that was actually recorded during
an experiment*. Such a path is a measurement, not a configuration value. New forward-looking
guidance must use the symbolic form (`<HELM_LOCAL_ROOT>\vm`, and so on) or refer to this document.

**Do not rewrite historical evidence when the local layout changes.** Evidence records what was
observed at the time it was recorded. If assets are later relocated, the recorded paths stay
exactly as they were. Editing them to match a newer layout would misrepresent what was observed,
which is a more serious defect than a stale-looking path.

**The current host layout may differ from historical experiment paths, and that is expected.** A
reader who finds a path in an evidence file that no longer exists on the machine has found a
correctly preserved historical record, not an error. Where current locations matter operationally,
they are stated in local-only inventories under `<HELM_LOCAL_ROOT>\private`, never in published
documentation.

## 4. Consequences for reviewers

- A large binary appearing anywhere under `helm-os\` is a defect, whether or not it is ignored.
- A new absolute host path in tracked documentation is a defect unless it is a dated observation.
- A change that edits existing dated evidence paths to match the current machine is a defect, and
  should be rejected rather than corrected in place.

Related: [Gate 0 lab runbook](../experiments/LAB-G0-RUNBOOK.md) records one lab's history and
applies this convention to its current operational guidance.
