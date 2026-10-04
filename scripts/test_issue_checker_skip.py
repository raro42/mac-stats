#!/usr/bin/env python3
"""Smoke tests for GitHub issue skip rules."""

from __future__ import annotations

import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location(
    "issue_checker", ROOT / "agents" / "issue_checker.py"
)
mod = importlib.util.module_from_spec(spec)
assert spec.loader is not None
spec.loader.exec_module(mod)


def test_skip_feedback_issue_three() -> None:
    issue = {
        "number": 3,
        "labels": [{"name": "help wanted"}, {"name": "question"}],
    }
    assert mod.skip_issue(issue) is not None


def test_keep_bug() -> None:
    issue = {"number": 14, "labels": [{"name": "bug"}]}
    assert mod.skip_issue(issue) is None


def test_skip_help_wanted_without_bug() -> None:
    issue = {"number": 99, "labels": [{"name": "help wanted"}]}
    assert mod.skip_issue(issue) is not None


if __name__ == "__main__":
    test_skip_feedback_issue_three()
    test_keep_bug()
    test_skip_help_wanted_without_bug()
    print("ok")
