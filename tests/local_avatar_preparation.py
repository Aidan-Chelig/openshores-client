"""Read supplied-server default DNA and test mounted CPU preparation; no sockets."""
import os
from pathlib import Path
import subprocess
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "reference/OpenShores-server/src"))
from openshores.protocol.login_reply import _default_dna


def _measure(installation: str, dna: bytes | None = None) -> str:
    """Ask the Rust received-DNA pipeline; never duplicate its clearance formula."""
    environment = dict(os.environ,
                       OPENSHORES_HAZERON_INSTALLATION=installation,
                       OPENSHORES_TEST_PLAYER_DNA_HEX=(dna if dna is not None else _default_dna()).hex())
    result = subprocess.run([
        "cargo", "test", "--offline", "--test", "local_avatar_preparation",
        "mounted_received_server_dna", "--", "--include-ignored", "--nocapture"],
        cwd=ROOT, env=environment, check=True, text=True, stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT)
    print(result.stdout, end="", flush=True)
    return result.stdout


def anchor_metrics(installation: str, dna: bytes | None = None) -> tuple[float, float, float]:
    metrics = [line.split()[1:] for line in _measure(installation, dna).splitlines()
               if line.startswith("AVATAR_ANCHOR_METRICS ")]
    if len(metrics) != 1 or len(metrics[0]) != 3:
        raise RuntimeError("Rust avatar clearance measurement missing or ambiguous")
    return tuple(map(float, metrics[0]))


def posture_metrics(installation: str, dna: bytes | None = None) -> tuple[float, float]:
    """Native adult Stand/Crouch CGHeight(true), never a Python height formula."""
    metrics = [line.split()[1:] for line in _measure(installation, dna).splitlines()
               if line.startswith("AVATAR_POSTURE_METRICS ")]
    if len(metrics) != 1 or len(metrics[0]) != 2:
        raise RuntimeError("Rust avatar posture clearances missing or ambiguous")
    return tuple(map(float, metrics[0]))

if __name__ == "__main__":
    installation = sys.argv[1] if len(sys.argv) > 1 else "reference/hazeron"
    environment = dict(os.environ,
                       OPENSHORES_HAZERON_INSTALLATION=installation,
                       OPENSHORES_TEST_PLAYER_DNA_HEX=_default_dna().hex())
    subprocess.run(["cargo", "test", "--offline", "--test", "local_avatar_preparation",
                    "--", "--include-ignored", "--nocapture"],
                   cwd=ROOT, env=environment, check=True)
