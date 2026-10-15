"""Static elbow pivot, branch and actual Bones-reference checks."""
import json
import runpy
import subprocess
from pathlib import Path

runpy.run_path("tests/native_dp_body_waist.py")
target = Path("reference/openshores-full/OpenShores.exe")
def assembly(symbol):
    return subprocess.check_output(["objdump", "-d", "--no-show-raw-insn", "--disassemble=" + symbol, str(target)], text=True)

wrapper = assembly("_ZN12DpArmAndHand11RotateElbowEf")
for anchor in ["ucomiss", "jp ", "je ", "<_ZN12DpArmAndHand11RotateElbowEf.part.0>"]:
    assert anchor in wrapper, anchor
concrete = assembly("_ZN12DpArmAndHand11RotateElbowEf.part.0")
assert concrete.count("call   *0xa0(%rax)") == 2
assert concrete.count("call   *0x30(%rax)") == 1
assert concrete.count("<_ZNK11AuMatrix4x4mlERKS_>") == 3
assert concrete.count("<_ZN11AuMatrix4x47RotateXEd>") == 2
special = concrete[concrete.index("1412bde8a:"):concrete.index("1412be0e8:")]
ordinary = concrete[concrete.index("1412be0e8:"):]
assert special.count("<_ZNK11AuMatrix4x4mlERKS_>") == 1
assert ordinary.count("<_ZNK11AuMatrix4x4mlERKS_>") == 2
assert "call   *0x30(%rax)" in special
assert "<_ZNK11AuMatrix4x47ProjectEddd>" not in special
for anchor in ["0x19(%rbx)", "0x1a(%rbx)", "$0x6,%al", "$0x7,%eax", "$0x5,%eax", "$0x1,%al"]:
    assert anchor in concrete, anchor
# Second Bones out-ref is used as the Project matrix; field58/60/68 supplies
# x/y/z. Ignore the decompiler's apparent Project signature/register splice.
for anchor in ["lea    0x3c(%rsp),%rdx", "mov    %r12,%r8", "mov    %r13,%r9", "mov    0x60(%rsp),%edx", "0x58(%rbx),%xmm2", "0x60(%rbx),%xmm3", "0x68(%rbx),%xmm0", "<_ZNK11AuMatrix4x47ProjectEddd>", "mov    0x34(%rsp),%edx", "mov    0x38(%rsp),%edx"]:
    assert anchor in ordinary, anchor
assert "mov    0x34(%rsp),%edx" in special
# SIMD low/high lanes establish X/Y ordering of post-pivot additions.
for anchor in ["mulpd  %xmm6,%xmm2", "mulpd  %xmm0,%xmm1", "addpd  %xmm2,%xmm1", "addpd  %xmm6,%xmm0"]:
    assert anchor in special, anchor
for anchor in ["mulpd  %xmm0,%xmm2", "mulpd  %xmm6,%xmm1", "addpd  %xmm2,%xmm1", "addpd  %xmm0,%xmm6"]:
    assert anchor in ordinary, anchor
for name in ["native-dp-arm-rotate-elbow", "native-dp-arm-rotate-elbow-part0"]:
    evidence = json.loads(Path("docs/evidence", name + ".json").read_text())
    assert evidence["subject"]["digest"]["sha256"] == "285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80"
print("Verified elbow zero/NaN guard, wing/ordinary pivots, field58 Project register binding, output2 versus output2/3 writes and distinct SIMD pivot orders. Static evidence only.")
