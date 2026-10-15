"""Offline GPU captures and nonblank-image checks. No original target execution."""
import argparse
import os
from pathlib import Path
import subprocess
import tempfile


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("installation", nargs="?", default="reference/hazeron")
    args = parser.parse_args()
    installation = Path(args.installation).resolve(strict=True)
    if Path("target").resolve().is_relative_to(installation):
        raise RuntimeError("build/capture output must be outside the installation")
    subprocess.run(["cargo", "build", "--offline", "--features", "viewer", "--bin", "openshores-client"], check=True)
    directory = Path(tempfile.mkdtemp(prefix="avatar-textures-", dir="target")).resolve()
    if directory.is_relative_to(installation):
        raise RuntimeError("capture output must be outside the installation")
    print(f"Local captures (not redistributable game assets): {directory}", flush=True)
    for part in ["head", "torso"]:
        print(f"Rendering {part}; allow cold shader compilation to finish", flush=True)
        result = subprocess.run([
            "target/debug/openshores-client", "view-native-body", str(installation), part,
            "--smoke-frames", "1000", "--screenshot", str(directory / f"{part}.png"),
        ], text=True, capture_output=True, timeout=120)
        print(result.stdout, end="", flush=True)
        print(result.stderr, end="", flush=True)
        result.check_returncode()
        if "ERROR" in result.stdout + result.stderr:
            raise RuntimeError(f"{part}: renderer reported an error")
        if not (directory / f"{part}.png").is_file():
            raise RuntimeError(f"{part}: no GPU screenshot was produced")
    environment = dict(os.environ, OPENSHORES_AVATAR_SCREENSHOT_DIR=str(directory))
    subprocess.run(["cargo", "test", "--offline", "--features", "viewer", "--lib",
                    "captured_previews_are_nonblank", "--", "--ignored", "--nocapture"],
                   env=environment, check=True)
    print(f"GPU smoke captures passed. Inspect texture placement in {directory}.")


if __name__ == "__main__":
    main()
