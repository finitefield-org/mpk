#!/usr/bin/env python3
"""Source-free CPU guard for the local Go/Rust native release gate."""

import os
from pathlib import Path
import struct
import sys


def has_native_x86_64_flags(cpuinfo):
    # Preliminary rejection only; the complete native gate is still required.
    flags_seen = False
    for line in cpuinfo.splitlines():
        key, separator, value = line.partition(":")
        if separator and key.strip() == "vendor_id" and value.strip() == "VirtualApple":
            return False
        if separator and key.strip() == "flags":
            flags = set(value.split())
            if {"lm", "sse2"} <= flags:
                flags_seen = True
    return flags_seen


def main():
    if len(sys.argv) != 1:
        return 64
    try:
        host = os.uname()
        if host.sysname != "Linux" or host.machine != "x86_64" or struct.calcsize("P") != 8:
            raise ValueError("host ABI")
        with Path("/proc/cpuinfo").open("rb") as source:
            raw = source.read(16 * 1024 * 1024 + 1)
        if len(raw) > 16 * 1024 * 1024 or not has_native_x86_64_flags(raw.decode("utf-8")):
            raise ValueError("native CPU flags")
    except (OSError, UnicodeError, ValueError):
        print("native x86-64 Linux CPU required for the Go/Rust release gate", file=sys.stderr)
        return 69
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
