"""Concrete tail type, capability masks and Slither loop anchors; static only."""
import runpy
import struct
import subprocess

c = runpy.run_path("tests/native_dp_connections.py")
read = c["read"]
for vptr, _, _ in c["bindings"]:
    method = struct.unpack("<Q", read(vptr + 0x50, 8))[0]
    if vptr == 0x14247F2B0:
        assert read(method, 6) == bytes.fromhex("b8 01 00 00 00 c3")
    else:
        assert read(method, 3) == bytes.fromhex("31 c0 c3")
assert struct.unpack("<f", read(0x1422BF8A0, 4))[0] == 0.5
assert struct.unpack("<I", read(0x1422BFC7C, 4))[0] == 0x3E860A92

def assembly(name):
    return subprocess.check_output(["objdump", "-d", "--no-show-raw-insn", "--disassemble=" + name, str(c["target"])], text=True)

lateral = assembly("_ZNK6DpTail16CanBendLaterallyEv")
assert "test   $0x54,%dl" in lateral and "and    $0x88,%edx" in lateral
vertical = assembly("_ZNK6DpTail17CanBendVerticallyEv")
assert "cmp    $0x6,%al" in vertical and "cmp    $0x7,%al" in vertical
slither = assembly("_ZN6DpTail7SlitherEff")
for anchor in ["add    $0x21,%eax", "shr    $1,%eax", "cmp    $0x11,%r14d",
               "cmp    $0x10,%r14d", "<sincos>", "xorps", "mulss"]:
    assert anchor in slither, anchor
for name in ["_ZN6DpBody18DeadTailFacingDownEv", "_ZN6DpBody18DeadTailFacingLeftEv"]:
    body = assembly(name)
    assert "call   *0x50(%rax)" in body and "<_ZN6DpTail7SlitherEff>" in body
print("Verified concrete tail tests, capability masks, Slither reversal/root-half stages and both death-tail dispatchers. Static anchors, not native runtime parity.")
