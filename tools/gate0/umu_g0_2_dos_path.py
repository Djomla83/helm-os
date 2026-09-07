"""G0-2-only adapter: pass the frozen probes to UMU using a Wine DOS path."""

import hashlib
import json
import os
from pathlib import Path
import sys

directory = Path("/home/helmlab/g0/completion-20260907")
identities = {
    "dcomp_probe.exe": "b74011c0c154e1e742b6a27c2de7259befce4e9e528cb946f9221fdd2f445ce4",
    "g0_2_control.exe": "b28f3528e158e60fdc536e400e02763a186133eb1922560d92456d6e3bc5c50e",
}
path = Path(sys.argv[1]).resolve(strict=True)
if path.parent != directory or path.name not in identities:
    raise ValueError("Only the two frozen G0-2 executables are supported")
if hashlib.sha256(path.read_bytes()).hexdigest() != identities[path.name]:
    raise ValueError("Probe bytes changed")
prefix = Path(os.environ["WINEPREFIX"])
if (prefix / "dosdevices" / "z:").resolve(strict=True) != Path("/"):
    raise ValueError("Expected the existing Wine Z: mapping to the lab filesystem")
dos_path = "Z:" + str(path).replace("/", "\\")
command = ["/usr/bin/umu-run", dos_path] + sys.argv[2:]
print(json.dumps({"g0_2_path_adapter_argv": command}), file=sys.stderr, flush=True)
os.execv(command[0], command)
