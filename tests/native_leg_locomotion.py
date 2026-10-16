"""SHA-bound Walk/Run leg constants and consumers, never execute the target."""
import struct
import subprocess
from native_dp_connections import read, target

def assembly(symbol):
    return subprocess.check_output(["objdump", "-d", "--disassemble=" + symbol, str(target)], text=True)

floats = {
    0x1422BFA40: 0x3DB2B8C2, 0x1422BFC90: 0x3C8EFA35,
    0x1422BFB2C: 0x40490FDB, 0x1422BFA48: 0x3FC90FDB,
    0x1422BFCF8: 0x40A78D36, 0x1422BFCFC: 0x40EA9280,
    0x1422BFD00: 0x40C90FDB, 0x1422BFCF0: 0x40060A92,
    0x1422BFCF4: 0x40860A92,
}
for address, bits in floats.items():
    assert struct.unpack("<I", read(address, 4))[0] == bits
for address, value in {
    0x1422BFE10: -3., 0x1422BFDC8: 10., 0x1422BFE08: 40.,
    0x1422BFD48: float(struct.unpack("<f", struct.pack("<I", 0x3C8EFA35))[0]),
    0x1422BFE18: 0.8, 0x1422BF9D8: 0.5,
    0x1422BF8F0: 3., 0x1422BFE20: 6.,
}.items():
    assert struct.unpack("<d", read(address, 8))[0] == value

for verb, size, pose, start, end in [("Walk", 4, "26", 0x1412DAC60, 0x1412DAF60),
                                      ("Run", 3, "25", 0x1412DA2B0, 0x1412DA660)]:
    code = assembly(f"_ZN12DpLegAndFoot{size}{verb}Eff")
    assert f"{start:x}" in code and f"{end:x}" not in code
    assert f"$0x{pose}," in code and "DhDNA6Rhythm" in code
    assert "CalcStandAngles" in code and "HasAnkleBend" in code
    order = [code.index(name) for name in ["RotateHip", "RotateKnee", "RotateAnkle", "SplayHip"]]
    assert order == sorted(order)
    assert "call   *0xa8(%rax)" in code  # concrete IsLeft
    assert "sinf" in code and "cvtss2sd" in code and "cvtsd2ss" in code
    owner = assembly(f"_ZN6DpBody{size + 4}{verb}LegsEfif")
    assert "SpreadLegs" in owner and f"DpLegAndFoot{size}{verb}" in owner
    assert "0x4c(%rcx)" in owner and "cmp    $0x2,%edi" in owner
assert struct.unpack("<Q", read(0x142425910 + 0xA8, 8))[0] == 0x141917A10
assert struct.unpack("<Q", read(0x142434AC0 + 0xA8, 8))[0] == 0x141907A30
assert "mov    $0x1,%eax" in assembly("_ZNK16DpLeftLegAndFoot6IsLeftEv")
assert "xor    %eax,%eax" in assembly("_ZNK12DpLegAndFoot6IsLeftEv")
print("Verified native leg gait literals, rhythm IDs, side bindings, selector dispatch and joint call order. Static evidence, not runtime visual parity.")
