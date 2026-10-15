"""Static BowWing mirrored side, sequential pivots and output writes."""
import json
import runpy
import struct
import subprocess
from pathlib import Path

c = runpy.run_path("tests/native_dp_connections.py")
runpy.run_path("tests/native_dp_body_waist.py")
def assembly(symbol):
    return subprocess.check_output(["objdump", "-d", "--no-show-raw-insn", "--disassemble=" + symbol, str(c["target"])], text=True)

for vptr, expected in [(0x142425850, "b801000000c3"), (0x142434a00, "31c0c3")]:
    is_left = struct.unpack("<Q", c["read"](vptr + 0xa8, 8))[0]
    assert c["read"](is_left, len(bytes.fromhex(expected))) == bytes.fromhex(expected)
wrapper = assembly("_ZN12DpArmAndHand7BowWingEf")
for anchor in ["ucomiss", "jp ", "je ", "<_ZN12DpArmAndHand7BowWingEf.part.0>"]:
    assert anchor in wrapper, anchor
concrete = assembly("_ZN12DpArmAndHand7BowWingEf.part.0")
assert concrete.count("call   *0xa0(%rax)") == 3
assert concrete.count("call   *0xa8(%rax)") == 1
assert concrete.count("<_ZNK11AuMatrix4x4mlERKS_>") == 3
assert concrete.count("<_ZN11AuMatrix4x47RotateZEd>") == 2
assert concrete.count("<_ZNK11AuMatrix4x47ProjectEddd>") == 1
for anchor in ["xorps", "0x58(%rbx),%xmm2", "0x60(%rbx),%xmm4", "0x68(%rbx),%xmm3", "mov    0x60(%rsp),%eax", "mov    0x4c(%rsp),%edx", "mov    0x50(%rsp),%edx", "0x70(%rbx),%xmm2", "0x78(%rbx),%xmm3", "0x80(%rbx),%xmm0", "mov    0x5c(%rsp),%edx"]:
    assert anchor in concrete, anchor
products = [concrete.index("1412bf1ba:"), concrete.index("1412bf281:"), concrete.index("1412bf4e7:")]
project = concrete.index("1412bf379:")
assert products[0] < products[1] < project < products[2]
for anchor in ["addsd  %xmm1,%xmm12", "addsd  %xmm11,%xmm6", "addsd  %xmm5,%xmm9", "addsd  %xmm2,%xmm0", "addsd  0xd0(%rax),%xmm12", "addsd  0xe8(%rax),%xmm0", "divsd  %xmm0,%xmm12", "addpd  %xmm2,%xmm0", "addpd  %xmm3,%xmm2"]:
    assert anchor in concrete, anchor
for name in ["native-dp-arm-bow-wing", "native-dp-arm-bow-wing-part0"]:
    evidence = json.loads(Path("docs/evidence", name + ".json").read_text())
    assert evidence["subject"]["digest"]["sha256"] == "285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80"
print("Verified BowWing side vtables/sign, zero/NaN guard, inlined field58 pivot, output2/3 writes, subsequent field70 Project and output3-only second stage. Static checks only.")
