"""Source-bound RotateWrist guard, inlined projection and output selection."""
import json
import runpy
import subprocess
from pathlib import Path

runpy.run_path("tests/native_dp_body_waist.py")
target = Path("reference/openshores-full/OpenShores.exe")
def assembly(symbol):
    return subprocess.check_output(["objdump", "-d", "--no-show-raw-insn", "--disassemble=" + symbol, str(target)], text=True)

wrapper = assembly("_ZN12DpArmAndHand11RotateWristEf")
for anchor in ["ucomiss", "jp ", "je ", "<_ZN12DpArmAndHand11RotateWristEf.part.0>"]:
    assert anchor in wrapper, anchor
concrete = assembly("_ZN12DpArmAndHand11RotateWristEf.part.0")
assert concrete.count("call   *0xa0(%rax)") == 2
assert concrete.count("<_ZNK11AuMatrix4x4mlERKS_>") == 1
assert "<_ZNK11AuMatrix4x47ProjectEddd>" not in concrete
assert "call   *0x30(%rax)" not in concrete
for anchor in ["lea    0x58(%rsp),%r9", "mov    %r13,%r9", "mov    0x170(%rsp),%eax", "0x70(%rbx),%xmm3", "0x78(%rbx),%xmm1", "0x80(%rbx),%xmm4", "mov    0x58(%rsp),%edx", "mov    0x58(%rsp),%eax", "<sincos>", "cvtss2sd"]:
    assert anchor in concrete, anchor
# Ordinary and sparse projection numerators finish column sums before adding
# translation; origin separately accumulates every W-row element.
for anchor in ["addsd  %xmm0,%xmm10", "addsd  %xmm11,%xmm7", "addsd  %xmm5,%xmm8", "addsd  %xmm1,%xmm2", "addsd  0xd0(%rax),%xmm10", "addsd  0xd8(%rax),%xmm7", "addsd  0xe0(%rax),%xmm8", "addsd  0xe8(%rax),%xmm2", "addsd  0xa8(%rax),%xmm2", "addsd  0xc8(%rax),%xmm2", "divsd  %xmm2,%xmm10"]:
    assert anchor in concrete, anchor
# The wrist's expanded pivot adds initial Y after the product sum, unlike
# shoulder's initial-Y-first grouping. All products still use primary-only *.
for anchor in ["addsd  %xmm10,%xmm9", "addsd  %xmm7,%xmm4", "addsd  %xmm8,%xmm0", "addsd  %xmm11,%xmm0"]:
    assert anchor in concrete, anchor
for name in ["native-dp-arm-rotate-wrist", "native-dp-arm-rotate-wrist-part0"]:
    evidence = json.loads(Path("docs/evidence", name + ".json").read_text())
    assert evidence["subject"]["digest"]["sha256"] == "285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80"
print("Verified wrist zero/NaN guard, field70 inlined sparse/origin/W projection, third-output-only writes, expanded pivot and primary matrix multiplication. Static evidence only.")
