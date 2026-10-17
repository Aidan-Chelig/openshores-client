"""SHA-bound crouch predicate and shared limb dispatch; no target execution."""
import struct
import subprocess
from native_dp_connections import read, target

def assembly(symbol):
    return subprocess.check_output(["objdump", "-d", "--disassemble=" + symbol, str(target)], text=True)

predicate = assembly("_ZNK10DaCreature10IsCrouchedEv")
for anchor in ("mov    0x48(%rax),%eax", "mov    0x6c(%rax),%eax",
               "sub    $0x10,%eax", "cmp    $0x1,%eax", "setbe  %r13b"):
    assert anchor in predicate, anchor

def destination(table, pose):
    return table + struct.unpack("<i", read(table + 4 * pose, 4))[0]

# Both leg selectors and both arm sides, not merely one first switch entry.
for table, bias in ((0x1422BF404, 0), (0x1422BF534, 4), (0x1422BF080, 0), (0x1422BF1B0, 0)):
    assert destination(table, 0x10 - bias) == destination(table, 0x24 - bias)
    assert destination(table, 0x11 - bias) == destination(table, 0x26 - bias)
    assert destination(table, 0x10 - bias) != destination(table, 0x11 - bias)
legs = assembly("_ZN6DpBody12PositionLegsEff")
arms = assembly("_ZN6DpBody12PositionArmsEff")
for anchor in ("1422bf404", "1422bf534", "StandLegs", "WalkLegs", "0x4c(%rcx)", "0x3c(%r12)", "sub    $0x4,%ecx"):
    assert anchor in legs, anchor
for anchor in ("1422bf080", "1422bf1b0", "StandArms", "WalkArms", "0x44(%rcx)", "0x50(%r12)"):
    assert anchor in arms, anchor
print("Verified Crouched means0x10/0x11 and both leg/arm switches share Stand/Walk stages. Static evidence only.")
