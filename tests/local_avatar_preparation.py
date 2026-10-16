"""Read supplied-server default DNA and test mounted CPU preparation; no sockets."""
import os
from pathlib import Path
import subprocess
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "reference/OpenShores-server/src"))
from openshores.protocol.login_reply import _default_dna

if __name__ == "__main__":
    installation = sys.argv[1] if len(sys.argv) > 1 else "reference/hazeron"
    environment = dict(os.environ,
                       OPENSHORES_HAZERON_INSTALLATION=installation,
                       OPENSHORES_TEST_PLAYER_DNA_HEX=_default_dna().hex())
    subprocess.run(["cargo", "test", "--offline", "--test", "local_avatar_preparation",
                    "--", "--include-ignored", "--nocapture"],
                   cwd=ROOT, env=environment, check=True)
