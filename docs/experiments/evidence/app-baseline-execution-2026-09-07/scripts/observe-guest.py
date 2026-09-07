"""Small A0-7ZIP observation script; does not launch or repair the application."""
import hashlib
import json
import os
from pathlib import Path
import sys

root = Path('/home/helmlab/exp009-a0-7zip')
sys.path.insert(0, str(root / 'tools'))
from app_baseline import capture, write_record

mode, output = sys.argv[1:]
assert os.getuid() != 0
record = {'mode': mode, 'uid': os.getuid(), 'groups': os.getgroups(),
          'source_commit': '5a83f8cb2fbb5a3597ddff6b5cae9cb813f1c5ae',
          'observation_script_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
          'boot_id': Path('/proc/sys/kernel/random/boot_id').read_text().strip(),
          'environment': {k: os.environ.get(k) for k in ['DISPLAY', 'WAYLAND_DISPLAY', 'XDG_SESSION_TYPE',
              'XDG_RUNTIME_DIR', 'XAUTHORITY', 'DBUS_SESSION_BUS_ADDRESS', 'WINEPREFIX', 'WINEARCH',
              'WINEDLLOVERRIDES', 'WINEDEBUG', 'SSH_AUTH_SOCK', 'DOCKER_HOST']}}
if mode == 'environment':
    commands = [['cat','/etc/os-release'],['uname','-a'],['id'],['findmnt','-no','FSTYPE,SOURCE,TARGET','/'],
                ['df','-B1','/'],['gnome-shell','--version'],['loginctl','list-sessions','--no-legend'],
                ['ps','-eo','pid,uid,comm,args'],['glxinfo','-B'],['xrandr','--current'],['mokutil','--sb-state'],
                ['/opt/wine-devel/bin/wine','--version'],
                ['dpkg-query','-W','-f=${binary:Package}\t${Version}\t${Architecture}\n']]
    record['commands'] = [capture(command, cwd=str(root), timeout=30) for command in commands]
    sessions = next(item for item in record['commands'] if item['argv'][:2] == ['loginctl','list-sessions'])
    record['desktop_sessions'] = []
    for line in sessions.get('stdout','').splitlines():
        fields = line.split()
        if fields and fields[0].isalnum():
            record['desktop_sessions'].append(capture(['loginctl','show-session',fields[0],
                '-p','Name','-p','Type','-p','Desktop','-p','Remote','-p','Active','-p','Display',
                '-p','Seat','-p','Service'], cwd=str(root), timeout=20))
elif mode == 'installed':
    import pefile
    record['files'] = []
    for name in ['7zFM.exe','7z.exe','7z.dll']:
        path = root / 'prefix/drive_c/Program Files/7-Zip' / name
        data = path.read_bytes()
        pe = pefile.PE(data=data)
        versions = {}
        for group in getattr(pe, 'FileInfo', []):
            for item in group:
                for table in getattr(item, 'StringTable', []):
                    versions.update({k.decode(errors='replace'): v.decode(errors='replace')
                                     for k,v in table.entries.items()})
        record['files'].append({'path': str(path.relative_to(root)), 'bytes': len(data),
                                'sha256': hashlib.sha256(data).hexdigest(),
                                'pe_machine': hex(pe.FILE_HEADER.Machine), 'versions': versions})
        pe.close()
    record['all_x64'] = all(item['pe_machine'] == '0x8664' for item in record['files'])
    record['scope'] = 'Installed file identity; GUI operation and output content are separate checks'
elif mode == 'wine-processes':
    record['processes'] = []
    for proc in Path('/proc').iterdir():
        if not proc.name.isdigit():
            continue
        try:
            if proc.stat().st_uid != os.getuid():
                continue
            maps = (proc / 'maps').read_text()
            selected = sorted({line.split()[-1] for line in maps.splitlines()
                               if '/wine/' in line or 'wine-devel' in line})
            if selected:
                record['processes'].append({'pid': int(proc.name), 'comm': (proc/'comm').read_text().strip(),
                                            'runtime_mappings': selected})
        except (FileNotFoundError, PermissionError):
            pass
else:
    raise ValueError('Unknown observation mode')
write_record(Path(output), record)
print(json.dumps(record, ensure_ascii=True))
