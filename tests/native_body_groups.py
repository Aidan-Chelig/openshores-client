"""Static body-group defaults, point-record rules and position lookup anchors."""
import hashlib
import json
import subprocess
from pathlib import Path

target = Path("reference/openshores-full/OpenShores.exe")
digest = hashlib.sha256(target.read_bytes()).hexdigest()
assert digest == "285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80"
def assembly(symbol):
    return subprocess.check_output(["objdump", "-d", "--no-show-raw-insn", "--disassemble=" + symbol, str(target)], text=True)

members = assembly("_ZN15Au3dsGLSLVGroup20LoadVertexGroupChunkER11QDataStreami")
for anchor in ["cmp    $0x1,%si", "$0x3f987f800", "cmp    $0x3,%eax", "cmp    $0xa,%eax", "cmp    $0x21,%eax", "movsd  %xmm9,0x50(%r8)", "movsd  %xmm9,0x50(%rax)", "0x40(%r9),%xmm0", "0x44(%r9),%xmm0", "0x48(%r9),%xmm0", "ucomiss"]:
    assert anchor in members, anchor
for offset in range(0x50, 0x56):
    assert f"cmpb   $0x0,0x{offset:x}(%r15)" in members
for offset in [0x28, 0x2c, 0x30]:
    assert f"0x{offset:x}(%r15)" in members
loader = assembly("_ZN15Au3dsGLSLObject13LoadMeshChunkER11QDataStreamiR7AuPointS3_10MODEL_TYPE")
assert "movl   $0x1,0xb4(%rsp)" in loader
assert "<_ZN15Au3dsGLSLVGroup9LoadChunkER11QDataStreami>" in loader
apply = assembly("_ZNK15Au3dsGLSLObject16ApplyScaleFactorER7AuPoint")
assert "cvtsd2ss 0x10(%rdi)" in apply
assert "<_ZNK8QMapDataI15Au3dsGLSLVertexdE8findNodeERKS0_.isra.0>" in apply
assert [i for i in range(34) if (0x3f987f800 >> i) & 1] == [11,12,13,14,15,16,17,18,23,24,27,28,29,30,31,32,33]
for name in ["native-body-group-members", "native-body-group-classification", "native-body-group-object-loader"]:
    record = json.loads(Path("docs/evidence", name + ".json").read_text())
    assert record["subject"]["digest"]["sha256"] == digest
classification = json.loads(Path("docs/evidence/native-body-group-classification.json").read_text())["normalized_result"]
assert classification.index('"ScalePt"') < classification.index('"ConnectPt"') < classification.index('"MidPt"')
assert classification.index('"Torso"') < classification.index('"Hand"') < classification.index('"Foot"') < classification.index('"Neck"') < classification.index('"Tail"') < classification.index('"Head"')
print("Verified native group default, name precedence, one-record point mask, multi-record pivot flags, xyz key comparisons/replacement and deformation float lookup. Static anchors, not weighted deformation/render parity.")
