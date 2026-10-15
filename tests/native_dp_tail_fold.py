"""Source-bound tail Fold float/root constants and body call; static only."""
import runpy
import struct
import subprocess

c = runpy.run_path("tests/native_dp_connections.py")
read = c["read"]
assert struct.unpack("<I", read(0x1422BFA20, 4))[0] == 0x3F490FDB
for address, value in [(0x1422BFC60, -4.371139000186241e-08),
                       (0x1422BFC68, 0.999999999999999),
                       (0x1422BFC70, -0.999999999999999)]:
    assert struct.unpack("<d", read(address, 8))[0] == value

def assembly(name):
    return subprocess.check_output(["objdump", "-d", "--no-show-raw-insn", "--disassemble="+name, str(c["target"])], text=True)

fold = assembly("_ZN6DpTail4FoldEv")
for anchor in ["divss", "cmpb   $0x0,0x3a(%rax)", "cmp    $0x11,%ebp",
               "cmp    $0x10,%ebp", "<_ZNK6DpTail16CanBendLaterallyEv>", "<sincos>"]:
    assert anchor in fold, anchor
body = assembly("_ZN6DpBody8FoldTailEv")
assert "call   *0x50(%rax)" in body and "<_ZN6DpTail4FoldEv>" in body
print("Verified native fold angle/root constants, owner byte3a, descending bounds and concrete body dispatch. Static anchors, not runtime parity.")
