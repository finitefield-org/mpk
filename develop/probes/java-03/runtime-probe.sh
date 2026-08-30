#!/bin/sh
# Disposable, offline specification probe. Run only inside the documented
# container, with read-only /inputs/jdk and /inputs/probes mounts.
set -eu
case "${1:-runtime}" in
    runtime) probe_class=RuntimeProbe ;;
    compiler) probe_class=CompilerProbe ;;
    *) echo 'expected runtime or compiler probe' >&2; exit 2 ;;
esac
test "$(cat /sys/fs/cgroup/memory.max)" = 1073741824
test "$(cat /sys/fs/cgroup/memory.swap.max)" = 0
test "$(cat /sys/fs/cgroup/pids.max)" = 128
root=/tmp/java-root
mkdir -p /tmp/probe-classes "$root/mpk/toolchain/jdk" "$root/lib/x86_64-linux-gnu" \
    "$root/lib64" "$root/proc" "$root/dev" "$root/mpk/tmp" "$root/mpk/source" \
    "$root/mpk/frontend" "$root/mpk/empty-home"
mkdir -p /tmp/probe-poison/META-INF/services "$root/work/poison-source/demo"
mount --bind "$root" "$root"
chmod 1777 /tmp/probe-classes /tmp/probe-poison
setpriv --reuid=65534 --regid=65534 --clear-groups --no-new-privs \
    /inputs/jdk/bin/javac -proc:none -implicit:none --release 25 -d /tmp/probe-classes \
    /inputs/probes/RuntimeProbe.java /inputs/probes/CompilerProbe.java
setpriv --reuid=65534 --regid=65534 --clear-groups --no-new-privs \
    /inputs/jdk/bin/javac -proc:none --release 25 -d /tmp/probe-poison \
    /inputs/probes/fixtures/poison/Injected.java /inputs/probes/fixtures/poison/ProbeProcessor.java
printf 'poison.ProbeProcessor\n' > /tmp/probe-poison/META-INF/services/javax.annotation.processing.Processor
setpriv --reuid=65534 --regid=65534 --clear-groups --no-new-privs \
    /inputs/jdk/bin/jar --create --file /tmp/poison.jar -C /tmp/probe-poison .
cp /tmp/poison.jar "$root/work/poison.jar"
cp /inputs/probes/fixtures/demo/Hidden.java "$root/work/poison-source/demo/Hidden.java"
mount --bind /inputs/jdk "$root/mpk/toolchain/jdk"
mount -o remount,bind,ro,nosuid,nodev "$root/mpk/toolchain/jdk"
cp /tmp/probe-classes/*.class "$root/mpk/frontend/"
for name in libc.so.6 libm.so.6 libdl.so.2 libpthread.so.0 librt.so.1; do
    cp -L "/lib/x86_64-linux-gnu/$name" "$root/lib/x86_64-linux-gnu/$name"
done
cp -L /lib64/ld-linux-x86-64.so.2 "$root/lib64/ld-linux-x86-64.so.2"
mount -t proc -o ro,nosuid,nodev,noexec proc "$root/proc"
for name in null urandom; do
    touch "$root/dev/$name"
    mount --bind "/dev/$name" "$root/dev/$name"
done
mount -t tmpfs -o noswap,nosuid,nodev,noexec,size=67108864,mode=1777 tmpfs "$root/mpk/tmp"
mount -o remount,bind,ro,nosuid,nodev "$root"
ulimit -c 0
ulimit -n 1024
ulimit -v 16777216
exec /usr/bin/env -i PATH=/nonexistent HOME=/mpk/empty-home TMPDIR=/mpk/tmp \
    LANG=C.UTF-8 LC_ALL=C.UTF-8 TZ=UTC \
    /usr/sbin/chroot --userspec=65534:65534 --groups=65534 "$root" \
    /mpk/toolchain/jdk/bin/java -Xint -Xshare:off -XX:+UseSerialGC \
    -XX:ActiveProcessorCount=1 -XX:+DisableAttachMechanism -XX:-UsePerfData \
    -Xms32m -Xmx512m -Xss1m -Dfile.encoding=UTF-8 -Duser.language=en \
    -Duser.country=US -Duser.timezone=UTC -Djava.io.tmpdir=/mpk/tmp \
    -Duser.home=/mpk/empty-home -Djava.library.path=/nonexistent \
    -XX:ErrorFile=/mpk/tmp/hs_err.log -XX:-CreateCoredumpOnCrash -XX:-HeapDumpOnOutOfMemoryError \
    --limit-modules java.base,java.compiler,jdk.compiler,jdk.zipfs \
    --add-modules java.compiler,jdk.compiler,jdk.zipfs \
    -cp /mpk/frontend "$probe_class"
