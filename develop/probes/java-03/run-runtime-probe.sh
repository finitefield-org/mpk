#!/bin/sh
# Local, offline T01 probe; never invokes a production frontend or release gate.
set -eu
if [ "$#" -ne 2 ]; then
    echo 'usage: run-runtime-probe.sh /absolute/pinned/jdk runtime|compiler' >&2
    exit 2
fi
case "$2" in runtime|compiler) ;; *) exit 2 ;; esac
jdk_root=$(CDPATH= cd -- "$1" && pwd)
probe_root=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
python3 "$probe_root/check-jdk.py" "$jdk_root"
exec docker run --rm --pull=never --platform linux/amd64 --network=none --ipc=none \
    --read-only --hostname=mpk-java-probe --cap-drop=ALL \
    --cap-add=SYS_ADMIN --cap-add=SETUID --cap-add=SETGID --cap-add=SYS_CHROOT \
    --security-opt=no-new-privileges --security-opt=seccomp=unconfined \
    --pids-limit=128 --memory=1073741824 --memory-swap=1073741824 \
    --tmpfs=/tmp:rw,nosuid,nodev,exec,size=134217728 \
    --mount "type=bind,src=$jdk_root,dst=/inputs/jdk,readonly" \
    --mount "type=bind,src=$probe_root,dst=/inputs/probes,readonly" \
    docker.io/library/python@sha256:db8e83a44af476c636a6a753adace39ad37863b63c0afd2862db7bbafeeb3944 \
    /bin/sh /inputs/probes/runtime-probe.sh "$2"
