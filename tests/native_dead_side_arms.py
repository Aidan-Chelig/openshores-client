"""SHA-bound side-facing dead arm stages; no original target execution."""
import struct
import subprocess
from native_dp_connections import read, target

for address, bits in {
    0x1422BFC84: 0x3F1C61AA, 0x1422BFD80: 0x41000000,
    0x1422BFD84: 0x42960000, 0x1422BFC90: 0x3C8EFA35,
    0x1422BFC7C: 0x3E860A92, 0x1422BFD30: 0xBD0EFA35,
    0x1422BFD8C: 0x3F91361E, 0x1422BFD90: 0xBEB2B8C2,
    0x1422BFB10: 0x41200000, 0x1422BFD88: 0x42FA0000,
    0x1422BFCC8: 0xBEDF66F3, 0x1422BFC80: 0x3E32B8C2,
}.items():
    assert struct.unpack("<I", read(address, 4))[0] == bits

def assembly(symbol):
    return subprocess.check_output(
        ["objdump", "-d", "--disassemble=" + symbol, str(target)], text=True)

for facing, length in [("Left", 18), ("Right", 19)]:
    body = assembly(f"_ZN6DpBody{length}DeadArmsFacing{facing}Ei")
    assert "1422bfc84" in body and "pxor   %xmm1,%xmm1" in body
    assert body.index("SpreadArms") < body.index("DeadFacing" + facing)
    assert "add    $0x8,%rbx" in body and "call   *0xa8(%rax)" in body
    assert "testb  $0x3,0x19(%r12)" in body and "testb  $0x6,0x1a(%r12)" in body
    part = assembly(f"_ZN12DpArmAndHand{length - 4}DeadFacing{facing}Ev")
    assert "cvtsi2ssl 0x50(%r12),%xmm1" in part
    assert "call   *0xa8(%rax)" in part
    for address in ["1422bfd80", "1422bfd84", "1422bfb10", "1422bfd88",
                    "1422bfc7c", "1422bfd30", "1422bfcc8", "1422bfd90"]:
        assert address in part
    assert part.count("HasWristBend") == 2
    assert part.index("RotateShoulder") < part.index("RotateWrist") < part.index("RotateElbow") < part.index("LiftShoulder") < part.index("SplayShoulder")
    # Left and Right reverse the first IsLeft branch, not the joint constants.
    branch = part[part.index("test   %al,%al"):].splitlines()[1]
    assert ("je " if facing == "Left" else "jne ") in branch
print("Verified side-facing dead arm constants, signed pair index, reversed side branch, wrist gates, traversal and joint order. Static evidence only.")
