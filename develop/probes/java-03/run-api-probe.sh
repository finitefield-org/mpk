#!/bin/sh
set -eu
# Provision the pinned JDK and container image separately. This probe never pulls.
if [ "$#" -ne 1 ]; then
    echo "usage: $0 /absolute/path/to/jdk-25.0.4.1+1" >&2
    exit 2
fi
jdk_root=$(CDPATH= cd -- "$1" && pwd)
probe_root=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
python3 "$probe_root/check-jdk.py" "$jdk_root"
exec docker run --rm --pull=never --platform linux/amd64 --network=none \
    --read-only --cap-drop=ALL --security-opt=no-new-privileges --user=65534:65534 \
    --pids-limit=64 --memory=1g --memory-swap=1g \
    --tmpfs /tmp:rw,nosuid,nodev,noexec,size=16m,mode=1777 \
    --tmpfs /work:rw,nosuid,nodev,noexec,size=128m,uid=65534,gid=65534,mode=0700 \
    --mount "type=bind,source=$jdk_root,target=/mpk/toolchain/jdk,readonly" \
    --mount "type=bind,source=$probe_root,target=/probe,readonly" \
    --workdir=/work \
    docker.io/library/python@sha256:db8e83a44af476c636a6a753adace39ad37863b63c0afd2862db7bbafeeb3944 \
    /usr/bin/env -i HOME=/nonexistent TMPDIR=/tmp PATH=/nonexistent LANG=C.UTF-8 LC_ALL=C.UTF-8 TZ=UTC \
    /bin/sh -eu -c '
        /bin/mkdir -p /work/classes /work/poison/META-INF/services /work/poison-source/demo
        /bin/cp /probe/fixtures/demo/Hidden.java /work/poison-source/demo/Hidden.java
        /mpk/toolchain/jdk/bin/javac -J-Xint -J-Xshare:off -J-XX:+UseSerialGC -J-XX:ActiveProcessorCount=1 -J-XX:+DisableAttachMechanism -J-XX:-UsePerfData -J-Xms32m -J-Xmx512m -J-Xss1m --release 25 -proc:none -d /work/classes /probe/CompilerProbe.java
        /mpk/toolchain/jdk/bin/javac -J-Xint -J-Xshare:off -J-XX:+UseSerialGC -J-XX:ActiveProcessorCount=1 -J-XX:+DisableAttachMechanism -J-XX:-UsePerfData -J-Xms32m -J-Xmx512m -J-Xss1m --release 25 -proc:none -d /work/poison /probe/fixtures/poison/Injected.java /probe/fixtures/poison/ProbeProcessor.java
        printf "poison.ProbeProcessor\n" > /work/poison/META-INF/services/javax.annotation.processing.Processor
        /mpk/toolchain/jdk/bin/jar --create --file /work/poison.jar -C /work/poison .
        exec /mpk/toolchain/jdk/bin/java -Xint -Xshare:off -XX:+UseSerialGC -XX:ActiveProcessorCount=1 -XX:+DisableAttachMechanism -XX:-UsePerfData -Xms32m -Xmx512m -Xss1m -Dfile.encoding=UTF-8 -Duser.language=en -Duser.country=US -Duser.timezone=UTC -Djava.io.tmpdir=/tmp --limit-modules java.base,java.compiler,jdk.compiler,jdk.zipfs --add-modules java.compiler,jdk.compiler,jdk.zipfs -cp /work/classes CompilerProbe
    '
