#!/bin/bash
# A0 synthetic fixtures and controls only, using the previously committed helper.
set -euo pipefail
test "$(id -un)" = helmlab
test "$(id -u)" != 0
cd /home/helmlab/exp009-a0-7zip
test ! -e prefix
test ! -e inputs
printf '%s\n' 'f2d8f84716f2989a9898c0fe11529f3ec6ea9780637545814277a5d3461e1b8e  source-5a83f8c.tar' 'f012436bc6868334cabdfe31818fd3ae67126d9737cb9a80084d2ae69cbfb1ae  observe-guest.py' '0859c524b8a63551848f0c246abddcb1d0b7b656b0fbfe879f8d85e61a9e6edd  7z2603-x64.exe' | sha256sum --check -
tar -xf source-5a83f8c.tar
printf '%s\n' '2ffec610149ad7aee94ed891af48c2c6623062937d3a5087449523a6d81e57e2  tools/app_baseline.py' '75d78bc6bc0f539df5ca8d54df34f406e44f2aa5ceb8a07bf3d856ce72747c8f  tools/fixtures/exp009-7zip.json' | sha256sum --check -
mkdir records controls outputs outputs/before-restart outputs/after-restart
python3 tools/app_baseline.py capture --cwd "$PWD" --timeout 60 records/guest-helper-tests.json -- python3 -m unittest discover -s tools/tests -p test_app_baseline.py -v
python3 tools/app_baseline.py capture --cwd "$PWD" --timeout 60 records/fixture-command.json -- python3 tools/app_baseline.py prepare inputs fixture-record.json
python3 tools/app_baseline.py capture --cwd "$PWD" --timeout 60 records/before-good-command.json -- python3 tools/app_baseline.py verify inputs/oracle-good.zip controls/before-good.json
set +e
python3 tools/app_baseline.py capture --cwd "$PWD" --timeout 60 records/before-bad-command.json -- python3 tools/app_baseline.py verify inputs/oracle-corrupted.zip controls/before-bad.json
broken_exit=$?
set -e
test "$broken_exit" = 1
printf 'Expected corrupted-control exit: %s\n' "$broken_exit"
echo HELM_A0_GUEST_CONTROLS_COMPLETE
