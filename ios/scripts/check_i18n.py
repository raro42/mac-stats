#!/usr/bin/env python3
"""Checks that every user-visible string in the iOS app goes through src/i18n.

Fails (exit 1) when:
- web code outside src/i18n/ has Spanish-only characters or assigns a quoted literal to
  textContent / placeholder / aria-label / title;
- index.html has a data-i18n* key that is not in src/i18n/messages.ts;
- a Swift error code is not in src/i18n/error-codes.ts;
- Swift sources, or Rust sources outside tests and benchmark/self-test data, contain
  Spanish-only characters.

`tsc` already checks that every dictionary has every key; this script catches text that
never reaches a dictionary. Run from anywhere: python3 ios/scripts/check_i18n.py
"""

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SPANISH = re.compile(r"[áéíóúñÁÉÍÓÚÑ¿¡«»]")
LITERAL_ASSIGN = re.compile(
    r"""(textContent|placeholder|innerText)\s*=\s*["'`][^"'`]*[A-Za-z]"""
    r"""|setAttribute\(\s*["'](aria-label|title|placeholder)["']\s*,\s*["'`]"""
)
# Rust files whose Spanish text is test data (benchmark and self-test prompts).
RUST_DATA_FILES = {"lab.rs", "selftest.rs"}

problems: list[str] = []


def report(path: Path, line_no: int, message: str, line: str) -> None:
    problems.append(f"{path.relative_to(ROOT)}:{line_no}: {message}: {line.strip()}")


def check_web() -> None:
    files = [ROOT / "index.html"] + sorted((ROOT / "src").rglob("*.ts"))
    for path in files:
        if "i18n" in path.relative_to(ROOT / "src").parts if path.suffix == ".ts" else False:
            continue
        for no, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
            if line.strip().startswith("//"):
                continue
            if SPANISH.search(line):
                report(path, no, "Spanish text outside src/i18n", line)
            if path.suffix == ".ts" and LITERAL_ASSIGN.search(line):
                report(path, no, "hard-coded UI text (use t())", line)


def check_html_keys() -> None:
    messages = (ROOT / "src/i18n/messages.ts").read_text(encoding="utf-8")
    keys = set(re.findall(r'^\s*"([\w.]+)":', messages, re.M))
    html = ROOT / "index.html"
    for no, line in enumerate(html.read_text(encoding="utf-8").splitlines(), 1):
        for key in re.findall(r'data-i18n(?:-[\w-]+)?="([^"]+)"', line):
            if key not in keys:
                report(html, no, f"unknown i18n key '{key}'", line)


def error_codes() -> set[str]:
    source = (ROOT / "src/i18n/error-codes.ts").read_text(encoding="utf-8")
    block = source.split("ERROR_CODES = [", 1)[1].split("]", 1)[0]
    return set(re.findall(r'"([a-z_]+)"', block))


def check_swift() -> None:
    codes = error_codes()
    swift_dir = ROOT / "plugins/tauri-plugin-llm/ios/tauri-plugin-llm"
    for path in sorted(swift_dir.glob("*.swift")):
        text = path.read_text(encoding="utf-8")
        in_code = False
        for no, line in enumerate(text.splitlines(), 1):
            if SPANISH.search(line) and not line.strip().startswith("//"):
                report(path, no, "Spanish text in Swift", line)
            if re.search(r"var code: String", line):
                in_code = True
            elif in_code and line.strip() == "}":
                in_code = False
            found = re.findall(r'return "([a-z_]+)"', line) if in_code or "func errorCode" in text else []
            found += re.findall(r'code: "([a-z_]+)"', line)
            for code in found:
                if code not in codes:
                    report(path, no, f"error code '{code}' missing from error-codes.ts", line)


def check_rust() -> None:
    for path in sorted((ROOT / "src-tauri/src").rglob("*.rs")):
        if path.name in RUST_DATA_FILES:
            continue
        in_tests = False
        for no, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
            if line.startswith("#[cfg(test)]"):
                in_tests = True
            stripped = line.strip()
            if in_tests or stripped.startswith("//"):
                continue
            if SPANISH.search(line):
                report(path, no, "Spanish text in Rust", line)


def main() -> int:
    check_web()
    check_html_keys()
    check_swift()
    check_rust()
    if problems:
        print("\n".join(problems))
        print(f"\n{len(problems)} i18n problem(s).")
        return 1
    print("i18n check passed.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
