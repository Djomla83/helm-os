#!/bin/bash
# One A0-7ZIP VM only; invoke once with guest administrative rights.
set -euo pipefail
test "$(id -u)" = 0
test "$(hostname)" = helm-a0-7zip
base=/home/helmsetup/a0-provision
mkdir "$base"
exec > >(tee "$base/stdout.log") 2> >(tee "$base/stderr.log" >&2)
date --utc --iso-8601=ns
cat /etc/os-release
uname -a
dpkg-query -W -f='${binary:Package}\t${Version}\t${Architecture}\n' > "$base/packages-before.tsv"
dpkg --add-architecture i386
apt-get update
DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends ca-certificates curl gnupg mesa-utils python3-pefile
mkdir -p /etc/apt/keyrings
curl --fail --location --show-error --output "$base/winehq.key" https://dl.winehq.org/wine-builds/winehq.key
printf 'd965d646defe94b3dfba6d5b4406900ac6c81065428bf9d9303ad7a72ee8d1b8  %s\n' "$base/winehq.key" | sha256sum --check -
gpg --batch --show-keys --with-colons "$base/winehq.key" > "$base/winehq-key-identity.txt"
grep --fixed-strings 'fpr:::::::::D43F640145369C51D786DDEA76F1A20FF987672F:' "$base/winehq-key-identity.txt"
gpg --batch --dearmor --output /etc/apt/keyrings/winehq-archive.key "$base/winehq.key"
chmod 644 /etc/apt/keyrings/winehq-archive.key
cat > /etc/apt/sources.list.d/winehq-noble.sources <<'EOF'
Types: deb
URIs: https://dl.winehq.org/wine-builds/ubuntu
Suites: noble
Components: main
Architectures: amd64 i386
Signed-By: /etc/apt/keyrings/winehq-archive.key
EOF
apt-get -o Debug::Acquire::gpgv=true update
find /var/lib/apt/lists -maxdepth 1 -type f -name 'dl.winehq.org_wine-builds_ubuntu_dists_noble_*' -exec sha256sum {} + > "$base/apt-index-sha256.txt"
apt-cache policy winehq-devel wine-devel wine-devel-amd64 wine-devel-i386:i386
apt-get -y --download-only --no-install-recommends install winehq-devel=11.17~noble-1 wine-devel=11.17~noble-1 wine-devel-amd64=11.17~noble-1 wine-devel-i386:i386=11.17~noble-1
cd /var/cache/apt/archives
cat > "$base/expected-wine-sha256.txt" <<'EOF'
eb42cc830c0e0582ecb1c5cf94cbfefe5d0a0235caae771035077b5e7b8ddd8f  wine-devel_11.17~noble-1_amd64.deb
9184a81f0848003460526f30a66bd470a997fa0bd43c0d59efcacce0a12adb63  wine-devel-amd64_11.17~noble-1_amd64.deb
62873c6d7c5b05710064bc5fa3d9e2be0595edd77bc94c2ed3ea7896fdcb7869  winehq-devel_11.17~noble-1_amd64.deb
599a34d04fd3443a55f5700c3a919f177cdc4ab77b5ee7d6ebcfb87de03cc966  wine-devel-i386_11.17~noble-1_i386.deb
EOF
sha256sum --check "$base/expected-wine-sha256.txt"
for package in wine-devel_11.17~noble-1_amd64.deb wine-devel-amd64_11.17~noble-1_amd64.deb winehq-devel_11.17~noble-1_amd64.deb wine-devel-i386_11.17~noble-1_i386.deb; do
    stat --format='%n %s bytes' "$package"
    dpkg-deb --field "$package" Package Version Architecture
done
find /var/cache/apt/archives -maxdepth 1 -type f -name '*.deb' -printf '%f\t%s\n' > "$base/downloaded-packages.tsv"
DEBIAN_FRONTEND=noninteractive apt-get -y --no-download --no-install-recommends install winehq-devel=11.17~noble-1 wine-devel=11.17~noble-1 wine-devel-amd64=11.17~noble-1 wine-devel-i386:i386=11.17~noble-1
apt-mark hold winehq-devel wine-devel wine-devel-amd64 wine-devel-i386:i386
dpkg-query -W -f='${binary:Package}\t${Version}\t${Architecture}\n' > "$base/packages-after.tsv"
sha256sum /opt/wine-devel/bin/wine /opt/wine-devel/bin/wineserver
/opt/wine-devel/bin/wine --version
test ! -d /home/helmlab
useradd --create-home --shell /bin/bash helmlab
id helmlab
install -d -m 700 -o helmlab -g helmlab /home/helmlab/.ssh
install -m 600 -o helmlab -g helmlab /home/helmsetup/.ssh/authorized_keys /home/helmlab/.ssh/authorized_keys
install -d -m 755 -o helmlab -g helmlab /home/helmlab/exp009-a0-7zip
timedatectl set-timezone Etc/UTC
df -B1 /
date --utc --iso-8601=ns
echo HELM_A0_PROVISION_COMPLETE
