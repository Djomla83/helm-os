"""Collect exact machine semantics for the same synthetic cases on two platforms."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def edit(root, name, change):
    path = root/name
    value = json.loads(path.read_bytes())
    change(value)
    path.write_bytes(json.dumps(value, ensure_ascii=True, indent=2).encode())
    if name != 'bundle.json':
        def repin(bundle):
            for artifact in bundle['artifacts']:
                if artifact['path'] == name:
                    artifact['sha256'] = digest(path.read_bytes())
        edit(root, 'bundle.json', repin)


def collect(binary):
    cases = {}
    names = ['synthetic', 'a0-7zip', 'missing', 'wrong-hash', 'wrong-destination',
             'blocked', 'experimental-fail', 'traversal', 'case-collision',
             'duplicate-field', 'invalid-utf8', 'overflow-u64', 'invalid-surrogate',
             'deep-json', 'unicode-storage', 'case-lookup', 'output-case',
             'file-link', 'directory-link', 'dangling', 'manifest-link', 'hardlink']
    with tempfile.TemporaryDirectory(prefix='helm-evidence-comparison-') as tmp:
        for name in names:
            root = Path(tmp)/name
            shutil.copytree(ROOT/'crates/helm-evidence/tests/fixtures'/('a0-7zip' if name == 'a0-7zip' else 'synthetic'), root)
            if name == 'missing':
                (root/'good.json').unlink()
            elif name == 'wrong-hash':
                (root/'good.json').write_bytes(b'synthetic wrong bytes')
            elif name == 'wrong-destination':
                edit(root, 'before.json', lambda r: r['output'].update(path='wrong/output.dat'))
            elif name == 'blocked':
                edit(root, 'before.json', lambda r: r['steps'][0].update(status='BLOCKED'))
            elif name == 'experimental-fail':
                edit(root, 'bundle.json', lambda b: b.update(experimental_verdict='FAIL'))
                edit(root, 'before.json', lambda r: r['steps'][0].update(status='FAIL'))
            elif name in ('traversal', 'unicode-storage', 'case-lookup'):
                value = {'traversal': '../SYNTHETIC_PRIVATE_CANARY', 'unicode-storage': 'café.json', 'case-lookup': 'GOOD.JSON'}[name]
                edit(root, 'bundle.json', lambda b: b['artifacts'][2].update(path=value))
            elif name == 'case-collision':
                edit(root, 'bundle.json', lambda b: b['artifacts'][2].update(path='RESULT.JSON'))
            elif name == 'duplicate-field':
                path = root/'bundle.json'
                path.write_bytes(path.read_bytes().replace(b'{', b'{"version":"999",', 1))
            elif name == 'invalid-utf8':
                (root/'bundle.json').write_bytes(b'\xff')
            elif name == 'overflow-u64':
                edit(root, 'result.json', lambda r: r.update(archive_bytes=2**64))
            elif name == 'invalid-surrogate':
                (root/'bundle.json').write_bytes(b'{"schema":"\\ud800"}')
            elif name == 'deep-json':
                # Serde skips auxiliary metadata iteratively; depth alone need not reject it.
                path = root/'result.json'
                path.write_bytes(path.read_bytes().replace(b'{', b'{"extra":'+b'['*140+b'0'+b']'*140+b',', 1))
                edit(root, 'bundle.json', lambda b: b['artifacts'][1].update(sha256=digest(path.read_bytes())))
            elif name == 'output-case':
                edit(root, 'before.json', lambda r: r['output'].update(path='OUTPUTS/before/output.dat'))
            elif name in ('file-link', 'dangling', 'manifest-link', 'hardlink'):
                leaf = root/('bundle.json' if name == 'manifest-link' else 'good.json')
                saved = root/'saved.json'
                leaf.rename(saved)
                if name == 'hardlink':
                    os.link(saved, leaf)
                else:
                    leaf.symlink_to('absent.json' if name == 'dangling' else 'saved.json')
            elif name == 'directory-link':
                (root/'outputs/before').rename(root/'saved-directory')
                (root/'outputs/before').symlink_to('../saved-directory', target_is_directory=True)
            cmd = [str(binary), 'verify', str(root)]
            result = subprocess.run([*cmd, '--json'], capture_output=True, timeout=5)
            human = subprocess.run(cmd, capture_output=True, timeout=5)
            report = json.loads(result.stdout)
            raw = json.dumps(report, sort_keys=True, separators=(',', ':')).encode()
            assert not result.stderr and not human.stderr
            assert str(root).encode() not in result.stdout+human.stdout
            assert b'SYNTHETIC_PRIVATE_CANARY' not in result.stdout+human.stdout
            cases[name] = {'exit_code': result.returncode, 'verdict': report['verdict'],
                           'experimental_verdict': report['experimental_verdict'],
                           'report_sha256': digest(raw), 'sections': report['sections'],
                           'failures': [{k:c[k] for k in ('code','status','workflow','artifact','requirement')}
                                        for c in report['checks'] if c['status'] in ('INVALID','INCOMPLETE')]}
    return {'system': platform.system(), 'machine': platform.machine(),
            'privacy': 'No raw absolute fixture path or synthetic private marker in human/JSON output',
            'canonicalization': 'Entire parsed report serialized with sorted keys and compact separators',
            'binary_sha256': digest(binary.read_bytes()), 'cases': cases}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', required=True, type=Path)
    parser.add_argument('--output', required=True, type=Path)
    args = parser.parse_args()
    args.output.write_text(json.dumps(collect(args.binary.resolve()), indent=2)+'\n', encoding='utf-8', newline='\n')
