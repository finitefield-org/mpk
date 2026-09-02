#!/bin/sh
set -eu

printf '%s\n' \
  'check-csharp-frontend.sh was retired by JAVA-03-T10; run sudo ./scripts/check-java-frontend.sh' \
  >&2
exit 64
