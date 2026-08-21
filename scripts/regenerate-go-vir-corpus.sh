#!/bin/sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$repo_root"

mode=${1:---check}

case "$mode" in
  --update)
    (
      cd go-tools/go2vir
      MPK_UPDATE_GO_VIR_CORPUS=1 go test -count=1 -run TestRegenerateGoVIRFrontendCorpus
    )
    MPK_UPDATE_GO_VIR_CORPUS=1 cargo test -p mpk-vc --test go_vir_corpus
    python3 scripts/generate-release-report.py --check
    ;;
  --check)
    (
      cd go-tools/go2vir
      go test -count=1 -run TestRegenerateGoVIRFrontendCorpus
    )
    cargo test -p mpk-vc --test go_vir_corpus
    python3 scripts/generate-release-report.py --check
    ;;
  *)
    echo "usage: $0 [--check|--update]" >&2
    exit 2
    ;;
esac
