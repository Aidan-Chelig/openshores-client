"""Standing-arm constants and complete static records, never native execution."""
import runpy
import struct
import subprocess

context = runpy.run_path("tests/native_dp_connections.py")
read = context["read"]
for address, bits, size in [
    (0x1422BFC90, 0x3C8EFA35, 4), (0x1422BFCA0, 0x3E567750, 4),
    (0x1422BFC94, 0x3EB2B8C2, 4), (0x1422BFC08, 0x3F733333, 4),
    (0x1422BFCB0, 0xBEE4F8B588E368F1, 8), (0x1422BFCB8, 0x4016CBE4, 4),
    (0x1422BFCBC, 0x3D0EFA35, 4), (0x1422BFA50, 0xBFC90FDB, 4),
    (0x1422BFD30, 0xBD0EFA35, 4), (0x1422BF8E8, 0x40400000, 4),
    (0x1422BFB0C, 0x3E800000, 4), (0x1422BFDDC, 0xC0000000, 4),
]:
    assert int.from_bytes(read(address, size), "little") == bits, hex(address)

for symbol, evidence, anchors in [
    ("_ZNK12DpArmAndHand15CalcStandAnglesEfRfS0_S0_", "native-arm-calc-stand-angles-gnu.txt",
     ["0x4088(%rcx)", "0x40b4(%rcx)", "<_ZNK6DpBody10HandLengthEb>",
      "0xe0(%rax)", "0xe8(%rax)", "0x88(%rax)", "0xa8(%rax)", "0xc8(%rax)",
      "1422bfcb0", "1422bfddc", "<acosf>", "<asinf>"]),
    ("_ZN6DpBody10SpreadArmsEffi", "native-dp-spread-arms-gnu.txt",
     ["0x1a(%rdx)", "call   *0xa8(%rax)", "movlps %xmm0,0x48(%rdi)", "1422bf8e8", "1422bfb0c"]),
    ("_ZN6DpBody9StandArmsEfib", "native-dp-stand-arms-gnu.txt",
     ["<_ZNK5DhDNA12WalksOnHandsEv>", "<_ZN6DpBody10SpreadArmsEffi>", "<_ZN12DpArmAndHand5RelaxEfb>", "xor    %ebp,%ebp"]),
    ("_ZN12DpArmAndHand5RelaxEfb", "native-arm-relax-gnu.txt",
     ["<_ZNK12DpArmAndHand15CalcStandAnglesEfRfS0_S0_>", "0x30(%rax)", "0x4c(%r12)", "0x48(%r12)"]),
    ("_ZN12DpArmAndHand12LiftShoulderEf", "native-arm-lift-gnu.txt",
     ["call   *0xa0(%rax)", "call   *0xa8(%rax)", "call   *0x30(%rax)", "<sincos>", "mulpd", "<_ZNK11AuMatrix4x4mlERKS_>"]),
    ("_ZN12DpArmAndHand13SplayShoulderEf.part.0", "native-arm-splay-gnu.txt",
     ["call   *0xa0(%rax)", "call   *0xa8(%rax)", "call   *0x30(%rax)", "<sincos>", "0xac(%rsp),%edx", "<_ZNK11AuMatrix4x4mlERKS_>"]),
]:
    assembly = subprocess.check_output(["objdump", "-d", "--disassemble=" + symbol, str(context["target"])], text=True)
    assert assembly.strip() == (context["Path"]("docs/evidence") / evidence).read_text().strip()
    for anchor in anchors:
        assert anchor in assembly, (symbol, anchor)

for vptr, returned in [(0x142425850, bytes.fromhex("b801000000c3")),
                       (0x142434A00, bytes.fromhex("31c0c3"))]:
    implementation = struct.unpack("<Q", read(vptr + 0xA8, 8))[0]
    assert read(implementation, len(returned)) == returned
print("Verified standing-arm constants, concrete side selection and complete static records, including shoulder lift/splay and Relax/StandArms composition anchors.")
