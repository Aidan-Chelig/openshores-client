"""Retain complete static REA evidence without flooding the terminal.

Requires an existing REA launcher and Ghidra/JDK environment; never executes
the selected target. Output must be a new file (existing evidence is preserved).
"""

import argparse
import json
import os
from pathlib import Path
import subprocess


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("launcher", type=Path)
    parser.add_argument("target", type=Path)
    parser.add_argument("procedure")
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    environment = os.environ.copy()
    environment["REA_GHIDRA_STARTUP_TIMEOUT_MS"] = "1800000"
    with args.output.open("xb") as evidence:
        result = subprocess.run(
            [str(args.launcher.resolve()), "decompile", str(args.target.resolve()),
             "--procedure", args.procedure, "--provider", "ghidra",
             "--format", "json", "--full-output"],
            env=environment, stdout=evidence, check=False,
        )
    print(f"REA exit={result.returncode}; retained output: {args.output}")
    if result.returncode:
        raise SystemExit(result.returncode)
    with args.output.open() as evidence:
        record = json.load(evidence)
    print("Complete JSON validated; envelope keys:", ", ".join(record))


if __name__ == "__main__":
    main()
