"""Bounded offline animation regressions; no original runtime or new analysis."""
from pathlib import Path
import os
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
sys.dont_write_bytecode = True

def run(*command):
    subprocess.run(command, cwd=ROOT, check=True)

for check in (
    "native_pose_rhythm.py", "native_leg_locomotion.py",
    "native_arm_locomotion.py", "native_dead_pose.py", "native_wave_pose.py", "native_jump_pose.py", "native_fold_pose.py", "native_point_pose.py",
    "native_dp_height.py", "native_dp_position_tail.py",
    "native_dp_position_torso.py", "native_dp_position_heads.py", "native_head_tilt.py", "native_crouch_pose.py",
):
    run(sys.executable, str(ROOT / "tests" / check))
run("cargo", "test", "--offline", "--features", "client", "--lib", "--quiet")
run(sys.executable, str(ROOT / "tests/local_avatar_preparation.py"),
    sys.argv[1] if len(sys.argv) > 1 else "reference/hazeron")
environment = dict(os.environ, OPENSHORES_HAZERON_INSTALLATION=(
    sys.argv[1] if len(sys.argv) > 1 else "reference/hazeron"))
subprocess.run(["cargo", "test", "--offline", "--test", "native_body_mounted_standing",
                "--", "--ignored"], cwd=ROOT, env=environment, check=True)
print("Offline avatar animation anchors, library and mounted received-DNA checks passed.", flush=True)
