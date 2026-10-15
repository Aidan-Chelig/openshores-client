"""Static shoulder wrapper, bone selection and primary-only product checks."""
import json
import runpy
import subprocess
from pathlib import Path

c = runpy.run_path("tests/native_dp_body_waist.py")
target = Path("reference/openshores-full/OpenShores.exe")
def assembly(symbol):
    return subprocess.check_output(["objdump", "-d", "--no-show-raw-insn", "--disassemble=" + symbol, str(target)], text=True)

wrapper = assembly("_ZN12DpArmAndHand14RotateShoulderEf")
for anchor in ["ucomiss", "jp ", "je ", "<_ZN12DpArmAndHand14RotateShoulderEf.part.0>"]:
    assert anchor in wrapper, anchor
concrete = assembly("_ZN12DpArmAndHand14RotateShoulderEf.part.0")
for anchor in ["call   *0xa0(%rax)", "call   *0x30(%rax)", "<sincos>", "cvtss2sd", "0x19(%rbx)", "0x1a(%rbx)", "$0x6,%al", "$0x7,%eax", "$0x5,%eax", "$0x1,%al"]:
    assert anchor in concrete, anchor
assert concrete.count("<_ZNK11AuMatrix4x4mlERKS_>") == 4
assert concrete.count("call   *0xa0(%rax)") == 1
assert concrete.count("call   *0x30(%rax)") == 1
# Native supplies three stack out references; the wing branch calls operator*
# for the first only, while the other branch calls it for all three.
single = concrete[concrete.index("1412ba279:"):concrete.index("1412ba3a0:")]
triple = concrete[concrete.index("1412ba3a0:"):]
assert single.count("<_ZNK11AuMatrix4x4mlERKS_>") == 1
assert triple.count("<_ZNK11AuMatrix4x4mlERKS_>") == 3
assert "0x44(%rsp)" in single
for reference in ["0x44(%rsp)", "0x48(%rsp)", "0x4c(%rsp)"]:
    assert reference in triple
# Preserve expanded pivot arithmetic instead of assuming a generic transform
# has identical float/zero/NaN behavior or reassociating flattened pseudocode.
for anchor in ["mulsd  %xmm8,%xmm6", "mulsd  %xmm8,%xmm11", "mulsd  %xmm8,%xmm9", "addsd  %xmm11,%xmm2", "subsd  %xmm13,%xmm15", "addsd  %xmm13,%xmm6", "addsd  %xmm9,%xmm1", "addsd  %xmm5,%xmm7", "addsd  %xmm10,%xmm0"]:
    assert anchor in concrete, anchor
for name in ["native-dp-arm-rotate-shoulder", "native-dp-arm-rotate-shoulder-part0"]:
    evidence = json.loads(Path("docs/evidence", name + ".json").read_text())
    assert evidence["subject"]["digest"]["sha256"] == "285047485af36d98bc9fe3c1c0a1f7bed4693f7ef3abcafc2c6d39d5e0d81d80"
print("Verified shoulder zero/NaN guard, actual Bones/current attachment calls, single/all-three selection, expanded pivot and matrix operator call sites. Static checks, not native execution.")
