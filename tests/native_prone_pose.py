"""SHA-bound stationary prone dispatch and arms; no target execution."""
import struct
import subprocess
from native_dp_connections import read, target

def assembly(symbol):
    return subprocess.check_output(["objdump", "-d", "--disassemble=" + symbol, str(target)], text=True)

for address, bits in {
    0x1422BFCDC: 0x3FB2B8C2, 0x1422BFC80: 0x3E32B8C2,
    0x1422BFDBC: 0xC0384E89, 0x1422BFDA0: 0x40384E88,
    0x1422BFDA4: 0x40384E89, 0x1422BFD30: 0xBD0EFA35,
}.items():
    assert struct.unpack("<I", read(address, 4))[0] == bits
for table, bias in [(0x1422BF404, 0), (0x1422BF534, 4)]:
    def destination(pose):
        return table + struct.unpack("<i", read(table + (pose - bias) * 4, 4))[0]
    assert destination(0x20) == destination(0x12)
arms = assembly("_ZN6DpBody9ProneArmsEib")
for anchor in ["SpreadArms", "1422bfcdc", "1422bfc80", "1422bfdbc", "1422bfda0",
               "1422bfda4", "1422bfd30", "addss  0x30(%rax),%xmm6",
               "call   *0xa8(%rax)", "add    $0x8,%rbx", "xor    %ebp,%ebp"]:
    assert anchor in arms
assert arms.index("RotateShoulder") < arms.index("RotateElbow") < arms.index("SplayShoulder") < arms.index("LiftShoulder")
assert "ProneArms" in assembly("_ZN6DpBody12PositionArmsEff")
for table, destination in [(0x1422BF080, 0x1412EE6B0), (0x1422BF1B0, 0x1412EEA00)]:
    assert table + struct.unpack("<i", read(table + 0x20 * 4, 4))[0] == destination
    branch = subprocess.check_output(["objdump", "-d", "-C", f"--start-address={destination:#x}",
                                      f"--stop-address={destination + 0x38:#x}", str(target)], text=True)
    assert "DpBody::ProneArms(int, bool)" in branch
    assert "xor    %r8d,%r8d" in branch and "cmpb   $0x0,0x6c(%r12)" in branch
table = 0x1421C664C
assert table + struct.unpack("<i", read(table + (0x20 - 4) * 4, 4))[0] == 0x1409808E7
table = 0x1421C5E34
assert table + struct.unpack("<i", read(table + (0x20 - 3) * 4, 4))[0] == 0x140978BE7
offset = subprocess.check_output(["objdump", "-d", "-C", "--start-address=0x140978BE7",
                                  "--stop-address=0x140978C2E", str(target)], text=True)
assert "DpBody::HeadLocation() const" in offset and "xorpd" in offset
print("Verified prone shared death-leg dispatch, arm spacing, fixed joint literals/order, first-selected pitch override and no render rotation. Static evidence only.")
