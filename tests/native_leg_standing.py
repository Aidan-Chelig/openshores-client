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
]:
    assembly = subprocess.check_output(
        ["objdump", "-d", "--disassemble=" + symbol, str(context["target"])], text=True
    )
    retained = (context["Path"]("docs/evidence") / evidence).read_text()
    assert assembly.strip() == retained.strip(), evidence
    for anchor in anchors:
        assert anchor in assembly, (symbol, anchor)
print("Verified standing-leg selectors, constants and retained full disassembly. Static evidence, not native runtime parity.")
