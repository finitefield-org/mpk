#!/usr/bin/env python3
"""Stable entry point for the active successor release report."""

from __future__ import annotations

import sys

from successor_release_report import ReleaseReportError, main


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except ReleaseReportError as error:
        print(f"release report generation failed: {error}", file=sys.stderr)
        raise SystemExit(1)
