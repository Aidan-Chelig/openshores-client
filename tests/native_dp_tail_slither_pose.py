"""Static tail pose-Slither masks/constants and body composition anchors."""
import runpy
import struct
import subprocess

c = runpy.run_path("tests/native_dp_connections.py")
read = c["read"]
for address, bits in [(0x1422BFC84, 0x3F1C61AA), (0x1422BFC78, 0x3F060A92),
                      (0x1422BFB2C, 0x40490FDB)]:
    assert struct.unpack("<I", read(address, 4))[0] == bits
for method in ["_ZN6DpTail7SlitherERK6AuPosef", "_ZN6DpBody11SlitherTailERK6AuPosef"]:
    asm = subprocess.check_output(["objdump", "-d", "--no-show-raw-insn", "--disassemble=" + method, str(c["target"])], text=True)
    for anchor in ["$0x2000000000085", "$0x202c0000000000", "$0x30000",
                   "<_ZNK5DhDNA6RhythmERK6AuPoseff>", "<_ZN6DpTail7SlitherEff>", "ucomiss"]:
        assert anchor in asm, (method, anchor)
    assert "cmp    $0x4b,%eax" in asm
# The low-pose guards add17/22/24/25. The masks cover only26..75.
masked = [p for p in range(26, 76) if (1 << (p-26)) & (0x2000000000085 | 0x1800 | 0x30000)]
assert masked == [26, 28, 33, 37, 38, 42, 43, 75]
assert [p for p in range(54) if (1 << p) & 0x202C0000000000] == [42, 43, 45, 53]
print("Verified tail/body pose-Slither selection masks, float constants and rhythm/concrete call anchors. Static checks, not native execution.")
