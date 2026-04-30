#!/usr/bin/env python3

from __future__ import annotations

from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
REQUIRED_FILES = [
    "AGENTS.md",
    "README.md",
    "Cargo.toml",
    "src/lib.rs",
]
ALLOWED_WORKFLOWS = {"guard.yml"}


def main() -> int:
    violations: list[str] = []

    for relative_path in REQUIRED_FILES:
        if not (ROOT / relative_path).is_file():
            violations.append(f"missing {relative_path}")

    workflows_dir = ROOT / ".github" / "workflows"
    if workflows_dir.is_dir():
        for workflow_path in workflows_dir.iterdir():
            if workflow_path.is_file() and workflow_path.name not in ALLOWED_WORKFLOWS:
                violations.append(
                    f"unexpected workflow {workflow_path.relative_to(ROOT)}"
                )

    cargo_toml = ROOT / "Cargo.toml"
    if cargo_toml.is_file():
        cargo_text = cargo_toml.read_text(encoding="utf-8")
        if "../stim-proto/crates/stim-proto" not in cargo_text:
            violations.append(
                "Cargo.toml must keep stim-proto as the explicit shared contract dependency"
            )

    if violations:
        print("guard failed: server boundary is not controllable.\n")
        for violation in violations:
            print(f"- {violation}")
        return 1

    print("guard passed: server boundary baseline is controllable.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
