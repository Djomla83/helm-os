"""Capture one command in the single A0 guest; no inherited SSH settings or agent."""
import base64
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import time

private = Path(__file__).parent
sys.path.insert(0, str(Path.cwd()/'tools'))
from app_baseline import capture, write_record

label, account, remote_command = sys.argv[1:4]
assert account in ('helmsetup','helmlab')
assert label.replace('-','').isalnum()
target = private/(label+'.json')
assert not target.exists()
guest = json.loads((private/'ssh/guest.json').read_text())
argv = ['ssh.exe','-F','NUL','-T','-o','BatchMode=yes','-o','IdentitiesOnly=yes',
        '-o','IdentityAgent=none','-o','ForwardAgent=no','-o','ForwardX11=no',
        '-o','StrictHostKeyChecking=yes','-o','ConnectTimeout=15',
        '-o','UserKnownHostsFile='+str(private/'ssh/known_hosts'),
        '-i',str(private/'ssh/id_ed25519'),account+'@'+guest['address'],remote_command]
if len(sys.argv) > 4:
    assert account == 'helmsetup' and sys.argv[4] == '--sudo'
    assert remote_command.startswith("sudo -S -p '' -- ")
    credentials = json.loads((private/'guest-credentials.private.json').read_text())
    stdin = (credentials['setup_password']+'\n').encode()
    if len(sys.argv) > 5:
        assert sys.argv[5] == '--set-app-password' and remote_command.endswith('chpasswd')
        stdin += ('helmlab:'+credentials['app_password']+'\n').encode()
    record = {'argv':argv, 'cwd':str(Path.cwd()),
              'started_utc':datetime.now(timezone.utc).isoformat(),
              'stdin':'Private guest credential supplied on stdin; not retained in this record'}
    start = time.monotonic()
    try:
        result = subprocess.run(argv,input=stdin,capture_output=True,timeout=1800)
        record.update(exit_code=result.returncode,stdout=result.stdout.decode('utf-8',errors='replace'),
                      stderr=result.stderr.decode('utf-8',errors='replace'),
                      stdout_base64=base64.b64encode(result.stdout).decode(),
                      stderr_base64=base64.b64encode(result.stderr).decode())
    except subprocess.TimeoutExpired as error:
        record.update(exit_code=None,timeout_seconds=1800,
                      stdout_base64=base64.b64encode(error.stdout or b'').decode(),
                      stderr_base64=base64.b64encode(error.stderr or b'').decode())
    record.update(ended_utc=datetime.now(timezone.utc).isoformat(),duration_seconds=time.monotonic()-start)
else:
    record = capture(argv,cwd=str(Path.cwd()),timeout=600)
record['scope']='SSH command capture in dedicated guest; command exit is not an application verdict'
record['script_sha256']=hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
write_record(target,record)
print(json.dumps({'record':target.name,'exit_code':record['exit_code'],
                  'duration_seconds':record['duration_seconds'],
                  'stdout_bytes':len(base64.b64decode(record['stdout_base64'])),
                  'stderr_bytes':len(base64.b64decode(record['stderr_base64']))}))
sys.exit(record['exit_code'] if record['exit_code'] is not None else 125)
