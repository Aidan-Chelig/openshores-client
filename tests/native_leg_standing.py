"""Pin standing-leg constants and branch anchors to the supplied PE, never execute it."""
import runpy
import struct
import subprocess

context = runpy.run_path("tests/native_dp_connections.py")
read = context["read"]  # Also validates the complete executable SHA-256.
constants = {
    0x1422BFA20: ("<f", 0x3F490FDB),
    0x1422BFA48: ("<f", 0x3FC90FDB),
    0x1422BFB2C: ("<f", 0x40490FDB),
    0x1422BFCC0: ("<f", 0x3F3504F3),
    0x1422BFDD8: ("<d", 0xC000000000000000),
    0x1422BFDE0: ("<d", 0x3FF921FB60000000),
    0x1422BFC94: ("<f", 0x3EB2B8C2),
}
for address, (kind, bits) in constants.items():
    size = struct.calcsize(kind)
    actual = int.from_bytes(read(address, size), "little")
    assert actual == bits, (hex(address), hex(actual), hex(bits))

for symbol, evidence, anchors in [
    ("_ZNK12DpLegAndFoot15CalcStandAnglesEfRfS0_S0_", "native-leg-calc-stand-angles-gnu.txt",
     ["0x409c(%rcx)", "0x4080(%rcx)", "<_ZNK6DpBody10FootLengthEb>",
      "<_ZNK5DhDNA13IsExoskeletalEv>", "<_ZNK5DhDNA11LegVerticesEv>",
      "<atanf>", "<cosf>", "<acosf>", "<asinf>", "divsd", "cvtsd2ss"]),
    ("_ZNK5DhDNA11LegVerticesEv", "native-dna-leg-vertices-gnu.txt",
     ["0x14(%rcx)", "0x1c(%rcx)", "0x9(%rcx)", "div    %ebx"]),
    ("_ZNK5DhDNA13IsExoskeletalEv", "native-dna-is-exoskeletal-gnu.txt",
     ["$0x21", "0x18(%rcx)"]),
    ("_ZN12DpLegAndFoot9RotateHipEf.part.0", "native-leg-hip-gnu.txt",
     ["<_ZNK12DpLegAndFoot17CurrentConnectLocEv>", "<sincos>", "<_ZNK11AuMatrix4x4mlERKS_>"]),
    ("_ZN12DpLegAndFoot10RotateKneeEf.part.0", "native-leg-knee-gnu.txt",
     ["0x50(%rbx)", "0x60(%rsp),%eax", "0x54(%rsp),%edx", "0x58(%rsp),%edx"]),
    ("_ZN12DpLegAndFoot11RotateAnkleEf.part.0", "native-leg-ankle-gnu.txt",
     ["0x68(%rbx)", "0x58(%rsp),%edx", "0x170(%rsp),%eax"]),
    ("_ZN12DpLegAndFoot8SplayHipEf.part.0", "native-leg-splay-gnu.txt",
     ["call   *0xa8(%rax)", "xorps", "<sincos>"]),
    ("_ZN12DpLegAndFoot5StandEf", "native-leg-stand-gnu.txt",
     ["0x48(%r12)", "<_ZNK5DhDNA12HasAnkleBendEv>"]),
    ("_ZN6DpBody9StandLegsEfi", "native-dp-stand-legs-gnu.txt",
     ["<_ZN6DpBody10SpreadLegsEfi>", "1422bfc94"]),
    ("_ZN6DpBody10SpreadLegsEfi", "native-dp-spread-legs-gnu.txt",
     ["0x4c(%rsi)", "0x48(%rsi)", "<_ZNK6DpBody10FootLengthEb>"]),
]:
    assembly = subprocess.check_output(
        ["objdump", "-d", "--disassemble=" + symbol, str(context["target"])], text=True
    )
    retained = (context["Path"]("docs/evidence") / evidence).read_text()
    assert assembly.strip() == retained.strip(), evidence
    for anchor in anchors:
        assert anchor in assembly, (symbol, anchor)
for vptr, implementation, returned in [(0x142425910, 0x141917A10, bytes.fromhex("b801000000c3")),
                                       (0x142434AC0, 0x141907A30, bytes.fromhex("31c0c3"))]:
    assert struct.unpack("<Q", read(vptr + 0xA8, 8))[0] == implementation
    assert read(implementation, len(returned)) == returned
print("Verified standing-leg selectors, constants, IsLeft vtables and retained full disassembly. Static evidence, not native runtime parity.")
