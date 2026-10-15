"""Verify native object group's full17-entry dispatch and metadata writes."""
import runpy
import struct
import subprocess

c = runpy.run_path("tests/native_dp_connections.py")
assembly = subprocess.check_output(["objdump", "-d", "--no-show-raw-insn", "--disassemble=_ZN15Au3dsGLSLObject13LoadMeshChunkER11QDataStreamiR7AuPointS3_10MODEL_TYPE", str(c["target"])], text=True)
table = 0x1422cf1e0
cases = {4:0x1413a5b92, 10:0x1413a5b7e, 11:0x1413a5b6a, 12:0x1413a5b56, 13:0x1413a5b42, 14:0x1413a5b2e, 19:0x1413a5b1a, 20:0x1413a5b06}
for kind, relative in enumerate(struct.unpack("<17i", c["read"](table, 68)), 4):
    assert table + relative == cases.get(kind, 0x1413a5af0), kind
for anchor in ["cmp    $0x3,%edx", "je     1413a5a10", "cmp    $0x1,%edx", "movb   $0x1,0x3a9(%r15)", "<_ZN7QVectorI15Au3dsGLSLVGroupE6appendERKS0_>"]:
    assert anchor in assembly, anchor
for offset in [0x1cc,0x200,0x234,0x304,0x268,0x29c,0x2d0,0x3ac,0x3e0]:
    assert f"lea    0x{offset:x}(%r15),%rcx" in assembly, hex(offset)
midpoint = assembly[assembly.index("1413a5b92:"):assembly.index("1413a5bb5:")]
assert "<_ZN15Au3dsGLSLVertexaSERKS_.isra.0>" in midpoint
assert "<_ZN7QVectorI15Au3dsGLSLVGroupE6appendERKS0_>" in midpoint
assert "jmp " not in assembly[assembly.index("1413a5b92:"):assembly.index("1413a5ba1:")] # MidPt falls through to retained append.
print("Verified all17 object group switch entries, separate ConnectPt case, nine metadata destinations, MidPt append exception and kind1 flag write. Static checks only.")
